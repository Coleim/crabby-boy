pub struct Joypad {
    p1: u8,
}

impl Joypad {
    pub fn new() -> Self {
        Joypad { p1: 0 }
    }

    pub fn read(&self) -> u8 {
        self.p1
    }

    pub fn write(&mut self, val: u8) {
        self.p1 = val;
    }
}
