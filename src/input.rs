#[inline(always)]
pub fn check_input() -> bool {
    #[cfg(unix)]
    {
        let mut fds: libc::fd_set = unsafe { std::mem::zeroed() };
        unsafe { libc::FD_ZERO(&mut fds) };
        unsafe { libc::FD_SET(0, &mut fds) };
        let mut tv = libc::timeval { tv_sec: 0, tv_usec: 0 };
        let ret = unsafe { libc::select(1, &mut fds, std::ptr::null_mut(), std::ptr::null_mut(), &mut tv) };
        if ret > 0 {
            let mut buf = [0u8; 32];
            let n = unsafe { libc::read(0, buf.as_mut_ptr() as *mut _, 32) };
            if n > 0 {
                if buf[0] == 3 {
                        return true;
                }
                if buf[0] == 27 {
                    if n == 1 {
                        return true;
                    }
                    if n >= 3 && buf[1] == b'[' {
                        return false; 
                   }
                    if n >= 2 && buf[1] != b'[' {
                        return true;
                }
            }
        }
   }
}
    #[cfg(windows)]
    {
        use std::io;
        use std::os::windows::io::AsRawHandle;
        let handle = io::stdin().as_raw_handle();
        let mut num: u32 = 0;
        if unsafe { winapi::um::consoleapi::GetNumberOfConsoleInputEvents(handle as _, &mut num) } != 0 && num > 0 {
            let mut rec: winapi::um::wincon::INPUT_RECORD = unsafe { std::mem::zeroed() };
            let mut read = 0;
            if unsafe { winapi::um::consoleapi::ReadConsoleInputW(handle as _, &mut rec, 1, &mut read) } != 0 {
                if rec.EventType == 1 {
                    let key = unsafe { rec.Event.KeyEvent() };
                    if key.bKeyDown != 0 {
                        let ch = unsafe { *key.uChar.UnicodeChar() };
                        return ch == 27 || ch == 3;
                    }
                }
            }
        }
    }
    false
}