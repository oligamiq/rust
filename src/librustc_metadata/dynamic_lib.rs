//! Dynamic library facilities.
//!
//! A simple wrapper over the platform's dynamic library facilities

use std::ffi::CString;
use std::path::Path;

pub struct DynamicLibrary(());

impl Drop for DynamicLibrary {
    fn drop(&mut self) {}
}

impl DynamicLibrary {
    /// Lazily open a dynamic library.
    pub fn open(filename: &Path) -> Result<DynamicLibrary, String> {
        Err("dylib loading not supported".to_string())
    }

    /// Accesses the value at the symbol of the dynamic library.
    pub unsafe fn symbol<T>(&self, symbol: &str) -> Result<*mut T, String> {
        unreachable!();
    }
}

#[cfg(test)]
mod tests;

#[cfg(disabled)]
mod dl {
    use std::ffi::{CStr, CString, OsStr};
    use std::os::unix::prelude::*;
    use std::ptr;
    use std::str;

    pub(super) fn open(filename: &OsStr) -> Result<*mut u8, String> {
        check_for_errors_in(|| unsafe {
            let s = CString::new(filename.as_bytes()).unwrap();
            libc::dlopen(s.as_ptr(), libc::RTLD_LAZY) as *mut u8
        })
    }

    fn check_for_errors_in<T, F>(f: F) -> Result<T, String>
    where
        F: FnOnce() -> T,
    {
        use std::sync::{Mutex, Once};
        static INIT: Once = Once::new();
        static mut LOCK: *mut Mutex<()> = ptr::null_mut();
        unsafe {
            INIT.call_once(|| {
                LOCK = Box::into_raw(Box::new(Mutex::new(())));
            });
            // dlerror isn't thread safe, so we need to lock around this entire
            // sequence
            let _guard = (*LOCK).lock();
            let _old_error = libc::dlerror();

            let result = f();

            let last_error = libc::dlerror() as *const _;
            if ptr::null() == last_error {
                Ok(result)
            } else {
                let s = CStr::from_ptr(last_error).to_bytes();
                Err(str::from_utf8(s).unwrap().to_owned())
            }
        }
    }

    pub(super) unsafe fn symbol(
        handle: *mut u8,
        symbol: *const libc::c_char,
    ) -> Result<*mut u8, String> {
        check_for_errors_in(|| libc::dlsym(handle as *mut libc::c_void, symbol) as *mut u8)
    }

    pub(super) unsafe fn close(handle: *mut u8) {
        libc::dlclose(handle as *mut libc::c_void);
    }
}

#[cfg(windows)]
mod dl {
    use std::ffi::OsStr;
    use std::io;
    use std::os::windows::prelude::*;
    use std::ptr;

    use winapi::shared::minwindef::HMODULE;
    use winapi::um::errhandlingapi::SetThreadErrorMode;
    use winapi::um::libloaderapi::{FreeLibrary, GetProcAddress, LoadLibraryW};
    use winapi::um::winbase::SEM_FAILCRITICALERRORS;

    pub(super) fn open(filename: &OsStr) -> Result<*mut u8, String> {
        // disable "dll load failed" error dialog.
        let prev_error_mode = unsafe {
            let new_error_mode = SEM_FAILCRITICALERRORS;
            let mut prev_error_mode = 0;
            let result = SetThreadErrorMode(new_error_mode, &mut prev_error_mode);
            if result == 0 {
                return Err(io::Error::last_os_error().to_string());
            }
            prev_error_mode
        };

        let filename_str: Vec<_> = filename.encode_wide().chain(Some(0)).collect();
        let result = unsafe { LoadLibraryW(filename_str.as_ptr()) } as *mut u8;
        let result = ptr_result(result);

        unsafe {
            SetThreadErrorMode(prev_error_mode, ptr::null_mut());
        }

        result
    }

    pub(super) unsafe fn symbol(
        handle: *mut u8,
        symbol: *const libc::c_char,
    ) -> Result<*mut u8, String> {
        let ptr = GetProcAddress(handle as HMODULE, symbol) as *mut u8;
        ptr_result(ptr)
    }

    pub(super) unsafe fn close(handle: *mut u8) {
        FreeLibrary(handle as HMODULE);
    }

    fn ptr_result<T>(ptr: *mut T) -> Result<*mut T, String> {
        if ptr.is_null() { Err(io::Error::last_os_error().to_string()) } else { Ok(ptr) }
    }
}
