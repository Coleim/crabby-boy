use crate::bus::iobridge::IOBridge;

// Start	End	Description	Notes
// 0000	3FFF	16 KiB ROM bank 00	From cartridge, usually a fixed bank
// 4000	7FFF	16 KiB ROM Bank 01–NN	From cartridge, switchable bank via mapper (if any)
// 8000	9FFF	8 KiB Video RAM (VRAM)	In CGB mode, switchable bank 0/1
// A000	BFFF	8 KiB External RAM	From cartridge, switchable bank if any
// C000	CFFF	4 KiB Work RAM (WRAM)
// D000	DFFF	4 KiB Work RAM (WRAM)	In CGB mode, switchable bank 1–7
// E000	FDFF	Echo RAM (mirror of C000–DDFF)	Nintendo says use of this area is prohibited.
// FE00	FE9F	Object attribute memory (OAM)
// FEA0	FEFF	Not Usable	Nintendo says use of this area is prohibited.
// FF00	FF7F	I/O Registers
// FF80	FFFE	High RAM (HRAM)
// FFFF	FFFF	Interrupt Enable register (IE)
pub struct Bus {
    rom: Vec<u8>, // full ROM, any size
    vram: [u8; 0x2000],
    eram: [u8; 0x2000],
    wram: [u8; 0x2000],
    oam: [u8; 0x2000],
    io: IOBridge,
    // serial: Serial,
    hram: [u8; 0x2000],
    ie: u8,
}

impl Bus {
    pub fn new(rom_data: Vec<u8>) -> Self {
        Bus {
            rom: rom_data,
            vram: [0; 0x2000],
            eram: [0; 0x2000],
            wram: [0; 0x2000],
            oam: [0; 0x2000],
            io: IOBridge::new(),
            hram: [0; 0x2000],
            ie: 0,
        }
    }
    pub fn tick(&mut self, cycles: u8) {
        self.io.tick(cycles);
    }
    pub fn get_rom(&self) -> &Vec<u8> {
        &self.rom
    }
    pub fn clear_if(&mut self, mask: u8) {
        self.io.clear_if(mask);
    }
    pub fn get_io(&self) -> &IOBridge {
        &self.io
    }
    pub fn get_ie(&self) -> u8 {
        self.ie
    }
    pub fn read(&self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x3FFF => self.rom[addr as usize], // Banking 0
            // 0x4000..=0x7FFF => self.rom[addr as usize], // For now no banking // Banking 1
            0x4000..=0x7FFF => self.rom[addr as usize], // For now no banking // Banking 1
            0x8000..=0x9FFF => self.vram[(addr - 0x8000) as usize],
            0xA000..=0xBFFF => self.eram[(addr - 0xA000) as usize],
            0xC000..=0xDFFF => self.wram[(addr - 0xC000) as usize],
            0xE000..=0xFDFF => {
                std::panic::panic_any("Not Usable	Nintendo says use of this area is prohibited.");
            }
            0xFE00..=0xFE9F => self.oam[(addr - 0xFE00) as usize],
            0xFEA0..=0xFEFF => {
                std::panic::panic_any("Not Usable	Nintendo says use of this area is prohibited.")
            }

            0xFF00..=0xFF7F => self.io.read(addr),
            0xFF80..=0xFFFE => self.hram[(addr - 0xFF80) as usize],
            0xFFFF => self.ie,
        }
    }

    pub fn write(&mut self, addr: u16, val: u8) {
        //TODO: https://doc.rust-lang.org/rust-by-example/conversion/from_into.html
        match addr {
            0x0000..=0x7FFF => {} // TODO : Implement MBC switch
            0x8000..=0x9FFF => self.vram[(addr - 0x8000) as usize] = val,
            0xA000..=0xBFFF => self.eram[(addr - 0xA000) as usize] = val,
            0xC000..=0xDFFF => {
                // println!("WRITE TO WRAM {:02x}, val: {:02x}", addr, val);
                self.wram[(addr - 0xC000) as usize] = val;
            }
            0xE000..=0xFDFF => self.wram[(addr - 0xE000) as usize] = val, // echo of WRAM
            0xFE00..=0xFE9F => self.oam[(addr - 0xFE00) as usize] = val,
            0xFF00..=0xFF7F => self.io.write(addr, val),
            0xFF80..=0xFFFE => {
                // println!(
                //     "HRAM OUTPUT 0x{:04X} = 0x{:02X} '{}'",
                //     addr, val, val as char
                // );
                self.hram[(addr - 0xFF80) as usize] = val;
            }
            0xFFFF => self.ie = val,
            _ => {
                println!("[BUS] WRITE NOT IMPLEMENTED FOR ADDR: {:02X}", addr);
                std::panic::panic_any("[BUS] WRITE NOT IMPLEMENTED");
            }
        }
    }
}

// let rom_data = std::fs::read(file_path).unwrap();
// let mut bus = Bus {
//     rom: rom_data,
//     current_bank: 1,   // if needed
//     wram: [0; 0x2000],
//     serial: Serial::new(),
// };
