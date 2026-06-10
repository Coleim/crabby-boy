#![allow(dead_code, non_upper_case_globals)]


use std::collections::HashMap;


const COND_NZ: u8 = 0;
const COND_Z: u8 = 1;
const COND_NC: u8 = 2;
const COND_C: u8 = 3;

const LCDC: u8 = 0x40;
const BGP: u8 = 0x47;
const OBP0: u8 = 0x48;
const LY: u8 = 0x44;
const JOYP: u8 = 0x00;

const NR52: u8 = 0x26; const NR51: u8 = 0x25; const NR50: u8 = 0x24; const NR30: u8 = 0x1A; const NR32: u8 = 0x1C; const NR33: u8 = 0x1D; const NR34: u8 = 0x1E; const WAVE_RAM: u16 = 0xFF30;

const PLAYER_X: u16 = 0xC000;
const Y_HI: u16 = 0xC001; const Y_LO: u16 = 0xC002; const VY_HI: u16 = 0xC003; const VY_LO: u16 = 0xC004;
const ON_GROUND: u16 = 0xC005;
const CHUNK_CNT: u16 = 0xC006; const SPTR_LO: u16 = 0xC007; const SPTR_HI: u16 = 0xC008;

const DIV: u8 = 0x04;
const TIMA: u8 = 0x05;
const TMA: u8 = 0x06;
const TAC: u8 = 0x07;
const IE: u16 = 0xFFFF;

const GROUND_PIX: u8 = 104; const GRAVITY: u16 = 0x0040; const JUMP_VEL: u16 = 0xFC00; 
const TILE_BLANK: u8 = 0;
const TILE_GROUND: u8 = 1;
const TILE_CAT_L: u8 = 2; const TILE_CAT_R: u8 = 4; const TILE_LOGO_BASE: u8 = 6; const TILE_PROMPT_BASE: u8 = 38; 
const LOGO_WORD: &str = "CatAnnaDev";
const LOGO_UNIQUE: &str = "CatAnDev"; const PROMPT_LETTERS: &str = "PRESATOMW";


const NINTENDO_LOGO: [u8; 48] = [
    0xCE, 0xED, 0x66, 0x66, 0xCC, 0x0D, 0x00, 0x0B, 0x03, 0x73, 0x00, 0x83, 0x00, 0x0C, 0x00, 0x0D,
    0x00, 0x08, 0x11, 0x1F, 0x88, 0x89, 0x00, 0x0E, 0xDC, 0xCC, 0x6E, 0xE6, 0xDD, 0xDD, 0xD9, 0x99,
    0xBB, 0xBB, 0x67, 0x63, 0x6E, 0x0E, 0xEC, 0xCC, 0xDD, 0xDC, 0x99, 0x9F, 0xBB, 0xB9, 0x33, 0x3E,
];


enum Fixup {
    Abs16 { at: usize, label: String },
    Rel8 { at: usize, label: String },
}

struct Asm {
    base: u16,
    code: Vec<u8>,
    labels: HashMap<String, u16>,
    fixups: Vec<Fixup>,
}

impl Asm {
    fn new(base: u16) -> Self {
        Asm { base, code: Vec::new(), labels: HashMap::new(), fixups: Vec::new() }
    }

    fn here(&self) -> u16 {
        self.base + self.code.len() as u16
    }
    fn label(&mut self, name: &str) {
        let addr = self.here();
        self.labels.insert(name.to_string(), addr);
    }
    fn b(&mut self, byte: u8) {
        self.code.push(byte);
    }
    fn bytes(&mut self, data: &[u8]) {
        self.code.extend_from_slice(data);
    }

        fn di(&mut self) {
        self.b(0xF3);
    }
    fn xor_a(&mut self) {
        self.b(0xAF);
    }
    fn inc_a(&mut self) {
        self.b(0x3C);
    }
    fn dec_a(&mut self) {
        self.b(0x3D);
    }
    fn add_a_a(&mut self) {
        self.b(0x87);
    }
    fn inc_de(&mut self) {
        self.b(0x13);
    }
    fn dec_bc(&mut self) {
        self.b(0x0B);
    }
    fn add_hl_de(&mut self) {
        self.b(0x19);
    }
    fn or_c(&mut self) {
        self.b(0xB1);
    }
    fn ld_a_b(&mut self) {
        self.b(0x78);
    }
    fn ld_b_a(&mut self) {
        self.b(0x47);
    }
    fn ld_l_a(&mut self) {
        self.b(0x6F);
    }
    fn ld_h_a(&mut self) {
        self.b(0x67);
    }
    fn ld_e_a(&mut self) {
        self.b(0x5F);
    }
    fn ld_d_a(&mut self) {
        self.b(0x57);
    }
    fn ld_a_l(&mut self) {
        self.b(0x7D);
    }
    fn ld_a_h(&mut self) {
        self.b(0x7C);
    }
    fn ld_a_de(&mut self) {
        self.b(0x1A);
    }
    fn ld_a_hl(&mut self) {
        self.b(0x7E);
    }
    fn ld_a_hlp(&mut self) {
        self.b(0x2A);
    }
    fn ld_hlp_a(&mut self) {
        self.b(0x22);
    }
    fn ld_de_a(&mut self) {
        self.b(0x12);
    }
    fn or_a(&mut self) {
        self.b(0xB7);
    }
    fn ei(&mut self) {
        self.b(0xFB);
    }
    fn reti(&mut self) {
        self.b(0xD9);
    }
    fn push_af(&mut self) {
        self.b(0xF5);
    }
    fn pop_af(&mut self) {
        self.b(0xF1);
    }
    fn push_hl(&mut self) {
        self.b(0xE5);
    }
    fn pop_hl(&mut self) {
        self.b(0xE1);
    }
    fn push_de(&mut self) {
        self.b(0xD5);
    }
    fn pop_de(&mut self) {
        self.b(0xD1);
    }
    fn ld_sp(&mut self, nn: u16) {
        self.b(0x31);
        self.b(nn as u8);
        self.b((nn >> 8) as u8);
    }

        fn ld_a_n(&mut self, n: u8) {
        self.b(0x3E);
        self.b(n);
    }
    fn and_n(&mut self, n: u8) {
        self.b(0xE6);
        self.b(n);
    }
    fn add_n(&mut self, n: u8) {
        self.b(0xC6);
        self.b(n);
    }
    fn cp_n(&mut self, n: u8) {
        self.b(0xFE);
        self.b(n);
    }

        fn ld_hl(&mut self, nn: u16) {
        self.b(0x21);
        self.b(nn as u8);
        self.b((nn >> 8) as u8);
    }
    fn ld_de(&mut self, nn: u16) {
        self.b(0x11);
        self.b(nn as u8);
        self.b((nn >> 8) as u8);
    }
    fn ld_bc(&mut self, nn: u16) {
        self.b(0x01);
        self.b(nn as u8);
        self.b((nn >> 8) as u8);
    }

        fn ldh_to(&mut self, n: u8) {
        self.b(0xE0);
        self.b(n);
    }
    fn ldh_from(&mut self, n: u8) {
        self.b(0xF0);
        self.b(n);
    }
    fn ld_to(&mut self, nn: u16) {
        self.b(0xEA);
        self.b(nn as u8);
        self.b((nn >> 8) as u8);
    }
    fn ld_from(&mut self, nn: u16) {
        self.b(0xFA);
        self.b(nn as u8);
        self.b((nn >> 8) as u8);
    }

        fn ld_hl_label(&mut self, label: &str) {
        self.b(0x21);
        let at = self.code.len();
        self.b(0);
        self.b(0);
        self.fixups.push(Fixup::Abs16 { at, label: label.to_string() });
    }
    fn ld_de_label(&mut self, label: &str) {
        self.b(0x11);
        let at = self.code.len();
        self.b(0);
        self.b(0);
        self.fixups.push(Fixup::Abs16 { at, label: label.to_string() });
    }
    fn jp(&mut self, label: &str) {
        self.b(0xC3);
        let at = self.code.len();
        self.b(0);
        self.b(0);
        self.fixups.push(Fixup::Abs16 { at, label: label.to_string() });
    }
    fn jr(&mut self, label: &str) {
        self.b(0x18);
        let at = self.code.len();
        self.b(0);
        self.fixups.push(Fixup::Rel8 { at, label: label.to_string() });
    }
    fn jr_cc(&mut self, cond: u8, label: &str) {
        self.b(0x20 | (cond << 3));
        let at = self.code.len();
        self.b(0);
        self.fixups.push(Fixup::Rel8 { at, label: label.to_string() });
    }

    fn finish(mut self) -> Vec<u8> {
        for fixup in &self.fixups {
            match fixup {
                Fixup::Abs16 { at, label } => {
                    let addr = self.labels[label];
                    self.code[*at] = addr as u8;
                    self.code[*at + 1] = (addr >> 8) as u8;
                }
                Fixup::Rel8 { at, label } => {
                    let target = self.labels[label] as i32;
                    let next = self.base as i32 + *at as i32 + 1;
                    let rel = target - next;
                    assert!((-128..=127).contains(&rel), "jr out of range to {}", label);
                    self.code[*at] = rel as i8 as u8;
                }
            }
        }
        self.code
    }
}


fn solid_rows(rows: &[u8]) -> Vec<u8> {
    let mut tile = Vec::with_capacity(rows.len() * 2);
    for &r in rows {
        tile.push(r);
        tile.push(r);
    }
    tile
}

fn cat_rows() -> [u16; 16] {
    let art = [
        "..XX......XX....",
        ".X..X....X..X...",
        ".X..XXXXXX..X...",
        ".XXXXXXXXXXXX...",
        ".XX.XXXXXX.XX...",
        ".XXXXXXXXXXXX...",
        ".XX.XXXXXX.XX...",
        ".XXXXXXXXXXXX...",
        ".XXXXXXXXXXXX.X.",
        ".XXXXXXXXXXXXXX.",
        ".XXXXXXXXXXXX.X.",
        ".XXXXXXXXXXXX...",
        ".XXXXXXXXXXXX...",
        ".XXX.XXXX.XXX...",
        ".XX...XX...XX...",
        "................",
    ];
    let mut rows = [0u16; 16];
    for (i, line) in art.iter().enumerate() {
        let mut v = 0u16;
        for (c, ch) in line.chars().enumerate() {
            if ch == 'X' {
                v |= 1 << (15 - c);
            }
        }
        rows[i] = v;
    }
    rows
}

fn cat_tiles() -> Vec<u8> {
    let rows = cat_rows();
    let mut out = Vec::new();
    let mut emit = |range: std::ops::Range<usize>, left: bool| {
        for r in range {
            let mask = if left { (rows[r] >> 8) as u8 } else { rows[r] as u8 };
            out.push(mask);
            out.push(mask);
        }
    };
    emit(0..8, true);
    emit(8..16, true);
    emit(0..8, false);
    emit(8..16, false);
    out
}

fn ground_tile() -> Vec<u8> {
    let mut tile = vec![0xFF, 0xFF];
    for _ in 1..8 {
        tile.push(0x00);
        tile.push(0xFF);
    }
    tile
}


fn glyph(ch: char) -> [u8; 8] {
    match ch {
        'C' => [0x70, 0x88, 0x80, 0x80, 0x80, 0x88, 0x70, 0x00],
        'a' => [0x00, 0x00, 0x70, 0x08, 0x78, 0x88, 0x78, 0x00],
        't' => [0x40, 0x40, 0xF0, 0x40, 0x40, 0x48, 0x30, 0x00],
        'A' => [0x70, 0x88, 0x88, 0xF8, 0x88, 0x88, 0x88, 0x00],
        'n' => [0x00, 0x00, 0xB0, 0xC8, 0x88, 0x88, 0x88, 0x00],
        'D' => [0xF0, 0x88, 0x88, 0x88, 0x88, 0x88, 0xF0, 0x00],
        'e' => [0x00, 0x00, 0x70, 0x88, 0xF8, 0x80, 0x70, 0x00],
        'v' => [0x00, 0x00, 0x88, 0x88, 0x88, 0x50, 0x20, 0x00],
        'P' => [0xF0, 0x88, 0x88, 0xF0, 0x80, 0x80, 0x80, 0x00],
        'R' => [0xF0, 0x88, 0x88, 0xF0, 0xA0, 0x90, 0x88, 0x00],
        'E' => [0xF8, 0x80, 0x80, 0xF0, 0x80, 0x80, 0xF8, 0x00],
        'S' => [0x78, 0x80, 0x80, 0x70, 0x08, 0x08, 0xF0, 0x00],
        'T' => [0xF8, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x00],
        'O' => [0x70, 0x88, 0x88, 0x88, 0x88, 0x88, 0x70, 0x00],
        'M' => [0x88, 0xD8, 0xA8, 0x88, 0x88, 0x88, 0x88, 0x00],
        'W' => [0x88, 0x88, 0x88, 0xA8, 0xA8, 0xD8, 0x88, 0x00],
        _ => [0; 8],
    }
}

fn scale2(g: [u8; 8]) -> [u16; 16] {
    let mut out = [0u16; 16];
    for r in 0..8 {
        let mut wide = 0u16;
        for c in 0..8 {
            if g[r] & (0x80 >> c) != 0 {
                wide |= 0b11 << (14 - 2 * c);
            }
        }
        out[2 * r] = wide;
        out[2 * r + 1] = wide;
    }
    out
}

fn logo_letter_tiles(ch: char) -> Vec<u8> {
    let rows = scale2(glyph(ch));
    let mut out = Vec::new();
    let mut emit = |range: std::ops::Range<usize>, left: bool| {
        for r in range {
            let mask = if left { (rows[r] >> 8) as u8 } else { rows[r] as u8 };
            out.push(mask);
            out.push(mask);
        }
    };
    emit(0..8, true);
    emit(0..8, false);
    emit(8..16, true);
    emit(8..16, false);
    out
}

fn tile_data() -> Vec<u8> {
    let mut data = Vec::new();
    data.extend(ground_tile());
    data.extend(cat_tiles());
    for ch in LOGO_UNIQUE.chars() {
        data.extend(logo_letter_tiles(ch));
    }
    for ch in PROMPT_LETTERS.chars() {
        data.extend(solid_rows(&glyph(ch)));
    }
    data
}

fn logo_map() -> Vec<u8> {
    let mut map = vec![TILE_BLANK; 2 * 32];
    for (i, ch) in LOGO_WORD.chars().enumerate() {
        let unit = LOGO_UNIQUE.find(ch).unwrap() as u8;
        let base = TILE_LOGO_BASE + unit * 4;
        let col = i * 2;
        map[col] = base;
        map[col + 1] = base + 1;
        map[32 + col] = base + 2;
        map[32 + col + 1] = base + 3;
    }
    map
}


fn sample_data() -> Vec<u8> {
    let raw = std::fs::read("assets/meow.raw")
        .expect("assets/meow.raw missing — decode the mp3 with ffmpeg first");
    let active = |b: u8| (b as i32 - 128).abs() > 6;
    let start = raw.iter().position(|&b| active(b)).unwrap_or(0);
    let end = raw.iter().rposition(|&b| active(b)).map(|i| i + 1).unwrap_or(raw.len());
    let region = &raw[start..end];

        let lo = *region.iter().min().unwrap() as i32;
    let hi = *region.iter().max().unwrap() as i32;
    let span = (hi - lo).max(1);
    let mut nibbles: Vec<u8> =
        region.iter().map(|&b| (((b as i32 - lo) * 15) / span) as u8).collect();
    while nibbles.len() % 32 != 0 {
        nibbles.push(8);
    }
    nibbles.chunks(2).map(|c| (c[0] << 4) | c[1]).collect()
}

fn num_chunks() -> usize {
    sample_data().len() / 16
}


fn build_game() -> Vec<u8> {
    let mut a = Asm::new(0x0150);

        a.jp("main");

            a.label("isr");
    a.push_af();
    a.push_hl();
    a.push_de();
    a.ld_from(CHUNK_CNT);
    a.or_a();
    a.jr_cc(COND_Z, "isr_stop");
    a.ld_from(SPTR_LO);
    a.ld_l_a();
    a.ld_from(SPTR_HI);
    a.ld_h_a();
    a.ld_de(WAVE_RAM);
    for _ in 0..16 {
        a.ld_a_hlp();
        a.ld_de_a();
        a.inc_de();
    }
    a.ld_a_l();
    a.ld_to(SPTR_LO);
    a.ld_a_h();
    a.ld_to(SPTR_HI);
    a.ld_from(CHUNK_CNT);
    a.dec_a();
    a.ld_to(CHUNK_CNT);
    a.jr("isr_done");
    a.label("isr_stop");
    a.xor_a();
    a.ldh_to(NR30);
    a.label("isr_done");
    a.pop_de();
    a.pop_hl();
    a.pop_af();
    a.reti();

        a.label("main");
    a.di();
    a.ld_sp(0xFFFE);
    a.ld_a_n(0x00);
    a.ldh_to(LCDC);

        a.ld_hl(0x8010);
    a.ld_de_label("tiledata");
    a.ld_bc(tile_data().len() as u16);
    a.label("copy_tiles");
    a.ld_a_de();
    a.ld_hlp_a();
    a.inc_de();
    a.dec_bc();
    a.ld_a_b();
    a.or_c();
    a.jr_cc(COND_NZ, "copy_tiles");

        a.ld_hl(0x9820);
    a.ld_de_label("logomap");
    a.ld_bc(logo_map().len() as u16);
    a.label("copy_logo");
    a.ld_a_de();
    a.ld_hlp_a();
    a.inc_de();
    a.dec_bc();
    a.ld_a_b();
    a.or_c();
    a.jr_cc(COND_NZ, "copy_logo");

        let p = TILE_PROMPT_BASE;
    let prompt = [
        p,
        p + 1,
        p + 2,
        p + 3,
        p + 3,
        TILE_BLANK,
        p + 4,
        TILE_BLANK,
        p + 5,
        p + 6,
        TILE_BLANK,
        p + 7,
        p + 2,
        p + 6,
        p + 8,
    ];
    a.ld_hl(0x98A2);
    for tile in prompt {
        a.ld_a_n(tile);
        a.ld_hlp_a();
    }

        a.ld_hl(0x99E0);
    a.ld_bc(96);
    a.label("fill_ground");
    a.ld_a_n(TILE_GROUND);
    a.ld_hlp_a();
    a.dec_bc();
    a.ld_a_b();
    a.or_c();
    a.jr_cc(COND_NZ, "fill_ground");

        a.ld_a_n(0xE4);
    a.ldh_to(BGP);
    a.ld_a_n(0xE4);
    a.ldh_to(OBP0);

        a.ld_a_n(0x80);
    a.ldh_to(NR52);
    a.ld_a_n(0xFF);
    a.ldh_to(NR51);
    a.ld_a_n(0x77);
    a.ldh_to(NR50);
    a.ld_a_n(0x80);
    a.ldh_to(NR30);
    a.ld_a_n(0x20);
    a.ldh_to(NR32);

        a.ld_a_n(240);
    a.ldh_to(TMA);
    a.ld_a_n(0x04);
    a.ldh_to(TAC);
    a.ld_a_n(0x04);
    a.ld_to(IE);
    a.ei();

        a.ld_a_n(72);
    a.ld_to(PLAYER_X);
    a.ld_a_n(GROUND_PIX);
    a.ld_to(Y_HI);
    a.xor_a();
    a.ld_to(Y_LO);
    a.ld_to(VY_HI);
    a.ld_to(VY_LO);
    a.ld_to(CHUNK_CNT);
    a.ld_a_n(1);
    a.ld_to(ON_GROUND);

        a.ld_a_n(0x97);
    a.ldh_to(LCDC);

        a.label("loop");

    a.label("wait_leave");
    a.ldh_from(LY);
    a.cp_n(144);
    a.jr_cc(COND_Z, "wait_leave");
    a.label("wait_vblank");
    a.ldh_from(LY);
    a.cp_n(144);
    a.jr_cc(COND_NZ, "wait_vblank");

        a.ld_a_n(0x10);
    a.ldh_to(JOYP);
    a.ldh_from(JOYP);
    a.ldh_from(JOYP);
    a.and_n(0x01);
    a.jr_cc(COND_NZ, "no_action");

    a.ld_from(ON_GROUND);
    a.cp_n(0);
    a.jr_cc(COND_Z, "no_jump");
    a.ld_a_n(JUMP_VEL as u8);
    a.ld_to(VY_LO);
    a.ld_a_n((JUMP_VEL >> 8) as u8);
    a.ld_to(VY_HI);
    a.label("no_jump");

        a.ld_from(CHUNK_CNT);
    a.or_a();
    a.jr_cc(COND_NZ, "no_meow");
                a.xor_a();
    a.ldh_to(DIV);
    a.ld_a_n(240);
    a.ldh_to(TIMA);
        a.ld_hl_label("sampledata");
    a.ld_de(WAVE_RAM);
    for _ in 0..16 {
        a.ld_a_hlp();
        a.ld_de_a();
        a.inc_de();
    }
    a.ld_a_l();
    a.ld_to(SPTR_LO);
    a.ld_a_h();
    a.ld_to(SPTR_HI);
    a.ld_a_n((num_chunks() - 1) as u8);
    a.ld_to(CHUNK_CNT);
        a.ld_a_n(0x80);
    a.ldh_to(NR30);
    a.ld_a_n(0x20);
    a.ldh_to(NR32);
    a.xor_a();
    a.ldh_to(NR33);
    a.ld_a_n(0x87);
    a.ldh_to(NR34);
    a.label("no_meow");
    a.label("no_action");

        a.ld_a_n(0x20);
    a.ldh_to(JOYP);
    a.ldh_from(JOYP);
    a.ldh_from(JOYP);
    a.ld_b_a();

    a.ld_a_b();
    a.and_n(0x01);
    a.jr_cc(COND_NZ, "no_right");
    a.ld_from(PLAYER_X);
    a.cp_n(144);
    a.jr_cc(COND_NC, "no_right");
    a.inc_a();
    a.ld_to(PLAYER_X);
    a.label("no_right");

    a.ld_a_b();
    a.and_n(0x02);
    a.jr_cc(COND_NZ, "no_left");
    a.ld_from(PLAYER_X);
    a.cp_n(9);
    a.jr_cc(COND_C, "no_left");
    a.dec_a();
    a.ld_to(PLAYER_X);
    a.label("no_left");

        a.ld_from(VY_LO);
    a.ld_l_a();
    a.ld_from(VY_HI);
    a.ld_h_a();
    a.ld_de(GRAVITY);
    a.add_hl_de();
    a.ld_a_l();
    a.ld_to(VY_LO);
    a.ld_a_h();
    a.ld_to(VY_HI);

        a.ld_from(Y_LO);
    a.ld_l_a();
    a.ld_from(Y_HI);
    a.ld_h_a();
    a.ld_from(VY_LO);
    a.ld_e_a();
    a.ld_from(VY_HI);
    a.ld_d_a();
    a.add_hl_de();
    a.ld_a_l();
    a.ld_to(Y_LO);
    a.ld_a_h();
    a.ld_to(Y_HI);

        a.cp_n(GROUND_PIX);
    a.jr_cc(COND_C, "airborne");
    a.ld_a_n(GROUND_PIX);
    a.ld_to(Y_HI);
    a.xor_a();
    a.ld_to(Y_LO);
    a.ld_to(VY_HI);
    a.ld_to(VY_LO);
    a.ld_a_n(1);
    a.ld_to(ON_GROUND);
    a.jr("grounded_done");
    a.label("airborne");
    a.xor_a();
    a.ld_to(ON_GROUND);
    a.label("grounded_done");

        a.ld_from(Y_HI);
    a.add_n(16);
    a.ld_b_a();
    a.ld_hl(0xFE00);
    a.ld_a_b();
    a.ld_hlp_a();
    a.ld_from(PLAYER_X);
    a.add_n(8);
    a.ld_hlp_a();
    a.ld_a_n(TILE_CAT_L);
    a.ld_hlp_a();
    a.xor_a();
    a.ld_hlp_a();
    a.ld_a_b();
    a.ld_hlp_a();
    a.ld_from(PLAYER_X);
    a.add_n(16);
    a.ld_hlp_a();
    a.ld_a_n(TILE_CAT_R);
    a.ld_hlp_a();
    a.xor_a();
    a.ld_hlp_a();

    a.jp("loop");

        a.label("tiledata");
    let tiles = tile_data();
    a.bytes(&tiles);
    a.label("logomap");
    let logo = logo_map();
    a.bytes(&logo);
    a.label("sampledata");
    let sample = sample_data();
    a.bytes(&sample);

    a.finish()
}


fn build_rom(title: &str, code: Vec<u8>) -> Vec<u8> {
    let mut rom = vec![0u8; 0x8000];

            rom[0x0050] = 0xC3;
    rom[0x0051] = 0x53;
    rom[0x0052] = 0x01;

    rom[0x0100] = 0x00;
    rom[0x0101] = 0xC3;
    rom[0x0102] = 0x50;
    rom[0x0103] = 0x01;

    rom[0x0104..0x0134].copy_from_slice(&NINTENDO_LOGO);

    for (i, byte) in title.bytes().take(11).enumerate() {
        rom[0x0134 + i] = byte;
    }

    rom[0x0144] = b'C';
    rom[0x0145] = b'A';
    rom[0x0146] = 0x00;
    rom[0x0147] = 0x00;
    rom[0x0148] = 0x00;
    rom[0x0149] = 0x00;
    rom[0x014A] = 0x01;
    rom[0x014B] = 0x33;

    let code_base = 0x0150;
    assert!(code.len() <= 0x8000 - code_base, "code too large");
    rom[code_base..code_base + code.len()].copy_from_slice(&code);

    let mut header_checksum: u8 = 0;
    for addr in 0x0134..=0x014C {
        header_checksum = header_checksum.wrapping_sub(rom[addr]).wrapping_sub(1);
    }
    rom[0x014D] = header_checksum;

    let mut global: u16 = 0;
    for (i, &byte) in rom.iter().enumerate() {
        if i != 0x014E && i != 0x014F {
            global = global.wrapping_add(byte as u16);
        }
    }
    rom[0x014E] = (global >> 8) as u8;
    rom[0x014F] = global as u8;

    rom
}


fn main() -> std::io::Result<()> {
    let code = build_game();
    let rom = build_rom("CATANNADEV", code);

    std::fs::create_dir_all("forge-out")?;
    let path = "forge-out/catannadev.gb";
    std::fs::write(path, &rom)?;
    println!("CatAnnaDev built: {} ({} bytes)", path, rom.len());
    println!("Play it:  cargo run --release {}", path);
    Ok(())
}
