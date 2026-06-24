use std::sync::{Arc, Mutex};
use std::time::Duration;

use crate::audio::audio_buffer::AudioBuffer;
use crate::bus::bus::Bus;
use crate::cpu::cpu::CPU;
use crate::cpu::header::CartdrigeHeader;
use crate::display::fps_counter::FpsCounter;

const RUNTIME_STEPS_PER_SEC: f64 = 400_000.0;

pub struct CrabbyBoy {
    fps: FpsCounter,
    pub bus: Bus,
    pub audio_buffer: Arc<Mutex<AudioBuffer>>,
    pub header: CartdrigeHeader,
    pub cpu: CPU,
}

impl CrabbyBoy {
    pub fn new(file_path: &str) -> Result<Self, String> {
        let fps_counter = FpsCounter::new();
        let rom_data = std::fs::read(file_path).unwrap();
        println!("ROM size: {} bytes", rom_data.len());
        let mut bus: Bus = Bus::new(rom_data);

        let audio_buffer = Arc::new(Mutex::new(AudioBuffer::new(8192)));
        // let audio_buffer = Arc::new(Mutex::new(AudioBuffer::new(32768)));
        bus.set_audio_buffer(audio_buffer.clone());

        let header: CartdrigeHeader = CartdrigeHeader::new(&bus.get_rom());
        header.is_valid()?;
        let cpu: CPU = CPU::new();

        Ok(CrabbyBoy {
            fps: fps_counter,
            bus,
            audio_buffer,
            header,
            cpu,
        })
    }

    pub fn set_audio_sample_rate(&mut self, sample_rate: u32) {
        self.bus.set_audio_sample_rate(sample_rate);
    }

    pub fn tick_for_duration(&mut self, frame_delta: Duration) {
        let dt = frame_delta.as_secs_f64().clamp(0.001, 0.5);
        let number_of_steps = (RUNTIME_STEPS_PER_SEC * dt) as usize;
        for _ in 0..number_of_steps {
            self.tick();
        }
    }

    pub fn tick(&mut self) -> bool {
        if self.cpu.stopped {
            println!("CPU STOPPED. Waiting interrupts");
            return false;
        }
        if self.cpu.halt {
            self.bus.internal_tick();
            let ie = self.bus.get_ie();
            let if_flag = self.bus.get_io().get_if();
            if (ie & if_flag) != 0 {
                self.cpu.halt = false;
            }
            return false;
        }

        self.cpu.handle_interrupts(&mut self.bus);
        self.cpu.execute(&mut self.bus);

        self.fps.tick();

        true
    }
}

#[cfg(test)]
fn check_test_results(bus: &Bus) -> bool {
    let serial_str = std::str::from_utf8(bus.get_io().get_serial().serial_output()).unwrap_or("");
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

#[cfg(test)]
fn log_test_diagnostics(bus: &Bus) {
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

#[cfg(test)]
macro_rules! cpu_instr_test {
    ($name: ident, $path: expr) => {
        #[test]
        fn $name() {
            let mut crabby = CrabbyBoy::new($path).unwrap();

            let test_max_loop = 500_000_000;
            let mut success = false;

            for i in 0..test_max_loop {
                if crabby.cpu.stopped {
                    break;
                }
                let prev_pc = crabby.cpu.pc;
                let cpu_executed = crabby.tick();
                if (!cpu_executed) {
                    continue;
                }

                if crabby.cpu.pc == prev_pc {
                    // Check eram to verify test results
                    if check_test_results(&crabby.bus) {
                        success = true;
                        break;
                    } else {
                        log_test_diagnostics(&crabby.bus);
                        panic!("Test ROM reported failure - Iteration {i}");
                    }
                }
            }

            assert!(
                success,
                "Max loop ({test_max_loop}) reached without satisfying condition"
            );
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
