pub struct Serial {
    sb: u8, // data register (0xFF01)
    sc: u8, // control register (0xFF02)
}

impl Serial {
    pub fn new() -> Self {
        Serial { sb: 0, sc: 0 }
    }

    pub fn read(&self, addr: u16) -> u8 {
        match addr {
            0xFF01 => self.sb,
            0xFF02 => self.sc,
            _ => 0,
        }
    }

    pub fn write(&mut self, addr: u16, val: u8) {
        match addr {
            0xFF01 => self.sb = val,
            0xFF02 => {
                self.sc = val;
                if self.sc == 0x81 {
                    print!("{}", self.sb as char);
                    self.sc = 0x00;
                }
            }
            _ => {}
        }
    }
}

// if addr == 0xFF02 && value == 0x81 {
//     serial.start_transfer();
// }
// And inside serial.start_transfer():
// let byte = bus.read(0xFF01);
// output(byte);
// bus.write(0xFF02, 0x01); // bit 7 cleared
