pub struct TimerAndDivider {
    internal_div: u16, // The real 16-bit counter // FF04 — DIV: Divider register
    tima: u8,          // FF05 — TIMA: Timer counter
    tma: u8,           // FF06 — TMA: Timer modulo
    tac: u8,           // FF07 — TAC: Timer control
}

impl TimerAndDivider {
    pub fn new() -> Self {
        TimerAndDivider {
            internal_div: 0,
            tima: 0,
            tma: 0,
            tac: 0,
        }
    }
    pub fn read(&self, addr: u16) -> u8 {
        match addr {
            0xFF04 => (self.internal_div >> 8) as u8,
            0xFF05 => self.tima,
            0xFF06 => self.tma,
            0xFF07 => self.tac,
            _ => std::panic::panic_any("Cannot read at this address"),
        }
    }

    pub fn write(&mut self, addr: u16, val: u8) {
        match addr {
            0xFF04 => self.internal_div = 0,
            0xFF05 => self.tima = val,
            0xFF06 => self.tma = val,
            0xFF07 => self.tac = val,
            _ => std::panic::panic_any("Cannot write at this address"),
        }
    }
}
