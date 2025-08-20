mod header;
use header::CartdrigeHeader;

fn main() -> Result<(), String> {
    // let file_path = "./Tetris.gb";
    let file_path = "./cpu_instrs.gb";
    let rom: Vec<u8> = std::fs::read(file_path).unwrap();
    let header: CartdrigeHeader = CartdrigeHeader::new(&rom);
    header.print();

    header.is_valid()?;

    // create cpu

    Ok(())
}
