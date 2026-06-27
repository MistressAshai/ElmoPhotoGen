
#[cfg(windows)]
fn main() {
    let mut res = tauri_winres::WindowsResource::new();
    res.set_icon("m200.ico");
    res.compile().unwrap();
}

#[cfg(unix)]
fn main() {}