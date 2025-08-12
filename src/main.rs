mod memory;
use memory::Memory;

fn main() {
    // let file_path = "./Tetris.gb";
    let file_path = "./cpu_instrs.gb";
    let rom = std::fs::read(file_path).unwrap();
    let memory = Memory::new(&rom);

    // read header

    // create cpu

    let opcode = rom[0];
    println!("opcode: 0b{:08b} - 0x{:X}", opcode, opcode);

    let logo: [u8; 48] = read_nintendo_logo(&rom);
    print_logo_ascii(&logo);
}

fn read_nintendo_logo(rom: &[u8]) -> [u8; 48] {
    let logo_start = 0x0104;
    let logo_end = 0x0133;
    let mut logo: [u8; 48] = [0; 48];
    logo.copy_from_slice(&rom[logo_start..=logo_end]);
    logo
}

fn print_logo_ascii(logo: &[u8; 48]) {
    for str in logo {
        print!("{:02X} ", str);
    }
}
