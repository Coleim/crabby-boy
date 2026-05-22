mod bus;
mod cpu;
mod emulator;
mod hardware;

use crate::bus::bus::Bus;
use crate::emulator::CrabbyBoy;

fn main() -> Result<(), String> {
    // let file_path = "./Tetris.gb";
    // let file_path = "./tests/halt_bug.gb";
    let file_path = "./tests/interrupt_time.gb";
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

    let mut crabby = CrabbyBoy::new();
    crabby.run(file_path)
}
