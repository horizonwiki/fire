use std::{
    io::{self, Write},
    sync::atomic::{AtomicBool, Ordering},
};

#[cfg(unix)]
use std::os::unix::io::AsRawFd;

pub struct Terminal {
    #[cfg(unix)]
    orig: libc::termios,
    #[cfg(windows)]
    orig_mode: u32,
}

pub static EXIT_REQUESTED: AtomicBool = AtomicBool::new(false);

#[cfg(unix)]
pub static RESIZE_REQUESTED: AtomicBool = AtomicBool::new(true);

impl Terminal {
    pub fn new() -> io::Result<Self> {
        #[cfg(unix)]
        {
            let fd = io::stdin().as_raw_fd();
            let mut orig = unsafe { std::mem::zeroed() };
            if unsafe { libc::tcgetattr(fd, &mut orig) } != 0 {
                return Err(io::Error::last_os_error());
            }
            let mut raw = orig;
            unsafe { libc::cfmakeraw(&mut raw) };
            raw.c_lflag &= !(libc::ECHO | libc::ICANON);
            if unsafe { libc::tcsetattr(fd, libc::TCSANOW, &raw) } != 0 {
                return Err(io::Error::last_os_error());
            }
            unsafe {
                libc::signal(libc::SIGINT, handle_sigint as *const () as libc::sighandler_t);
                libc::signal(libc::SIGTERM, handle_sigint as *const () as libc::sighandler_t);
                libc::signal(libc::SIGWINCH, handle_sigwinch as *const () as libc::sighandler_t);
            }
            Ok(Self { orig })
        }
        #[cfg(windows)]
        {
            use std::os::windows::io::AsRawHandle;
            let handle = io::stdin().as_raw_handle();
            unsafe {
                let mut mode: u32 = 0;
                if winapi::um::consoleapi::GetConsoleMode(handle as _, &mut mode) == 0 {
                    return Err(io::Error::last_os_error());
                }
                let new_mode = mode & !(0x0002 | 0x0004 | 0x0010 | 0x0020);
                if winapi::um::consoleapi::SetConsoleMode(handle as _, new_mode) == 0 {
                    return Err(io::Error::last_os_error());
                }
                unsafe extern "system" fn ctrl_handler(_: u32) -> i32 {
                    EXIT_REQUESTED.store(true, Ordering::Relaxed);
                    1
                }
                winapi::um::consoleapi::SetConsoleCtrlHandler(Some(ctrl_handler), 1);
                Ok(Self { orig_mode: mode })
            }
        }
        #[cfg(not(any(unix, windows)))]
        {
            Ok(Self {})
        }
    }
}

impl Drop for Terminal {
    fn drop(&mut self) {
        restore_terminal();

        #[cfg(unix)]
        {
            let fd = io::stdin().as_raw_fd();
            unsafe { libc::tcsetattr(fd, libc::TCSANOW, &self.orig) };
        }
        #[cfg(windows)]
        {
            use std::os::windows::io::AsRawHandle;
            let handle = io::stdin().as_raw_handle();
            unsafe { winapi::um::consoleapi::SetConsoleMode(handle as _, self.orig_mode) };
        }
    }
}

pub fn get_size() -> (usize, usize) {
    #[cfg(unix)]
    {
        let mut ws: libc::winsize = unsafe { std::mem::zeroed() };
        if unsafe { libc::ioctl(1, libc::TIOCGWINSZ, &mut ws) } == 0 {
            return (ws.ws_col as usize, ws.ws_row as usize);
        }
    }
    #[cfg(windows)]
    {
        use std::os::windows::io::AsRawHandle;
        let handle = io::stdout().as_raw_handle();
        let mut info: winapi::um::wincon::CONSOLE_SCREEN_BUFFER_INFO = unsafe { std::mem::zeroed() };
        if unsafe { winapi::um::wincon::GetConsoleScreenBufferInfo(handle as _, &mut info) } != 0 {
            let w = (info.srWindow.Right - info.srWindow.Left + 1) as usize;
            let h = (info.srWindow.Bottom - info.srWindow.Top + 1) as usize;
            return (w, h);
        }
    }
    (80, 24)
}

pub fn restore_terminal() {
    let mut stdout = io::stdout();
    let _ = stdout.write_all(b"\x1b[0m\x1b[?25h\x1b[?1049l");
    let _ = stdout.flush();
}

#[cfg(unix)]
extern "C" fn handle_sigint(_: i32) {
    EXIT_REQUESTED.store(true, Ordering::Relaxed);
}

#[cfg(unix)]
extern "C" fn handle_sigwinch(_: i32) {
    RESIZE_REQUESTED.store(true, Ordering::Relaxed);
}
