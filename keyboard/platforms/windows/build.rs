fn main() {
    println!("cargo:rerun-if-changed=resources/chkey.ico");
    println!("cargo:rerun-if-changed=resources/windows.rc");

    #[cfg(target_os = "windows")]
    {
        let mut res = winres::WindowsResource::new();
        res.set_icon("resources/chkey.ico");
        res.set("ProductName", "CHKey");
        res.set("FileDescription", "CHKey Vietnamese Input Method");
        if let Err(e) = res.compile() {
            eprintln!("cargo:warning=Failed to compile Windows resource: {e}");
        }
    }
}
