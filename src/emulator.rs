use std::sync::{Arc, Mutex};

use crate::audio::audio_buffer::AudioBuffer;
use crate::audio::audio_output::AudioOutput;
use crate::bus::bus::Bus;
use crate::cpu::cpu::CPU;
use crate::cpu::header::CartdrigeHeader;

pub struct CrabbyBoy {
    #[cfg(test)]
    test_max_loop: u32,
    audio_output: Option<AudioOutput>,
}

impl CrabbyBoy {
    pub fn new() -> Self {
        CrabbyBoy {
            #[cfg(test)]
            test_max_loop: 500_000_000,
            audio_output: None,
        }
    }

    pub fn run(&mut self, file_path: &str) -> Result<(), String> {
        let rom_data: Vec<u8> = std::fs::read(file_path).unwrap();
        println!("ROM size: {} bytes", rom_data.len());
        let mut bus: Bus = Bus::new(rom_data);

        let audio_buffer = Arc::new(Mutex::new(AudioBuffer::new(8192)));
        if let Some((output, sample_rate)) = AudioOutput::new(audio_buffer.clone()) {
            // Very important to play sound
            self.audio_output = Some(output);
            bus.set_audio_sample_rate(sample_rate);
        }
        let sync_buffer = audio_buffer.clone();
        bus.set_audio_buffer(audio_buffer);
        let audio_active = self.audio_output.is_some();

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

            cpu.handle_interrupts(&mut bus);
            cpu.execute(&mut bus);

            // if audio_active {
            //     while sync_buffer.lock().unwrap().count() >= 6144 {
            //         // 6144 = nombre de sample minimal
            //         std::thread::sleep(std::time::Duration::from_millis(1));
            //     }
            // }

            // #[cfg(test)]
            if cpu.pc == prev_pc {
                // Check eram to verify test results
                if self.check_test_results(&bus) {
                    return Ok(());
                } else {
                    self.log_test_diagnostics(&bus);
                    return Err("Test ROM reported failure".to_string());
                }
            }
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

    fn log_test_diagnostics(&self, bus: &Bus) {
        let serial_bytes = bus.get_io().get_serial().serial_output();
        let serial_str = std::str::from_utf8(serial_bytes).unwrap_or("");
        if !serial_str.is_empty() {
            eprintln!("[DIAG] serial: {}", serial_str);
        }
        let eram = bus.get_eram();
        if eram.len() >= 8 {
            eprintln!(
                "[DIAG] eram status={:02X} sig={:02X} {:02X} {:02X}",
                eram[0], eram[1], eram[2], eram[3]
            );
            let text: String = eram[4..]
                .iter()
                .take_while(|&&b| b != 0)
                .map(|&b| b as char)
                .collect();
            println!("[DIAG] eram text: {}", text);
        }
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

    // dmg_sounds
    cpu_instr_test!(sound_01, "./tests/dmg_sound/01-registers.gb");
    cpu_instr_test!(sound_02, "./tests/dmg_sound/02-len ctr.gb");
    cpu_instr_test!(sound_03, "./tests/dmg_sound/03-trigger.gb");
    cpu_instr_test!(sound_04, "./tests/dmg_sound/04-sweep.gb");
    cpu_instr_test!(sound_05, "./tests/dmg_sound/05-sweep details.gb");
    cpu_instr_test!(sound_06, "./tests/dmg_sound/06-overflow on trigger.gb");
    cpu_instr_test!(sound_07, "./tests/dmg_sound/07-len sweep period sync.gb");
    cpu_instr_test!(sound_08, "./tests/dmg_sound/08-len ctr during power.gb");
    cpu_instr_test!(sound_09, "./tests/dmg_sound/09-wave read while on.gb");
    cpu_instr_test!(sound_10, "./tests/dmg_sound/10-wave trigger while on.gb");
    cpu_instr_test!(sound_11, "./tests/dmg_sound/11-regs after power.gb");
    cpu_instr_test!(sound_12, "./tests/dmg_sound/12-wave write while on.gb");

    cpu_instr_test!(sound_all, "./tests/dmg_sound.gb");
}
