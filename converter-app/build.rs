fn main() {
    #[cfg(windows)]
    {
        let mut res = winres::WindowsResource::new();
        res.set_icon("../assets/icon.ico");
        res.set("ProductName", "Tempo");
        res.set("FileDescription", "Tempo - Audio Converter");
        res.set("LegalCopyright", "Copyright (c) 2026 Hemil Shah");
        let _ = res.compile();
    }
}
