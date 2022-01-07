#[macro_use]
extern crate log;
use env_logger::Env;
use std::path::Path;
use std::io::ErrorKind;
use std::process;
use std::mem;
use std::io;
use std::ptr::null_mut;
use core::ffi::c_void;
use std::ffi::CString;
use windows::{
	Win32::Foundation::*, Win32::System::Memory::*, 
	Win32::System::Threading::*, Win32::System::Diagnostics::ToolHelp::*, 
	Win32::System::LibraryLoader::*, Win32::System::Diagnostics::Debug::*
};

const PROCESS_NAME: &str = "win32calc.exe";
const DLL_PATH: &str = "target\\release\\dllgenerator.dll";

fn get_process_entry(proc_name: &str) -> Result<PROCESSENTRY32, std::io::Error> {
	let mut process_entry: PROCESSENTRY32 = PROCESSENTRY32 { 
		dwSize: mem::size_of::<PROCESSENTRY32>() as u32, 
		cntUsage: 0, 
		th32ProcessID: 0, 
		th32DefaultHeapID: 0, 
		th32ModuleID: 0, 
		cntThreads: 0, 
		th32ParentProcessID: 0, 
		pcPriClassBase: 0, 
		dwFlags: 0, 
		szExeFile: [0; 260] 
	};
	let process_snapshot_flags = 2; // TH32CS_SNAPPROCESS 0x00000002
	unsafe {
		let process_snapshot_handle: HANDLE = CreateToolhelp32Snapshot(process_snapshot_flags, 0);
		if Process32First(process_snapshot_handle, &mut process_entry).as_bool() {
			while Process32Next(process_snapshot_handle, &mut process_entry).as_bool() {
				if String::from_utf8(process_entry.szExeFile.to_vec()) 
					.expect("UTF-8 error, could not convert process name to String.")
					.trim_matches(char::from(0)) == PROCESS_NAME {
						return Ok(process_entry);
				}
				process_entry.szExeFile = [0; 260];
			}
		}
		return Err(io::Error::new(ErrorKind::Other, format!("Unable to find {}", proc_name)))
	}
}

fn main() {
	let env = Env::default()
        .default_filter_or("info");
    env_logger::init_from_env(env);
	unsafe {		
		match get_process_entry(PROCESS_NAME) {
			Ok(process_entry) => {
				info!("Found {}.", PROCESS_NAME);
				info!("Opening target process.");
				let process_handle = OpenProcess(
					PROCESS_CREATE_THREAD | PROCESS_VM_OPERATION | PROCESS_VM_WRITE,
					BOOL{0: 0}, process_entry.th32ProcessID);

				let dll_file = Path::new(DLL_PATH);
                if !dll_file.exists() {
                    error!("{}", format!("No DLL found in {}", DLL_PATH));
                    process::exit(0x0100);
                }
				let dll_file_path = Path::new(DLL_PATH);
				let dll_path = CString::new(dll_file_path.canonicalize().unwrap().to_str().unwrap()).unwrap();
				let dll_path_size = dll_path.as_bytes_with_nul().len();
				info!("Allocating memory in target process.");
				let process_alloc = VirtualAllocEx(
					process_handle, 
					null_mut(), 
					dll_path_size, 
					MEM_COMMIT | MEM_RESERVE, 
					PAGE_READWRITE
				);
				info!("Writing DLL path in allocated memory.");
				WriteProcessMemory(
					process_handle, 
					process_alloc, 
					dll_path.as_ptr() as *const c_void, 
					dll_path_size, 
					null_mut()
				);
				
				info!("Preparing to inject DLL code.");
				let kernel_mod = CString::new("Kernel32.dll").unwrap();
				let loadlibrary_fn = CString::new("LoadLibraryA").unwrap();
				let mod_handle = GetModuleHandleA(PSTR {0: kernel_mod.as_ptr() as *mut u8});
				
				let mod_address = GetProcAddress(mod_handle,PSTR {0: loadlibrary_fn.as_ptr() as *mut u8}).unwrap();
				
				let injection_start_address: LPTHREAD_START_ROUTINE = mem::transmute(mod_address);
				info!("Executing DLL code on behalf of target process.");
				let rhandle = CreateRemoteThread(
					process_handle, 
					null_mut(), 
					0, 
					injection_start_address, 
					process_alloc, 
					0, 
					null_mut()
				);
				info!("Waiting 5 seconds before stopping DLL injection.");
				WaitForSingleObject(rhandle, 5000);
				info!("Cleaning DLL injection from target process.");
				TerminateThread(rhandle, 0);
				CloseHandle(rhandle);
				VirtualFreeEx(
					process_handle, 
					process_alloc, 
					dll_path_size, 
					MEM_RELEASE
				);
				CloseHandle(process_handle);
			},
			Err(e) => {
				error!("{}.", e);
			}
		}		
	}
}