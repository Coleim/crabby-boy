use std::sync::{Arc, Mutex};

use crate::audio::buffer::AudioBuffer;
use crate::audio::output::AudioOutput;
use crate::bus::Bus;
use crate::cartridge::header::CartridgeHeader;
use crate::cpu::CPU;

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

        let capturing = std::env::var("CRABBY_WAV").is_ok();
        let cap = if capturing { 4_000_000 } else { 8192 };
        let audio_buffer = Arc::new(Mutex::new(AudioBuffer::new(cap)));
        let mut sample_rate: Option<u32> = None;
        if !capturing {
            if let Some((output, sr)) = AudioOutput::new(audio_buffer.clone()) {
                self.audio_output = Some(output);
                sample_rate = Some(sr);
            }
        }

        let header: CartridgeHeader = CartridgeHeader::new(&rom_data);
        header.print();
        if let Err(e) = header.is_valid() {
            eprintln!("Warning: {}", e);
        }

        #[cfg(test)]
        {
            let _ = sample_rate;
            let mut bus = Bus::new(rom_data);
            bus.set_audio_buffer(audio_buffer);
            return self.run_headless(CPU::new(), bus);
        }

        #[cfg(not(test))]
        {
            if std::env::var("CRABBY_MOONEYE").is_ok() {
                return Self::run_mooneye(CPU::new(), Bus::new(rom_data));
            }
            if let Ok(out) = std::env::var("CRABBY_SHOT") {
                let frames = std::env::var("CRABBY_FRAMES")
                    .ok()
                    .and_then(|f| f.parse().ok())
                    .unwrap_or(600u32);
                let mut bus = Bus::new(rom_data);
                bus.set_audio_buffer(audio_buffer);
                return Self::run_screenshot(CPU::new(), bus, &out, frames);
            }
            let sync_buffer = if self.audio_output.is_some() {
                Some(audio_buffer.clone())
            } else {
                None
            };
            self.run_windowed(rom_data, file_path, audio_buffer, sample_rate, sync_buffer)
        }
    }

    #[cfg(not(test))]
    const THEMES: [[u32; 4]; 4] = [
        crate::hardware::ppu::SHADES,
        [0x00FFFFFF, 0x00A8A8A8, 0x00505050, 0x00000000],
        [0x00FFF6D3, 0x00F9A875, 0x00EB6B6F, 0x007C3F58],
        [0x00C4F0C2, 0x005AB9A8, 0x001E606E, 0x002D1B00],
    ];

    #[cfg(test)]
    fn run_headless(&mut self, mut cpu: CPU, mut bus: Bus) -> Result<(), String> {
        let mut loop_count: u32 = 0;
        loop {
            loop_count = loop_count.wrapping_add(1);
            if loop_count > self.test_max_loop {
                return Err(format!(
                    "Infinite Loop. Reaching loop number: {}",
                    loop_count
                ));
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
        rom_data: Vec<u8>,
        file_path: &str,
        audio_buffer: Arc<Mutex<AudioBuffer>>,
        sample_rate: Option<u32>,
        sync_buffer: Option<Arc<Mutex<AudioBuffer>>>,
    ) -> Result<(), String> {
        use crate::hardware::joypad;
        use minifb::{Key, KeyRepeat, Scale, Window, WindowOptions};

        let save_path = format!("{}.sav", file_path);
        let state_path = format!("{}.state", file_path);

        let build = |theme: usize| -> (CPU, Bus) {
            let mut bus = Bus::new(rom_data.clone());
            if let Some(sr) = sample_rate {
                bus.set_audio_sample_rate(sr);
            }
            bus.set_audio_buffer(audio_buffer.clone());
            bus.set_palette(Self::THEMES[theme]);
            if let Ok(data) = std::fs::read(&save_path) {
                bus.load_save(&data);
            }
            (CPU::new(), bus)
        };

        let mut theme = 0usize;
        let (mut cpu, mut bus) = build(theme);
        let mut paused = false;
        let mut shot_count = 0u32;

        let mut window = Window::new(
            "Crabby Boy",
            160,
            144,
            WindowOptions {
                scale: Scale::X4,
                ..WindowOptions::default()
            },
        )
        .map_err(|e| e.to_string())?;
        if sync_buffer.is_none() {
            window.set_target_fps(60);
        }

        while window.is_open() && !window.is_key_down(Key::Escape) {
            if window.is_key_pressed(Key::P, KeyRepeat::No) {
                paused = !paused;
            }
            if window.is_key_pressed(Key::R, KeyRepeat::No) {
                let m = build(theme);
                cpu = m.0;
                bus = m.1;
            }
            if window.is_key_pressed(Key::C, KeyRepeat::No) {
                theme = (theme + 1) % Self::THEMES.len();
                bus.set_palette(Self::THEMES[theme]);
            }
            if window.is_key_pressed(Key::F12, KeyRepeat::No) {
                let name = format!("{}.shot{}.ppm", file_path, shot_count);
                Self::write_ppm(bus.framebuffer(), &name);
                shot_count += 1;
            }
            if window.is_key_pressed(Key::F5, KeyRepeat::No) {
                match bus.save_state(&cpu) {
                    Ok(data) => {
                        let _ = std::fs::write(&state_path, data);
                        println!("State saved to {}", state_path);
                    }
                    Err(e) => eprintln!("Save state failed: {}", e),
                }
            }
            if window.is_key_pressed(Key::F8, KeyRepeat::No) {
                if let Ok(data) = std::fs::read(&state_path) {
                    if let Err(e) = bus.load_state(&mut cpu, &data) {
                        eprintln!("Load state failed: {}", e);
                    } else {
                        println!("State loaded from {}", state_path);
                    }
                }
            }

            let mut pressed = 0u8;
            for (key, bit) in [
                (Key::Z, joypad::BUTTON_A),
                (Key::X, joypad::BUTTON_B),
                (Key::Backspace, joypad::BUTTON_SELECT),
                (Key::Enter, joypad::BUTTON_START),
                (Key::Right, joypad::BUTTON_RIGHT),
                (Key::Left, joypad::BUTTON_LEFT),
                (Key::Up, joypad::BUTTON_UP),
                (Key::Down, joypad::BUTTON_DOWN),
            ] {
                if window.is_key_down(key) {
                    pressed |= bit;
                }
            }
            bus.set_joypad(pressed);

            let fast = window.is_key_down(Key::Tab);
            if !paused {
                let frames = if fast { 8 } else { 1 };
                for _ in 0..frames {
                    Self::run_frame(&mut cpu, &mut bus, if fast { &None } else { &sync_buffer });
                }
            }

            window
                .update_with_buffer(bus.framebuffer(), 160, 144)
                .map_err(|e| e.to_string())?;
        }

        if let Some(data) = bus.save_data() {
            let _ = std::fs::write(&save_path, data);
        }
        Ok(())
    }

    #[cfg(not(test))]
    fn run_frame(cpu: &mut CPU, bus: &mut Bus, sync: &Option<Arc<Mutex<AudioBuffer>>>) {
        loop {
            if cpu.stopped || cpu.halt {
                bus.internal_tick();
                let ie = bus.get_ie();
                let if_flag = bus.get_io().get_if();
                if (ie & if_flag) != 0 {
                    cpu.stopped = false;
                    cpu.halt = false;
                }
            } else {
                cpu.handle_interrupts(bus);
                cpu.execute(bus);
            }
            if let Some(buffer) = sync {
                while buffer.lock().unwrap().count() >= 4096 {
                    std::thread::sleep(std::time::Duration::from_micros(500));
                }
            }
            if bus.take_frame_ready() {
                break;
            }
        }
    }

    #[cfg(not(test))]
    fn write_ppm(fb: &[u32], path: &str) {
        let mut ppm = String::from("P3\n160 144\n255\n");
        for &px in fb.iter() {
            ppm.push_str(&format!(
                "{} {} {}\n",
                (px >> 16) & 0xFF,
                (px >> 8) & 0xFF,
                px & 0xFF
            ));
        }
        if std::fs::write(path, ppm).is_ok() {
            println!("Screenshot written to {}", path);
        }
    }

    #[cfg(not(test))]
    fn run_mooneye(mut cpu: CPU, mut bus: Bus) -> Result<(), String> {
        let mut budget: u64 = 40_000_000;
        loop {
            if bus.peek(cpu.pc) == 0x40 {
                let fib = cpu.b == 3
                    && cpu.c == 5
                    && cpu.d == 8
                    && cpu.e == 13
                    && cpu.h == 21
                    && cpu.l == 34;
                if fib {
                    println!("MOONEYE_PASS");
                    return Ok(());
                }
                println!("MOONEYE_FAIL");
                return Err("mooneye fail".to_string());
            }
            if cpu.halt || cpu.stopped {
                bus.internal_tick();
                let ie = bus.get_ie();
                let if_flag = bus.get_io().get_if();
                if ie & if_flag != 0 {
                    cpu.halt = false;
                    cpu.stopped = false;
                }
            } else {
                cpu.handle_interrupts(&mut bus);
                cpu.execute(&mut bus);
            }
            budget -= 1;
            if budget == 0 {
                println!("MOONEYE_TIMEOUT");
                return Err("mooneye timeout".to_string());
            }
        }
    }

    #[cfg(not(test))]
    fn run_screenshot(mut cpu: CPU, mut bus: Bus, out: &str, frames: u32) -> Result<(), String> {
        for frame in 0..frames {
            if let Ok(script) = std::env::var("CRABBY_INPUT") {
                use crate::hardware::joypad;
                let mut pressed = 0u8;
                for tok in script.split(',') {
                    let mut it = tok.split(':');
                    if let (Some(f), Some(btn)) = (it.next(), it.next()) {
                        let dur: u32 = it.next().and_then(|d| d.parse().ok()).unwrap_or(4);
                        if f.parse::<u32>()
                            .map_or(false, |ff| frame >= ff && frame < ff + dur)
                        {
                            pressed |= match btn {
                                "start" => joypad::BUTTON_START,
                                "select" => joypad::BUTTON_SELECT,
                                "a" => joypad::BUTTON_A,
                                "b" => joypad::BUTTON_B,
                                "up" => joypad::BUTTON_UP,
                                "down" => joypad::BUTTON_DOWN,
                                "left" => joypad::BUTTON_LEFT,
                                "right" => joypad::BUTTON_RIGHT,
                                _ => 0,
                            };
                        }
                    }
                }
                bus.set_joypad(pressed);
            }
            loop {
                if cpu.stopped || cpu.halt {
                    bus.internal_tick();
                    let ie = bus.get_ie();
                    let if_flag = bus.get_io().get_if();
                    if (ie & if_flag) != 0 {
                        cpu.stopped = false;
                        cpu.halt = false;
                    }
                } else {
                    cpu.handle_interrupts(&mut bus);
                    cpu.execute(&mut bus);
                }
                if bus.take_frame_ready() {
                    break;
                }
            }
        }

        let fb = bus.framebuffer();
        let mut ppm = format!("P3\n160 144\n255\n");
        for &px in fb.iter() {
            let r = (px >> 16) & 0xFF;
            let g = (px >> 8) & 0xFF;
            let b = px & 0xFF;
            ppm.push_str(&format!("{} {} {}\n", r, g, b));
        }
        std::fs::write(out, ppm).map_err(|e| e.to_string())?;
        println!("Screenshot written to {} after {} frames", out, frames);
        if let Ok(path) = std::env::var("CRABBY_WAV") {
            if let Some(buf) = bus.audio_buffer_handle() {
                let mut guard = buf.lock().unwrap();
                let mut samples = Vec::with_capacity(guard.count());
                while !guard.empty() {
                    samples.push(guard.pop());
                }
                let bytes: Vec<u8> =
                    samples.iter().flat_map(|s| s.to_le_bytes()).collect();
                let _ = std::fs::write(&path, bytes);
                println!("Audio: {} samples written to {}", samples.len(), path);
            }
        }
        if let Ok(probe) = std::env::var("CRABBY_PROBE") {
            if let Ok(addr) = u16::from_str_radix(probe.trim_start_matches("0x"), 16) {
                println!("PROBE {:04X} = {:02X}", addr, bus.peek(addr));
            }
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

    fn step_frames(cpu: &mut CPU, bus: &mut Bus, frames: u32) {
        for _ in 0..frames {
            loop {
                if cpu.halt || cpu.stopped {
                    bus.internal_tick();
                    if bus.get_ie() & bus.get_io().get_if() != 0 {
                        cpu.halt = false;
                        cpu.stopped = false;
                    }
                } else {
                    cpu.handle_interrupts(bus);
                    cpu.execute(bus);
                }
                if bus.take_frame_ready() {
                    break;
                }
            }
        }
    }

    #[test]
    fn save_state_roundtrip() {
        let rom = std::fs::read("./tests/Tetris.gb").unwrap();
        let mut cpu = CPU::new();
        let mut bus = Bus::new(rom);
        step_frames(&mut cpu, &mut bus, 200);

        let snapshot = bus.save_state(&cpu).unwrap();
        step_frames(&mut cpu, &mut bus, 30);
        let reference: Vec<u32> = bus.framebuffer().to_vec();

        bus.load_state(&mut cpu, &snapshot).unwrap();
        step_frames(&mut cpu, &mut bus, 30);
        assert_eq!(bus.framebuffer(), reference.as_slice());
    }
}
