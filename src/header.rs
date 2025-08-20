pub struct CartdrigeHeader {
    entry_point: u8,
    nintendo_logo: [u8; 48],
    title: String,
    manufacturer_code: String,
    publisher: String,
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
            manufacturer_code: String::new(),
            publisher: String::new(),
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
        println!("  publisher: {}", self.publisher);
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
}
