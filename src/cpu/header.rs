use super::mappings::cartridge_type_map::CARTRIDGE_TYPE_MAP;
use super::mappings::licensee_map::NEW_LICENSEE_MAP;
use super::mappings::licensee_map::OLD_LICENSEE_MAP;
use super::mappings::size_map::RAM_SIZE_MAP;
use super::mappings::size_map::ROM_SIZE_MAP;

// https://gbdev.io/pandocs/The_Cartridge_Header.html

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
            cartridge_type: Self::parse_cartidge(rom),
            rom_size: Self::parse_rom_size(rom),
            ram_size: Self::parse_ram_size(rom),
            destination_code: Self::parse_destination_code(rom),
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
            return format!(
                "[NEW] {}",
                NEW_LICENSEE_MAP
                    .get(code_cow.as_ref())
                    .unwrap_or(&"Unknown")
            );
        }
        format!(
            "[OLD] {}",
            OLD_LICENSEE_MAP.get(old_code).unwrap_or(&"Unknown")
        )
    }

    fn parse_sgb(rom: &[u8]) -> String {
        let flag = &rom[0x146];
        if *flag == 0x03 {
            return "Support SGB functions".to_string();
        }
        "Does not support SGB functions".to_string()
    }

    fn parse_cartidge(rom: &[u8]) -> String {
        CARTRIDGE_TYPE_MAP
            .get(&rom[0x0147])
            .unwrap_or(&"None")
            .to_string()
    }

    fn parse_rom_size(rom: &[u8]) -> String {
        ROM_SIZE_MAP
            .get(&rom[0x0148])
            .unwrap_or(&"Unknown")
            .to_string()
    }

    fn parse_ram_size(rom: &[u8]) -> String {
        RAM_SIZE_MAP
            .get(&rom[0x149])
            .unwrap_or(&"Unknown")
            .to_string()
    }
    fn parse_destination_code(rom: &[u8]) -> String {
        if *&rom[0x014A] == 0x00 {
            return "Japan (and possibly overseas)".to_string();
        }
        "Overseas only".to_string()
    }
}
