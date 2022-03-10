use std::env;
mod emulator;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: rschip8 <PROGRAM>");
        return;
    }
    if let Err(e) = emulator::run(&args[1]) {
        eprintln!("Error: {}", e);
    }
}
