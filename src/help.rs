pub fn print_help(program_name: &str) {
    println!(
"
Usage:
  {program_name} [options]

Options:
  -f,    --fps <number>   - set FPS (default: 30, range: 15-120)
  -t,    --theme <name>   - set theme (default: std)
  -n-c,  --no-color       - disable colors (ASCII only)

Available themes:
  ash         - monochrome smoke & gray fire
  blue        - blue neon fire
  classic     - alternative classic fire
  copper      - turquoise copper-oxide flame
  crimson     - aggressive crimson-red fire
  ember       - glowing amber coals
  emerald     - deep emerald chemical fire
  forest      - mystical green fire
  ghost       - ethereal violet magic flame
  gold        - luxury metallic golden shimmer
  ice         - ice fire
  magma       - viscous glow of molten lava
  nebula      - cosmic pink & blue fire
  pink        - pink neon fire
  plasma      - electric indigo plasma
  rainbow     - multicolor spectrum fire
  solar       - blinding white-hot solar flares
  std         - classic fire
  sulfur      - ghostly blue flame

  custom      - use a user-defined theme (see format below)

Custom Theme Format:
  custom:#hex.#hex.#hex.#hex
  Provide 1 to 4 HEX colors separated by dots (e.g., custom:#ff0000.#00ff00)

Examples:
  {program_name} -f 60
  {program_name} --theme copper --fps 45
  {program_name} -t custom:#ff0055.#ffcc00.#ffffff

Controls:
  ESC or Ctrl+C - exit
"
    );
}