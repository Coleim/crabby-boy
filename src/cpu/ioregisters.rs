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
            println!("[IOREG] READ NOT IMPLEMENTED FOR ADDR: {:02X}", addr);
            std::panic::panic_any("[IOREG] READ NOT IMPLEMENTED");
            0x00
        }
    }

    pub fn write(&mut self, addr: u16, val: u8) {
        if matches!(addr, 0xFF01..=0xFF02) {
            self.serial.write(addr, val);
        } else {
            println!("[IOREG] WRITE NOT IMPLEMENTED FOR ADDR: {:02X}", addr);
            std::panic::panic_any("[IOREG] WRITE NOT IMPLEMENTED");
        }
    }
}
