mod cpu;
mod memory;
mod network;
mod process;
mod tui;
use color_eyre::Result;

pub fn main() -> Result<()> {
    // process::main();
    // cpu::main();
    // memory::main();
    tui::main()
}
