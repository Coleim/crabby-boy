#[derive(serde::Serialize, serde::Deserialize)]
pub struct Serial {
    sb: u8,
    sc: u8,
    irq: bool,
    pub serial_output: Vec<u8>,
}

impl Serial {
    pub fn new() -> Self {
        Serial {
            sb: 0,
            sc: 0,
            irq: false,
            serial_output: Vec::new(),
        }
    }

    #[cfg(test)]
    pub fn serial_output(&self) -> &Vec<u8> {
        &self.serial_output
    }

    pub fn take_irq(&mut self) -> bool {
        let irq = self.irq;
        self.irq = false;
        irq
    }

    pub fn read(&self, addr: u16) -> u8 {
        match addr {
            0xFF01 => self.sb,
            0xFF02 => self.sc | 0x7E,
            _ => 0xFF,
        }
    }

    pub fn write(&mut self, addr: u16, val: u8) {
        match addr {
            0xFF01 => self.sb = val,
            0xFF02 => {
                self.sc = val;
                if val & 0x81 == 0x81 {
                    print!("{}", self.sb as char);
                    self.serial_output.push(self.sb);
                    self.sb = 0xFF;
                    self.sc &= !0x80;
                    self.irq = true;
                }
            }
            _ => {}
        }
    }
}
