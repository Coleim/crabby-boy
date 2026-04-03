use crate::hardware::{apu::APU, joypad::Joypad, ppu::PPU, serial::Serial, timer::Timer};

// IOHandler {
//     apu: APU,
//     ppu: PPU,
//     timer: Timer,
//     joypad: Joypad,
//     interrupt_flag: u8
// }

pub struct IOBridge {
    joypad: Joypad,
    serial: Serial,
    timer: Timer,       // $FF04	$FF07	DMG	Timer and divider
    interrupt_flag: u8, // FF0F — IF: Interrupt flag
    audio: APU,         // $FF10	$FF26	DMG	Audio
    ppu: PPU,           // $FF40 - $FF4B - LCD Control, Status, Position, Scrolling, and Palettes
    key1_spd: u8,       // KEY1/SPD (CGB Mode only): Prepare speed switch
}

impl IOBridge {
    pub fn new() -> Self {
        IOBridge {
            joypad: Joypad::new(),
            serial: Serial::new(),
            timer: Timer::new(),
            interrupt_flag: 0,
            audio: APU::new(),
            ppu: PPU::new(),
            key1_spd: 0,
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
            0xFF00 => self.joypad.read(),
            0xFF01..=0xFF02 => self.serial.read(addr),
            0xFF04..=0xFF07 => self.timer.read(addr),
            0xFF0F => self.interrupt_flag,
            0xFF10..=0xFF26 => self.audio.read(addr),
            0xFF40..=0xFF4B => self.ppu.read(addr),
            0xFF4D => self.key1_spd,
            _ => {
                println!("[IOREG] READ NOT IMPLEMENTED FOR ADDR: {:02X}", addr);
                0x00
            }
        }
    }

    pub fn write(&mut self, addr: u16, val: u8) {
        // println!(" WRITE IO addr: {:02x}, val: {:02x}", addr, val);
        match addr {
            0xFF00 => self.joypad.write(val),
            0xFF01..=0xFF02 => self.serial.write(addr, val),
            0xFF04..=0xFF07 => self.timer.write(addr, val),
            0xFF0F => self.interrupt_flag = val,
            0xFF10..=0xFF26 => self.audio.write(addr, val),
            0xFF40..=0xFF4B => self.ppu.write(addr, val),
            0xFF4D => self.key1_spd = val,
            _ => {
                println!(
                    "[IOREG] WRITE NOT IMPLEMENTED FOR ADDR: {:02X}, VAL: {:02X}",
                    addr, val
                )
            }
        }
    }
}
