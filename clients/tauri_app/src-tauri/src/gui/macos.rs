#[cfg(target_os = "macos")]
extern "C" {
    pub fn devflow_set_macos_dock_icon(bytes: *const u8, len: usize);
}

#[cfg(target_os = "macos")]
pub fn apply_macos_dock_icon() {
    static ICON_PNG: &[u8] = include_bytes!("../../icons/icon.png");
    unsafe {
        devflow_set_macos_dock_icon(ICON_PNG.as_ptr(), ICON_PNG.len());
    }
}

#[cfg(not(target_os = "macos"))]
pub fn apply_macos_dock_icon() {}
