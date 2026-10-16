// Embeds the NexusCard icon and version info into nexuscard.exe (Windows only).
fn main() {
    println!("cargo:rerun-if-changed=assets/icon.ico");
    println!("cargo:rerun-if-changed=build.rs");

    #[cfg(windows)]
    {
        let mut resource = winresource::WindowsResource::new();
        resource.set_icon("assets/icon.ico");
        resource.set("ProductName", "NexusCard");
        resource.set("FileDescription", "NexusCard - Apple Wallet card designs from Windows");
        resource.set("CompanyName", "NexusCard");
        resource.set("LegalCopyright", "MIT License");
        if let Err(error) = resource.compile() {
            // A missing resource compiler must not block building the app itself.
            println!("cargo:warning=Could not embed the Windows icon: {error}");
        }
    }
}
