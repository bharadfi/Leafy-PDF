<p align="center">
  <img src="assets/app-icon/leafypdf-256.png" width="128" height="128" alt="Leafy PDF icon">
</p>

<h1 align="center">Leafy PDF</h1>

<p align="center">
  <b>A fast, private PDF reader and editor, written in Rust.</b><br>
  Read, organize, combine, split, sign-check and secure PDFs in a native app that works entirely on your own computer.<br>
  Windows x64 installer · choose your own install folder · built-in signature validation
</p>

<p align="center">
  <img alt="License: MIT OR Apache-2.0" src="https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-2e7d32?style=flat-square">
  <img alt="Written in Rust" src="https://img.shields.io/badge/written%20in-Rust-2e7d32?style=flat-square">
  <img alt="Windows x64" src="https://img.shields.io/badge/installer-Windows%20x64-2e7d32?style=flat-square">
  <img alt="No account, no telemetry" src="https://img.shields.io/badge/no%20account-no%20telemetry-1b5e20?style=flat-square">
</p>

<p align="center">
  <a href="#highlights">Highlights</a> ·
  <a href="#what-this-build-adds">What this build adds</a> ·
  <a href="#signature-validation">Signature validation</a> ·
  <a href="#read-anything-beautifully">Read</a> ·
  <a href="#organize-pages-like-cards-on-a-table">Organize</a> ·
  <a href="#combine-and-split-without-losing-a-thing">Combine &amp; split</a> ·
  <a href="#open-protected-documents-and-respect-their-rules">Protect</a> ·
  <a href="#comments-forms-layers-and-attachments">Forms &amp; layers</a> ·
  <a href="#runs-locally-stays-yours">Private</a> ·
  <a href="#built-for-agents-too">Agents</a> ·
  <a href="#how-its-built">How it's built</a> ·
  <a href="#get-started">Get started</a> ·
  <a href="#whats-next">What's next</a> ·
  <a href="#downloads">Downloads</a>
</p>

---

## Highlights

<table>
<tr>
<td width="25%" valign="top">

### Faithful
Real-world typography: world scripts, vertical Japanese, colour emoji, gradients, soft masks and transparency render the way the author intended.

</td>
<td width="25%" valign="top">

### Fearless
Every save appends your changes and leaves the original bytes untouched. Writes are atomic, undo runs deep, and nothing is lost if you close by mistake.

</td>
<td width="25%" valign="top">

### Yours
No account, no telemetry, no cloud, no update check. It works offline and opens instantly.

</td>
<td width="25%" valign="top">

### Trustworthy
Digital signatures are checked on open and shown on the page: a green tick, an amber question mark or a red cross, on screen and in print.

</td>
</tr>
</table>

---

## What this build adds

Leafy PDF is a custom build of an open-source Rust PDF engine, shaped for everyday use on Windows. On top of a stock build of that kind of viewer, and of the custom forks made from it, it adds:

| Area | What Leafy PDF does |
|---|---|
| **Signatures** | Verdict marks drawn *behind* the signer's text, on screen and in print: green tick, amber question mark, red cross. One-click **Add to trusted certificates** and **Remove from trusted certificates**. Certified documents read *Document certified by NAME <email>.* An optional, **off-by-default** setting also trusts the root certificates Windows trusts. The signed file is never touched. |
| **Installer** | A signed Windows x64 installer that asks who should use the app and **where to install it** (any drive and folder). About 55.7 MB, with the fonts for Japanese, Chinese, Korean, Arabic, Hebrew, Thai, Devanagari and Georgian and the offline OCR models built in. No command-line tools and no portable zip, so the install stays small. |
| **Explorer** | A snapshot of page 1 as the thumbnail of each PDF, and the green Leafy PDF tile (never a page) for password-protected files. The PDF file icon is the same tile. |
| **Reading aloud** | *Read this page aloud*, *Read to the end of the document aloud*, *Pause or resume* and *Stop*, using the voice Windows already has. No speech engine is shipped. |
| **Scanning** | *Create ▸ PDF from a scanner* opens Windows' own scan window. *Scan another page* gathers several scans into one PDF. |
| **Compare** | Compare only the pages you choose in each file, in the dialog and for scripts. The report and the comments follow the same pages. |
| **Preferences** | How often unsaved changes are autosaved (1 to 15 minutes) and how many recent files are remembered. |
| **Interface** | The window title bar and the document tabs share a single row. Its own logo and icons. |
| **Quiet by design** | No account, no telemetry, no update check, no banners or promotional links, and no cloud or AI features. Its settings and trusted certificates live in their own `LeafyPdf` folder and are shared with no other program. |

**Deliberately left out:** cloud services, an AI assistant, online signing and review services, and installers for other operating systems (the installer and testing are Windows x64 only).

---

## Signature validation

Leafy PDF checks every digital signature when a document opens and shows the result on the signature itself, the way the best desktop readers do:

| Mark | Meaning |
|---|---|
| **Green tick** | The signature is intact and its signer is trusted. |
| **Amber question mark** | The signature is intact, but you haven't trusted the signer yet, or validity could not be determined. |
| **Red cross** | The signed content changed, the signature does not verify, or later changes were not permitted. |

- **Behind the text, never over it.** The mark is a soft round badge drawn in the signature's own background layer, so the signer's name, date and reason stay fully readable.
- **Prints the same way.** The same mark is in the printed page and in *Print to PDF*.
- **Trust a signer once, with a click.** Open the *Signatures* panel, expand a signature and choose **Add to trusted certificates**: its question mark turns into a green tick. **Remove from trusted certificates** takes the trust away again, and the tick goes back to a question mark. Trusted certificates are stored locally, nowhere else.
- **Windows roots, if you want them.** *Preferences ▸ Trust root certificates from Windows when validating signatures* is off by default. When it is on, a signature whose certificate chains to one of the roots Windows trusts shows as valid without adding the certificate yourself. Those roots are kept apart from your own list and are never saved into it.
- **Certified documents say so.** For a certified signature that verifies, the caption reads *Document certified by NAME <email>.* instead of "Signature Not Verified". The email is read from the signer's certificate.
- **Your file is never touched.** The verdict is drawn into the copy you view and print. The signed bytes, and the file on disk, stay exactly as they were.
- **Details when you want them.** The panel lists, for each signature, why it got its verdict: what was checked, who signed, with which algorithm, when, and whether the document changed afterwards. You can also view the signed version, show the certificate and export it.

Validation runs on your computer against the certificates you choose to trust.

---

## Read anything, beautifully

Leafy PDF renders PDFs with care for the details that make a page feel right: kerning and ligatures, right-to-left and complex scripts, vertical CJK, colour emoji, shadings, blend modes, soft masks and optional content.

- **Deep zoom stays sharp.** Large pages render in tiles, so text stays crisp at any magnification.
- **Built to survive bad files.** Every page renders in isolation and damaged documents are repaired.
- **Layouts for every task:** continuous, single page, two-up, view rotation, full screen and a distraction-free Read mode.
- **Light and dark themes**, both designed to be easy on the eyes for long sessions.
- **Find it, select it, copy it.** Search as you type and select text that comes out in the right reading order, for columns, right-to-left runs and CJK too.
- **Listen to it.** View ▸ *Read this page aloud* reads a page with the Windows voice; you can pause, resume and stop.
- **Navigate long documents.** Bookmarks, page thumbnails and the document's own page labels (i, ii, 1, 2…) keep you oriented. Right-click a tab to close it, reopen the last closed file or reveal it in Explorer.

## Organize pages like cards on a table

Open **Organize pages** to see every page at once:

- **Select pages:** click, Ctrl-click or Shift-click.
- **Change them:** rotate, delete, insert blank pages, insert pages from another file, and move them earlier or later.
- **Undo anything:** each change is one step in a history you can walk backwards and forwards; undo still works after you save.

**Saves you can trust:** *incremental* (the original bytes stay intact), *atomic* (written to a temporary copy, then swapped in) and *recoverable* (unsaved changes are autosaved, every minute unless you choose another interval; if the app ever quits unexpectedly it offers to recover your work). Unsaved documents carry a dot on their tab, and closing asks before anything is lost. Encrypted documents stay encrypted on disk.

## Combine and split without losing a thing

**Combine files** merges any number of PDFs into one, with a bookmark for each file. **Extract** copies the pages you select into a new document. **Split** divides a document every *n* pages, or before the pages you choose. Links and named destinations are rewired to the copied pages, form fields stay interactive, layers keep their on/off defaults and attachments come along.

## Open protected documents and respect their rules

Leafy PDF implements the PDF standard security handler: every revision from 40-bit RC4 to AES-256, user and owner passwords (including Unicode passwords), crypt filters and attachment-only encryption.

Documents restricted by their author show a clear notice, and Leafy PDF honours their permissions: with only the open password, changes the author has locked stay locked. Enter the owner password and the restrictions lift. Edits to encrypted documents are saved encrypted, under the same keys.

**Document Properties** shows the title, author, subject and keywords (editable), the fonts used and whether each is embedded, the PDF version, page size, tags, fields, layers and attachments, and the full security picture.

## Comments, forms, layers and attachments

- **Forms:** every field with its current value, field highlighting, and checkboxes, radio buttons, lists and signatures drawn the way their author designed them. Fill & Sign adds typed or drawn signatures, stamps and marks.
- **Layers:** switch optional content on and off and the page re-renders instantly.
- **Comments** appear as threaded conversations; **attachments** can be opened or saved.
- **Scan & OCR:** recognise text in scans with the bundled offline models, or scan straight from a scanner.
- **Compare files:** word-by-word text differences and visual differences, side by side, for whole files or chosen pages.
- **More tools on the home screen:** Export, Create, Combine, Organize, Edit, Add comments, Scan & OCR, Protect, Redact, Compress, Prepare a form, Add a stamp and Use a certificate. Press Ctrl-K to search every tool and command.

---

## Runs locally, stays yours

- **Private by design.** Documents never leave your machine. There's no account, no telemetry and no cloud processing.
- **Engine first.** Parsing, rendering, signing and editing live in reusable library crates. The interface is one swappable layer on top.
- **Scriptable.** The same engine is available without the interface (see below).

---

## Built for agents, too

Every engine feature is reachable without the GUI through one table of JSON-Schema-described tools: open, inspect, render pages to PNG, extract and find text, rotate, delete, move and insert pages, edit bookmarks and page labels, add and reply to comments, list and fill form fields, protect with passwords, compare files (optionally only some pages), set metadata, undo and redo, save, combine, extract and split.

The source also builds a command-line tool and an opt-in server that agents can use over standard input and output; the Windows installer does not include them. The MCP-style server is opt-in: nothing starts it on its own and it opens no network port, and a root folder confines every file an agent can read or write.

The running app can be driven too: start it with `leafypdf.exe --control <file>` and an agent can see and operate the real interface (widget tree, clicks, typing, keys, commands, screenshots). This is off by default, listens only on loopback, and every connection must present the random token written to that file.

The command lines, options and examples are in [docs/development.md](docs/development.md).

---

## How it's built

Leafy PDF is a Cargo workspace of focused crates, layered so the core never depends on the UI: stream filters and the standard security handler at the bottom, then the PDF object layer, signatures, page operations, comments, forms and rendering, then one engine that every front end uses (sessions, edits, undo, saving and the tool catalogue), and on top the desktop interface and the agent tools. A small separate library provides the Explorer thumbnails; it is the only place in the project that needs `unsafe` code, and it never parses a PDF itself.

Leafy PDF's behaviour comes from the ISO 32000 specification and black-box observation. Every icon, font and image is openly licensed and listed in [ATTRIBUTION.md](ATTRIBUTION.md); `cargo xtask assets` checks that list.
A full record of the changes in this build is in [docs/leafy-pdf.md](docs/leafy-pdf.md).

## Get started

Building the app needs Rust (MSVC toolchain). Building the installer also needs the .NET 8 SDK, the Windows SDK and WiX v5.

```powershell
git clone https://github.com/bharadfi/Leafy-PDF
cd Leafy-PDF
cargo xtask models        # fetch the OCR models Scan & OCR needs
```

How to run the app from source, the environment variables and the other development notes are in [docs/development.md](docs/development.md).

The program text in Japanese, Chinese, Korean, Arabic, Hebrew, Thai, Devanagari and Georgian relies on the [craft-fonts](https://github.com/storytold/craft-fonts) font set (SIL Open Font License 1.1), a build input. The installer build requires it and refuses to continue without it. Check out the commit pinned in the repository's packaging notes:

```powershell
git clone https://github.com/storytold/craft-fonts ..\craft-fonts
$env:CRAFT_FONTS_DIR = '..\craft-fonts'; $env:CRAFT_FONTS_REQUIRED = '1'
dotnet tool install --global wix --version 5.0.2
powershell -File packaging\windows\package.ps1 -Arch x64
```

The interface language is chosen in **Edit → Preferences…** (see [docs/localization.md](docs/localization.md)). The steps for building and publishing the installer are in [docs/releasing.md](docs/releasing.md).

## What's next

Leafy PDF is **alpha** software. Everyday reading, commenting, form filling, signing and page organizing work, and the signature checks above are in place. Areas that are still thin or missing:

- reliable editing of existing text (especially CJK and right-to-left);
- OCR beyond Latin script;
- office-document import and export;
- certificate-based password protection and tools for tagging documents for accessibility;
- signature tokens and online long-term validation;
- preflight and PDF/X, PDF/UA conformance;
- full XFA forms.

Not yet verified for this build: printing a validated signature on a physical printer, scanning from a real scanner, and installers for other operating systems (this build's installer and testing are Windows x64 only).

The wider roadmap is in [ROADMAP.md](ROADMAP.md); the full assessment is in [docs/target-app-parity.md](docs/target-app-parity.md), the ranked gaps in [docs/gaps.md](docs/gaps.md), and the milestones in [docs/roadmap.md](docs/roadmap.md).

---

## Downloads

Get the installer from the [latest release](https://github.com/bharadfi/Leafy-PDF/releases/latest); [all releases](https://github.com/bharadfi/Leafy-PDF/releases) lists earlier versions with their notes and checksums.

### Windows (x64)

| Build | Installer |
|---|---|
| x64 (64-bit Intel/AMD) | `leafypdf-<ver>-windows-x64.msi` |

- **Where it installs.** The installer asks who should use Leafy PDF (just you, or everyone on the computer) and then the folder: type a path or Browse to any drive.
- **Signed with a self-made certificate.** The installer and `leafypdf.exe` are signed, but the certificate is not issued by a public authority, so Windows shows *Unknown publisher* (and SmartScreen may ask for confirmation) until you choose to trust it. You can compare the file's SHA-256, shown in the release notes, with `Get-FileHash <file> -Algorithm SHA256`.
- **Shortcuts and menus.** It adds a Start Menu shortcut, an optional desktop shortcut, and optional "Edit with Leafy PDF" and "Create PDF with Leafy PDF…" entries in the right-click menu of PDF files and images in File Explorer (on Windows 11, under "Show more options"). It does not change which app opens PDFs by default. When Leafy PDF is the default, PDF files show page thumbnails (or the Leafy PDF tile if protected); Windows may need a moment, or a restart of File Explorer, to replace thumbnails it has already cached.
- **Unattended install.** Set a property to `0` to leave an item out:

```powershell
msiexec /i "leafypdf-<ver>-windows-x64.msi" /qn /norestart INSTALLDESKTOPSHORTCUT=0
```

| Property | `=0` leaves out |
|---|---|
| `INSTALLDESKTOPSHORTCUT` | the desktop shortcut |
| `INSTALLPDFCONTEXTMENU` | "Edit with Leafy PDF" on PDF files |
| `INSTALLIMAGECONTEXTMENU` | "Create PDF with Leafy PDF…" on images |

An upgrade keeps what the installed version has: a shortcut or menu entry that is there stays, and one that is missing stays missing; a value on the command line always wins.

---

## License and credits

Leafy PDF is dual-licensed under [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE), at your option.
It is built on open-source code and keeps its authors' copyright and attribution notices; the required notices are in [NOTICE](NOTICE) and in the two licence files.

Bundled fonts, icons, images and other assets keep their own open licenses; each one is listed with its author, source and license in [ATTRIBUTION.md](ATTRIBUTION.md). The Leafy PDF logo in `assets/brand/` is under the same licences ([assets/brand/LICENSE.txt](assets/brand/LICENSE.txt)). Release builds also embed the fonts of [craft-fonts](https://github.com/storytold/craft-fonts/blob/main/ATTRIBUTION.md) (SIL Open Font License 1.1).
