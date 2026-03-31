mod cpu;

use crate::cpu::bus::Bus;
use crate::cpu::cpu::CPU;
use crate::cpu::header::CartdrigeHeader;

fn main() -> Result<(), String> {
    // let file_path = "./Tetris.gb";
    let file_path = "./tests/halt_bug.gb";
    // let file_path = "./tests/cpu_instrs.gb";
    // let file_path = "./tests/cpu_instrs/01-special.gb";
    // let file_path = "./tests/cpu_instrs/02-interrupts.gb";
    // let file_path = "./tests/cpu_instrs/03-op_sp,hl.gb";
    // let file_path = "./tests/cpu_instrs/04-op r,imm.gb";
    // let file_path = "./tests/cpu_instrs/05-op rp.gb";
    // let file_path = "./tests/cpu_instrs/06-ld r,r.gb";
    // let file_path = "./tests/cpu_instrs/07-jr,jp,call,ret,rst.gb";
    // let file_path = "./tests/cpu_instrs/08-misc instrs.gb";
    // let file_path = "./tests/cpu_instrs/09-op r,r.gb";
    // let file_path = "./tests/cpu_instrs/10-bit ops.gb";
    // let file_path = "./tests/cpu_instrs/11-op a,(hl).gb";
    // Init Mem

    let rom_data: Vec<u8> = std::fs::read(file_path).unwrap();
    println!("ROM size: {} bytes", rom_data.len());
    let mut bus: Bus = Bus::new(rom_data);
    let header: CartdrigeHeader = CartdrigeHeader::new(&bus.get_rom());
    header.print();
    header.is_valid()?;
    // create cpu
    let mut cpu: CPU = CPU::new();
    // let mut cycle_count = 0;
    loop {
        // cycle_count += 1;
        // if cycle_count == 500_000 {
        //     println!("=== WRAM DUMP ===");
        //     for addr in (0xC000u16..=0xC500u16).step_by(1) {
        //         if (addr & 0xF) == 0 {
        //             print!("\n{:04X}: ", addr);
        //         }
        //         print!("{:02X} ", bus.read(addr));
        //     }
        //     println!();
        // }

        if cpu.stopped {
            println!("CPU STOPPED. Waiting interrupts");
            // CPU does nothing until an interrupt or button press wakes it
            // check_interrupts(&mut cpu);
            break;
        }
        if cpu.halt {
            // println!("CPU HALTED");
            bus.tick(4);
            // println!("HALT CHECK IF: 0x{:02X}", bus.get_io().get_if());
            let ie = bus.get_ie();
            let if_flag = bus.get_io().get_if();
            if (ie & if_flag) != 0 {
                // println!("CPU IE: {} IF: {} ; IME : {}", ie, if_flag, cpu.ime);
                cpu.halt = false;
                // ADD THIS:
                // println!("HALT EXIT → PC: 0x{:04X}", cpu.pc);
                // println!("  IE: {:08b} ({:02X})", ie, ie);
                // println!("  IF: {:08b} ({:02X})", if_flag, if_flag);
                // println!("  IME: {}", cpu.ime);
            } else {
                // println!("CONTINUE ppu_cycles: {}", bus.get_io().get_ppu_cycles());
                continue;
            }
        }

        match cpu.execute(&mut bus) {
            Some(tick) => {
                bus.tick(tick);
                cpu.handle_interrupts(&mut bus);
            }
            None => {
                panic!(" Error in getting the cycles ");
            }
        }
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
