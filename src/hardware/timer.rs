#[derive(serde::Serialize, serde::Deserialize)]
pub struct Timer {
    div: u16,
    tima: u8,
    tma: u8,
    tac: u8,
    reload_pending: bool,
    reload_delay: u8,
    reloaded_this_cycle: bool,
}

impl Timer {
    pub fn new() -> Self {
        Timer {
            div: 0xABCC,
            tima: 0,
            tma: 0,
            tac: 0xF8,
            reload_pending: false,
            reload_delay: 0,
            reloaded_this_cycle: false,
        }
    }

    fn select_bit(&self) -> u8 {
        match self.tac & 0x03 {
            0 => 9,
            1 => 3,
            2 => 5,
            _ => 7,
        }
    }

    fn timer_signal(&self) -> bool {
        self.tac & 0x04 != 0 && (self.div >> self.select_bit()) & 1 != 0
    }

    fn inc_tima(&mut self) {
        let (next, overflow) = self.tima.overflowing_add(1);
        self.tima = next;
        if overflow {
            self.tima = 0;
            self.reload_pending = true;
            self.reload_delay = 4;
        }
    }

    pub fn tick(&mut self) -> bool {
        let mut irq = false;
        for _ in 0..4 {
            self.reloaded_this_cycle = false;
            if self.reload_pending {
                self.reload_delay -= 1;
                if self.reload_delay == 0 {
                    self.tima = self.tma;
                    self.reload_pending = false;
                    self.reloaded_this_cycle = true;
                    irq = true;
                }
            }
            let before = self.timer_signal();
            self.div = self.div.wrapping_add(1);
            let after = self.timer_signal();
            if before && !after {
                self.inc_tima();
            }
        }
        irq
    }

    pub fn read(&self, addr: u16) -> u8 {
        match addr {
            0xFF04 => (self.div >> 8) as u8,
            0xFF05 => self.tima,
            0xFF06 => self.tma,
            0xFF07 => self.tac | 0xF8,
            _ => 0xFF,
        }
    }

    pub fn write(&mut self, addr: u16, val: u8) {
        match addr {
            0xFF04 => {
                let before = self.timer_signal();
                self.div = 0;
                if before && !self.timer_signal() {
                    self.inc_tima();
                }
            }
            0xFF05 => {
                if !self.reloaded_this_cycle {
                    self.tima = val;
                    self.reload_pending = false;
                }
            }
            0xFF06 => {
                self.tma = val;
                if self.reloaded_this_cycle {
                    self.tima = val;
                }
            }
            0xFF07 => {
                let before = self.timer_signal();
                self.tac = val & 0x07;
                if before && !self.timer_signal() {
                    self.inc_tima();
                }
            }
            _ => {}
        }
    }
}
