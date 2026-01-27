mod cpu;
mod header;
mod mappings;

use std::default;

use cpu::CPU;
use header::CartdrigeHeader;

fn main() -> Result<(), String> {
    // let file_path = "./Tetris.gb";
    let file_path = "./cpu_instrs.gb";
    let mut mem: Vec<u8> = std::fs::read(file_path).unwrap();
    let header: CartdrigeHeader = CartdrigeHeader::new(&mem);
    header.print();

    header.is_valid()?;

    // create cpu
    let mut cpu: CPU = CPU::new();

    while (cpu.execute(&mut mem)) {}

    Ok(())
}
