use std::sync::{Arc, Mutex};

use crate::{
    audio::{apu::APU, buffer::AudioBuffer},
    hardware::{joypad::Joypad, ppu::PPU, serial::Serial, timer::Timer},
};

#[derive(serde::Serialize, serde::Deserialize)]
pub struct IOBridge {
    joypad: Joypad,
    serial: Serial,
    timer: Timer,
    interrupt_flag: u8,
    audio: APU,
    ppu: PPU,
    key1_spd: u8,
    frame_ready: bool,
}

impl IOBridge {
    pub fn new() -> Self {
        IOBridge {
            joypad: Joypad::new(),
            serial: Serial::new(),
            timer: Timer::new(),
            interrupt_flag: 0,
            audio: APU::new(),
            ppu: PPU::new(),
            key1_spd: 0,
            frame_ready: false,
        }
    }

    pub fn tick(&mut self) {
        if self.timer.tick() {
            self.interrupt_flag |= 0b0000_0100;
        }
        let (vblank, stat) = self.ppu.tick();
        if vblank {
            self.interrupt_flag |= 0b0000_0001;
            self.frame_ready = true;
        }
        if stat {
            self.interrupt_flag |= 0b0000_0010;
        }
        if self.joypad.take_irq() {
            self.interrupt_flag |= 0b0001_0000;
        }
        if self.serial.take_irq() {
            self.interrupt_flag |= 0b0000_1000;
        }
        self.audio.tick();
    }

    pub fn take_frame_ready(&mut self) -> bool {
        let ready = self.frame_ready;
        self.frame_ready = false;
        ready
    }

    pub fn set_joypad(&mut self, pressed: u8) {
        self.joypad.set_pressed(pressed);
    }

    pub fn framebuffer(&self) -> &[u32] {
        self.ppu.framebuffer()
    }

    pub fn set_palette(&mut self, palette: [u32; 4]) {
        self.ppu.set_palette(palette);
    }

    pub fn vram_read(&self, addr: u16) -> u8 {
        self.ppu.vram_read(addr)
    }
    pub fn vram_write(&mut self, addr: u16, val: u8) {
        self.ppu.vram_write(addr, val);
    }
    pub fn oam_read(&self, addr: u16) -> u8 {
        self.ppu.oam_read(addr)
    }
    pub fn oam_write(&mut self, addr: u16, val: u8) {
        self.ppu.oam_write(addr, val);
    }
    pub fn dma_write(&mut self, index: usize, val: u8) {
        self.ppu.dma_write(index, val);
    }

    pub fn set_audio_buffer(&mut self, buffer: Arc<Mutex<AudioBuffer>>) {
        self.audio.set_audio_buffer(buffer);
    }

    pub fn audio_buffer_handle(&self) -> Option<Arc<Mutex<AudioBuffer>>> {
        self.audio.audio_buffer_handle()
    }

    pub fn set_audio_sample_rate(&mut self, sample_rate: u32) {
        self.audio.set_sample_rate(sample_rate);
    }

    pub fn clear_if(&mut self, if_bit: u8) {
        self.interrupt_flag = self.interrupt_flag & !if_bit;
    }
    pub fn get_if(&self) -> u8 {
        self.interrupt_flag
    }
    #[cfg(test)]
    pub fn get_serial(&self) -> &Serial {
        &self.serial
    }

    pub fn read(&self, addr: u16) -> u8 {
        match addr {
            0xFF00 => self.joypad.read(),
            0xFF01..=0xFF02 => self.serial.read(addr),
            0xFF04..=0xFF07 => self.timer.read(addr),
            0xFF0F => self.interrupt_flag | 0b1110_0000,
            0xFF10..=0xFF26 | 0xFF30..=0xFF3F => self.audio.read(addr),
            0xFF40..=0xFF4B => self.ppu.read(addr),
            0xFF4D => 0xFF,
            _ => 0xFF,
        }
    }

    pub fn write(&mut self, addr: u16, val: u8) {
        match addr {
            0xFF00 => self.joypad.write(val),
            0xFF01..=0xFF02 => self.serial.write(addr, val),
            0xFF04..=0xFF07 => self.timer.write(addr, val),
            0xFF0F => self.interrupt_flag = val & 0b0001_1111,
            0xFF10..=0xFF26 | 0xFF30..=0xFF3F => self.audio.write(addr, val),
            0xFF40..=0xFF4B => self.ppu.write(addr, val),
            0xFF4D => self.key1_spd = val,
            _ => {}
        }
    }
}
