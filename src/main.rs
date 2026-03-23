mod cpu;

use crate::cpu::bus::Bus;
use crate::cpu::cpu::CPU;
use crate::cpu::header::CartdrigeHeader;

fn main() -> Result<(), String> {
    // let file_path = "./Tetris.gb";
    // let file_path = "./tests/cpu_instrs.gb";
    let file_path = "./tests/cpu_instrs/01-special.gb";
    // let file_path = "./tests/cpu_instrs/03-op_sp,hl.gb";
    // let file_path = "./02-interrupts.gb";
    // Init Mem
    let rom_data: Vec<u8> = std::fs::read(file_path).unwrap();
    println!("ROM size: {} bytes", rom_data.len());
    let mut bus: Bus = Bus::new(rom_data);
    let header: CartdrigeHeader = CartdrigeHeader::new(&bus.get_rom());
    header.print();
    header.is_valid()?;
    // create cpu
    let mut cpu: CPU = CPU::new();

    for addr in 0x0200..=0x0215u16 {
        println!("ROM[0x{:04X}] = 0x{:02X}", addr, bus.read(addr));
    }

    loop {
        if cpu.stopped {
            println!("CPU STOPPED. Waiting interrupts");
            // CPU does nothing until an interrupt or button press wakes it
            // check_interrupts(&mut cpu);
            continue;
        }
        if cpu.halt {
            break; // for emulator convenience
        }

        if !cpu.execute(&mut bus) {
            break;
        }
        // check_serial(&mut mem);
    }

    Ok(())
}

// Aussi, ton halt devrait ne pas break mais plutôt attendre une interruption comme stopped :
// if cpu.halt {
//     // Sortir du halt uniquement si une interruption est pending
//     let ie = bus.read(0xFFFF);
//     let if_ = bus.read(0xFF0F);
//     if ie & if_ != 0 {
//         cpu.halt = false;
//     }
//     continue; // pas break !
// }
