use once_cell::sync::Lazy;
use std::collections::HashMap;

pub struct CartdrigeHeader {
    entry_point: u8,
    nintendo_logo: [u8; 48],
    title: String,
    manufacturer_code: String,
    cgb_flag: String,
    licensee: String,
    sgb_flag: String,
    cartridge_type: String,
    rom_size: String, // This byte indicates how much ROM is present on the cartridge. In most cases, the ROM size is given by 32 KiB × (1 << <value>):
    ram_size: String,
    destination_code: String,
    old_publisher: String,
    version_number: u8,
    header_checksum: u8, // uint8_t checksum = 0; for (uint16_t address = 0x0134; address <= 0x014C; address++) { checksum = checksum - rom[address] - 1; }
    global_checksum: u8,
}

impl CartdrigeHeader {
    pub fn new(rom: &[u8]) -> Self {
        CartdrigeHeader {
            entry_point: 0,
            nintendo_logo: Self::read_nintendo_logo(rom),
            title: Self::parse_title(rom),
            manufacturer_code: Self::parse_manufacturercode(rom),
            cgb_flag: Self::parse_cgb_flag(rom),
            licensee: Self::parse_licensee(rom),
            sgb_flag: Self::parse_sgb(rom),
            cartridge_type: String::new(),
            rom_size: String::new(),
            ram_size: String::new(),
            destination_code: String::new(),
            old_publisher: String::new(),
            version_number: 0,
            header_checksum: 0,
            global_checksum: 0,
        }
    }

    pub fn print(&self) {
        println!("CartdrigeHeader {{");
        println!("  entry_point: {}", self.entry_point);
        println!("  nintendo_logo: {}", self.logo_hex());
        println!("  title: {}", self.title);
        println!("  manufacturer_code: {}", self.manufacturer_code);
        println!("  cgb flag: {}", self.cgb_flag);
        println!("  licensee: {}", self.licensee);
        println!("  sgb flag: {}", self.sgb_flag);
        println!("  cartridge_type: {}", self.cartridge_type);
        println!("  rom_size: {}", self.rom_size);
        println!("  ram_size: {}", self.ram_size);
        println!("  destination_code: {}", self.destination_code);
        println!("  old_publisher: {}", self.old_publisher);
        println!("  version_number: {}", self.version_number);
        println!("  header_checksum: {}", self.header_checksum);
        println!("  global_checksum: {}", self.global_checksum);
        println!("}}");
    }

    pub fn logo_hex(&self) -> String {
        self.nintendo_logo
            .iter()
            .map(|b| format!("{:02X}", b))
            .collect::<Vec<String>>()
            .join(" ")
    }

    pub fn is_valid(&self) -> Result<(), String> {
        if self.logo_hex().eq("CE ED 66 66 CC 0D 00 0B 03 73 00 83 00 0C 00 0D 00 08 11 1F 88 89 00 0E DC CC 6E E6 DD DD D9 99 BB BB 67 63 6E 0E EC CC DD DC 99 9F BB B9 33 3E") {
            println!("Nintendo Logo is valid");
            Ok(())
        } else {
            Err("Nintendo Logo is invalid".to_string())
        }
    }

    fn read_nintendo_logo(rom: &[u8]) -> [u8; 48] {
        let logo_start = 0x0104;
        let logo_end = 0x0133;
        let mut logo: [u8; 48] = [0; 48];
        logo.copy_from_slice(&rom[logo_start..=logo_end]);
        logo
    }

    fn parse_title(rom: &[u8]) -> String {
        let title_start = 0x0134;
        let title_end = 0x0143;
        let title_bytes = &rom[title_start..=title_end];
        String::from_utf8_lossy(title_bytes).to_string()
    }

    fn parse_manufacturercode(rom: &[u8]) -> String {
        let start = 0x013F;
        let end = 0x0142;
        let code = &rom[start..=end];
        String::from_utf8_lossy(code).to_string()
    }

    fn parse_cgb_flag(rom: &[u8]) -> String {
        let flag = &rom[0x143];
        if *flag == 0x80 {
            return "The game supports CGB enhancements, but is backwards compatible with monochrome Game Boys".to_string();
        } else if *flag == 0xC0 {
            return "The game works on CGB only".to_string();
        }
        "No flags".to_string()
    }

    fn parse_licensee(rom: &[u8]) -> String {
        let old_code = &rom[0x014B];
        if *old_code == 0x33 {
            let new_code = &rom[0x0144..=0x0145];
            let code_cow = String::from_utf8_lossy(new_code);
            return NEW_LICENSEE_MAP
                .get(code_cow.as_ref())
                .unwrap_or(&"Unknown")
                .to_string();
        }
        OLD_LICENSEE_MAP
            .get(old_code)
            .unwrap_or(&"Unknown")
            .to_string()
    }

    fn parse_sgb(rom: &[u8]) -> String {
        let flag = &rom[0x146];
        if *flag == 0x03 {
            return "Support SGB functions".to_string();
        }
        "Does not support SGB functions".to_string()
    }
}

// Use &str as key type to support non-hex codes like "9H", "BL", "DK"
static NEW_LICENSEE_MAP: Lazy<HashMap<&'static str, &'static str>> = Lazy::new(|| {
    let mut m = HashMap::new();
    m.insert("00", "None");
    m.insert("01", "Nintendo Research & Development 1");
    m.insert("08", "Capcom");
    m.insert("13", "EA (Electronic Arts)");
    m.insert("18", "Hudson Soft");
    m.insert("19", "B-AI");
    m.insert("20", "KSS");
    m.insert("22", "Planning Office WADA");
    m.insert("24", "PCM Complete");
    m.insert("25", "San-X");
    m.insert("28", "Kemco");
    m.insert("29", "SETA Corporation");
    m.insert("30", "Viacom");
    m.insert("31", "Nintendo");
    m.insert("32", "Bandai");
    m.insert("33", "Ocean Software/Acclaim Entertainment");
    m.insert("34", "Konami");
    m.insert("35", "HectorSoft");
    m.insert("37", "Taito");
    m.insert("38", "Hudson Soft");
    m.insert("39", "Banpresto");
    m.insert("41", "Ubi Soft1");
    m.insert("42", "Atlus");
    m.insert("44", "Malibu Interactive");
    m.insert("46", "Angel");
    m.insert("47", "Bullet-Proof Software2");
    m.insert("49", "Irem");
    m.insert("50", "Absolute");
    m.insert("51", "Acclaim Entertainment");
    m.insert("52", "Activision");
    m.insert("53", "Sammy USA Corporation");
    m.insert("54", "Konami");
    m.insert("55", "Hi Tech Expressions");
    m.insert("56", "LJN");
    m.insert("57", "Matchbox");
    m.insert("58", "Mattel");
    m.insert("59", "Milton Bradley Company");
    m.insert("60", "Titus Interactive");
    m.insert("61", "Virgin Games Ltd.3");
    m.insert("64", "Lucasfilm Games4");
    m.insert("67", "Ocean Software");
    m.insert("69", "EA (Electronic Arts)");
    m.insert("70", "Infogrames5");
    m.insert("71", "Interplay Entertainment");
    m.insert("72", "Broderbund");
    m.insert("73", "Sculptured Software6");
    m.insert("75", "The Sales Curve Limited7");
    m.insert("78", "THQ");
    m.insert("79", "Accolade");
    m.insert("80", "Misawa Entertainment");
    m.insert("83", "lozc");
    m.insert("86", "Tokuma Shoten");
    m.insert("87", "Tsukuda Original");
    m.insert("91", "Chunsoft Co.8");
    m.insert("92", "Video System");
    m.insert("93", "Ocean Software/Acclaim Entertainment");
    m.insert("95", "Varie");
    m.insert("96", "Yonezawa/s’pal");
    m.insert("97", "Kaneko");
    m.insert("99", "Pack-In-Video");
    m.insert("9H", "Bottom Up");
    m.insert("A4", "Konami (Yu-Gi-Oh!)");
    m.insert("BL", "MTO");
    m.insert("DK", "Kodansha");
    m
});

static OLD_LICENSEE_MAP: Lazy<HashMap<u8, &'static str>> = Lazy::new(|| {
    let mut m = HashMap::new();
    m.insert(0x00, "None");
    m.insert(0x01, "Nintendo");
    m.insert(0x08, "Capcom");
    m.insert(0x09, "HOT-B");
    m.insert(0x0A, "Jaleco");
    m.insert(0x0B, "Coconuts Japan");
    m.insert(0x0C, "Elite Systems");
    m.insert(0x13, "EA (Electronic Arts)");
    m.insert(0x18, "Hudson Soft");
    m.insert(0x19, "ITC Entertainment");
    m.insert(0x1A, "Yanoman");
    m.insert(0x1D, "Japan Clary");
    m.insert(0x1F, "Virgin Games Ltd.3");
    m.insert(0x24, "PCM Complete");
    m.insert(0x25, "San-X");
    m.insert(0x28, "Kemco");
    m.insert(0x29, "SETA Corporation");
    m.insert(0x30, "Infogrames5");
    m.insert(0x31, "Nintendo");
    m.insert(0x32, "Bandai");
    m.insert(0x33, "New licensee code should be used");
    m.insert(0x34, "Konami");
    m.insert(0x35, "HectorSoft");
    m.insert(0x38, "Capcom");
    m.insert(0x39, "Banpresto");
    m.insert(0x3C, "Entertainment Interactive (stub)");
    m.insert(0x3E, "Gremlin");
    m.insert(0x41, "Ubi Soft1");
    m.insert(0x42, "Atlus");
    m.insert(0x44, "Malibu Interactive");
    m.insert(0x46, "Angel");
    m.insert(0x47, "Spectrum HoloByte");
    m.insert(0x49, "Irem");
    m.insert(0x4A, "Virgin Games Ltd.3");
    m.insert(0x4D, "Malibu Interactive");
    m.insert(0x4F, "U.S. Gold");
    m.insert(0x50, "Absolute");
    m.insert(0x51, "Acclaim Entertainment");
    m.insert(0x52, "Activision");
    m.insert(0x53, "Sammy USA Corporation");
    m.insert(0x54, "GameTek");
    m.insert(0x55, "Park Place13");
    m.insert(0x56, "LJN");
    m.insert(0x57, "Matchbox");
    m.insert(0x59, "Milton Bradley Company");
    m.insert(0x5A, "Mindscape");
    m.insert(0x5B, "Romstar");
    m.insert(0x5C, "Naxat Soft14");
    m.insert(0x5D, "Tradewest");
    m.insert(0x60, "Titus Interactive");
    m.insert(0x61, "Virgin Games Ltd.3");
    m.insert(0x67, "Ocean Software");
    m.insert(0x69, "EA (Electronic Arts)");
    m.insert(0x6E, "Elite Systems");
    m.insert(0x6F, "Electro Brain");
    m.insert(0x70, "Infogrames5");
    m.insert(0x71, "Interplay Entertainment");
    m.insert(0x72, "Broderbund");
    m.insert(0x73, "Sculptured Software6");
    m.insert(0x75, "The Sales Curve Limited7");
    m.insert(0x78, "THQ");
    m.insert(0x79, "Accolade15");
    m.insert(0x7A, "Triffix Entertainment");
    m.insert(0x7C, "MicroProse");
    m.insert(0x7F, "Kemco");
    m.insert(0x80, "Misawa Entertainment");
    m.insert(0x83, "LOZC G.");
    m.insert(0x86, "Tokuma Shoten");
    m.insert(0x8B, "Bullet-Proof Software2");
    m.insert(0x8C, "Vic Tokai Corp.16");
    m.insert(0x8E, "Ape Inc.17");
    m.insert(0x8F, "I’Max18");
    m.insert(0x91, "Chunsoft Co.8");
    m.insert(0x92, "Video System");
    m.insert(0x93, "Tsubaraya Productions");
    m.insert(0x95, "Varie");
    m.insert(0x96, "Yonezawa19/S’Pal");
    m.insert(0x97, "Kemco");
    m.insert(0x99, "Arc");
    m.insert(0x9A, "Nihon Bussan");
    m.insert(0x9B, "Tecmo");
    m.insert(0x9C, "Imagineer");
    m.insert(0x9D, "Banpresto");
    m.insert(0x9F, "Nova");
    m.insert(0xA1, "Hori Electric");
    m.insert(0xA2, "Bandai");
    m.insert(0xA4, "Konami");
    m.insert(0xA6, "Kawada");
    m.insert(0xA7, "Takara");
    m.insert(0xA9, "Technos Japan");
    m.insert(0xAA, "Broderbund");
    m.insert(0xAC, "Toei Animation");
    m.insert(0xAD, "Toho");
    m.insert(0xAF, "Namco");
    m.insert(0xB0, "Acclaim Entertainment");
    m.insert(0xB1, "ASCII Corporation or Nexsoft");
    m.insert(0xB2, "Bandai");
    m.insert(0xB4, "Square Enix");
    m.insert(0xB6, "HAL Laboratory");
    m.insert(0xB7, "SNK");
    m.insert(0xB9, "Pony Canyon");
    m.insert(0xBA, "Culture Brain");
    m.insert(0xBB, "Sunsoft");
    m.insert(0xBD, "Sony Imagesoft");
    m.insert(0xBF, "Sammy Corporation");
    m.insert(0xC0, "Taito");
    m.insert(0xC2, "Kemco");
    m.insert(0xC3, "Square");
    m.insert(0xC4, "Tokuma Shoten");
    m.insert(0xC5, "Data East");
    m.insert(0xC6, "Tonkin House");
    m.insert(0xC8, "Koei");
    m.insert(0xC9, "UFL");
    m.insert(0xCA, "Ultra Games");
    m.insert(0xCB, "VAP, Inc.");
    m.insert(0xCC, "Use Corporation");
    m.insert(0xCD, "Meldac");
    m.insert(0xCE, "Pony Canyon");
    m.insert(0xCF, "Angel");
    m.insert(0xD0, "Taito");
    m.insert(0xD1, "SOFEL (Software Engineering Lab)");
    m.insert(0xD2, "Quest");
    m.insert(0xD3, "Sigma Enterprises");
    m.insert(0xD4, "ASK Kodansha Co.");
    m.insert(0xD6, "Naxat Soft14");
    m.insert(0xD7, "Copya System");
    m.insert(0xD9, "Banpresto");
    m.insert(0xDA, "Tomy");
    m.insert(0xDB, "LJN");
    m.insert(0xDD, "Nippon Computer Systems");
    m.insert(0xDE, "Human Ent.");
    m.insert(0xDF, "Altron");
    m.insert(0xE0, "Jaleco");
    m.insert(0xE1, "Towa Chiki");
    m.insert(0xE2, "Yutaka # Needs more info");
    m.insert(0xE3, "Varie");
    m.insert(0xE5, "Epoch");
    m.insert(0xE7, "Athena");
    m.insert(0xE8, "Asmik Ace Entertainment");
    m.insert(0xE9, "Natsume");
    m.insert(0xEA, "King Records");
    m.insert(0xEB, "Atlus");
    m.insert(0xEC, "Epic/Sony Records");
    m.insert(0xEE, "IGS");
    m.insert(0xF0, "A Wave");
    m.insert(0xF3, "Extreme Entertainment");
    m.insert(0xFF, "LJN");
    m
});
