use crate::cpu::{audio::Audio, serial::Serial, timer_and_divider::TimerAndDivider};

pub struct IORegisters {
    serial: Serial,
    timer_and_diviers: TimerAndDivider, // $FF04	$FF07	DMG	Timer and divider
    interrupt_flag: u8,                 // FF0F — IF: Interrupt flag
    interrupt_enable: u8,               // FFFF — IE: Interrupt enable
    audio: Audio,                       // $FF10	$FF26	DMG	Audio
}

impl IORegisters {
    pub fn new() -> Self {
        IORegisters {
            serial: Serial::new(),
            timer_and_diviers: TimerAndDivider::new(),
            interrupt_enable: 0,
            interrupt_flag: 0,
            audio: Audio::new(),
        }
    }
    pub fn read(&self, addr: u16) -> u8 {
        match addr {
            0xFF01..=0xFF02 => self.serial.read(addr),
            0xFF04..=0xFF07 => self.timer_and_diviers.read(addr),
            0xFF0F => self.interrupt_flag,
            0xFF10..=0xFF26 => self.audio.read(addr),
            0xFFFF => self.interrupt_enable,
            _ => {
                println!("[IOREG] READ NOT IMPLEMENTED FOR ADDR: {:02X}", addr);
                // std::panic::panic_any("[IOREG] READ NOT IMPLEMENTED");
                0x00
            }
        }
    }

    pub fn write(&mut self, addr: u16, val: u8) {
        match addr {
            0xFF01..=0xFF02 => self.serial.write(addr, val),
            0xFF04..=0xFF07 => self.timer_and_diviers.write(addr, val),
            0xFF0F => self.interrupt_flag = val,
            0xFF10..=0xFF26 => self.audio.write(addr, val),
            0xFFFF => self.interrupt_enable = val,
            _ => {
                println!(
                    "[IOREG] WRITE NOT IMPLEMENTED FOR ADDR: {:02X} - val: {:02X}",
                    addr, val
                );
                // std::panic::panic_any("[IOREG] WRITE NOT IMPLEMENTED");
            }
        }
    }
}
