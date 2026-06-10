use std::sync::{Arc, Mutex};

use crate::{audio::buffer::AudioBuffer, cartridge::Cartridge, cpu::CPU, io::IOBridge};
use serde_big_array::BigArray;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Bus {
    cartridge: Cartridge,
    #[serde(with = "BigArray")]
    wram: [u8; 0x2000],
    io: IOBridge,
    #[serde(with = "BigArray")]
    hram: [u8; 0x80],
    ie: u8,
    dma_active: bool,
    dma_base: u16,
    dma_index: u16,
    dma_delay: u8,
}

impl Bus {
    pub fn new(rom_data: Vec<u8>) -> Self {
        Bus {
            cartridge: Cartridge::new(rom_data),
            wram: [0; 0x2000],
            io: IOBridge::new(),
            hram: [0; 0x80],
            ie: 0,
            dma_active: false,
            dma_base: 0,
            dma_index: 0,
            dma_delay: 0,
        }
    }

    pub fn set_audio_buffer(&mut self, buffer: Arc<Mutex<AudioBuffer>>) {
        self.io.set_audio_buffer(buffer);
    }

    pub fn set_audio_sample_rate(&mut self, sample_rate: u32) {
        self.io.set_audio_sample_rate(sample_rate);
    }

    pub fn take_frame_ready(&mut self) -> bool {
        self.io.take_frame_ready()
    }

    pub fn set_joypad(&mut self, pressed: u8) {
        self.io.set_joypad(pressed);
    }

    pub fn framebuffer(&self) -> &[u32] {
        self.io.framebuffer()
    }

    pub fn set_palette(&mut self, palette: [u32; 4]) {
        self.io.set_palette(palette);
    }

    pub fn audio_buffer_handle(&self) -> Option<Arc<Mutex<AudioBuffer>>> {
        self.io.audio_buffer_handle()
    }

    pub fn load_save(&mut self, data: &[u8]) {
        self.cartridge.load_ram(data);
    }

    pub fn save_data(&self) -> Option<&[u8]> {
        self.cartridge.battery_ram()
    }

    pub fn save_state(&self, cpu: &CPU) -> Result<Vec<u8>, String> {
        bincode::serialize(&(cpu, self)).map_err(|e| e.to_string())
    }

    pub fn load_state(&mut self, cpu: &mut CPU, data: &[u8]) -> Result<(), String> {
        let (new_cpu, mut new_bus): (CPU, Bus) =
            bincode::deserialize(data).map_err(|e| e.to_string())?;
        new_bus.cartridge.set_rom(self.cartridge.take_rom());
        if let Some(buffer) = self.io.audio_buffer_handle() {
            new_bus.io.set_audio_buffer(buffer);
        }
        *cpu = new_cpu;
        *self = new_bus;
        Ok(())
    }
    pub fn clear_if(&mut self, mask: u8) {
        self.io.clear_if(mask);
    }
    pub fn get_io(&self) -> &IOBridge {
        &self.io
    }
    pub fn get_ie(&self) -> u8 {
        self.ie
    }

    pub fn internal_tick(&mut self) {
        self.io.tick();
        self.step_dma();
        self.cartridge.tick();
    }

    fn step_dma(&mut self) {
        if !self.dma_active {
            return;
        }
        if self.dma_delay > 0 {
            self.dma_delay -= 1;
            return;
        }
        let byte = self.raw_read(self.dma_base.wrapping_add(self.dma_index));
        self.io.dma_write(self.dma_index as usize, byte);
        self.dma_index += 1;
        if self.dma_index >= 0xA0 {
            self.dma_active = false;
        }
    }

    fn dma_conflict(&self, addr: u16) -> bool {
        self.dma_active && self.dma_delay == 0 && addr < 0xFF00
    }

    pub fn read(&mut self, addr: u16) -> u8 {
        let val = self._read(addr);
        self.internal_tick();
        val
    }

    pub fn peek(&self, addr: u16) -> u8 {
        self.raw_read(addr)
    }

    pub fn write(&mut self, addr: u16, val: u8) {
        self._write(addr, val);
        self.internal_tick();
    }

    fn _read(&self, addr: u16) -> u8 {
        if self.dma_conflict(addr) {
            return 0xFF;
        }
        self.raw_read(addr)
    }

    fn raw_read(&self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x7FFF => self.cartridge.read_rom(addr),
            0x8000..=0x9FFF => self.io.vram_read(addr),
            0xA000..=0xBFFF => self.cartridge.read_ram(addr),
            0xC000..=0xDFFF => self.wram[(addr - 0xC000) as usize],
            0xE000..=0xFDFF => self.wram[(addr - 0xE000) as usize],
            0xFE00..=0xFE9F => self.io.oam_read(addr),
            0xFEA0..=0xFEFF => 0xFF,
            0xFF00..=0xFF7F => self.io.read(addr),
            0xFF80..=0xFFFE => self.hram[(addr - 0xFF80) as usize],
            0xFFFF => self.ie,
        }
    }

    fn _write(&mut self, addr: u16, val: u8) {
        if self.dma_conflict(addr) {
            return;
        }
        match addr {
            0x0000..=0x7FFF => self.cartridge.write_control(addr, val),
            0x8000..=0x9FFF => self.io.vram_write(addr, val),
            0xA000..=0xBFFF => self.cartridge.write_ram(addr, val),
            0xC000..=0xDFFF => self.wram[(addr - 0xC000) as usize] = val,
            0xE000..=0xFDFF => self.wram[(addr - 0xE000) as usize] = val,
            0xFE00..=0xFE9F => self.io.oam_write(addr, val),
            0xFEA0..=0xFEFF => {}
            0xFF46 => {
                self.io.write(addr, val);
                self.dma_base = (val as u16) << 8;
                self.dma_index = 0;
                self.dma_delay = 2;
                self.dma_active = true;
            }
            0xFF00..=0xFF7F => self.io.write(addr, val),
            0xFF80..=0xFFFE => self.hram[(addr - 0xFF80) as usize] = val,
            0xFFFF => self.ie = val,
        }
    }

    #[cfg(test)]
    pub fn get_eram(&self) -> &[u8] {
        self.cartridge.ram()
    }
}
