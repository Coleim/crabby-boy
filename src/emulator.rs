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
        // Very important to play sound
        if let Some((output, sample_rate)) = AudioOutput::new(audio_buffer.clone()) {
            self.audio_output = Some(output);
            bus.set_audio_sample_rate(sample_rate);
        }
        #[cfg(not(test))]
        let sync_buffer = if self.audio_output.is_some() {
            Some(audio_buffer.clone())
        } else {
            None
        };
        bus.set_audio_buffer(audio_buffer);

        let header: CartdrigeHeader = CartdrigeHeader::new(&bus.get_rom());
        header.print();
        header.is_valid()?;

        let cpu: CPU = CPU::new();

        #[cfg(test)]
        return self.run_headless(cpu, bus);

        #[cfg(not(test))]
        self.run_windowed(cpu, bus, sync_buffer)
    }

    #[cfg(test)]
    fn run_headless(&mut self, mut cpu: CPU, mut bus: Bus) -> Result<(), String> {
        let mut loop_count: u32 = 0;
        loop {
            loop_count = loop_count.wrapping_add(1);
            if loop_count > self.test_max_loop {
                return Err(format!("Infinite Loop. Reaching loop number: {}", loop_count));
            }
            if cpu.stopped {
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
            let prev_pc = cpu.pc;
            cpu.handle_interrupts(&mut bus);
            cpu.execute(&mut bus);
            if cpu.pc == prev_pc {
                if self.check_test_results(&bus) {
                    return Ok(());
                } else {
                    return Err("Test ROM reported failure".to_string());
                }
            }
        }
        Ok(())
    }

    #[cfg(not(test))]
    fn run_windowed(
        &mut self,
        mut cpu: CPU,
        mut bus: Bus,
        sync_buffer: Option<Arc<Mutex<AudioBuffer>>>,
    ) -> Result<(), String> {
        use crate::hardware::joypad;
        use minifb::{Key, Scale, Window, WindowOptions};

        let mut window = Window::new(
            "Crabby Boy",
            160,
            144,
            WindowOptions { scale: Scale::X4, ..WindowOptions::default() },
        )
        .map_err(|e| e.to_string())?;
        if sync_buffer.is_none() {
            window.set_target_fps(60);
        }

        let mut framebuffer = vec![0u32; 160 * 144];

        while window.is_open() && !window.is_key_down(Key::Escape) {
            let mut pressed = 0u8;
            if window.is_key_down(Key::Z) {
                pressed |= joypad::BUTTON_A;
            }
            if window.is_key_down(Key::X) {
                pressed |= joypad::BUTTON_B;
            }
            if window.is_key_down(Key::Backspace) {
                pressed |= joypad::BUTTON_SELECT;
            }
            if window.is_key_down(Key::Enter) {
                pressed |= joypad::BUTTON_START;
            }
            if window.is_key_down(Key::Right) {
                pressed |= joypad::BUTTON_RIGHT;
            }
            if window.is_key_down(Key::Left) {
                pressed |= joypad::BUTTON_LEFT;
            }
            if window.is_key_down(Key::Up) {
                pressed |= joypad::BUTTON_UP;
            }
            if window.is_key_down(Key::Down) {
                pressed |= joypad::BUTTON_DOWN;
            }
            bus.set_joypad(pressed);

            loop {
                if cpu.stopped {
                    bus.internal_tick();
                    let ie = bus.get_ie();
                    let if_flag = bus.get_io().get_if();
                    if (ie & if_flag) != 0 {
                        cpu.stopped = false;
                    }
                } else if cpu.halt {
                    bus.internal_tick();
                    let ie = bus.get_ie();
                    let if_flag = bus.get_io().get_if();
                    if (ie & if_flag) != 0 {
                        cpu.halt = false;
                    }
                } else {
                    cpu.handle_interrupts(&mut bus);
                    cpu.execute(&mut bus);
                }
                if let Some(buffer) = &sync_buffer {
                    while buffer.lock().unwrap().count() >= 4096 {
                        std::thread::sleep(std::time::Duration::from_micros(500));
                    }
                }
                if bus.take_frame_ready() {
                    break;
                }
            }

            bus.render_background(&mut framebuffer);
            window
                .update_with_buffer(&framebuffer, 160, 144)
                .map_err(|e| e.to_string())?;
        }

        Ok(())
    }

    #[cfg(test)]
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
}
