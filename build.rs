fn main() {
    #[cfg(windows)]
    {
        let mut res = winres::WindowsResource::new();
        res.set_icon("app.ico");
        res.set_icon_with_id("app.ico", "app-icon");
        res.set("ProductName", "圖書管理借還系統");
        res.set("FileDescription", "現代化單機版圖書管理借還系統");
        res.set("ProductVersion", "0.5.0.0");
        res.set("FileVersion", "0.5.0.0");
        if let Err(e) = res.compile() {
            eprintln!("Warning: Failed to compile Windows resource: {}", e);
        }
    }
}
