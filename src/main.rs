mod cpu;
mod process;

mod tui;
use color_eyre::Result;

pub fn main() -> Result<()> {
    // let mut processes = process::get_all_processes();
    // process::sort_by_memory(&mut processes);
    //
    // println!("=== SYSTEM PROCESSES ===");
    // println!("{}", "-".repeat(100));
    //
    // for process in processes.iter().take(20) {
    //     // Show top 20 by CPU
    //     println!("{}", process);
    cpu::main();
    tui::main()
}
