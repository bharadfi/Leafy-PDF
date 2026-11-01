//! Windows only: embed the app icon and version info (VERSIONINFO) into `leafypdf.exe`, so it
//! shows in Explorer, the taskbar, the Start menu and Alt-Tab. The same icon is also resource ID 2,
//! which the installer gives to PDF files (`leafypdf.exe,-2`) so they show the Leafy PDF tile.
//!
//! On every other target this does nothing. A missing resource compiler is a warning, so a
//! cross-compile from macOS or Linux still links, unless `PDFCRAFT_REQUIRE_WINRES=1` turns it
//! into an error (for release builds).

#[path = "src/windows_manifest.rs"]
mod windows_manifest;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=../../assets/app-icon/leafypdf.ico");
    println!("cargo:rerun-if-changed=src/windows_manifest.rs");
    println!("cargo:rerun-if-env-changed=PDFCRAFT_REQUIRE_WINRES");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }
    let mut res = winresource::WindowsResource::new();
    res.set_icon("../../assets/app-icon/leafypdf.ico")
        .set_icon_with_id("../../assets/app-icon/leafypdf.ico", "2")
        .set_manifest(windows_manifest::WINDOWS_MANIFEST)
        .set("ProductName", "Leafy PDF")
        .set("FileDescription", "Leafy PDF workbench")
        // The original authors' notice stays: Leafy PDF is built from PdfCraft's source.
        .set("LegalCopyright", "Copyright (c) the PdfCraft contributors. MIT OR Apache-2.0.")
        .set("OriginalFilename", "leafypdf.exe")
        .set("InternalName", "leafypdf");
    if let Err(e) = res.compile() {
        if std::env::var_os("PDFCRAFT_REQUIRE_WINRES").is_some() {
            println!("cargo::error=embedding Windows resources failed: {e}");
            return;
        }
        println!("cargo:warning=leafypdf.exe built without icon/version resources: {e}");
    }
}
