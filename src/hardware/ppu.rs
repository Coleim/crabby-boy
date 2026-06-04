const SHADES: [u32; 4] = [0x00E0F8D0, 0x0088C070, 0x00346856, 0x00081820];

pub struct PPU {
    lcd_control: u8, // FF40 — LCDC: LCD control
    scy: u8,         // FF42–FF43 — SCY, SCX
    scx: u8,
    lcd_y_coord: u8, // FF44 — LY: LCD Y coordinate [read-only]
    bgp: u8,         // FF47 — BGP
    dots: u16,       // T-cycle counter
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
            dots: 0,
        }
    }

    pub fn tick(&mut self) -> bool {
        self.dots += 4;
        if self.dots >= 456 {
            self.dots -= 456;
            self.lcd_y_coord += 1;
            if self.lcd_y_coord >= 154 {
                self.lcd_y_coord = 0;
            }
            if self.lcd_y_coord == 144 {
                return true;
            }
        }
        false
    }

    pub fn render_background(&self, vram: &[u8], framebuffer: &mut [u32]) {
        if self.lcd_control & 0x80 == 0 {
            framebuffer.fill(SHADES[0]);
            return;
        }
        let map_base: u16 = if self.lcd_control & 0x08 != 0 { 0x9C00 } else { 0x9800 };
        let unsigned_tiles = self.lcd_control & 0x10 != 0;
        for screen_y in 0..144usize {
            let bg_y = (screen_y as u8).wrapping_add(self.scy);
            let tile_row = (bg_y / 8) as u16;
            let line = (bg_y % 8) as u16;
            for screen_x in 0..160usize {
                let bg_x = (screen_x as u8).wrapping_add(self.scx);
                let tile_col = (bg_x / 8) as u16;
                let map_addr = map_base + tile_row * 32 + tile_col;
                let tile_index = vram[(map_addr - 0x8000) as usize];
                let tile_addr = if unsigned_tiles {
                    0x8000 + tile_index as u16 * 16
                } else {
                    0x9000u16.wrapping_add((tile_index as i8 as i16 * 16) as u16)
                };
                let row_addr = (tile_addr + line * 2 - 0x8000) as usize;
                let lo = vram[row_addr];
                let hi = vram[row_addr + 1];
                let bit = 7 - (bg_x % 8);
                let color_id = (((hi >> bit) & 1) << 1) | ((lo >> bit) & 1);
                let shade = (self.bgp >> (color_id * 2)) & 0x3;
                framebuffer[screen_y * 160 + screen_x] = SHADES[shade as usize];
            }
        }
    }

    pub fn read(&self, addr: u16) -> u8 {
        match addr {
            0xFF40 => self.lcd_control,
            0xFF42 => self.scy,
            0xFF43 => self.scx,
            0xFF44 => self.lcd_y_coord,
            0xFF47 => self.bgp,
            _ => 0xFF,
        }
    }

    pub fn write(&mut self, addr: u16, val: u8) {
        match addr {
            0xFF40 => self.lcd_control = val,
            0xFF42 => self.scy = val,
            0xFF43 => self.scx = val,
            0xFF44 => self.lcd_y_coord = val,
            0xFF47 => self.bgp = val,
            _ => {}
        }
    }
}
