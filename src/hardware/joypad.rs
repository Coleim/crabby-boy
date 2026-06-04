pub const BUTTON_A: u8 = 1 << 0;
pub const BUTTON_B: u8 = 1 << 1;
pub const BUTTON_SELECT: u8 = 1 << 2;
pub const BUTTON_START: u8 = 1 << 3;
pub const BUTTON_RIGHT: u8 = 1 << 4;
pub const BUTTON_LEFT: u8 = 1 << 5;
pub const BUTTON_UP: u8 = 1 << 6;
pub const BUTTON_DOWN: u8 = 1 << 7;

pub struct Joypad {
    select: u8,
    pressed: u8,
}

impl Joypad {
    pub fn new() -> Self {
        Joypad { select: 0x30, pressed: 0 }
    }

    pub fn read(&self) -> u8 {
        let mut low = 0x0F;
        if self.select & 0x20 == 0 {
            low &= !(self.pressed & 0x0F);
        }
        if self.select & 0x10 == 0 {
            low &= !((self.pressed >> 4) & 0x0F);
        }
        0xC0 | (self.select & 0x30) | (low & 0x0F)
    }

    pub fn write(&mut self, val: u8) {
        self.select = val & 0x30;
    }

    pub fn set_pressed(&mut self, pressed: u8) {
        self.pressed = pressed;
    }
}
