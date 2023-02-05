extern crate winres;

fn main() {
  if cfg!(target_os = "windows") {
    let mut res = winres::WindowsResource::new();
    res.set_icon_with_id("assets/icon.ico", "window-icon");
    res.compile().unwrap();
  }
}