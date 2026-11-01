//! File ▸ Create ▸ PDF from a scanner: Windows' own scan window, then the usual "create a PDF from images" step.

use crate::PdfCraftApp;
use crate::platform::scanner::Scanned;

impl PdfCraftApp {
    /// Open the scan window.
    pub(crate) fn create_from_scanner(&mut self) {
        let message = if self.scanner.start() { tl!("Opening the scan window…") } else { tl!("A scan window is already open.") };
        self.notify(message.to_string());
    }

    /// A scanned page: added to the PDF being made from images when there is one (Scan another page), or the
    /// first page of a new one.
    pub fn take_scanned_page(&mut self, png: Vec<u8>) {
        if let Some(draft) = self.image_import.as_mut() {
            let number = draft.images.len() + 1;
            draft.images.push((format!("Scan {number}.png"), png));
            self.dialog = Some(crate::Dialog::CreateImages);
            return;
        }
        self.begin_image_import(vec![("Scan 1.png".into(), png)]);
        if let Some(draft) = self.image_import.as_mut() {
            draft.from_scanner = true;
        }
    }

    /// Take a finished scan into the new-PDF-from-images step, or say why there is none.
    pub(crate) fn poll_scanner(&mut self) {
        match self.scanner.take() {
            Some(Scanned::Picture(png)) => self.take_scanned_page(png),
            Some(Scanned::Problem(problem)) => self.notify(problem),
            Some(Scanned::Cancelled) | None => {}
        }
        // The scan window belongs to another process: look again soon, without waiting for input here.
        if self.scanner.busy()
            && let Some(ctx) = &self.ctx
        {
            ctx.request_repaint_after(std::time::Duration::from_millis(300));
        }
    }
}
