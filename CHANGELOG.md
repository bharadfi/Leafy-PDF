# Changelog

## 0.22.0 (2026-11-23) - based on pdfcraft v0.22.0

Leafy PDF now follows upstream pdfcraft 0.22.0 (previously 0.17.0).

### Changed
- **Signatures now build on upstream's status layers.** Upstream redraws a signer's validity layers itself; Leafy PDF keeps its extras on top: a round translucent badge behind the tick, question mark or cross so the signer's text stays readable, the *Document certified by NAME <email>.* caption for a verifying certification signature, and the signer's email read from the certificate's alternative name.
- Trust root certificates from Windows (optional, off by default) and Remove from trusted certificates are kept.
- Install folder: the installer uses upstream's own folder chooser (Leafy PDF's separate page was dropped).
- Compare: uses upstream's Pages field. The separate old/new page ranges of 0.17.0 are gone.
- Translations now come from the craft-i18n catalogs; strings renamed to "Leafy PDF" show in English in other languages until translated upstream.

### Kept from earlier Leafy PDF releases
Merged title and tab row, Leafy logo and icons, no update check, no Discord or community links, Explorer thumbnails and the Leafy file icon, Read Out Loud, scanning from a scanner, autosave interval, Windows x64 MSI only.

### Installer
`leafypdf-0.22.0-windows-x64.msi`, signed with a new self-made certificate (Windows shows "Unknown publisher"; it is a different publisher identity from earlier releases).

### Known limits
Not yet checked through a real install: Explorer thumbnails and the file icon. Remove from trusted certificates, the certified-caption wording, Read Out Loud and the scanner were not exercised in the real app on this build.
