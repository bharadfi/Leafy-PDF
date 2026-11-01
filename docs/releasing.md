# Releasing Leafy PDF

Leafy PDF is released as a **Windows x64 MSI**. It is built and signed on a developer machine; there is no
automated release pipeline in this repository. Packaging for other operating systems (Linux, macOS, FreeBSD)
is not part of this project, and `packaging/web/` is untested.

## Build the installer

You need the Rust MSVC toolchain, the .NET 8 SDK with WiX v5 (`dotnet tool install --global wix --version 5.0.2`),
the Windows SDK (`signtool`, `rc.exe`), a network connection for the OCR models, and the
[craft-fonts](https://github.com/storytold/craft-fonts) font set at the commit the release pins. The build
refuses to continue without the fonts, because the installer would otherwise ship without the fonts for
Japanese, Chinese, Korean, Arabic, Hebrew, Thai, Devanagari and Georgian text.

```powershell
$env:CRAFT_FONTS_DIR = '..\craft-fonts'; $env:CRAFT_FONTS_REQUIRED = '1'
$env:DIST = 'dist\release'
# optional: sign with a code-signing certificate (base64 .pfx), self-signed or issued
$env:WINDOWS_CERTIFICATE = [Convert]::ToBase64String([IO.File]::ReadAllBytes('codesign.pfx'))
$env:WINDOWS_CERTIFICATE_PASSWORD = '<password>'
$env:WINDOWS_SELF_SIGNED = '<thumbprint>'     # only for a self-signed certificate
powershell -File packaging\windows\package.ps1 -Arch x64
```

The result is `leafypdf-<version>-windows-x64.msi` in `$env:DIST`. `package.ps1` builds the app, signs
`leafypdf.exe`, stages the OCR models and font notices, runs the install-policy and MSI checks
(`check-install-policy.ps1`, `test-msi.ps1`), builds the MSI with WiX and signs it. Without signing secrets it
leaves the files unsigned and says so. `packaging/windows/sign.ps1` is the one place that changes when signing
changes. `INSTALLATION.md` in the same folder explains the installer's pages and properties.

## Check before you publish

- `cargo test -p pdfcraft-sign --lib`, `cargo test -p pdfcraft-engine --lib signature`,
  `cargo test -p pdfcraft-ui-egui --lib canvas`
- `cargo xtask assets` (every asset has an `ATTRIBUTION.toml` entry with an allowed licence)
- Run the app: open a signed PDF, check the three signature marks and trust/untrust in the Signatures panel.
- Compare `Get-FileHash <msi> -Algorithm SHA256` with the checksum you put in the release notes.

## Publish

Create a GitHub release for the tag `v<version>` with the MSI attached, for example with the GitHub CLI:

```powershell
gh release create v<version> leafypdf-<version>-windows-x64.msi --title "Leafy PDF <version>" --notes-file RELEASE-NOTES.md
```

Say in the notes if the installer is signed with a self-made certificate: Windows shows *Unknown publisher*
for it until the certificate is trusted.
