use crate::cpu::serial::Serial;

pub struct IORegisters {
    serial: Serial,
}

impl IORegisters {
    pub fn new() -> Self {
        IORegisters {
            serial: Serial::new(),
        }
    }
    pub fn read(&self, addr: u16) -> u8 {
        if matches!(addr, 0xFF01..=0xFF02) {
            self.serial.read(addr)
        } else {
            0x00
        }
    }

    pub fn write(&mut self, addr: u16, val: u8) {
        if matches!(addr, 0xFF01..=0xFF02) {
            self.serial.write(addr, val);
        } else {
        }
    }
}
