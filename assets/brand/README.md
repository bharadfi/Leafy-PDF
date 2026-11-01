# Leafy PDF logo

Provided by the project owner as `pdf_app_logo_alt.svg` (a teal rounded app icon with two stacked pages, a
highlighted line and a text cursor). The shapes were extracted into the clean vector files below, without the
wordmark text or the file's embedded metadata.

| File | Used for |
|---|---|
| `leafy-logo.svg` | the full logo: Home, About, and every icon larger than 48 px |
| `leafy-icon-small.svg` | the simplified icon (no text cursor, fewer lines): taskbar, title bar and Explorer sizes up to 48 px |
| `leafy-mark.svg` | the one-colour outline mark, for monochrome or very small uses (not shown in the app yet) |
| `make-icons.ps1` | renders `../app-icon/leafypdf.ico` (16–256 px), `leafypdf-256.png` and `leafypdf-1024.png` from those shapes |

The Windows executable embeds `leafypdf.ico` (`apps/pdfcraft/build.rs`); the window icon is `leafypdf-256.png`
(`apps/pdfcraft/src/main.rs`); the installer uses `leafypdf.ico` too (`packaging/windows/package.ps1`).

The PDF document icon (`assets/app-icon/pdfcraft-document.ico`, a Lucide outline) and the PdfCraft lion files in
`assets/app-icon/` are unchanged: the lion files are still used by the Linux, macOS and FreeBSD packaging.

Licence: `MIT OR Apache-2.0`, like the rest of Leafy PDF (see [LICENSE.txt](LICENSE.txt)). The logo was generated with AI
assistance at the owner's request; the licence text says so. Every file above has an entry in `ATTRIBUTION.toml`
(checked by `cargo xtask assets`).
