pub const SHADES: [u32; 4] = [0x00E0F8D0, 0x0088C070, 0x00346856, 0x00081820];

const MODE_HBLANK: u8 = 0;
const MODE_VBLANK: u8 = 1;
const MODE_OAM: u8 = 2;
const MODE_DRAW: u8 = 3;

const OAM_END: u16 = 80;
const DRAW_END: u16 = 252;
const LINE_DOTS: u16 = 456;
const VBLANK_LINE: u8 = 144;
const FRAME_LINES: u8 = 154;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct PPU {
    #[serde(with = "serde_big_array::BigArray")]
    vram: [u8; 0x2000],
    #[serde(with = "serde_big_array::BigArray")]
    oam: [u8; 0xA0],

    lcdc: u8,
    stat: u8,
    scy: u8,
    scx: u8,
    ly: u8,
    lyc: u8,
    bgp: u8,
    obp0: u8,
    obp1: u8,
    wy: u8,
    wx: u8,
    dma: u8,

    dots: u16,
    window_line: u8,
    line_rendered: bool,
    stat_line: bool,
    palette: [u32; 4],
    #[serde(with = "serde_big_array::BigArray")]
    framebuffer: [u32; 160 * 144],
}

impl PPU {
    pub fn new() -> Self {
        PPU {
            vram: [0; 0x2000],
            oam: [0; 0xA0],
            lcdc: 0x91,
            stat: 0x85,
            scy: 0,
            scx: 0,
            ly: 0,
            lyc: 0,
            bgp: 0xFC,
            obp0: 0xFF,
            obp1: 0xFF,
            wy: 0,
            wx: 0,
            dma: 0xFF,
            dots: 0,
            window_line: 0,
            line_rendered: false,
            stat_line: false,
            palette: SHADES,
            framebuffer: [SHADES[0]; 160 * 144],
        }
    }

    pub fn set_palette(&mut self, palette: [u32; 4]) {
        self.palette = palette;
    }

    fn lcd_on(&self) -> bool {
        self.lcdc & 0x80 != 0
    }

    fn mode(&self) -> u8 {
        if !self.lcd_on() {
            return MODE_HBLANK;
        }
        if self.ly >= VBLANK_LINE {
            return MODE_VBLANK;
        }
        if self.dots < OAM_END {
            return MODE_OAM;
        }
        if self.dots < DRAW_END {
            return MODE_DRAW;
        }
        MODE_HBLANK
    }

    pub fn tick(&mut self) -> (bool, bool) {
        if !self.lcd_on() {
            return (false, false);
        }

        let mut vblank = false;
        self.dots += 4;

        if self.dots < LINE_DOTS {
            if self.ly < VBLANK_LINE && self.dots >= OAM_END && !self.line_rendered {
                self.render_scanline();
                self.line_rendered = true;
            }
        } else {
            self.dots -= LINE_DOTS;
            self.line_rendered = false;
            self.ly += 1;
            if self.ly >= FRAME_LINES {
                self.ly = 0;
                self.window_line = 0;
            }
            if self.ly == VBLANK_LINE {
                vblank = true;
            }
        }

        let stat = self.refresh_stat();
        (vblank, stat)
    }

    fn reported_ly(&self) -> u8 {
        if self.ly == 153 && self.dots >= 4 {
            0
        } else {
            self.ly
        }
    }

    fn refresh_stat(&mut self) -> bool {
        let mode = self.mode();
        let coincidence = self.reported_ly() == self.lyc;
        self.stat = (self.stat & 0x78) | 0x80 | mode | if coincidence { 0x04 } else { 0 };

        let oam_source = mode == MODE_OAM || (self.ly == VBLANK_LINE && self.dots < OAM_END);
        let line = (self.stat & 0x40 != 0 && coincidence)
            || (self.stat & 0x20 != 0 && oam_source)
            || (self.stat & 0x10 != 0 && mode == MODE_VBLANK)
            || (self.stat & 0x08 != 0 && mode == MODE_HBLANK);
        let irq = line && !self.stat_line;
        self.stat_line = line;
        irq
    }

    pub fn framebuffer(&self) -> &[u32] {
        &self.framebuffer
    }

    fn render_scanline(&mut self) {
        let ly = self.ly as usize;
        let base = ly * 160;
        let mut bg_ids = [0u8; 160];

        if self.lcdc & 0x01 != 0 {
            let window_active = self.lcdc & 0x20 != 0 && (self.wy as usize) <= ly;
            let mut window_drawn = false;
            for x in 0..160usize {
                let in_window = window_active && (x as i32) >= (self.wx as i32 - 7);
                let color_id = if in_window {
                    window_drawn = true;
                    let wx = (x as i32 - (self.wx as i32 - 7)) as u16;
                    self.tile_pixel(true, wx, self.window_line as u16)
                } else {
                    let bx = (x as u8).wrapping_add(self.scx) as u16;
                    let by = (self.ly).wrapping_add(self.scy) as u16;
                    self.tile_pixel(false, bx, by)
                };
                bg_ids[x] = color_id;
                let shade = (self.bgp >> (color_id * 2)) & 0x3;
                self.framebuffer[base + x] = self.palette[shade as usize];
            }
            if window_drawn {
                self.window_line += 1;
            }
        } else {
            for x in 0..160usize {
                self.framebuffer[base + x] = self.palette[0];
            }
        }

        if self.lcdc & 0x02 != 0 {
            self.render_sprites(ly, base, &bg_ids);
        }
    }

    fn tile_pixel(&self, window: bool, x: u16, y: u16) -> u8 {
        let map_bit = if window { 0x40 } else { 0x08 };
        let map_base: usize = if self.lcdc & map_bit != 0 {
            0x1C00
        } else {
            0x1800
        };
        let tile_col = (x / 8) & 31;
        let tile_row = (y / 8) & 31;
        let tile_index = self.vram[map_base + (tile_row * 32 + tile_col) as usize];
        let tile_addr = if self.lcdc & 0x10 != 0 {
            tile_index as usize * 16
        } else {
            (0x1000_isize + (tile_index as i8 as isize) * 16) as usize
        };
        let row = (y % 8) as usize;
        let lo = self.vram[tile_addr + row * 2];
        let hi = self.vram[tile_addr + row * 2 + 1];
        let bit = 7 - (x % 8);
        (((hi >> bit) & 1) << 1) | ((lo >> bit) & 1)
    }

    fn render_sprites(&mut self, ly: usize, base: usize, bg_ids: &[u8; 160]) {
        let height: i32 = if self.lcdc & 0x04 != 0 { 16 } else { 8 };
        let mut chosen: Vec<usize> = Vec::with_capacity(10);
        for i in 0..40usize {
            let sy = self.oam[i * 4] as i32 - 16;
            if (ly as i32) >= sy && (ly as i32) < sy + height {
                chosen.push(i);
                if chosen.len() == 10 {
                    break;
                }
            }
        }
        chosen.sort_by_key(|&i| (self.oam[i * 4 + 1], i));

        for &i in chosen.iter().rev() {
            let oy = self.oam[i * 4] as i32 - 16;
            let ox = self.oam[i * 4 + 1] as i32 - 8;
            let tile = self.oam[i * 4 + 2];
            let attrs = self.oam[i * 4 + 3];

            let flip_y = attrs & 0x40 != 0;
            let flip_x = attrs & 0x20 != 0;
            let palette = if attrs & 0x10 != 0 {
                self.obp1
            } else {
                self.obp0
            };
            let behind_bg = attrs & 0x80 != 0;

            let mut row = ly as i32 - oy;
            if flip_y {
                row = height - 1 - row;
            }
            let tile_index = if height == 16 { tile & 0xFE } else { tile };
            let addr = tile_index as usize * 16 + row as usize * 2;
            let lo = self.vram[addr];
            let hi = self.vram[addr + 1];

            for px in 0..8i32 {
                let bit = if flip_x { px } else { 7 - px };
                let color_id = (((hi >> bit) & 1) << 1) | ((lo >> bit) & 1);
                if color_id == 0 {
                    continue;
                }
                let sx = ox + px;
                if sx < 0 || sx >= 160 {
                    continue;
                }
                if behind_bg && bg_ids[sx as usize] != 0 {
                    continue;
                }
                let shade = (palette >> (color_id * 2)) & 0x3;
                self.framebuffer[base + sx as usize] = self.palette[shade as usize];
            }
        }
    }

    pub fn vram_read(&self, addr: u16) -> u8 {
        self.vram[(addr - 0x8000) as usize]
    }
    pub fn vram_write(&mut self, addr: u16, val: u8) {
        self.vram[(addr - 0x8000) as usize] = val;
    }
    pub fn oam_read(&self, addr: u16) -> u8 {
        self.oam[(addr - 0xFE00) as usize]
    }
    pub fn oam_write(&mut self, addr: u16, val: u8) {
        self.oam[(addr - 0xFE00) as usize] = val;
    }
    pub fn dma_write(&mut self, index: usize, val: u8) {
        self.oam[index] = val;
    }

    pub fn read(&self, addr: u16) -> u8 {
        match addr {
            0xFF40 => self.lcdc,
            0xFF41 => self.stat | 0x80,
            0xFF42 => self.scy,
            0xFF43 => self.scx,
            0xFF44 => self.reported_ly(),
            0xFF45 => self.lyc,
            0xFF46 => self.dma,
            0xFF47 => self.bgp,
            0xFF48 => self.obp0,
            0xFF49 => self.obp1,
            0xFF4A => self.wy,
            0xFF4B => self.wx,
            _ => 0xFF,
        }
    }

    pub fn write(&mut self, addr: u16, val: u8) {
        match addr {
            0xFF40 => {
                let was_on = self.lcd_on();
                self.lcdc = val;
                if was_on && !self.lcd_on() {
                    self.ly = 0;
                    self.dots = 0;
                    self.window_line = 0;
                    self.line_rendered = false;
                    self.stat_line = false;
                    self.framebuffer.fill(self.palette[0]);
                }
            }
            0xFF41 => self.stat = (self.stat & 0x87) | (val & 0x78),
            0xFF42 => self.scy = val,
            0xFF43 => self.scx = val,
            0xFF44 => {}
            0xFF45 => self.lyc = val,
            0xFF46 => self.dma = val,
            0xFF47 => self.bgp = val,
            0xFF48 => self.obp0 = val,
            0xFF49 => self.obp1 = val,
            0xFF4A => self.wy = val,
            0xFF4B => self.wx = val,
            _ => {}
        }
    }
}
