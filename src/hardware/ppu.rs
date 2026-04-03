pub struct PPU {
    lcd_control: u8, // FF40 — LCDC: LCD control
    scy: u8,         // FF42–FF43 — SCY, SCX
    scx: u8,
    lcd_y_coord: u8, // FF44 — LY: LCD Y coordinate [read-only]
    bgp: u8,         // FF47 — BGP
}

// FF40 — LCDC: LCD control
//
// LCDC is the main LCD Control register. Its bits toggle what elements are displayed on the screen, and how.
// 7	6	5	4	3	2	1	0
// LCD & PPU enable	Window tile map	Window enable	BG & Window tiles	BG tile map	OBJ size	OBJ enable	BG & Window enable / priority
//
//     LCD & PPU enable: 0 = Off; 1 = On
//     Window tile map area: 0 = 9800–9BFF; 1 = 9C00–9FFF
//     Window enable: 0 = Off; 1 = On
//     BG & Window tile data area: 0 = 8800–97FF; 1 = 8000–8FFF
//     BG tile map area: 0 = 9800–9BFF; 1 = 9C00–9FFF
//     OBJ size: 0 = 8×8; 1 = 8×16
//     OBJ enable: 0 = Off; 1 = On
//     BG & Window enable / priority [Different meaning in CGB Mode]: 0 = Off; 1 = On

impl PPU {
    pub fn new() -> Self {
        PPU {
            lcd_control: 0,
            scy: 0,
            scx: 0,
            lcd_y_coord: 0,
            bgp: 0,
        }
    }

    pub fn read(&self, addr: u16) -> u8 {
        match addr {
            0xFF40 => self.lcd_control,
            0xFF42 => self.scy,
            0xFF43 => self.scx,
            0xFF44 => self.lcd_y_coord,
            0xFF47 => self.bgp,
            _ => {
                println!("[PPU] READ NOT IMPLEMENTED FOR ADDR: {:02X}", addr);
                0x00
            }
        }
    }

    pub fn write(&mut self, addr: u16, val: u8) {
        match addr {
            0xFF40 => self.lcd_control = val,
            0xFF42 => self.scy = val,
            0xFF43 => self.scx = val,
            0xFF44 => self.lcd_y_coord = val,
            0xFF47 => self.bgp = val,
            _ => {
                println!(
                    "[PPU] WRITE NOT IMPLEMENTED FOR ADDR: {:02X}, VAL: {:02X}",
                    addr, val
                )
            }
        }
    }
}
