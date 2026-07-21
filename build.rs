fn main() {
    #[cfg(windows)]
    {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("assets/icon.ico")
            .set("ProductName", "Niko")
            .set("FileDescription", "Niko OneShot")
            .set("CompanyName", "Artezon");
        res.compile().unwrap();
    }
}
