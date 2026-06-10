mod audio;
mod bus;
mod cartridge;
mod cpu;
mod emulator;
mod hardware;
mod io;

use crate::emulator::CrabbyBoy;

fn main() -> Result<(), String> {
    let file_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "./tests/Tetris.gb".to_string());

    let mut crabby = CrabbyBoy::new();
    crabby.run(&file_path)
}
