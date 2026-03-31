use crate::hardware::{apu::APU, serial::Serial, timer::Timer};

// IOHandler {
//     apu: APU,
//     ppu: PPU,
//     timer: Timer,
//     joypad: Joypad,
//     interrupt_flag: u8
// }

pub struct IOBridge {
    serial: Serial,
    timer: Timer,       // $FF04	$FF07	DMG	Timer and divider
    interrupt_flag: u8, // FF0F — IF: Interrupt flag
    audio: APU,         // $FF10	$FF26	DMG	Audio
}

impl IOBridge {
    pub fn new() -> Self {
        IOBridge {
            serial: Serial::new(),
            timer: Timer::new(),
            interrupt_flag: 0,
            audio: APU::new(),
        }
    }

    pub fn tick(&mut self, cycles: u8) {
        if self.timer.tick(cycles) {
            self.interrupt_flag |= 0b0000_0100;
        }
    }

    pub fn clear_if(&mut self, if_bit: u8) {
        self.interrupt_flag = self.interrupt_flag & !if_bit;
    }
    pub fn get_if(&self) -> u8 {
        self.interrupt_flag
    }

    pub fn read(&self, addr: u16) -> u8 {
        match addr {
            0xFF01..=0xFF02 => self.serial.read(addr),
            0xFF04..=0xFF07 => self.timer.read(addr),
            0xFF0F => self.interrupt_flag,
            0xFF10..=0xFF26 => self.audio.read(addr),
            0xFF40 => 0x91, // LCDC - LCD enabled, BG enabled
            0xFF41 => 0x85, // STAT - mode 1 (VBlank)
            0xFF42 => 0x00, // SCY
            0xFF43 => 0x00, // SCX
            0xFF44 => 0x00, // LY: 0-153
            0xFF47 => 0xFC, // BGP
            _ => std::panic!("[IOREG] READ NOT IMPLEMENTED FOR ADDR: {:02X}", addr),
        }
    }

    pub fn write(&mut self, addr: u16, val: u8) {
        // println!(" WRITE IO addr: {:02x}, val: {:02x}", addr, val);
        match addr {
            0xFF01..=0xFF02 => self.serial.write(addr, val),
            0xFF04..=0xFF07 => self.timer.write(addr, val),
            0xFF0F => {
                // println!("WRITE IF ← 0x{:02X}", val);
                // println!("WRITE IF ← 0x{:02X} (from where?)", val);
                self.interrupt_flag = val;
            }
            0xFF10..=0xFF26 => self.audio.write(addr, val),
            0xFF40..=0xFF4B => {} // PPU registers - silently ignore for now
            _ => {
                // println!(
                //     "[IOREG] WRITE NOT IMPLEMENTED FOR ADDR: {:02X} - val: {:02X}",
                //     addr, val
                // );
                // std::panic::panic_any("[IOREG] WRITE NOT IMPLEMENTED");
            }
        }
    }
}
