//! `leafypdf.exe --thumbnail <pdf> <out> <size>`: the picture Explorer shows for a PDF.
//!
//! The Windows thumbnail handler (`crates/thumbs`) runs this in a child process and hands the result to
//! Explorer, so a damaged or hostile file can only ever crash this process. It opens no window, writes no
//! log file and never hands its file to a running Leafy PDF.
//!
//! `out` holds the bytes `LFTH`, then width and height as little-endian `u32`, then `width * height`
//! premultiplied BGRA pixels, the longest side at most `size`. A page is drawn over white paper. A file that
//! needs a password gets the Leafy PDF tile instead, so a protected document never shows a page.
//!
//! Exit status: `0` when `out` was written, `1` otherwise (Explorer then shows the file icon).

use std::io::Write as _;
use std::process::ExitCode;
use std::sync::Arc;

use pdfcraft_render::{OpenError, PageRenderer, RenderConfig, RenderRequest, RequestKind, inspect};

/// The tile shown for a password-protected file: the app icon.
const TILE_PNG: &[u8] = include_bytes!("../../../assets/app-icon/leafypdf-256.png");
const MIN_SIZE: u32 = 16;
const MAX_SIZE: u32 = 1024;
/// A PDF larger than this gets no thumbnail: reading it just to draw a small picture is not worth it.
const MAX_FILE_BYTES: u64 = 256 * 1024 * 1024;

/// A picture: width, height and premultiplied RGBA8 pixels.
struct Picture {
    width: u32,
    height: u32,
    rgba: Vec<u8>,
}

pub(crate) fn run(args: &[String]) -> ExitCode {
    // A panic must end in "no thumbnail", not in a crash report from the process that hosts Explorer's handler.
    let result = std::panic::catch_unwind(|| try_run(args));
    match result {
        Ok(Ok(())) => ExitCode::SUCCESS,
        Ok(Err(reason)) => {
            log_failure(&reason);
            ExitCode::FAILURE
        }
        Err(_) => {
            log_failure("internal error (panic)");
            ExitCode::FAILURE
        }
    }
}

/// With `LEAFYPDF_THUMBNAIL_LOG=<file>` set, why a picture could not be made is appended to that file
/// (the process has no console, and Explorer only learns that there is no thumbnail).
fn log_failure(reason: &str) {
    let Some(path) = std::env::var_os("LEAFYPDF_THUMBNAIL_LOG") else { return };
    if let Ok(mut file) = std::fs::OpenOptions::new().create(true).append(true).open(path) {
        let _ = writeln!(file, "{reason}");
    }
}

fn try_run(args: &[String]) -> Result<(), String> {
    let [pdf, out, size] = args else {
        return Err("usage: --thumbnail <pdf> <out> <size>".into());
    };
    let size: u32 = size.parse().map_err(|_| format!("bad size {size:?}"))?;
    let size = size.clamp(MIN_SIZE, MAX_SIZE);
    let picture = picture_of(pdf, size)?;
    write_picture(out, &picture)
}

fn picture_of(pdf: &str, size: u32) -> Result<Picture, String> {
    let length = std::fs::metadata(pdf).map_err(|e| format!("{pdf}: {e}"))?.len();
    if length > MAX_FILE_BYTES {
        return Err(format!("{pdf}: {length} bytes is too large for a thumbnail"));
    }
    let bytes = Arc::new(std::fs::read(pdf).map_err(|e| format!("{pdf}: {e}"))?);
    let info = match inspect(bytes.clone(), None) {
        Ok(info) => info,
        // Needs a password to open: show the tile, never a page.
        Err(OpenError::NeedsPassword) => return tile(size),
        Err(e) => return Err(format!("{pdf}: {e:?}")),
    };
    let page = info.pages.first().ok_or("the document has no pages")?;
    let longest = page.width.max(page.height).max(1.0);
    let scale = (size as f32 / longest).clamp(0.01, 8.0);
    let display = pdfcraft_engine::display_bytes_for(bytes, None);
    let mut renderer = PageRenderer::new(display, RenderConfig { password: None, reject_oversize: true, ..Default::default() });
    let drawn = renderer.render(RenderRequest { page: 0, kind: RequestKind::Pixels, tile: None, scale, tag: 0, preview: false });
    if let Some(error) = drawn.error {
        return Err(error);
    }
    let mut rgba = drawn.rgba.to_vec();
    over_white(&mut rgba);
    Ok(Picture { width: drawn.width, height: drawn.height, rgba })
}

/// Paper is white: composite premultiplied pixels over it, so the page is opaque whatever the renderer left clear.
fn over_white(rgba: &mut [u8]) {
    for px in rgba.chunks_exact_mut(4) {
        let clear = 255 - px[3];
        px[0] = px[0].saturating_add(clear);
        px[1] = px[1].saturating_add(clear);
        px[2] = px[2].saturating_add(clear);
        px[3] = 255;
    }
}

/// The Leafy PDF tile at `size` (never enlarged past its own 256 px; Explorer scales it).
fn tile(size: u32) -> Result<Picture, String> {
    let mut decoder = png::Decoder::new(std::io::Cursor::new(TILE_PNG));
    decoder.set_transformations(png::Transformations::normalize_to_color8());
    let mut reader = decoder.read_info().map_err(|e| e.to_string())?;
    let mut buffer = vec![0; reader.output_buffer_size().ok_or("the tile is too large")?];
    let frame = reader.next_frame(&mut buffer).map_err(|e| e.to_string())?;
    let (width, height) = (frame.width, frame.height);
    let data = &buffer[..frame.buffer_size()];
    let mut rgba = Vec::with_capacity(width as usize * height as usize * 4);
    match frame.color_type {
        png::ColorType::Rgba => rgba.extend_from_slice(data),
        png::ColorType::Rgb => data.chunks_exact(3).for_each(|p| rgba.extend_from_slice(&[p[0], p[1], p[2], 255])),
        other => return Err(format!("the tile has an unsupported colour type {other:?}")),
    }
    // The PNG holds straight alpha; the picture format is premultiplied.
    for px in rgba.chunks_exact_mut(4) {
        let a = u32::from(px[3]);
        for c in &mut px[..3] {
            *c = ((u32::from(*c) * a + 127) / 255) as u8;
        }
    }
    let picture = Picture { width, height, rgba };
    Ok(if size < picture.width { shrink(&picture, size) } else { picture })
}

/// Area-average `picture` down to a square `size` (the tile is square).
fn shrink(picture: &Picture, size: u32) -> Picture {
    let (sw, sh) = (picture.width as usize, picture.height as usize);
    let size = size as usize;
    let mut rgba = vec![0u8; size * size * 4];
    for y in 0..size {
        let (y0, y1) = (y * sh / size, ((y + 1) * sh / size).max(y * sh / size + 1));
        for x in 0..size {
            let (x0, x1) = (x * sw / size, ((x + 1) * sw / size).max(x * sw / size + 1));
            let mut sum = [0u32; 4];
            for sy in y0..y1.min(sh) {
                for sx in x0..x1.min(sw) {
                    let at = (sy * sw + sx) * 4;
                    for (c, total) in sum.iter_mut().enumerate() {
                        *total += u32::from(picture.rgba[at + c]);
                    }
                }
            }
            let n = ((y1.min(sh) - y0) * (x1.min(sw) - x0)).max(1) as u32;
            let at = (y * size + x) * 4;
            for c in 0..4 {
                rgba[at + c] = (sum[c] / n) as u8;
            }
        }
    }
    Picture { width: size as u32, height: size as u32, rgba }
}

fn write_picture(out: &str, picture: &Picture) -> Result<(), String> {
    let mut data = Vec::with_capacity(12 + picture.rgba.len());
    data.extend_from_slice(b"LFTH");
    data.extend_from_slice(&picture.width.to_le_bytes());
    data.extend_from_slice(&picture.height.to_le_bytes());
    for px in picture.rgba.chunks_exact(4) {
        data.extend_from_slice(&[px[2], px[1], px[0], px[3]]); // RGBA -> BGRA
    }
    let mut file = std::fs::File::create(out).map_err(|e| format!("{out}: {e}"))?;
    file.write_all(&data).map_err(|e| format!("{out}: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paper_is_opaque_white_where_the_page_left_it_clear() {
        let mut rgba = [0, 0, 0, 0, 10, 20, 30, 255, 50, 50, 50, 128];
        over_white(&mut rgba);
        assert_eq!(rgba, [255, 255, 255, 255, 10, 20, 30, 255, 177, 177, 177, 255]);
    }

    #[test]
    fn the_tile_is_the_app_icon_at_every_requested_size() {
        let big = tile(1024).unwrap();
        assert_eq!((big.width, big.height), (256, 256), "not enlarged");
        assert_eq!(big.rgba.len(), 256 * 256 * 4);
        let small = tile(48).unwrap();
        assert_eq!((small.width, small.height, small.rgba.len()), (48, 48, 48 * 48 * 4));
        let centre = (24 * 48 + 24) * 4;
        assert_eq!(small.rgba[centre + 3], 255, "the middle of the tile is solid");
        assert_eq!(small.rgba[3], 0, "its rounded corner is clear");
    }

    #[test]
    fn a_bad_call_fails_instead_of_panicking() {
        assert!(try_run(&[]).is_err());
        assert!(try_run(&["a.pdf".into(), "b".into(), "wide".into()]).is_err());
        assert!(try_run(&["Z:\\no\\such\\file.pdf".into(), "b".into(), "64".into()]).is_err());
    }
}
