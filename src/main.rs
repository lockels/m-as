mod cpu;
mod process;
mod tui;
use color_eyre::Result;

pub fn main() -> Result<()> {
    // process::main();
    // cpu::main();
    tui::main()
}
