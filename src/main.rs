// Fire-CLI || Github: https://github.com/horizonwiki/fire || v0.1.3 || 2026-04-13
use std::{env, io};

mod terminal;
mod theme;
mod simulation;
mod renderer;
mod input;
mod help;

use terminal::Terminal;
use theme::{Theme, ColorMode};
use help::print_help;
use renderer::run_main_loop;
use fire_cli::{VERSION, BUILD_DATE};

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    let program_name = args.get(0).map(|s| s.as_str()).unwrap_or("fire-cli");

    let mut theme = Theme::std();
    let mut color_mode = ColorMode::Theme;
    let mut fps: u32 = 30;
    let mut use_color = true;
    let mut i = 1;
    
    while i < args.len() {
        match args[i].to_lowercase().as_str() {
            "-t" | "--theme" => {
                if i + 1 < args.len() {
                    let theme_arg = args[i + 1].to_lowercase();
                    let theme_str = theme_arg.as_str();
                    match theme_str {
                        "ash"     => theme = Theme::ash(),
                        "aurora"  => theme = Theme::aurora(),
                        "blue"    => theme = Theme::blue(),
                        "classic" => theme = Theme::classic(),
                        "copper"  => theme = Theme::copper(),
                        "crimson" => theme = Theme::crimson(),
                        "dusk"    => theme = Theme::dusk(),
                        "ember"   => theme = Theme::ember(),
                        "emerald" => theme = Theme::emerald(),
                        "forest"  => theme = Theme::forest(),
                        "frost"   => theme = Theme::frost(),
                        "ghost"   => theme = Theme::ghost(),
                        "gold"    => theme = Theme::gold(),
                        "ice"     => theme = Theme::ice(),
                        "magma"   => theme = Theme::magma(),
                        "nebula"  => theme = Theme::nebula(),
                        "pink"    => theme = Theme::pink(),
                        "plasma"  => theme = Theme::plasma(),
                        "rainbow" => color_mode = ColorMode::Rainbow,
                        "sakura"  => theme = Theme::sakura(),
                        "solar"   => theme = Theme::solar(),
                        "std"     => theme = Theme::std(),
                        "sulfur"  => theme = Theme::sulfur(),

                        "custom"  => {
                            eprintln!("Missing colors after 'custom:'. Example: -t custom:#ff0055.#ffcc00.#ffffff");
                            print_help(program_name);
                            return Ok(());
                        }
                        s if s.starts_with("custom:") => {
                            let input = &s["custom:".len()..];
                            match theme::parse_custom_theme(input) {
                                Some(t) => { theme = t; color_mode = ColorMode::Theme; }
                                None => {
                                    eprintln!("Invalid custom theme. Example: -t custom:#ff0055.#ffcc00.#ffffff");
                                    print_help(program_name);
                                    return Ok(());
                                }
                            }
                        }
                        _ => {
                            eprintln!("Invalid theme: {}", args[i + 1]);
                            print_help(program_name);
                            return Ok(());
                        }
                    }
                    i += 1;
                } else {
                    eprintln!("Missing theme name");
                    print_help(program_name);
                    return Ok(());
                }
            }
            "-f" | "--fps" => {
                if i + 1 < args.len() {
                    if let Ok(f) = args[i + 1].parse::<u32>() {
                        fps = f.clamp(15, 120);
                        i += 1;
                    } else {
                        eprintln!("Invalid FPS value: {}", args[i + 1]);
                        print_help(program_name);
                        return Ok(());
                    }
                } else {
                    eprintln!("Missing FPS value");
                    print_help(program_name);
                    return Ok(());
                }
            }
            "--no-color" | "-n-c" | "--nocolor" => use_color = false,
            "-h" | "--help"  => {
                print_help(program_name);
                return Ok(());
            }
            "-v" | "--version" => {
                println!("fire-cli v{} ({}-{}) built {}",
                VERSION,
                std::env::consts::ARCH,
                std::env::consts::OS,
                BUILD_DATE
                );
                return Ok(());
            }
            _ => {
                eprintln!("Invalid option: {}", args[i]);
                print_help(program_name);
                return Ok(());
            }
        }
        i += 1;
    }

    let _term = Terminal::new()?;
    
    std::panic::set_hook(Box::new(|info| {
        terminal::restore_terminal();
        eprintln!("panic: {}", info);
    }));
    
    run_main_loop(&theme, color_mode, fps, use_color)?;

    Ok(())
}