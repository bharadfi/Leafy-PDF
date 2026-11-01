use super::types::{Modification, Status, TrustStore};
use super::validate::{is_doc_timestamp, validate_doc_timestamp, validate_into};
use super::{Arc, Dict, DigestAlg, Document, HashMap, HashSet, ObjRef, Object, SignatureInfo, Stream};

/// Digests of signed byte ranges (and parsed signed revisions), kept across revalidations of
/// one document: incremental edits only append to the file, so the signed bytes and their
/// digest don't change, and rehashing a large file on every edit was the expensive part of
/// validation (120 MB: 84 ms per comment edit before, 6 ms after). Keyed by the ranges, the
/// algorithm and a sampled fingerprint of the covered bytes; use one cache per document, whose
/// bytes only ever grow.
#[derive(Debug, Default)]
pub struct DigestCache {
    map: std::sync::Mutex<HashMap<RangeKey, Vec<u8>>>,
    /// Signed revisions opened for change classification, by length and fingerprint.
    revisions: std::sync::Mutex<HashMap<(usize, [u8; 32]), Document>>,
    /// Candidate identities only: validation still uses current bytes and current trust.
    pub(crate) discovery: std::sync::Mutex<Option<TimestampCandidates>>,
    #[cfg(test)]
    pub(crate) discovery_scans: std::sync::atomic::AtomicUsize,
}

/// Keep only the most recently visited source/overlay. In particular, this must not retain
/// a Document, its bytes, or parsed signature dictionaries across edits and saves.
#[derive(Debug)]
pub(crate) struct TimestampCandidates {
    source: pdfcraft_cos::SourceIdentity,
    modified: Vec<u32>,
    pub(crate) candidates: Vec<ObjRef>,
}

pub(crate) const DISCOVERY_CACHE_LIMIT: usize = 4096;

pub(crate) fn is_document_timestamp(object: &Object) -> bool {
    object.as_dict().is_some_and(|d| is_doc_timestamp(d) && d.contains(b"ByteRange"))
}

/// Only dictionary/null edits have a local effect on candidate membership. Changes to an
/// object stream or a scalar used as an indirect /Length can affect unedited objects too.
/// Checking both the original and current objects also covers freeing those dependencies.
/// Unsupported or unreadable edits deliberately fall back to the complete bounded scan.
fn changed_timestamp_candidates(doc: &Document, numbers: &[u32]) -> Option<Vec<ObjRef>> {
    for (reference, object) in doc.scan_original_objects_subset(numbers.to_vec()) {
        match object.ok()?.as_ref() {
            Object::Dict(_) => {}
            Object::Null if matches!(doc.xref_entry(reference.num), None | Some(pdfcraft_cos::XrefEntry::Free { .. })) => {}
            _ => return None,
        }
    }
    let mut candidates = Vec::new();
    for (reference, object) in doc.scan_objects_subset(numbers.to_vec()) {
        let object = object.ok()?;
        if !matches!(object.as_ref(), Object::Dict(_) | Object::Null) {
            return None;
        }
        if is_document_timestamp(&object) {
            candidates.push(reference);
        }
    }
    Some(candidates)
}

/// (end of first range, start of second, end of second, algorithm, fingerprint).
type RangeKey = (usize, usize, usize, DigestAlg, [u8; 32]);

/// 64 KiB sampled from across `covered`, plus its length.
fn fingerprint(covered: &[u8]) -> [u8; 32] {
    let mut parts: Vec<&[u8]> = (0..64)
        .map(|i| {
            let at = covered.len().saturating_sub(1024) * i / 63;
            &covered[at..(at + 1024).min(covered.len())]
        })
        .collect();
    let len = (covered.len() as u64).to_be_bytes();
    parts.push(&len);
    DigestAlg::Sha256.digest(&parts).try_into().unwrap_or([0; 32])
}

impl DigestCache {
    pub(crate) fn timestamp_candidates(&self, doc: &Document) -> Vec<ObjRef> {
        let mut slot = self.discovery.lock().ok();
        let previous = slot.as_mut().and_then(|slot| slot.take());
        let source = doc.source_identity();
        let modified = doc.modified_objects();
        // Reconstructed/unknown input and output-security changes take the original path.
        let eligible = slot.is_some()
            && doc.repair_log().is_empty()
            && !doc.revisions().is_empty()
            && !doc.encryption_changed()
            && modified.len() <= DISCOVERY_CACHE_LIMIT;
        if eligible && let Some(previous) = previous.filter(|previous| previous.source == source) {
            // Undo/redo can remove overlays as well as add them. Revisit the union so an
            // original candidate replaced or freed in the last snapshot can reappear.
            let mut changed = previous.modified;
            changed.extend_from_slice(&modified);
            changed.sort_unstable();
            changed.dedup();
            if changed.len() <= DISCOVERY_CACHE_LIMIT
                && let Some(mut candidates) = changed_timestamp_candidates(doc, &changed)
            {
                candidates.extend(previous.candidates.into_iter().filter(|r| changed.binary_search(&r.num).is_err()));
                candidates.sort_unstable_by_key(|r| r.num);
                if candidates.len() <= DISCOVERY_CACHE_LIMIT
                    && let Some(slot) = slot.as_mut()
                {
                    **slot = Some(TimestampCandidates { source, modified, candidates: candidates.clone() });
                }
                return candidates;
            }
        }
        #[cfg(test)]
        self.discovery_scans.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let mut complete = true;
        let candidates: Vec<_> = doc
            .scan_objects_checked()
            .filter_map(|(reference, object)| match object {
                Ok(object) => is_document_timestamp(&object).then_some(reference),
                Err(_) => {
                    complete = false;
                    None
                }
            })
            .collect();
        if eligible
            && complete
            && candidates.len() <= DISCOVERY_CACHE_LIMIT
            && changed_timestamp_candidates(doc, &modified).is_some()
            && let Some(slot) = slot.as_mut()
        {
            **slot = Some(TimestampCandidates { source, modified, candidates: candidates.clone() });
        }
        candidates
    }

    /// The signed revision `bytes[..end]`, parsed (and kept). An encrypted one is opened with
    /// `doc`'s own key (the same `/Encrypt` and `/ID`), so a user password the document was
    /// opened with is neither needed again nor kept here (#1808).
    pub(super) fn revision(&self, doc: &Document, bytes: &[u8], end: usize) -> Option<Document> {
        let key = (end, fingerprint(&bytes[..end]));
        if let Some(d) = self.revisions.lock().ok().and_then(|m| m.get(&key).cloned()) {
            return Some(d);
        }
        let d = doc.open_with_same_credentials(Arc::new(bytes[..end].to_vec())).ok()?;
        if let Ok(mut m) = self.revisions.lock() {
            if m.len() > 8 {
                m.clear();
            }
            m.insert(key, d.clone());
        }
        Some(d)
    }

    pub(super) fn digest(&self, alg: DigestAlg, bytes: &[u8], l0: usize, o1: usize, end: usize) -> Vec<u8> {
        let key = (l0, o1, end, alg, fingerprint(&bytes[..end]));
        if let Some(d) = self.map.lock().ok().and_then(|m| m.get(&key).cloned()) {
            return d;
        }
        let d = alg.digest(&[&bytes[..l0], &bytes[o1..end]]);
        if let Ok(mut m) = self.map.lock() {
            if m.len() > 64 {
                m.clear();
            }
            m.insert(key, d.clone());
        }
        d
    }
}

pub(crate) fn text(doc: &Document, d: &Dict, key: &[u8]) -> Option<String> {
    let o = doc.resolve(d.get(key)?);
    let s = match &*o {
        Object::String(s) => s.to_text(),
        Object::Name(n) => String::from_utf8_lossy(n).into_owned(),
        _ => return None,
    };
    let s = s.trim_matches('\0').trim().to_string();
    (!s.is_empty()).then_some(s)
}

pub(super) struct Field {
    pub(super) name: String,
    /// The field dictionary (and its widget(s)).
    pub(super) r: Option<ObjRef>,
    pub(super) dict: Dict,
    pub(super) widgets: Vec<(Option<ObjRef>, Dict)>,
}

impl Field {
    /// The signature dictionary in /V. A field without one is unsigned, whether /V is missing,
    /// null or not a dictionary.
    pub(super) fn signature(&self, doc: &Document) -> Option<Dict> {
        self.dict.get(b"V").map(|v| doc.resolve(v)).and_then(|v| v.as_dict().cloned())
    }

    /// True when /V holds something that is neither null nor a signature dictionary: signing the
    /// field would overwrite a value we can't read as a signature.
    pub(super) fn has_foreign_value(&self, doc: &Document) -> bool {
        self.dict.get(b"V").map(|v| doc.resolve(v)).is_some_and(|v| !matches!(*v, Object::Null) && v.as_dict().is_none())
    }
}

/// Every signature field, depth first in `/Fields` order.
pub(super) fn sig_fields(doc: &Document) -> Vec<Field> {
    let mut out = Vec::new();
    let Some(root) = doc.root() else { return out };
    let Some(af) = doc.get(root).as_dict().and_then(|c| c.get(b"AcroForm").map(|a| doc.resolve(a))).and_then(|a| a.as_dict().cloned()) else {
        return out;
    };
    let Some(fields) = af.get(b"Fields").map(|f| doc.resolve(f)).and_then(|f| f.as_array().cloned()) else { return out };
    let mut seen = HashSet::new();
    let mut stack: Vec<(Object, String, Option<Vec<u8>>)> = fields.into_iter().rev().map(|f| (f, String::new(), None)).collect();
    while let Some((o, prefix, inherited_ft)) = stack.pop() {
        if let Some(r) = o.as_ref()
            && !seen.insert(r)
        {
            continue;
        }
        let Some(d) = doc.resolve(&o).as_dict().cloned() else { continue };
        let part = text(doc, &d, b"T");
        let name = match (&part, prefix.is_empty()) {
            (Some(p), true) => p.clone(),
            (Some(p), false) => format!("{prefix}.{p}"),
            (None, _) => prefix.clone(),
        };
        let ft = d.name(b"FT").map(<[u8]>::to_vec).or(inherited_ft);
        let kids: Vec<Object> = d.get(b"Kids").map(|k| doc.resolve(k)).and_then(|k| k.as_array().cloned()).unwrap_or_default();
        // Kids with a /T are fields; others are this field's widgets.
        let (sub, widgets): (Vec<Object>, Vec<Object>) = kids.into_iter().partition(|k| doc.resolve(k).as_dict().is_some_and(|kd| kd.contains(b"T")));
        for k in sub.into_iter().rev() {
            stack.push((k, name.clone(), ft.clone()));
        }
        if ft.as_deref() == Some(b"Sig") && part.is_some() {
            let mut ws: Vec<(Option<ObjRef>, Dict)> =
                widgets.iter().filter_map(|w| doc.resolve(w).as_dict().cloned().map(|wd| (w.as_ref(), wd))).collect();
            if d.name(b"Subtype") == Some(b"Widget") {
                ws.insert(0, (o.as_ref(), d.clone()));
            }
            out.push(Field { name, r: o.as_ref(), dict: d, widgets: ws });
        }
    }
    out
}

/// How deep a signature appearance's form XObjects are followed looking for validity layers.
const VALIDITY_LAYER_DEPTH: usize = 6;
/// The most of a layer's decoded content read to find its marker comment.
const VALIDITY_LAYER_PROBE: usize = 64 * 1024;

/// The verdict a validity layer stands for, from the marker comment its content opens with.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Mark {
    Unknown,
    Valid,
    Invalid,
    /// The signer's baked-in "Signature Not Verified" caption (Adobe's `n4`): a reader shows its
    /// own caption instead once the signature verifies.
    Caption,
}

/// The wording a signer bakes into the caption layer; Acrobat replaces it with the verdict's own.
const NOT_VERIFIED_CAPTION: &[u8] = b"(Signature Not Verified)";

/// The mark `stream` is a placeholder for, when it is one of Adobe's validity layers: a form
/// XObject whose content opens with a `% DSUnknown`, `% DSValid` or `% DSInvalid` comment.
/// Signing software bakes these in beside the signer's text (`n1`, drawn under it); a reader
/// replaces them with the verdict it reached. (`% DSBlank` layers are empty and need nothing.)
fn layer_mark(stream: &Stream) -> Option<Mark> {
    if stream.dict.name(b"Subtype") != Some(b"Form") {
        return None;
    }
    let content = stream.decoded_within(VALIDITY_LAYER_PROBE).ok()?;
    if content.windows(NOT_VERIFIED_CAPTION.len()).any(|w| w == NOT_VERIFIED_CAPTION) {
        return Some(Mark::Caption);
    }
    let rest = content.trim_ascii_start().strip_prefix(b"% DS")?;
    let word: Vec<u8> = rest.iter().copied().take_while(u8::is_ascii_alphabetic).collect();
    match word.as_slice() {
        b"Unknown" => Some(Mark::Unknown),
        b"Valid" => Some(Mark::Valid),
        b"Invalid" => Some(Mark::Invalid),
        _ => None,
    }
}

/// The validity layers reachable from `appearance` (a signature widget's `/AP /N`), by reference.
fn collect_validity_layers(doc: &Document, appearance: &Object, depth: usize, seen: &mut HashSet<ObjRef>, out: &mut Vec<(ObjRef, Mark)>) {
    let resolved = doc.resolve(appearance);
    let Object::Stream(stream) = &*resolved else { return };
    if let Some(r) = appearance.as_ref() {
        if !seen.insert(r) {
            return;
        }
        if let Some(mark) = layer_mark(stream) {
            out.push((r, mark));
            return;
        }
    }
    if depth >= VALIDITY_LAYER_DEPTH {
        return;
    }
    let xobjects =
        stream.dict.get(b"Resources").map(|r| doc.resolve(r)).and_then(|r| r.as_dict().and_then(|d| d.get(b"XObject").map(|x| doc.resolve(x))));
    let Some(xobjects) = xobjects.as_ref().and_then(|x| x.as_dict().cloned()) else { return };
    for (_, child) in xobjects.iter() {
        collect_validity_layers(doc, child, depth + 1, seen, out);
    }
}

/// The validity layers of one signature widget's normal appearance.
fn widget_validity_layers(doc: &Document, widget: &Dict) -> Vec<(ObjRef, Mark)> {
    let mut layers = Vec::new();
    if let Some(normal) = widget.get(b"AP").map(|ap| doc.resolve(ap)).and_then(|ap| ap.as_dict().and_then(|d| d.get(b"N").cloned())) {
        collect_validity_layers(doc, &normal, 0, &mut HashSet::new(), &mut layers);
    }
    layers
}

/// The content of a validity layer drawing `status`, in the layer's own `bbox` (Adobe lays these
/// out on a 100 x 100 square): a translucent round badge holding a tick, a cross or a question
/// mark, so the signer's text stays readable through it. The first line
/// is the marker comment, so the layer reads as the verdict it now shows.
fn mark_content(status: Status, bbox: [f64; 4]) -> Vec<u8> {
    let (sx, sy) = ((bbox[2] - bbox[0]).abs() / 100.0, (bbox[3] - bbox[1]).abs() / 100.0);
    let place = format!("q {sx} 0 0 {sy} {} {} cm 1 J 1 j", bbox[0].min(bbox[2]), bbox[1].min(bbox[3]));
    // A round badge: a pale tint of the verdict colour (so the signer's text reads through it) and
    // the glyph in the same colour, both translucent.
    let (marker, rgb, glyph) = match status {
        Status::Valid => ("% DSValid", "0.0 0.5 0.18", "9 w 30 50 m 45 34 l 72 68 l S"),
        Status::Invalid => ("% DSInvalid", "0.75 0.1 0.1", "9 w 33 33 m 67 67 l S 33 67 m 67 33 l S"),
        Status::Unknown => ("% DSUnknown", "0.75 0.38 0.0", "BT /DSF 62 Tf 31 30 Td (?) Tj ET"),
    };
    let lines: Vec<String> = vec![
        marker.into(),
        place,
        format!("/DSA gs {rgb} rg 94 50 m 94 74.3 74.3 94 50 94 c 25.7 94 6 74.3 6 50 c 6 25.7 25.7 6 50 6 c 74.3 6 94 25.7 94 50 c f"),
        format!("/DSB gs {rgb} RG {rgb} rg {glyph}"),
        "Q".into(),
    ];
    // One operator group per line, ending in a newline, so the marker comment stays on its own line.
    lines.into_iter().map(|l| l + "\n").collect::<String>().into_bytes()
}

/// `content` (a caption layer: one `Tf`, one `Tm`, `(Signature Not Verified) Tj`) with `text` in
/// place of the signer's words. A caption with an address after its name (`... <a@b>`) goes on two
/// lines, and the font shrinks so the longest line fits `bbox`'s width.
fn caption_content(content: &[u8], text: &str, bbox: Option<&[f64]>) -> Option<Vec<u8>> {
    content.windows(NOT_VERIFIED_CAPTION.len()).any(|w| w == NOT_VERIFIED_CAPTION).then_some(())?;
    let ops = String::from_utf8_lossy(content).into_owned();
    let tf = ops.find(" Tf")?;
    let size: f64 = ops[..tf].rsplit(|c: char| c.is_whitespace()).find(|t| !t.is_empty())?.parse().ok()?;
    let font = ops[..tf].rsplit('/').next().and_then(|f| f.split_whitespace().next())?.to_string();
    let (x0, y0, x1, y1) = match bbox {
        Some([a, b, c, d]) => (a.min(*c), b.min(*d), a.max(*c), b.max(*d)),
        _ => return None,
    };
    let lines: Vec<String> = match text.split_once(" <") {
        Some((name, address)) => vec![name.to_string(), format!("<{address}")],
        None => vec![text.to_string()],
    };
    let longest = lines.iter().map(|l| l.chars().count()).max().unwrap_or(1) as f64;
    // Helvetica averages about 0.6 em per character in capitals and lower case mixed.
    let fitted = ((x1 - x0 - 4.0) / (longest * 0.6)).min(size).max(3.0);
    let escape = |l: &str| -> String {
        l.chars()
            .map(|c| match c {
                '(' | ')' | '\\' => format!("\\{c}"),
                c if (' '..='~').contains(&c) => c.to_string(),
                _ => "?".to_string(),
            })
            .collect()
    };
    let mut out = String::from("BT\n");
    for (i, line) in lines.iter().enumerate() {
        let y = y1 - fitted * (i as f64 + 1.0);
        out += &format!("/{font} {fitted:.2} Tf 1 0 0 1 2 {:.2} Tm ({}) Tj\n", y.max(y0), escape(line));
    }
    out += "ET\n";
    Some(out.into_bytes())
}

/// The resources [`mark_content`] needs: only the question mark draws text.
fn mark_resources(status: Status) -> Dict {
    let mut resources = Dict::new();
    let mut states = Dict::new();
    for (name, alpha) in [(&b"DSA"[..], 0.28), (&b"DSB"[..], 1.0)] {
        let mut gs = Dict::new();
        gs.set(b"Type", Object::name("ExtGState"));
        gs.set(b"ca", Object::Real(alpha));
        gs.set(b"CA", Object::Real(alpha));
        states.set(name, Object::Dict(gs));
    }
    resources.set(b"ExtGState", Object::Dict(states));
    if status == Status::Unknown {
        let mut font = Dict::new();
        font.set(b"Type", Object::name("Font"));
        font.set(b"Subtype", Object::name("Type1"));
        font.set(b"BaseFont", Object::name("Helvetica-Bold"));
        let mut fonts = Dict::new();
        fonts.set(b"DSF", Object::Dict(font));
        resources.set(b"Font", Object::Dict(fonts));
    }
    resources
}

/// `doc` with each signature appearance's validity layer showing the verdict `verdict` gives for
/// its field, or `None` when nothing needs to change (no such layer, or it already shows that
/// verdict). Acrobat does this in memory: the layer the signer baked in as a placeholder is
/// replaced by the mark for the reader's own verdict and drawn where it was, under the signer's
/// text. For the copy a reader *displays or prints*; the signed file is never changed, since a
/// signature covers its own bytes.
///
/// `caption` gives, for a field whose signature verifies, the line Acrobat writes in place of the
/// signer's "Signature Not Verified" caption (e.g. "Document certified by NAME <email>."); a field it
/// gives none for keeps the signer's caption, which is then true.
pub fn with_validity_marks(
    doc: &Document,
    verdict: &dyn Fn(&str) -> Option<Status>,
    caption: &dyn Fn(&str) -> Option<String>,
) -> Option<Document> {
    enum Change {
        Mark(Status),
        Caption(String),
    }
    let mut changes: Vec<(ObjRef, Change)> = Vec::new();
    for field in sig_fields(doc) {
        let Some(status) = verdict(&field.name) else { continue };
        for (_, widget) in &field.widgets {
            for (r, mark) in widget_validity_layers(doc, widget) {
                if changes.iter().any(|(c, _)| *c == r) {
                    continue;
                }
                // Every validity layer is redrawn, even one that already says the same: the signer's
                // own artwork would otherwise show a different style from the verdict's.
                if mark != Mark::Caption {
                    changes.push((r, Change::Mark(status)));
                } else if let Some(text) = caption(&field.name) {
                    changes.push((r, Change::Caption(text)));
                }
            }
        }
    }
    if changes.is_empty() {
        return None;
    }
    let mut copy = doc.clone();
    for (r, change) in changes {
        let Object::Stream(s) = &*doc.get(r) else { continue };
        let status = match change {
            Change::Caption(text) => {
                // The signer's own font, position and size stay; only the words change, wrapped
                // and shrunk to the layer's width.
                let Ok(content) = s.decoded() else { continue };
                if let Some(replaced) = caption_content(&content, &text, nums(doc, &s.dict, b"BBox").as_deref()) {
                    copy.set(r, Object::Stream(Stream::flate(s.dict.clone(), &replaced)));
                }
                continue;
            }
            Change::Mark(status) => status,
        };
        // The dictionary stays (`/BBox`, `/Matrix`), so the layer is placed exactly as it was.
        let bbox = match nums(doc, &s.dict, b"BBox").as_deref() {
            Some([a, b, c, d]) => [*a, *b, *c, *d],
            _ => [0.0, 0.0, 100.0, 100.0],
        };
        let mut dict = s.dict.clone();
        dict.set(b"Resources", Object::Dict(mark_resources(status)));
        copy.set(r, Object::Stream(Stream::flate(dict, &mark_content(status, bbox))));
    }
    Some(copy)
}

/// Which page holds each annotation.
fn annot_pages(doc: &Document) -> HashMap<ObjRef, usize> {
    let mut map = HashMap::new();
    for (i, p) in pdfcraft_annot::page_refs(doc).unwrap_or_default().into_iter().enumerate() {
        if let Some(a) = doc.get(p).as_dict().and_then(|d| d.get(b"Annots").map(|a| doc.resolve(a))).and_then(|a| a.as_array().cloned()) {
            for r in a.iter().filter_map(Object::as_ref) {
                map.insert(r, i);
            }
        }
    }
    map
}

pub(super) fn nums(doc: &Document, d: &Dict, key: &[u8]) -> Option<Vec<f64>> {
    doc.resolve(d.get(key)?).as_array().map(|a| a.iter().filter_map(|x| doc.resolve(x).as_f64()).collect())
}

/// List and validate every signature field. `bytes` is the file as stored (the ranges index it).
pub fn list(doc: &Document, bytes: &[u8], trust: &TrustStore) -> Vec<SignatureInfo> {
    list_cached(doc, bytes, trust, &DigestCache::default())
}

/// [`list`], reusing digests and standalone-timestamp discovery from `cache`. Field/widget
/// structure and validation against `bytes` and `trust` are read again on every call.
pub fn list_cached(doc: &Document, bytes: &[u8], trust: &TrustStore, cache: &DigestCache) -> Vec<SignatureInfo> {
    let pages = annot_pages(doc);
    let mut out = Vec::new();
    let mut field_values: HashSet<ObjRef> = HashSet::new();
    for f in sig_fields(doc) {
        if let Some(v_ref) = f.dict.get(b"V").and_then(Object::as_ref) {
            field_values.insert(v_ref);
        }
        let widget = f.widgets.first();
        let page = widget.and_then(|(r, w)| r.and_then(|r| pages.get(&r).copied()).or_else(|| w.reference(b"P").and_then(|p| page_index(doc, p))));
        let rect = widget
            .and_then(|(_, w)| nums(doc, w, b"Rect"))
            .filter(|r| r.len() == 4)
            .map(|r| [r[0].min(r[2]), r[1].min(r[3]), r[0].max(r[2]), r[1].max(r[3])]);
        // Shown when it has a box, and its widget isn't Hidden (2) or NoView (32), as the forms
        // crate reads a widget's flags.
        let flags = widget.and_then(|(_, w)| w.get(b"F")).and_then(|o| doc.resolve(o).as_int()).unwrap_or(0);
        let visible = rect.is_some_and(|r| r[2] - r[0] > 0.5 && r[3] - r[1] > 0.5) && flags & (2 | 32) == 0;
        let v = f.signature(doc);
        let mut info = SignatureInfo {
            field: f.name.clone(),
            signed: v.is_some(),
            page,
            rect,
            visible,
            validity_layer: widget.is_some_and(|(_, w)| widget_validity_layers(doc, w).iter().any(|(_, m)| *m != Mark::Caption)),
            signer: None,
            certificate: None,
            chain: Vec::new(),
            date: None,
            signing_time: None,
            reason: None,
            location: None,
            contact: None,
            sub_filter: None,
            doc_timestamp: false,
            certify: None,
            revision: 0,
            signed_len: 0,
            digest: None,
            algorithm: None,
            timestamp: false,
            timestamp_time: None,
            status: Status::Unknown,
            modification: Modification::None,
            details: Vec::new(),
        };
        if let Some(v) = v {
            validate_into(doc, bytes, trust, &v, &mut info, cache);
        }
        out.push(info);
    }
    // Standalone document timestamps (ISO 32000-2 §12.8.2.2) live outside AcroForm fields.
    let candidates = cache.timestamp_candidates(doc).into_iter().map(|r| r.num).collect();
    for (r, o) in doc.scan_objects_subset(candidates) {
        if field_values.contains(&r) {
            continue;
        }
        let Ok(o) = o else { continue };
        let Some(d) = o.as_dict() else { continue };
        if !is_document_timestamp(&o) {
            continue;
        }
        let mut info = SignatureInfo {
            field: "DocumentTimestamp".to_string(),
            signed: true,
            page: None,
            rect: None,
            visible: false,
            validity_layer: false,
            signer: None,
            certificate: None,
            chain: Vec::new(),
            date: None,
            signing_time: None,
            reason: None,
            location: None,
            contact: None,
            sub_filter: Some("ETSI.RFC3161".to_string()),
            doc_timestamp: true,
            certify: None,
            revision: 0,
            signed_len: 0,
            digest: None,
            algorithm: None,
            timestamp: true,
            timestamp_time: None,
            status: Status::Unknown,
            modification: Modification::None,
            details: Vec::new(),
        };
        validate_doc_timestamp(doc, bytes, d, &mut info, cache, trust);
        out.push(info);
    }
    out
}

fn page_index(doc: &Document, p: ObjRef) -> Option<usize> {
    pdfcraft_annot::page_refs(doc).ok()?.iter().position(|r| *r == p)
}

/// A widget's `/MK /R` as a quarter turn (0, 90, 180 or 270); missing, malformed or not a
/// multiple of 90 counts as 0, as `pdfcraft-forms` reads it for the fields it draws.
pub(super) fn mk_rotation(doc: &Document, widget: &Dict) -> i64 {
    let r = widget
        .get(b"MK")
        .map(|mk| doc.resolve(mk))
        .and_then(|mk| mk.as_dict().and_then(|mk| mk.get(b"R")).map(|r| doc.resolve(r)))
        .and_then(|r| r.as_int().or_else(|| r.as_f64().filter(|v| v.is_finite() && v.abs() <= 1.0e6).map(|v| v.round() as i64)));
    r.map(|r| r.rem_euclid(360)).filter(|q| matches!(q, 90 | 180 | 270)).unwrap_or(0)
}

#[cfg(test)]
mod caption_tests {
    use super::caption_content;

    /// The caption layer (`n4`) of the Income Tax intimation: one line, 11.72 pt, in a 120 x 16.5 box.
    const LAYER: &[u8] = b"BT\n1 0 0 1 2 41.28 Tm\n/F1 11.72 Tf\n(Signature Not Verified)Tj\nET";

    #[test]
    fn a_certified_caption_replaces_the_signers_words_on_two_lines_that_fit_the_layer() {
        let text = "Document certified by NAGAMANICKAM SAYIRAJ <NAGAMANICKAM.SAYIRAJ@INCOMETAX.GOV.IN>.";
        let out = caption_content(LAYER, text, Some(&[0.0, 38.5, 120.0, 55.0])).unwrap();
        let out = String::from_utf8(out).unwrap();
        assert!(!out.contains("Signature Not Verified"), "{out}");
        assert!(out.contains("(Document certified by NAGAMANICKAM SAYIRAJ) Tj"), "{out}");
        assert!(out.contains("(<NAGAMANICKAM.SAYIRAJ@INCOMETAX.GOV.IN>.) Tj"), "{out}");
        let size: f64 = out.split(" Tf").next().unwrap().rsplit(' ').next().unwrap().parse().unwrap();
        assert!(size < 11.72 && size >= 3.0, "shrunk to fit: {size}");
        // The text stays inside the 120-wide layer at the shrunk size (0.6 em per character).
        assert!(size * 0.6 * 43.0 <= 120.0, "{size}");
    }

    #[test]
    fn parentheses_and_non_latin_letters_cannot_break_the_content_stream() {
        let out = caption_content(LAYER, "Document certified by A) B\\ \u{0928}.", Some(&[0.0, 0.0, 120.0, 55.0])).unwrap();
        let out = String::from_utf8(out).unwrap();
        assert!(out.contains("(Document certified by A\\) B\\\\ ?.) Tj"), "{out}");
    }
}