pub struct Cartdrige_Header {
    entry_point: u8,
    nintendo_logo: [u8; 50],
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
