//! Read Out Loud: speaks text with the voice Windows already has.
//!
//! Leafy PDF ships no speech engine. On Windows the text is sent, over standard input, to a hidden
//! PowerShell that hands it to the built-in `System.Speech` synthesizer; stopping ends that process. Standard
//! input keeps the document's text out of the command line, so no character in it is ever interpreted as
//! part of a command. Pausing and resuming go through a small control file the speaking process checks a few
//! times a second. Elsewhere this reports that no voice is available.

use std::path::PathBuf;
use std::process::Child;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

/// The most characters read in one go (about a 400-page novel).
const MAX_CHARS: usize = 1_500_000;

static SEQUENCE: AtomicU32 = AtomicU32::new(0);

/// What is being read aloud, if anything. Cloning shares it.
#[derive(Clone, Default)]
pub(crate) struct Voice {
    inner: Arc<Mutex<State>>,
}

#[derive(Default)]
struct State {
    /// Changes whenever reading is stopped or replaced, so a slow start can tell it was overtaken.
    generation: u64,
    /// The speaking process.
    child: Option<Child>,
    /// The file the speaking process is told to pause and resume through.
    control: Option<PathBuf>,
    /// Whether the reading is paused.
    paused: bool,
    /// Why the last start failed, until the interface has shown it.
    problem: Option<String>,
}

impl State {
    /// Ends the current reading and starts a new generation.
    fn stop(&mut self) -> u64 {
        self.generation += 1;
        self.problem = None;
        self.paused = false;
        if let Some(mut child) = self.child.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
        if let Some(control) = self.control.take() {
            let _ = std::fs::remove_file(control);
        }
        self.generation
    }
}

impl Drop for State {
    fn drop(&mut self) {
        self.stop();
    }
}

fn lock(inner: &Mutex<State>) -> MutexGuard<'_, State> {
    inner.lock().unwrap_or_else(PoisonError::into_inner)
}

/// A fresh control file name, in the temporary folder.
fn control_file() -> PathBuf {
    std::env::temp_dir().join(format!("leafypdf-speech-{}-{}.ctl", std::process::id(), SEQUENCE.fetch_add(1, Ordering::Relaxed)))
}

impl Voice {
    /// Stops what is being read and starts reading the text `text` produces. It runs on a worker thread:
    /// getting the text of a whole document takes time the interface should not wait for.
    pub(crate) fn read(&self, text: impl FnOnce() -> Result<String, String> + Send + 'static) {
        let generation = lock(&self.inner).stop();
        let inner = self.inner.clone();
        let work = move || {
            let control = control_file();
            let started = text().and_then(|t| speak(&t, &control, None));
            let mut state = lock(&inner);
            if state.generation != generation {
                // Stopped, or replaced by another reading, while this one was starting.
                if let Ok(mut child) = started {
                    let _ = child.kill();
                    let _ = child.wait();
                }
                let _ = std::fs::remove_file(&control);
                return;
            }
            match started {
                Ok(child) => {
                    state.child = Some(child);
                    state.control = Some(control);
                }
                Err(problem) => state.problem = Some(problem),
            }
        };
        if std::thread::Builder::new().name("pdfcraft-speech".into()).spawn(work).is_err() {
            lock(&self.inner).problem = Some("Read Out Loud could not start.".into());
        }
    }

    /// Stops reading.
    pub(crate) fn stop(&self) {
        lock(&self.inner).stop();
    }

    /// Pauses a reading, or resumes a paused one. `Some(true)` when it is now paused, `Some(false)` when it
    /// is reading again, `None` when nothing is being read (it may just have finished).
    pub(crate) fn toggle_pause(&self) -> Option<bool> {
        let mut state = lock(&self.inner);
        let finished = state.child.as_mut().is_none_or(|child| !matches!(child.try_wait(), Ok(None)));
        if finished {
            state.stop();
            return None;
        }
        let control = state.control.clone()?;
        let command = if state.paused { "resume" } else { "pause" };
        std::fs::write(control, command).ok()?;
        state.paused = !state.paused;
        Some(state.paused)
    }

    /// Why reading could not start, once.
    pub(crate) fn take_problem(&self) -> Option<String> {
        lock(&self.inner).problem.take()
    }
}

/// `text` limited to [`MAX_CHARS`], cut at a character boundary.
fn limited(text: &str) -> &str {
    match text.char_indices().nth(MAX_CHARS) {
        Some((end, _)) => &text[..end],
        None => text,
    }
}

/// Starts speaking `text`; `control` is the file the speaker watches for `pause` and `resume`. `volume` (0 to 100)
/// is for tests, which must not make a noise; `None` leaves the voice at Windows' own setting.
#[cfg(windows)]
fn speak(text: &str, control: &std::path::Path, volume: Option<u8>) -> Result<Child, String> {
    use std::io::Write as _;
    use std::os::windows::process::CommandExt as _;
    use std::process::{Command, Stdio};

    /// `CREATE_NO_WINDOW`: no console window flashes up.
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    /// Reads the text as UTF-8 from standard input and speaks it in the background with the voice Windows has
    /// chosen, then looks every 150 ms for a `pause` or `resume` written to the control file until the voice is
    /// done. The path and the volume travel in the environment, not in the script.
    const SCRIPT: &str = "[Console]::InputEncoding = [Text.Encoding]::UTF8; Add-Type -AssemblyName System.Speech; \
                          $text = [Console]::In.ReadToEnd(); \
                          $voice = New-Object System.Speech.Synthesis.SpeechSynthesizer; \
                          if ($env:LEAFYPDF_SPEECH_VOLUME) { $voice.Volume = [int]$env:LEAFYPDF_SPEECH_VOLUME }; \
                          Register-ObjectEvent -InputObject $voice -EventName SpeakCompleted -SourceIdentifier Done | Out-Null; \
                          $voice.SpeakAsync($text) | Out-Null; \
                          $control = $env:LEAFYPDF_SPEECH_CONTROL; \
                          while ($null -eq (Wait-Event -SourceIdentifier Done -Timeout 0.15)) { \
                            if ($control -and (Test-Path -LiteralPath $control)) { \
                              $command = (Get-Content -LiteralPath $control -Raw).Trim(); \
                              Remove-Item -LiteralPath $control -ErrorAction SilentlyContinue; \
                              if ($command -eq 'pause') { $voice.Pause() } elseif ($command -eq 'resume') { $voice.Resume() } \
                            } \
                          }";

    let text = limited(text);
    if text.trim().is_empty() {
        return Err("There is no text to read on these pages.".into());
    }
    let mut command = Command::new("powershell.exe");
    command
        .args(["-NoProfile", "-NonInteractive", "-Command", SCRIPT])
        .env("LEAFYPDF_SPEECH_CONTROL", control)
        .creation_flags(CREATE_NO_WINDOW)
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    if let Some(volume) = volume {
        command.env("LEAFYPDF_SPEECH_VOLUME", volume.min(100).to_string());
    }
    let mut child = command.spawn().map_err(|e| format!("Windows' speech could not be started: {e}"))?;
    let written = child.stdin.take().map(|mut stdin| stdin.write_all(text.as_bytes()));
    if !matches!(written, Some(Ok(()))) {
        let _ = child.kill();
        let _ = child.wait();
        return Err("The text could not be handed to Windows' speech.".into());
    }
    // Standard input closed when `stdin` went out of scope: the reader sees the end of the text.
    Ok(child)
}

#[cfg(not(windows))]
fn speak(_text: &str, _control: &std::path::Path, _volume: Option<u8>) -> Result<Child, String> {
    Err("Read Out Loud uses the voice built into Windows, which is not available here.".into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn long_text_is_cut_on_a_character_boundary() {
        let text = "é".repeat(MAX_CHARS + 10);
        let cut = limited(&text);
        assert_eq!(cut.chars().count(), MAX_CHARS);
        assert_eq!(limited("short"), "short");
    }

    #[test]
    fn a_reading_that_fails_to_start_reports_why_once() {
        let voice = Voice::default();
        voice.read(|| Err("no text layer".into()));
        for _ in 0..200 {
            if let Some(problem) = voice.take_problem() {
                assert_eq!(problem, "no text layer");
                assert!(voice.take_problem().is_none(), "reported once");
                return;
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        panic!("the problem was never reported");
    }

    #[test]
    fn stopping_overtakes_a_reading_that_is_still_starting() {
        let voice = Voice::default();
        voice.read(|| {
            std::thread::sleep(std::time::Duration::from_millis(150));
            Err("late".into())
        });
        voice.stop();
        std::thread::sleep(std::time::Duration::from_millis(400));
        assert!(voice.take_problem().is_none(), "an overtaken start says nothing");
    }

    #[test]
    fn pausing_with_nothing_being_read_does_nothing() {
        assert_eq!(Voice::default().toggle_pause(), None);
    }

    /// Needs Windows' speech voice and takes about twenty seconds, so it runs only when asked for
    /// (`cargo test -p pdfcraft-ui-egui --lib speech -- --ignored`). The voice is muted.
    #[cfg(windows)]
    #[test]
    #[ignore = "needs Windows' speech voice and takes about 20 seconds"]
    fn a_paused_voice_waits_and_a_resumed_one_finishes() {
        use std::time::{Duration, Instant};
        let control = control_file();
        let words = "One two three four five six seven eight nine ten eleven twelve thirteen fourteen fifteen.";
        let mut child = speak(words, &control, Some(0)).expect("the voice starts");
        std::thread::sleep(Duration::from_secs(1));
        std::fs::write(&control, "pause").unwrap();
        // Paused, it must outlast the time the sentence takes to say.
        std::thread::sleep(Duration::from_secs(9));
        assert!(matches!(child.try_wait(), Ok(None)), "a paused voice is still waiting");
        std::fs::write(&control, "resume").unwrap();
        let started = Instant::now();
        while matches!(child.try_wait(), Ok(None)) && started.elapsed() < Duration::from_secs(20) {
            std::thread::sleep(Duration::from_millis(100));
        }
        assert!(matches!(child.try_wait(), Ok(Some(status)) if status.success()), "resumed, it finishes by itself");
        let _ = std::fs::remove_file(&control);
    }
}
