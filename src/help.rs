pub fn print_help(program_name: &str) {
    println!(
"
Usage:
  {program_name} [options]

Options:
  -f,    --fps <number>   - set FPS (default: 30, range: 15-120)
  -n-c,  --no-color       - disable colors (ASCII only)
  -t,    --theme <name>   - set theme (default: std)
  -v,    --version        - show version info

Available themes:
  ash     - monochrome smoke & gray fire
  aurora  - shifting green-teal northern lights
  blue    - blue neon fire
  classic - alternative classic fire
  copper  - turquoise copper-oxide flame
  crimson - aggressive crimson-red fire
  dusk    - warm orange-purple sunset fire
  ember   - glowing amber coals
  emerald - deep emerald chemical fire
  forest  - mystical green fire
  frost   - cold blue-white icy fire
  ghost   - ethereal violet magic flame
  gold    - luxury metallic golden shimmer
  ice     - ice fire
  magma   - viscous glow of molten lava
  nebula  - cosmic pink & blue fire
  pink    - pink neon fire
  plasma  - electric indigo plasma
  rainbow - multicolor spectrum fire
  sakura  - soft pink cherry blossom fire
  solar   - blinding white-hot solar flares
  std     - classic fire
  sulfur  - ghostly blue flame

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