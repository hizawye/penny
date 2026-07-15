//! Build script. On Windows it embeds `assets/penny.ico` into the executable
//! as a resource so the app icon appears in Explorer, the taskbar, and any
//! shortcuts. A no-op everywhere else.

fn main() {
    #[cfg(windows)]
    {
        println!("cargo:rerun-if-changed=assets/penny.ico");
        let mut res = winresource::WindowsResource::new();
        res.set_icon("assets/penny.ico");
        if let Err(e) = res.compile() {
            println!("cargo:warning=could not embed Windows icon: {e}");
        }
    }
}
