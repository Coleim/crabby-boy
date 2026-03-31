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

    pub fn tick(&mut self, cycles: u8) -> bool {
        for _ in 0..cycles {
            let before = self.internal_div;
            self.internal_div = self.internal_div.wrapping_add(1);
            // Check TAC to increment TIMA
            if self.tac & 0b0000_0100 != 0 {
                // println!("TAC");
                let clock_select = self.tac & 0b0000_0011;
                let bit = match clock_select {
                    0 => 9,
                    1 => 3,
                    2 => 5,
                    3 => 7,
                    _ => panic!("Cannot tick {} cycles", cycles),
                };

                let was_set = (before >> bit) & 1 == 1;
                let is_set = (self.internal_div >> bit) & 1 == 1;

                if was_set && !is_set {
                    if (self.tima as u16).wrapping_add(1) > 0xFF {
                        self.tima = self.tma;
                        // println!("TICK - internaldiv {}", self.internal_div);
                        return true;
                    } else {
                        self.tima = self.tima.wrapping_add(1);
                    }
                }
            }
        }

        false
    }

    pub fn read(&self, addr: u16) -> u8 {
        match addr {
            0xFF04 => (self.internal_div >> 8) as u8,
            0xFF05 => self.tima,
            0xFF06 => self.tma,
            0xFF07 => self.tac,
            _ => panic!("Cannot read timer at addr: {:04X}", addr),
        }
    }

    pub fn write(&mut self, addr: u16, val: u8) {
        match addr {
            0xFF04 => self.internal_div = 0,
            0xFF05 => self.tima = val,
            0xFF06 => self.tma = val,
            0xFF07 => self.tac = val,
            _ => panic!("Cannot write timer at addr: {:04X}", addr),
        }
    }
}
