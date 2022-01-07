use windows::Win32::UI::WindowsAndMessaging::{MessageBoxA, MB_OK};

#[no_mangle]
pub extern "stdcall" fn DllMain() {
    unsafe {
        MessageBoxA(None, "Dll injected", "Rust PoC", MB_OK);
    }
}
