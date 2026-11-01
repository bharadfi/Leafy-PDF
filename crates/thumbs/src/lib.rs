//! Leafy PDF's thumbnail handler for Windows Explorer (`IThumbnailProvider`).
//!
//! Explorer hands this library the PDF as a stream. The library does not read the PDF: it writes the bytes
//! to a temporary file and runs `leafypdf.exe --thumbnail <pdf> <out> <size>` from the folder it is installed
//! in (see `apps/pdfcraft/src/thumbnail.rs`), waits for it with a time limit, and turns the picture it wrote
//! into the bitmap Explorer asked for. A damaged or hostile file can therefore only ever stop that child
//! process; a file that needs a password gets the Leafy PDF tile, never a page.
//!
//! The installer registers the handler for the `LeafyPdf.Document` ProgID, so it applies when Leafy PDF is
//! the default app for `.pdf`. It is the only crate in the workspace that uses `unsafe`: the COM calls
//! cannot be made without it.
#![cfg(windows)]
// The COM entry points are exported on purpose (Explorer finds them by name); the linker would rather have
// them marked private in an export definition, which says nothing a reader of this file needs.
#![allow(linker_messages)]

use std::cell::RefCell;
use std::ffi::c_void;
use std::os::windows::process::CommandExt as _;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicIsize, AtomicU32, Ordering};
use std::time::{Duration, Instant};

use windows::Win32::Foundation::{
    CLASS_E_CLASSNOTAVAILABLE, CLASS_E_NOAGGREGATION, E_FAIL, E_INVALIDARG, E_OUTOFMEMORY, E_UNEXPECTED, HMODULE, S_FALSE,
};
use windows::Win32::Graphics::Gdi::{BI_RGB, BITMAPINFO, BITMAPINFOHEADER, CreateDIBSection, DIB_RGB_COLORS, HBITMAP};
use windows::Win32::System::Com::{IClassFactory, IClassFactory_Impl, IStream};
use windows::Win32::System::LibraryLoader::GetModuleFileNameW;
use windows::Win32::System::SystemServices::DLL_PROCESS_ATTACH;
use windows::Win32::UI::Shell::PropertiesSystem::{IInitializeWithStream, IInitializeWithStream_Impl};
use windows::Win32::UI::Shell::{IThumbnailProvider, IThumbnailProvider_Impl, WTS_ALPHATYPE, WTSAT_ARGB};
use windows::core::{BOOL, GUID, HRESULT, IUnknown, Interface as _, Ref, Result, implement};

/// The handler's class id. The installer registers this same value (`packaging/windows/pdfcraft.wxs`).
const CLSID_LEAFY_THUMBNAIL: GUID = GUID::from_u128(0x93E5ACCA_84E1_4B0B_A11A_2AACD2262036);
const EXE_NAME: &str = "leafypdf.exe";
/// A PDF larger than this gets no thumbnail (the app refuses it as well).
const MAX_PDF_BYTES: usize = 256 * 1024 * 1024;
/// The child gets this long; a slow or stuck file then simply shows its icon.
const CHILD_TIMEOUT: Duration = Duration::from_secs(20);
/// A picture larger than this in either direction is not one the app wrote.
const MAX_SIDE: u32 = 4096;
/// `CREATE_NO_WINDOW | BELOW_NORMAL_PRIORITY_CLASS`: a background job for a folder full of PDFs.
const CHILD_FLAGS: u32 = 0x0800_0000 | 0x0000_4000;

static MODULE: AtomicIsize = AtomicIsize::new(0);
static SEQUENCE: AtomicU32 = AtomicU32::new(0);

#[unsafe(no_mangle)]
extern "system" fn DllMain(module: HMODULE, reason: u32, _reserved: *mut c_void) -> BOOL {
    if reason == DLL_PROCESS_ATTACH {
        MODULE.store(module.0 as isize, Ordering::Relaxed);
    }
    BOOL(1)
}

/// Explorer asks for the class factory of [`CLSID_LEAFY_THUMBNAIL`].
#[unsafe(no_mangle)]
unsafe extern "system" fn DllGetClassObject(clsid: *const GUID, iid: *const GUID, object: *mut c_void) -> HRESULT {
    if clsid.is_null() || iid.is_null() || object.is_null() {
        return E_INVALIDARG;
    }
    // SAFETY: COM passes valid pointers for the duration of the call (checked non-null above).
    unsafe {
        if *clsid != CLSID_LEAFY_THUMBNAIL {
            return CLASS_E_CLASSNOTAVAILABLE;
        }
        let factory: IClassFactory = Factory.into();
        factory.query(iid, object.cast())
    }
}

/// The library stays loaded while its host process runs (a thumbnail host is short-lived).
#[unsafe(no_mangle)]
extern "system" fn DllCanUnloadNow() -> HRESULT {
    S_FALSE
}

#[implement(IClassFactory)]
struct Factory;

impl IClassFactory_Impl for Factory_Impl {
    fn CreateInstance(&self, outer: Ref<'_, IUnknown>, iid: *const GUID, object: *mut *mut c_void) -> Result<()> {
        if outer.is_some() {
            return Err(CLASS_E_NOAGGREGATION.into());
        }
        if iid.is_null() || object.is_null() {
            return Err(E_INVALIDARG.into());
        }
        let provider: IUnknown = Provider::default().into();
        // SAFETY: `iid` and `object` are COM's out parameters, checked non-null above.
        unsafe { provider.query(iid, object.cast()).ok() }
    }

    fn LockServer(&self, _lock: BOOL) -> Result<()> {
        Ok(())
    }
}

#[implement(IInitializeWithStream, IThumbnailProvider)]
#[derive(Default)]
struct Provider {
    stream: RefCell<Option<IStream>>,
}

impl IInitializeWithStream_Impl for Provider_Impl {
    fn Initialize(&self, stream: Ref<'_, IStream>, _mode: u32) -> Result<()> {
        *self.stream.borrow_mut() = Some(stream.ok()?.clone());
        Ok(())
    }
}

impl IThumbnailProvider_Impl for Provider_Impl {
    fn GetThumbnail(&self, size: u32, bitmap: *mut HBITMAP, alpha: *mut WTS_ALPHATYPE) -> Result<()> {
        if bitmap.is_null() || alpha.is_null() {
            return Err(E_INVALIDARG.into());
        }
        let stream = self.stream.borrow().clone().ok_or_else(|| windows::core::Error::from(E_UNEXPECTED))?;
        let pdf = read_stream(&stream)?;
        let picture = picture_for(&pdf, size.clamp(16, 1024)).map_err(|_| windows::core::Error::from(E_FAIL))?;
        let handle = make_bitmap(&picture)?;
        // SAFETY: Explorer's out parameters, checked non-null above.
        unsafe {
            *bitmap = handle;
            *alpha = WTSAT_ARGB;
        }
        Ok(())
    }
}

/// A decoded picture: size and premultiplied BGRA pixels, top row first.
struct Picture {
    width: u32,
    height: u32,
    bgra: Vec<u8>,
}

fn read_stream(stream: &IStream) -> Result<Vec<u8>> {
    let mut data = Vec::new();
    let mut chunk = vec![0u8; 1 << 20];
    loop {
        let mut read = 0u32;
        // SAFETY: `chunk` is a valid buffer of the length given, and `read` outlives the call.
        unsafe { stream.Read(chunk.as_mut_ptr().cast(), chunk.len() as u32, Some(&mut read)).ok()? };
        if read == 0 {
            return Ok(data);
        }
        data.extend_from_slice(&chunk[..read as usize]);
        if data.len() > MAX_PDF_BYTES {
            return Err(E_OUTOFMEMORY.into());
        }
    }
}

/// Removes the temporary files when dropped, whichever way the call ends.
struct Temp(Vec<PathBuf>);

impl Drop for Temp {
    fn drop(&mut self) {
        for path in &self.0 {
            let _ = std::fs::remove_file(path);
        }
    }
}

/// The folder this library is installed in: `leafypdf.exe` sits beside it.
fn installed_exe() -> Option<PathBuf> {
    let mut buffer = [0u16; 1024];
    // SAFETY: `buffer` is valid for the length given; the handle is this library's own.
    let length = unsafe { GetModuleFileNameW(Some(HMODULE(MODULE.load(Ordering::Relaxed) as *mut c_void)), &mut buffer) } as usize;
    if length == 0 || length >= buffer.len() {
        return None;
    }
    let dll = PathBuf::from(String::from_utf16(&buffer[..length]).ok()?);
    Some(dll.parent()?.join(EXE_NAME))
}

fn picture_for(pdf: &[u8], size: u32) -> std::result::Result<Picture, String> {
    let exe = installed_exe().filter(|p| p.is_file()).ok_or("leafypdf.exe is not beside the library")?;
    let dir = std::env::temp_dir();
    let tag = format!("leafypdf-thumb-{}-{}", std::process::id(), SEQUENCE.fetch_add(1, Ordering::Relaxed));
    let (input, output) = (dir.join(format!("{tag}.pdf")), dir.join(format!("{tag}.bgra")));
    let _cleanup = Temp(vec![input.clone(), output.clone()]);
    std::fs::write(&input, pdf).map_err(|e| e.to_string())?;
    run_child(&exe, &input, &output, size)?;
    parse_picture(&std::fs::read(&output).map_err(|e| e.to_string())?)
}

fn run_child(exe: &Path, input: &Path, output: &Path, size: u32) -> std::result::Result<(), String> {
    let mut child = Command::new(exe)
        .arg("--thumbnail")
        .arg(input)
        .arg(output)
        .arg(size.to_string())
        .creation_flags(CHILD_FLAGS)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| e.to_string())?;
    let started = Instant::now();
    loop {
        match child.try_wait().map_err(|e| e.to_string())? {
            Some(status) if status.success() => return Ok(()),
            Some(status) => return Err(format!("leafypdf.exe exited with {status}")),
            None if started.elapsed() > CHILD_TIMEOUT => {
                let _ = child.kill();
                let _ = child.wait();
                return Err("timed out".into());
            }
            None => std::thread::sleep(Duration::from_millis(15)),
        }
    }
}

/// `LFTH`, width, height (little-endian `u32`), then `width * height` BGRA pixels.
fn parse_picture(data: &[u8]) -> std::result::Result<Picture, String> {
    let header = data.get(..12).filter(|h| &h[..4] == b"LFTH").ok_or("not a picture")?;
    let width = u32::from_le_bytes([header[4], header[5], header[6], header[7]]);
    let height = u32::from_le_bytes([header[8], header[9], header[10], header[11]]);
    if width == 0 || height == 0 || width > MAX_SIDE || height > MAX_SIDE {
        return Err(format!("picture of {width} x {height}"));
    }
    let pixels = &data[12..];
    if pixels.len() != width as usize * height as usize * 4 {
        return Err("the picture is truncated".into());
    }
    Ok(Picture { width, height, bgra: pixels.to_vec() })
}

fn make_bitmap(picture: &Picture) -> Result<HBITMAP> {
    let info = BITMAPINFO {
        bmiHeader: BITMAPINFOHEADER {
            biSize: size_of::<BITMAPINFOHEADER>() as u32,
            biWidth: picture.width as i32,
            // Negative: top row first, as the picture is stored.
            biHeight: -(picture.height as i32),
            biPlanes: 1,
            biBitCount: 32,
            biCompression: BI_RGB.0,
            ..Default::default()
        },
        ..Default::default()
    };
    let mut bits: *mut c_void = std::ptr::null_mut();
    // SAFETY: `info` describes a 32-bit top-down bitmap; `bits` receives the pixel buffer GDI owns.
    let bitmap = unsafe { CreateDIBSection(None, &info, DIB_RGB_COLORS, &mut bits, None, 0)? };
    if bits.is_null() {
        return Err(E_OUTOFMEMORY.into());
    }
    // SAFETY: the section holds exactly `width * height * 4` bytes, which is `picture.bgra.len()`.
    unsafe { std::ptr::copy_nonoverlapping(picture.bgra.as_ptr(), bits.cast::<u8>(), picture.bgra.len()) };
    Ok(bitmap)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn picture_bytes(width: u32, height: u32, pixels: usize) -> Vec<u8> {
        let mut data = b"LFTH".to_vec();
        data.extend_from_slice(&width.to_le_bytes());
        data.extend_from_slice(&height.to_le_bytes());
        data.extend(std::iter::repeat_n(7u8, pixels));
        data
    }

    #[test]
    fn a_well_formed_picture_is_read() {
        let picture = parse_picture(&picture_bytes(3, 2, 3 * 2 * 4)).unwrap();
        assert_eq!((picture.width, picture.height, picture.bgra.len()), (3, 2, 24));
    }

    #[test]
    fn anything_else_is_refused() {
        assert!(parse_picture(b"").is_err());
        assert!(parse_picture(b"PNG\0\0\0\0\0\0\0\0\0").is_err(), "wrong magic");
        assert!(parse_picture(&picture_bytes(0, 5, 0)).is_err(), "empty");
        assert!(parse_picture(&picture_bytes(MAX_SIDE + 1, 1, 4)).is_err(), "too wide");
        assert!(parse_picture(&picture_bytes(3, 2, 23)).is_err(), "truncated");
        assert!(parse_picture(&picture_bytes(3, 2, 25)).is_err(), "trailing bytes");
    }

    #[test]
    fn the_bitmap_holds_the_pixels() {
        let picture = Picture { width: 2, height: 1, bgra: vec![1, 2, 3, 255, 4, 5, 6, 255] };
        let handle = make_bitmap(&picture).unwrap();
        assert!(!handle.is_invalid());
    }
}
