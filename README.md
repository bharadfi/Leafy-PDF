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

Also worth knowing:

- **Compact.** The installer is about 55 MB and installs without the command-line tools, so it takes little space on disk.
- **Your folder, your choice.** The Windows installer asks who should use the app and then *where to install it*: type a path or click Browse and pick any drive and folder.
- **One row at the top.** The window title bar and the document tabs share a single row, so the page gets more room.
- **Fonts included.** The fonts for Japanese, Chinese, Korean, Arabic, Hebrew, Thai, Devanagari and Georgian text are built into the program, so editing text in those scripts needs nothing extra.
- **Offline OCR.** *Scan & OCR* uses models that ship with the installer. Nothing is uploaded.
- **Its own settings.** Settings, recent files and your list of trusted certificates live in their own `LeafyPdf` folder and are not shared with any other PDF program.
- **No promotions.** No community banners, no links out of the app unless you click a link in a document.

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
- **Navigate long documents.** Bookmarks, page thumbnails and the document's own page labels (i, ii, 1, 2…) keep you oriented. Right-click a tab to close it, reopen the last closed file or reveal it in Explorer.

## Organize pages like cards on a table

Open **Organize pages** to see every page at once:

- **Select pages:** click, Ctrl-click or Shift-click.
- **Change them:** rotate, delete, insert blank pages, insert pages from another file, and move them earlier or later.
- **Undo anything:** each change is one step in a history you can walk backwards and forwards; undo still works after you save.

**Saves you can trust:** *incremental* (the original bytes stay intact), *atomic* (written to a temporary copy, then swapped in) and *recoverable* (changes are autosaved every minute; if the app ever quits unexpectedly it offers to recover your work). Unsaved documents carry a dot on their tab, and closing asks before anything is lost. Encrypted documents stay encrypted on disk.

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
- **More tools on the home screen:** Export, Create, Combine, Organize, Edit, Add comments, Scan & OCR, Protect, Redact, Compress, Prepare a form, Add a stamp and Use a certificate. Press Ctrl-K to search every tool and command.

---

## Runs locally, stays yours

- **Private by design.** Documents never leave your machine. There's no account, no telemetry and no cloud processing.
- **Engine first.** Parsing, rendering, signing and editing live in reusable library crates. The interface is one swappable layer on top.
- **Scriptable.** The same engine is available without the interface (see below).

---

## Built for agents, too

Every engine feature is reachable without the GUI through one table of JSON-Schema-described tools: open, inspect, render pages to PNG, extract and find text, rotate, delete, move and insert pages, edit bookmarks and page labels, add and reply to comments, list and fill form fields, protect with passwords, set metadata, undo and redo, save, combine, extract and split.

The source builds a command-line tool, `pdfcraft-cli` (the crate names of the code base are unchanged). The Windows installer does not include it. Build it from source if you want it:

```powershell
cargo build --release -p pdfcraft-cli
pdfcraft-cli tools                                    # every tool and its JSON Schema
pdfcraft-cli info  form.pdf                           # structure as JSON
pdfcraft-cli text  paper.pdf --page 3                 # reading-order text
pdfcraft-cli combine report.pdf appendix.pdf --out combined.pdf
pdfcraft-cli mcp --root C:\path\to\your\pdfs          # opt-in MCP server over stdin/stdout
```

The MCP server is opt-in: nothing starts it on its own and it opens no network port. `--root` confines every file the agent can read or write to one directory.

### Driving the app itself

Start the app with `leafypdf.exe --control <file>` and an agent can see and operate the real interface: the widget tree with labels and positions, clicks, typing, keys, commands, view options and screenshots. This is off by default. It listens only on loopback, and every connection must present the random token written to that file.

```powershell
pdfcraft-cli ui --control <file> inspect query="Signature panel"
pdfcraft-cli ui --control <file> click label="Signature panel"
pdfcraft-cli ui --control <file> screenshot --out window.png
```

---

## How it's built

Leafy PDF is a Cargo workspace of focused crates, layered so the core never depends on the UI:

| Crate | What it does |
|---|---|
| `pdfcraft-filters` | Every PDF stream filter (Flate, LZW, ASCII85, RunLength, predictors), encode and decode |
| `pdfcraft-crypt` | The standard security handler: RC4, AES-128/256, revisions 2–6, permissions |
| `pdfcraft-cos` | The PDF object layer: tolerant parsing, repair, copy-on-write edits, incremental and full writing |
| `pdfcraft-sign` | Digital signatures: certificate chains, verification, trust, signing, and the verdict layers drawn on the page |
| `pdfcraft-organize` | Page operations, combine / extract / split, bookmarks, page labels, document information |
| `pdfcraft-annot` | Comments: builders and appearance streams for notes, text markup, shapes, ink and text boxes |
| `pdfcraft-forms` | Interactive forms: the field model, filling with regenerated appearances, Clear form |
| `pdfcraft-render` | Rendering, inspection and text extraction with reading order |
| `pdfcraft-engine` | The façade every frontend uses: sessions, edits, undo, saving, the tool catalogue |
| `pdfcraft-automation` | Agent control: the headless tool table, `pdfcraft-cli run`, and the opt-in MCP server |
| `pdfcraft-ui-egui` | The desktop interface |

Leafy PDF's behaviour comes from the ISO 32000 specification and black-box observation. Every icon, font and image is openly licensed and listed in [ATTRIBUTION.md](ATTRIBUTION.md); `cargo xtask assets` checks that list.
What this fork changes is recorded in [docs/leafy-pdf.md](docs/leafy-pdf.md).

## Get started

Building the app needs Rust (MSVC toolchain). Building the installer also needs the .NET 8 SDK, the Windows SDK and WiX v5.

```powershell
git clone https://github.com/bharadfi/Leafy-PDF
cd Leafy-PDF
cargo xtask models                                   # fetch the OCR models Scan & OCR needs
cargo run --release -p pdfcraft -- some.pdf          # run the desktop app on a PDF
```

The program text in Japanese, Chinese, Korean, Arabic, Hebrew, Thai, Devanagari and Georgian relies on the [craft-fonts](https://github.com/storytold/craft-fonts) font set (SIL Open Font License 1.1), a build input. The installer build requires it and refuses to continue without it. Check out the commit pinned in the repository's packaging notes:

```powershell
git clone https://github.com/storytold/craft-fonts ..\craft-fonts
$env:CRAFT_FONTS_DIR = '..\craft-fonts'; $env:CRAFT_FONTS_REQUIRED = '1'
dotnet tool install --global wix --version 5.0.2
powershell -File packaging\windows\package.ps1 -Arch x64
```

The interface language is chosen in **Edit → Preferences…** (see [docs/localization.md](docs/localization.md)). Logs, environment variables and other development notes are in [docs/development.md](docs/development.md).

## What's next

Leafy PDF is **alpha** software. Everyday reading, commenting, form filling, signing and page organizing work, and the signature checks above are in place. Areas that are still thin or missing:

- reliable editing of existing text (especially CJK and right-to-left);
- OCR beyond Latin script;
- office-document import and export;
- signature tokens and online long-term validation;
- preflight and PDF/X, PDF/UA conformance;
- full XFA forms.

Not yet verified for this fork: printing a validated signature on a physical printer, and the installers for other operating systems (this fork's installer and testing are Windows x64 only).

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
- **Shortcuts and menus.** It adds a Start Menu shortcut, an optional desktop shortcut, and optional "Edit with Leafy PDF" and "Create PDF with Leafy PDF…" entries in the right-click menu of PDF files and images in File Explorer (on Windows 11, under "Show more options"). It does not change which app opens PDFs by default.
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
It is built on the open-source [PdfCraft](https://github.com/storytold/pdfcraft) 0.16.1 code base and keeps its copyright and attribution notices; required notices are in [NOTICE](NOTICE).

Bundled fonts, icons, images and other assets keep their own open licenses; each one is listed with its author, source and license in [ATTRIBUTION.md](ATTRIBUTION.md). The Leafy PDF logo in `assets/brand/` is under the same licences ([assets/brand/LICENSE.txt](assets/brand/LICENSE.txt)). Release builds also embed the fonts of [craft-fonts](https://github.com/storytold/craft-fonts/blob/main/ATTRIBUTION.md) (SIL Open Font License 1.1).
