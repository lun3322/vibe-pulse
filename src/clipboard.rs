use std::{mem::size_of, ptr::copy_nonoverlapping};

use windows::{
    Win32::{
        Foundation::{GlobalFree, HANDLE, HWND},
        System::{
            DataExchange::{CloseClipboard, EmptyClipboard, OpenClipboard, SetClipboardData},
            Memory::{GMEM_MOVEABLE, GlobalAlloc, GlobalLock, GlobalUnlock},
        },
    },
    core::{Error, Result},
};

const UNICODE_TEXT: u32 = 13;

pub fn copy_text(owner: HWND, text: &str) -> Result<()> {
    let text: Vec<u16> = text.encode_utf16().chain([0]).collect();
    unsafe {
        let memory = GlobalAlloc(GMEM_MOVEABLE, text.len() * size_of::<u16>())?;
        let target = GlobalLock(memory).cast::<u16>();
        if target.is_null() {
            let _ = GlobalFree(Some(memory));
            return Err(Error::from_thread());
        }
        copy_nonoverlapping(text.as_ptr(), target, text.len());
        let _ = GlobalUnlock(memory);
        if let Err(error) = write(owner, HANDLE(memory.0)) {
            let _ = GlobalFree(Some(memory));
            return Err(error);
        }
    }
    Ok(())
}

unsafe fn write(owner: HWND, memory: HANDLE) -> Result<()> {
    unsafe { OpenClipboard(Some(owner))? };
    let result = unsafe {
        EmptyClipboard().and_then(|_| SetClipboardData(UNICODE_TEXT, Some(memory)).map(|_| ()))
    };
    result.and(unsafe { CloseClipboard() })
}
