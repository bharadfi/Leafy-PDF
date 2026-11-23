# Leafy PDF: what changed from PdfCraft 0.22.0

Leafy PDF is built from the PdfCraft 0.22.0 source. This file records what was changed, what was
removed and why, so the removed code does not need to be kept around. Internal crate names stay
`pdfcraft-*` (the project is the reference); only what a person or the operating system sees changed.

## Window

- **One bar instead of two (Windows).** The system title bar is gone. The tab strip is the title
  bar: Home, tabs, Open, the theme/help/settings buttons and flat Windows caption buttons (minimize,
  maximize, red close on hover) share one row, as in Acrobat. Dragging the empty strip moves the
  window, double-click maximises, thin zones on the edges resize. It reuses the app-drawn title bar
  PdfCraft already had for GNOME/Wayland (`apps/pdfcraft/src/decorations.rs`,
  `crates/ui-egui/src/platform/window_controls.rs`, `chrome.rs`). Preferences ▸ Use system title bar
  and `PDFCRAFT_TITLEBAR=system` still give the system frame back. macOS is unchanged.

## Removed

| What | Where it was |
|---|---|
| Discord button | tab strip (`chrome.rs`), `help.discord` command |
| "Join the ArtCraft community" card with its web page, GitHub and ArtCraft website buttons | Home (`home.rs`), `community_card_closed` state |
| Help entries: PdfCraft web page, PdfCraft on GitHub, ArtCraft website, Join the ArtCraft Discord | `menus.rs`, `commands.rs` (engine and UI) |
| The links table (`DISCORD`, `WEBSITE`, `APP_PAGE`, `GITHUB`, `LINKS`) | `crates/engine/src/links.rs` (the document-link safety checks in that file stay) |
| ArtCraft wordmark, ArtCraft mark and the community buttons in About | `widgets.rs`, `dialogs.rs` |
| Help ▸ Check for updates, the start-up update check and the update dialog | `platform/updates.rs`, the `craft-update` dependency, the "check at start" preference and the `--finish-update` hand-off; it would have offered PdfCraft releases |
| Discord and ArtCraft lines in the CLI banner and the MCP server info | `apps/pdfcraft-cli`, `crates/automation/src/mcp.rs` |
| Discord, About and Help links in the installer's Add/Remove Programs entry | `packaging/windows/pdfcraft.wxs` |
| PrintCraft settings migration (a new app has no such folders) | `apps/pdfcraft/src/main.rs` |
| The launcher-only branch of the install-policy checker | `packaging/windows/check-install-policy.ps1` |
| The `view.check-for-updates` parity entry | `parity/acrobat-features.toml` |
| Tests for the removed features | `updates.rs`, two update-prompt tests in `comments.rs`; the layout tests now measure against the shortcuts button instead of Discord |
| Linux (AppImage, deb, rpm, Flatpak), macOS (app bundle, DMG, notarization) and FreeBSD packaging | `packaging/linux/`, `packaging/macos/`, `packaging/freebsd/` (FreeBSD's script was built on the Linux folder) |
| Tests that read those files | the desktop-entry and flatpak tests in `apps/pdfcraft`, `flatpak_updates.rs`, the Info.plist check in `i18n/tests.rs` |
| The two macOS DMG background assets | `ATTRIBUTION.toml` / `ATTRIBUTION.md` |
| The red PDF file icon and its licence file | `assets/app-icon/pdfcraft-document.{svg,ico}`, `LICENSE-document.txt` (also dropped from the installer's files, policy and `NOTICE`); PDF files now use the Leafy PDF tile |
| Upstream's multi-platform release guide | `docs/releasing.md`, replaced by a short guide to the Windows installer build |

## Renamed to Leafy PDF

- **Visible text** in string literals: window title, About, Home, dialogs, menus, toasts, Windows
  file properties. Since 0.22.0 the translations live outside the repository (craft-i18n), so a renamed
  string shows in English in the other languages until it is translated there.
- **Program:** `leafypdf.exe` (Cargo `[[bin]] name = "leafypdf"`). The command-line tool keeps the name
  `pdfcraft-cli`.
- **Per-user data, a fresh start sharing nothing with PdfCraft:** `LeafyPdf` (settings, recovery,
  clipboard cache, single-instance socket) and `LeafyPdfData` with `LeafyPdf.portable` for portable mode.
- **Installer identity:** new UpgradeCode and ProductCode, new component GUIDs, `LeafyPdf.Document`
  ProgID, `Software\LeafyPdf` registry key, `Leafy PDF` Start Menu and desktop shortcuts, a flat
  `Leafy PDF` install folder. It installs beside PdfCraft instead of upgrading it.

## Added

- **Install folder:** upstream's installer has had its own folder chooser since 0.18 (an "Install to" box and Browse on the welcome page), so Leafy PDF uses that one.
- **Explorer thumbnails and the file icon** (crates/thumbs, pps/pdfcraft/src/thumbnail.rs): a small thumbnail-handler library, installed beside the app and registered on the LeafyPdf.Document ProgID, asks leafypdf.exe --thumbnail for a picture of page 1. A file that needs a password gets the Leafy PDF tile instead of a page. PDF files use the Leafy PDF tile as their icon instead of the red PDF icon. The new crate is the one place in the workspace that uses `unsafe` (COM), so it does not inherit the workspace lint.
- **Signatures, on top of upstream's status layers.** Since 0.22.0 upstream redraws the signer's named
  validity layers itself (`crates/sign/src/status_layers.rs`). Leafy PDF adds to it: a round, translucent
  badge behind the icon so the signer's text stays readable; *Document certified by NAME <email>.* in place of
  the baked caption for a verifying certification signature (`SignatureInfo::verified_caption`); Remove from
  trusted certificates in the signature panel and the Certificate Viewer; the email read from the
  certificate's subject alternative name (`x509::san_emails`).
- **Read Out Loud, Scanner, Preferences (autosave interval, Windows roots)**, as in the README.

## Deliberately unchanged

- The MIT OR Apache-2.0 licences, `NOTICE`, `ATTRIBUTION.*` and the "Copyright (c) the PdfCraft
  contributors" notice in the file properties and About credits. Leafy PDF is derived work and keeps them.
- Text written into PDFs (signature appearance wording, producer strings).
- The application code for other operating systems (windowing, file associations, keychain access) is
  unchanged and not tested here; this project builds and ships a Windows x64 installer only.

## Logo, fonts, size and signing

- **Logo:** `assets/brand/` holds the logo, the simplified small icon and the outline mark, extracted from the
  owner's SVG (see `assets/brand/README.md`). `assets/brand/make-icons.ps1` renders `assets/app-icon/leafypdf.ico`
  (16-256 px; the simplified icon up to 48 px) and the window-icon PNGs. The logo also appears on Home and in About.
- **Embedded fonts:** the build must include the craft-fonts set (Japanese, Chinese, Korean, Arabic, Hebrew, Thai,
  Devanagari, Georgian and symbol fonts, ~44 MB). Without it those scripts fall back to system fonts and cannot be
  edited reliably. The official release pins `storytold/craft-fonts` at commit
  `b076417b5880bd4154bd22f00695b1d28c6bea83`; build with `CRAFT_FONTS_DIR=<checkout>` and `CRAFT_FONTS_REQUIRED=1`
  so a missing set fails the build instead of passing silently.
- **Smaller install:** `pdfcraft-cli.exe` is no longer shipped. It is a separate scripting tool that carried its own
  copy of every font (~85 MB). Installed size is 128.7 MB, down from 217.3 MB; the installer is 52.9 MB, down from
  87.8 MB. Build it with `cargo build --release -p pdfcraft-cli` when needed.
- **Signing:** the program and the installer are signed with a self-signed certificate (`WINDOWS_CERTIFICATE`,
  `WINDOWS_CERTIFICATE_PASSWORD`, and `WINDOWS_SELF_SIGNED=<thumbprint>` so `sign.ps1` checks the signature is intact
  and made by that certificate instead of requiring a root Windows trusts). Windows still shows "Unknown publisher"
  until the public certificate is trusted on that PC.

## Open items

- The ArtCraft trademarked logos in `docs/brand/` (and the README header and notices that show them) are still in
  the repository, although the app no longer shows them. The project's asset policy says forks must remove them.
- Other-platform packaging is not updated.

## Building

```powershell
dotnet tool install --global wix --version 5.0.2     # once; needs the .NET 8 SDK and the Windows SDK
git clone https://github.com/storytold/craft-fonts ..\craft-fonts   # then check out the commit above
$env:CRAFT_FONTS_DIR = '..\craft-fonts'; $env:CRAFT_FONTS_REQUIRED = '1'
pwsh packaging/windows/package.ps1 -Arch x64         # leafypdf-<version>-windows-x64.msi in dist\release
```
