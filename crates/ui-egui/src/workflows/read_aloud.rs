//! View ▸ Read this page aloud, Read to the end of the document aloud and Stop reading aloud.

use pdfcraft_engine::export::Exporter;

use crate::PdfCraftApp;

impl PdfCraftApp {
    /// Read the current page, or everything from it to the end, aloud.
    pub(crate) fn read_aloud(&mut self, to_end: bool) {
        let Some((index, id)) = self.active_ids() else { return };
        if !self.may_copy(id) {
            return;
        }
        let Some(doc) = self.session.get(id) else { return };
        let src = doc.export_source();
        let first = self.views[index].current.min(src.pages.saturating_sub(1));
        let pages: Vec<usize> = if to_end { (first..src.pages).collect() } else { vec![first] };
        self.speech.read(move || {
            // Pages are separated by form feeds; a pause between them reads better as a blank line.
            let text = Exporter::from_source(src).text_of(&pages)?.replace('\u{c}', "\n\n");
            if text.trim().is_empty() { Err("There is no text to read on these pages. Recognize text (OCR) to add some.".into()) } else { Ok(text) }
        });
        self.notify(tl!("Reading aloud…").to_string());
    }

    /// Pause reading aloud, or carry on where it paused.
    pub(crate) fn pause_reading(&mut self) {
        let message = match self.speech.toggle_pause() {
            Some(true) => tl!("Reading paused"),
            Some(false) => tl!("Reading again"),
            None => tl!("Nothing is being read aloud"),
        };
        self.notify(message.to_string());
    }

    /// Stop reading aloud.
    pub(crate) fn stop_reading(&mut self) {
        self.speech.stop();
    }

    /// Show why reading could not start.
    pub(crate) fn poll_speech(&mut self) {
        if let Some(problem) = self.speech.take_problem() {
            self.notify(problem);
        }
    }
}
