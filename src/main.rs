// Fire-CLI || Github: https://github.com/horizonwiki/fire || v0.1.1 || 12.12.25
use std::{
    env,
    io::{self, Write, BufWriter},
    sync::atomic::{AtomicBool, Ordering},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

#[cfg(unix)]
use std::os::unix::io::AsRawFd;

struct Terminal {
    #[cfg(unix)]
    orig: libc::termios,
    #[cfg(windows)]
    orig_mode: u32,
}

impl Terminal {
    fn new() -> io::Result<Self> {
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

static EXIT_REQUESTED: AtomicBool = AtomicBool::new(false);

#[cfg(unix)]
extern "C" fn handle_sigint(_: i32) {
    EXIT_REQUESTED.store(true, Ordering::Relaxed);
}

fn get_size() -> (usize, usize) {
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

fn restore_terminal() {
    let mut stdout = io::stdout();
    let _ = stdout.write_all(b"\x1b[0m\x1b[?25h\x1b[?1049l");
    let _ = stdout.flush();
}

#[inline(always)]
fn check_input() -> bool {
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
                if buf[0] ==  27 {
                    if n == 1 {   
                        return true;
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

struct Rng(u32);
impl Rng {
    #[inline(always)]
    fn new() -> Self {
        let seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u32;
        Self(seed | 1)
    }
    
    #[inline(always)]
    fn next(&mut self) -> u32 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 17;
        self.0 ^= self.0 << 5;
        self.0
    }
}

#[derive(Copy, Clone)]
struct Theme {
    colors: [&'static str; 4],
}

impl Theme {
    
    pub const CHARS: [char; 10] = [' ', '.', ':', '^', '*', 'x', 's', 'S', '#', '$'];
    
    const STD: Theme = Theme {
        colors: ["38;2;255;0;0", "38;2;255;85;0", "38;2;255;170;0", "38;2;255;255;0"]
    };
    
    const ICE: Theme = Theme {
        colors: ["38;2;173;216;230", "38;2;135;206;250", "38;2;0;191;255", "38;2;240;248;255"]
    };
    
    const CLASSIC: Theme = Theme {
        colors: ["38;2;20;20;20", "38;2;220;50;47", "38;2;255;200;40", "38;2;70;130;180"]
    };
    
    const PINK: Theme = Theme {
        colors: ["38;2;255;105;180", "38;2;255;182;193", "38;2;255;240;245", "38;2;255;255;255"]
    };

    const BLUE: Theme = Theme {
        colors: ["38;2;10;15;28", "38;2;0;95;135", "38;2;0;175;175", "38;2;51;225;255"]
    };
}

fn print_help() {
    println!("\nUsage:");
    println!("  fire [options]\n");
    println!("Options:");
    println!("  -f,    --fps <number>  - set FPS (default: 30, range: 15-120)");
    println!("  -n-c,  --no-color      - disable colors (ASCII only)");
    println!("\nThemes:");
    println!("  -s,    --std           - classic fire (default)");
    println!("  -i,    --ice           - ice fire");
    println!("  -c,    --classic       - alternative classic fire");
    println!("  -p,    --pink          - pink neon fire");
    println!("  -b,    --blue          - blue neon fire");
    println!("\nExamples:");
    println!("  fire -i -f 60");
    println!("  fire --blue --fps 45");
    println!("  fire --no-color");
    println!("\nControls:");
    println!("  ESC or Ctrl+C - exit");
}

#[inline(always)]
fn simulate_step(buf: &mut [u8], w: usize, h: usize, rng: &mut Rng) {
    let size = w * h;
    
    for _ in 0..(w / 9).max(1) {
        let idx = (rng.next() as usize % w) + w * (h - 1);
        if idx < buf.len() {
            buf[idx] = 65;
        }
    }

    for i in 0..size {
        if i + w + 1 < buf.len() {
            let sum = buf[i] as u32
                + buf[i + 1] as u32
                + buf[i + w] as u32
                + buf[i + w + 1] as u32;
            buf[i] = (sum >> 2) as u8;
        }
    }
}

fn precompile_color_codes(theme: &Theme) -> [Vec<u8>; 4] {
    [
        format!("\x1b[{}m", theme.colors[0]).into_bytes(),
        format!("\x1b[{}m", theme.colors[1]).into_bytes(),
        format!("\x1b[{}m", theme.colors[2]).into_bytes(),
        format!("\x1b[{}m", theme.colors[3]).into_bytes(),
    ]
}

const fn precompile_chars() -> [[u8; 4]; 10] {
    let mut result = [[0u8; 4]; 10];
    let mut i = 0;
    while i < 10 {
        
        let ch = Theme::CHARS[i] as u32; 
        result[i][0] = ch as u8;
        result[i][1] = 0;
        result[i][2] = 0;
        result[i][3] = 0;
        i += 1;
    }
    result
}

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    
    let mut theme = Theme::STD;
    let mut fps: u32 = 30;
    let mut use_color = true;
    let mut i = 1;
    
    while i < args.len() {
        match args[i].to_lowercase().as_str() {
            "-s" | "--std" => theme = Theme::STD,
            "-i" | "--ice" => theme = Theme::ICE,
            "-c" | "--classic" => theme = Theme::CLASSIC,
            "-p" | "--pink" => theme = Theme::PINK,
            "-b" | "--blue" => theme = Theme::BLUE,
            "-f" | "--fps" => {
                if i + 1 < args.len() {
                    if let Ok(f) = args[i + 1].parse::<u32>() {
                        fps = f.clamp(15, 120);
                        i += 1;
                    } else {
                        eprintln!("Invalid FPS value: {}", args[i + 1]);
                        print_help();
                        return Ok(());
                    }
                } else {
                    eprintln!("Missing FPS value");
                    print_help();
                    return Ok(());
                }
            }
            "--no-color" | "-n-c" | "--nocolor" => use_color = false,
            "-h" | "--help"  => {
                print_help();
                return Ok(());
            }
            _ => {
                eprintln!("Invalid option: {}", args[i]);
                print_help();
                return Ok(());
            }
        }
        i += 1;
    }

    let _term = Terminal::new()?;
    
    std::panic::set_hook(Box::new(|info| {
        restore_terminal();
        eprintln!("panic: {}", info);
    }));
    
    let stdout = io::stdout();
    let mut stdout = BufWriter::with_capacity(128 * 1024, stdout);
    
    stdout.write_all(b"\x1b[?1049h\x1b[?25l\x1b[2J\x1b[H")?;
    stdout.flush()?;
    
    let mut current_size = get_size();
    let (mut w, mut h) = current_size;
    let mut size = w * h;
    let mut buf = vec![0u8; size + w + 1];
    
    let render_interval = Duration::from_secs_f64(1.0 / fps as f64);
    let physics_step = 1.0 / fps as f64;
    
    let mut screen = Vec::with_capacity((w + 1) * h * 30);
    let mut rng = Rng::new();
    let mut last_instant = Instant::now();
    let mut accumulator = 0.0f64;

    let color_codes = precompile_color_codes(&theme);
    let char_bytes = precompile_chars();

    loop {
        let frame_start = Instant::now();

        let new_size = get_size();
        if new_size != current_size {
            current_size = new_size;
            w = current_size.0;
            h = current_size.1;
            size = w * h;
            buf = vec![0u8; size + w + 1];
            screen = Vec::with_capacity((w + 1) * h * 30);
            stdout.write_all(b"\x1b[2J")?;
            stdout.flush()?;
        }

        let now = Instant::now();
        let dt = now.duration_since(last_instant);
        last_instant = now;
        accumulator += dt.as_secs_f64();
        
        if accumulator > 0.25 {
            accumulator = 0.25;
        }
        
        let mut steps = 0;
        while accumulator >= physics_step && steps < 5 {
            simulate_step(&mut buf, w, h, &mut rng);
            accumulator -= physics_step;
            steps += 1;
        }

        screen.clear();
        screen.extend_from_slice(b"\x1b[1;1H");

        let mut last_color_idx: Option<usize> = None;

        for y in 0..h {
            for x in 0..w {
                let i = y * w + x;
                let heat = buf[i] as usize;

                if use_color {
                    let color_idx = match heat {
                        0..=4 => 0,
                        5..=9 => 1,
                        10..=15 => 2,
                        _ => 3,
                    };

                    if last_color_idx != Some(color_idx) {
                        screen.extend_from_slice(&color_codes[color_idx]);
                        last_color_idx = Some(color_idx);
                    }
                }

                let ch_idx = heat.min(9);
                let ch_byte = char_bytes[ch_idx][0];
                screen.push(ch_byte);
            }
            
            if use_color {
                screen.extend_from_slice(b"\x1b[0m");
                last_color_idx = None;
            }
            
            screen.extend_from_slice(b"\x1b[0K");
            if y < h - 1 {
                screen.extend_from_slice(b"\r\n");
            }
        }

        stdout.write_all(&screen)?;
        stdout.flush()?;

        if check_input() || EXIT_REQUESTED.load(Ordering::Relaxed) {
            break;
        }

        let frame_elapsed = frame_start.elapsed();
        if frame_elapsed < render_interval {
            std::thread::sleep(render_interval - frame_elapsed);
        }
    }

    Ok(())
}