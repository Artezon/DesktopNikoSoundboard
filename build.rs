fn main() {
    #[cfg(windows)]
    {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("assets/icon.ico")
            .set("ProductName", "Niko OneShot")
            .set("FileDescription", "OneShot characters soundboard :3")
            .set("CompanyName", "Artezon");
        res.compile().unwrap();
    }
}
