use crate::bus::bus::Bus;
use crate::cpu::cpu::CPU;
use crate::cpu::header::CartdrigeHeader;

pub struct CrabbyBoy {
    #[cfg(test)]
    test_max_loop: u32,
}

impl CrabbyBoy {
    pub fn new() -> Self {
        CrabbyBoy {
            #[cfg(test)]
            test_max_loop: 500_000_000,
        }
    }

    pub fn run(&mut self, file_path: &str) -> Result<(), String> {
        let rom_data: Vec<u8> = std::fs::read(file_path).unwrap();
        println!("ROM size: {} bytes", rom_data.len());
        let mut bus: Bus = Bus::new(rom_data);
        let header: CartdrigeHeader = CartdrigeHeader::new(&bus.get_rom());
        header.print();
        header.is_valid()?;
        // create cpu
        let mut cpu: CPU = CPU::new();
        #[cfg(test)]
        let mut loop_count: u32 = 0;
        loop {
            #[cfg(test)]
            {
                loop_count = loop_count.wrapping_add(1);
                if loop_count > self.test_max_loop {
                    return Err(format!(
                        "Infinite Loop. Reaching loop number: {}",
                        loop_count
                    ));
                }
            }

            if cpu.stopped {
                println!("CPU STOPPED. Waiting interrupts");
                // CPU does nothing until an interrupt or button press wakes it
                break;
            }
            if cpu.halt {
                bus.internal_tick();
                let ie = bus.get_ie();
                let if_flag = bus.get_io().get_if();
                if (ie & if_flag) != 0 {
                    cpu.halt = false;
                }
                continue;
            }

            // #[cfg(test)]
            let prev_pc = cpu.pc;

            cpu.execute(&mut bus);

            // #[cfg(test)]
            if cpu.pc == prev_pc {
                // Check eram to verify test results
                if self.check_test_results(&bus) {
                    return Ok(());
                } else {
                    return Err("Test ROM reported failure".to_string());
                }
            }

            cpu.handle_interrupts(&mut bus);
        }

        Ok(())
    }

    // #[cfg(test)]
    fn check_test_results(&self, bus: &Bus) -> bool {
        let serial_str =
            std::str::from_utf8(bus.get_io().get_serial().serial_output()).unwrap_or("");
        if serial_str.contains("Passed") {
            return true;
        }
        if serial_str.contains("Failed") {
            eprintln!("Test ROM reported failure: {}", serial_str);
            return false;
        }

        let eram = bus.get_eram();

        if eram[0] == 0x00 {
            let text: String = eram[4..]
                .iter()
                .take_while(|&&b| b != 0)
                .map(|&b| b as char)
                .collect();
            println!("{}", text);
            if text.contains("Passed") {
                return true;
            }
            if text.contains("Failed") {
                eprintln!("Test ROM reported failure: {}", text);
                return false;
            }
        }

        false
    }
}

#[cfg(test)]
macro_rules! cpu_instr_test {
    ($name: ident, $path: expr) => {
        #[test]
        fn $name() {
            let mut crabby = CrabbyBoy::new();
            assert_eq!(crabby.run($path), Ok(()));
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    cpu_instr_test!(test_01_special, "./tests/cpu_instrs/01-special.gb");
    cpu_instr_test!(test_02_interrupts, "./tests/cpu_instrs/02-interrupts.gb");
    cpu_instr_test!(test_03_op_sp_hl, "./tests/cpu_instrs/03-op_sp,hl.gb");
    cpu_instr_test!(test_04_op_r_imm, "./tests/cpu_instrs/04-op r,imm.gb");
    cpu_instr_test!(test_05_op_rp, "./tests/cpu_instrs/05-op rp.gb");
    cpu_instr_test!(test_06_ld_r_r, "./tests/cpu_instrs/06-ld r,r.gb");
    cpu_instr_test!(
        test_07_jr_jp_call_ret_rst,
        "./tests/cpu_instrs/07-jr,jp,call,ret,rst.gb"
    );
    cpu_instr_test!(test_08_misc_instrs, "./tests/cpu_instrs/08-misc instrs.gb");
    cpu_instr_test!(test_09_op_r_r, "./tests/cpu_instrs/09-op r,r.gb");
    cpu_instr_test!(test_10_bit_ops, "./tests/cpu_instrs/10-bit ops.gb");
    cpu_instr_test!(test_11_op_a_hl, "./tests/cpu_instrs/11-op a,(hl).gb");

    // Too long
    // cpu_instr_test!(test_all_cpu_instrs, "./tests/cpu_instrs.gb");

    cpu_instr_test!(read_timing, "./tests/mem_timing/01-read_timing.gb");
    cpu_instr_test!(write_timing, "./tests/mem_timing/02-write_timing.gb");
    cpu_instr_test!(modify_timing, "./tests/mem_timing/03-modify_timing.gb");
    cpu_instr_test!(mem_timing, "./tests/mem_timing.gb");

    cpu_instr_test!(read_timing2, "./tests/mem_timing-2/01-read_timing.gb");
    cpu_instr_test!(write_timing2, "./tests/mem_timing-2/02-write_timing.gb");
    cpu_instr_test!(modify_timing2, "./tests/mem_timing-2/03-modify_timing.gb");
    cpu_instr_test!(mem_timing2, "./tests/mem_timing-2.gb");

    cpu_instr_test!(instr_timing, "./tests/instr_timing.gb");

    cpu_instr_test!(halt_bug, "./tests/halt_bug.gb");

    cpu_instr_test!(interrupt_time, "./tests/interrupt_time.gb");
}
