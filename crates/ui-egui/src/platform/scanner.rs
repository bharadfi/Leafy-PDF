//! Scan with the scanner Windows already knows.
//!
//! Leafy PDF ships no scanner library. On Windows a hidden PowerShell opens the system's own scan window
//! (Windows Image Acquisition, WIA), saves what the scanner returned as a PNG file, and the picture is read
//! back. The user picks the scanner, the colour mode and the resolution in that window, as in any other
//! Windows program. Elsewhere this reports that no scanner support is available.

use std::sync::{Arc, Mutex, PoisonError};

/// What a scan came to.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Scanned {
    /// A PNG picture.
    Picture(Vec<u8>),
    /// The scan window was closed without scanning.
    Cancelled,
    /// Why no picture was made (no scanner, scanner busy, the service is off ...).
    Problem(String),
}

/// A scan in progress, if any. Cloning shares it.
#[derive(Clone, Default)]
pub(crate) struct Scanner {
    /// `Some` while the scan window is open or its result has not been taken.
    running: Arc<Mutex<Option<Option<Scanned>>>>,
}

impl Scanner {
    /// Opens the scan window. `false` when a scan is already running.
    pub(crate) fn start(&self) -> bool {
        self.start_with(scan)
    }

    /// Runs `scan` on a worker thread as the scan. `false` when a scan is already running.
    fn start_with(&self, scan: impl FnOnce() -> Scanned + Send + 'static) -> bool {
        {
            let mut running = self.running.lock().unwrap_or_else(PoisonError::into_inner);
            if running.is_some() {
                return false;
            }
            *running = Some(None);
        }
        let running = self.running.clone();
        let work = move || {
            let result = scan();
            *running.lock().unwrap_or_else(PoisonError::into_inner) = Some(Some(result));
        };
        if std::thread::Builder::new().name("pdfcraft-scan".into()).spawn(work).is_err() {
            *self.running.lock().unwrap_or_else(PoisonError::into_inner) = Some(Some(Scanned::Problem("The scan could not be started.".into())));
        }
        true
    }

    /// Whether a scan window is open, or its result has not been taken yet.
    pub(crate) fn busy(&self) -> bool {
        self.running.lock().unwrap_or_else(PoisonError::into_inner).is_some()
    }

    /// The result, once, when the scan has finished.
    pub(crate) fn take(&self) -> Option<Scanned> {
        let mut running = self.running.lock().unwrap_or_else(PoisonError::into_inner);
        match running.take() {
            Some(Some(done)) => Some(done),
            other => {
                *running = other;
                None
            }
        }
    }
}

#[cfg(windows)]
fn scan() -> Scanned {
    use std::os::windows::process::CommandExt as _;
    use std::process::{Command, Stdio};

    /// `CREATE_NO_WINDOW`: no console window; the scan window itself is a normal window.
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    /// The exit status that means "closed without scanning".
    const CANCELLED: i32 = 2;
    /// Opens the scan window, asking for a colour PNG, and saves it where `LEAFYPDF_SCAN_OUT` says. The path
    /// travels in the environment, not in the script, so no character in it is ever read as code.
    const SCRIPT: &str = "$ErrorActionPreference = 'Stop'; \
        try { \
          $dialog = New-Object -ComObject WIA.CommonDialog; \
          $image = $dialog.ShowAcquireImage(1, 1, 131072, '{B96B3CAF-0728-11D3-9D7B-0000F81EF32E}', $false, $true, $false); \
          if ($null -eq $image) { exit 2 }; \
          $image.SaveFile($env:LEAFYPDF_SCAN_OUT); \
          exit 0 \
        } catch { [Console]::Out.Write($_.Exception.Message); exit 3 }";

    static COUNTER: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
    let out =
        std::env::temp_dir().join(format!("leafypdf-scan-{}-{}.png", std::process::id(), COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed)));
    let _ = std::fs::remove_file(&out);
    let outcome = Command::new("powershell.exe")
        .args(["-NoProfile", "-NonInteractive", "-Command", SCRIPT])
        .env("LEAFYPDF_SCAN_OUT", &out)
        .creation_flags(CREATE_NO_WINDOW)
        .stdin(Stdio::null())
        .output();
    let scanned = match outcome {
        Err(e) => Scanned::Problem(format!("Windows' scanner support could not be started: {e}")),
        Ok(o) if o.status.code() == Some(CANCELLED) => Scanned::Cancelled,
        Ok(o) if o.status.success() => match std::fs::read(&out) {
            Ok(bytes) if !bytes.is_empty() => Scanned::Picture(bytes),
            _ => Scanned::Problem("The scanner returned no picture.".into()),
        },
        Ok(o) => {
            let said = String::from_utf8_lossy(&o.stdout).trim().to_string();
            Scanned::Problem(if said.is_empty() {
                "The scan failed.".into()
            } else if said.contains("No WIA device") {
                "No scanner was found. Connect a scanner and try again.".into()
            } else {
                said
            })
        }
    };
    let _ = std::fs::remove_file(&out);
    scanned
}

#[cfg(not(windows))]
fn scan() -> Scanned {
    Scanned::Problem("Scanning uses the scanner support built into Windows, which is not available here.".into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_second_scan_cannot_start_while_one_is_open_and_the_result_is_taken_once() {
        // A stand-in for the scan window: the real one would open on a machine that has a scanner.
        fn stub() -> Scanned {
            std::thread::sleep(std::time::Duration::from_millis(200));
            Scanned::Picture(vec![1, 2, 3])
        }
        let scanner = Scanner::default();
        assert!(scanner.take().is_none(), "nothing yet");
        assert!(scanner.start_with(stub));
        assert!(!scanner.start_with(stub), "one scan window at a time");
        // Whatever this machine has, the scan ends in a result.
        let mut result = None;
        for _ in 0..300 {
            if let Some(done) = scanner.take() {
                result = Some(done);
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
        assert_eq!(result, Some(Scanned::Picture(vec![1, 2, 3])));
        assert!(scanner.take().is_none(), "taken once");
        assert!(scanner.start_with(stub), "a new scan can start afterwards");
    }
}
