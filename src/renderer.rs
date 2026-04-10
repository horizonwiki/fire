use std::{
    io::{self, Write, BufWriter},
    time::{Duration, Instant},
    sync::atomic::Ordering,
};

use crate::theme::{Theme, ColorMode};
use crate::simulation::Rng;
use crate::input::check_input;
use crate::terminal::{get_size, EXIT_REQUESTED};

#[cfg(unix)]
use crate::terminal::RESIZE_REQUESTED;

pub fn precompile_color_codes(theme: &Theme) -> [Vec<u8>; 4] {
    [
        format!("\x1b[{}m", theme.colors[0]).into_bytes(),
        format!("\x1b[{}m", theme.colors[1]).into_bytes(),
        format!("\x1b[{}m", theme.colors[2]).into_bytes(),
        format!("\x1b[{}m", theme.colors[3]).into_bytes(),
    ]
}

pub const fn precompile_chars() -> [[u8; 4]; 10] {
    let mut result = [[0u8; 4]; 10];
    let mut i = 0;
    while i < 10 {
        let ch = Theme::CHARS[i] as u32;
        assert!(ch <= 127, "CHARS must contain only ASCII characters");
        result[i][0] = ch as u8;
        i += 1;
    }
    result
}

#[inline(always)]
pub fn push_truecolor(buf: &mut Vec<u8>, r: u8, g: u8, b: u8) {
    #[inline(always)]
    fn push_u8(buf: &mut Vec<u8>, mut n: u8) {
        if n >= 100 {
            buf.push(b'0' + n / 100);
            n %= 100;
            buf.push(b'0' + n / 10);
            buf.push(b'0' + n % 10);
        } else if n >= 10 {
            buf.push(b'0' + n / 10);
            buf.push(b'0' + n % 10);
        } else {
            buf.push(b'0' + n);
        }
    }
    buf.extend_from_slice(b"\x1b[38;2;");
    push_u8(buf, r);
    buf.push(b';');
    push_u8(buf, g);
    buf.push(b';');
    push_u8(buf, b);
    buf.push(b'm');
}

pub fn run_main_loop(
    theme: &Theme,
    color_mode: ColorMode,
    fps: u32,
    use_color: bool,
) -> io::Result<()> {
    use crate::simulation::simulate_step;
    use crate::theme::hue_to_color_bytes;

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

    let mut screen = Vec::with_capacity((w + 1) * h * 20);
    let mut rng = Rng::new();
    let mut last_instant = Instant::now();
    let mut accumulator = 0.0f64;

    let color_codes = precompile_color_codes(theme);
    let char_bytes = precompile_chars();
    let mut rainbow_offset: f32 = 0.0;

    loop {
        let frame_start = Instant::now();

        #[cfg(unix)]
        if RESIZE_REQUESTED.load(Ordering::Relaxed) {
            RESIZE_REQUESTED.store(false, Ordering::Relaxed);
            let new_size = get_size();
            if new_size != current_size {
                current_size = new_size;
                w = current_size.0;
                h = current_size.1;
                size = w * h;
                buf = vec![0u8; size + w + 1];
                screen = Vec::with_capacity((w + 1) * h * 20);
                stdout.write_all(b"\x1b[2J")?;
                stdout.flush()?;
            }
        }
        #[cfg(windows)]
        {
            let new_size = get_size();
            if new_size != current_size {
                current_size = new_size;
                w = current_size.0;
                h = current_size.1;
                size = w * h;
                buf = vec![0u8; size + w + 1];
                screen = Vec::with_capacity((w + 1) * h * 20);
                stdout.write_all(b"\x1b[2J")?;
                stdout.flush()?;
            }
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
        if color_mode == ColorMode::Rainbow {
            rainbow_offset = (rainbow_offset + 0.5).rem_euclid(360.0);
        }
        screen.clear();
        screen.extend_from_slice(b"\x1b[1;1H");

        let mut last_color_idx: Option<usize> = None;
        let mut last_rgb: Option<(u8, u8, u8)> = None;

        for y in 0..h {
            let row_start = y * w;
            for i in row_start..row_start + w {
                let heat = buf[i] as usize;

                if use_color {
                    match color_mode {
                        ColorMode::Theme => {
                            let color_idx = match heat {
                                0..=4 => 0,
                                5..=9 => 1,
                                10..=15 => 2,
                                _ => 3,
                            };
                            if last_color_idx != Some(color_idx) {
                                screen.extend_from_slice(&color_codes[color_idx]);
                                last_color_idx = Some(color_idx);
                                last_rgb = None;
                            }
                        }
                        ColorMode::Rainbow => {
                            if heat > 0 {
                                let x = i % w;
                                let hue = (rainbow_offset + x as f32 * 360.0 / w as f32)
                                    .rem_euclid(360.0);
                                let [r, g, b] = hue_to_color_bytes(hue, heat);
                                if last_rgb != Some((r, g, b)) {
                                    push_truecolor(&mut screen, r, g, b);
                                    last_rgb = Some((r, g, b));
                                    last_color_idx = None;
                                }
                            } else {
                                last_rgb = None;
                            }
                        }
                    }
                }

                let ch_idx = heat.min(9);
                screen.push(char_bytes[ch_idx][0]);
            }

            if use_color {
                screen.extend_from_slice(b"\x1b[0m");
                last_color_idx = None;
                last_rgb = None;
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
