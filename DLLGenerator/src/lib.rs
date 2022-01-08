use core::ffi::c_void;
use windows::Win32::Foundation::*;
use windows::Win32::UI::WindowsAndMessaging::{MessageBoxA, MB_OK};
const MB_TITLE: &str = "g0rsec - DLL Experimentation";

#[no_mangle]
pub extern "stdcall" fn DllMain(
    _dll_module: HINSTANCE,
    call_reason: u32,
    _reserved: c_void,
) -> BOOL {
    match call_reason {
        0 => dll_detach(),
        1 => injection(),
        2 => reinject(),
        3 => thread_detach(),
        _ => (),
    }
    BOOL { 0: 1 }
}

fn injection() {
    unsafe {
        MessageBoxA(None, "DLL injected !", MB_TITLE, MB_OK);
    }
}

fn reinject() {
    unsafe {
        MessageBoxA(None, "DLL re-injected !", MB_TITLE, MB_OK);
    }
}

fn thread_detach() {
    unsafe {
        MessageBoxA(None, "DLL thread detached !", MB_TITLE, MB_OK);
    }
}

fn dll_detach() {
    unsafe {
        MessageBoxA(None, "DLL detached !", MB_TITLE, MB_OK);
    }
}
