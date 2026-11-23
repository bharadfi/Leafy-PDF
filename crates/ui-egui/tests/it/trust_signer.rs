//! The Signatures panel's one-click "Add to trusted certificates" must store the signer shown in
//! the row, not the chain top the document embedded in the CMS (audit v3-098). Signing with a
//! certificate whose CMS carries its issuer chain gives a signature whose `chain.last()` (the
//! self-signed root) differs from its `certificate` (the signer); the click must trust the signer.

use crate::support;
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use pdfcraft_engine::SignatureStatus;
use pdfcraft_ui_egui::{PdfCraftApp, SignStep};

const FIXTURE: &[u8] = b"%PDF-1.7
1 0 obj << /Type /Catalog /Pages 2 0 R >> endobj
2 0 obj << /Type /Pages /Kids [3 0 R] /Count 1 >> endobj
3 0 obj << /Type /Page /Parent 2 0 R /MediaBox [0 0 300 200] /Contents 4 0 R /Resources << /Font << /F1 5 0 R >> >> >> endobj
4 0 obj << /Length 45 >> stream
BT /F1 14 Tf 20 150 Td (Please sign below) Tj ET
endstream endobj
5 0 obj << /Type /Font /Subtype /Type1 /BaseFont /Helvetica >> endobj
trailer << /Root 1 0 R >>
%%EOF";

fn dir() -> std::path::PathBuf {
    static NEXT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let n = NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let d = std::env::temp_dir().join(format!("pdfcraft-trust-signer-{}-{n}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

fn harness(dir: std::path::PathBuf) -> Harness<'static, PdfCraftApp> {
    let mut h = Harness::builder().with_size(egui::vec2(1400.0, 900.0)).build_eframe(move |_cc| {
        let mut app = PdfCraftApp::new();
        app.set_option("language", "en").unwrap();
        app.open_bytes("contract.pdf", None, FIXTURE.to_vec()).expect("opens");
        app.set_option("left", "closed").unwrap();
        app.export_dir_override = Some(dir.to_string_lossy().into_owned());
        app.save_override = Some(dir.join("contract_signed.pdf").to_string_lossy().into_owned());
        app
    });
    h.run_steps(6);
    h
}

/// A document signed with chain.p12, whose CMS embeds the signer's issuer chain (signer + self-signed root).
fn signed_with_chain() -> Harness<'static, PdfCraftApp> {
    let mut h = harness(dir());
    let p12 = concat!(env!("CARGO_MANIFEST_DIR"), "/../sign/tests/data/chain.p12");
    let id = pdfcraft_engine::sign::pkcs12::open(&std::fs::read(p12).unwrap(), "test").unwrap();
    let entry = h.state_mut().add_digital_id(p12, &id.certificate);
    h.state_mut().start_signing(0, None, None, None);
    {
        let d = h.state_mut().sign_draft.as_mut().unwrap();
        d.selected = Some(entry);
        d.step = SignStep::SignAs;
        d.password = "test".into();
    }
    h.run_steps(2);
    support::outside_menus(&h, "Sign").click();
    h.run_steps(6);
    h
}

#[test]
fn removing_a_trusted_signer_takes_it_off_the_list_and_the_signature_goes_back_to_unknown() {
    let mut h = signed_with_chain();
    h.get_by_label_contains("Signed by").click();
    h.run_steps(2);
    h.get_by_label("Add to trusted certificates").click();
    h.run_steps(3);
    {
        let s = h.state();
        assert_eq!(s.session.trusted_certificates().len(), 1, "the signer is trusted");
        let sig = s.session.get(s.views[0].id).unwrap().signatures.iter().find(|x| x.signed).cloned().unwrap();
        assert_eq!(sig.status, SignatureStatus::Valid);
    }
    // The trust button is gone and a remove button takes its place.
    assert!(h.query_by_label("Add to trusted certificates").is_none());
    h.get_by_label("Remove from trusted certificates").click();
    h.run_steps(3);
    let s = h.state();
    assert!(s.session.trusted_certificates().is_empty(), "the signer is no longer trusted");
    let sig = s.session.get(s.views[0].id).unwrap().signatures.iter().find(|x| x.signed).cloned().unwrap();
    assert_eq!(sig.status, SignatureStatus::Unknown, "an untrusted signer shows an unknown identity again");
}

#[test]
fn one_click_trust_stores_the_displayed_signer_not_the_embedded_root() {
    // Sign with chain.p12, whose CMS embeds the signer's issuer chain (signer + self-signed root).
    let mut h = signed_with_chain();

    // The signature validates as Unknown and carries a two-certificate chain whose top (the root)
    // is a different certificate from the displayed signer.
    let (signer, root) = {
        let s = h.state();
        let sig = s.session.get(s.views[0].id).unwrap().signatures.iter().find(|x| x.signed).cloned().expect("a signed field");
        assert_eq!(sig.status, SignatureStatus::Unknown, "untrusted before the click");
        assert_eq!(sig.chain.len(), 2, "the CMS embeds the signer and its root: {:?}", sig.chain.len());
        let signer = sig.certificate.clone().expect("a signer certificate");
        let root = sig.chain.last().cloned().expect("a chain top");
        assert_ne!(signer.raw, root.raw, "the embedded chain top must differ from the signer for this test to be meaningful");
        (signer, root)
    };

    // Expand the signature row and use the one-click trust button.
    h.get_by_label_contains("Signed by").click();
    h.run_steps(2);
    h.get_by_label("Add to trusted certificates").click();
    h.run_steps(3);

    // The trust store holds exactly the displayed signer, never the document-supplied root.
    let trusted = h.state().session.trusted_certificates().to_vec();
    assert!(trusted.iter().any(|c| c.raw == signer.raw), "the displayed signer should be trusted");
    assert!(!trusted.iter().any(|c| c.raw == root.raw), "the embedded root must not be trusted by the one-click action");
}
