mod bus;
mod cpu;
mod hardware;

use crate::bus::bus::Bus;
use crate::cpu::cpu::CPU;
use crate::cpu::header::CartdrigeHeader;

fn main() -> Result<(), String> {
    // let file_path = "./Tetris.gb";
    // let file_path = "./tests/halt_bug.gb";
    let file_path = "./tests/cpu_instrs.gb";
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
    loop {
        if cpu.stopped {
            println!(
                "EXECUTE PC: 0x{:04X} OP: 0x{:02X} SP: 0x{:04X}",
                cpu.pc,
                bus.read(cpu.pc),
                cpu.sp
            );
            println!("CPU STOPPED. Waiting interrupts");
            // CPU does nothing until an interrupt or button press wakes it
            break;
        }
        if cpu.halt {
            bus.tick(4);
            let ie = bus.get_ie();
            let if_flag = bus.get_io().get_if();
            if (ie & if_flag) != 0 {
                cpu.halt = false;
            }
            continue;
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
