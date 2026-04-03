use crate::Bus;

const CYCLES: [u8; 256] = [
    4, 12, 8, 8, 4, 4, 8, 4, 20, 8, 8, 8, 4, 4, 8, 4, // 0x00-0x0F
    4, 12, 8, 8, 4, 4, 8, 4, 12, 8, 8, 8, 4, 4, 8, 4, // 0x10-0x1F
    12, 12, 8, 8, 4, 4, 8, 4, 12, 8, 8, 8, 4, 4, 8, 4, // 0x20-0x2F
    12, 12, 8, 8, 12, 12, 12, 4, 12, 8, 8, 8, 4, 4, 8, 4, // 0x30-0x3F
    4, 4, 4, 4, 4, 4, 8, 4, 4, 4, 4, 4, 4, 4, 8, 4, // 0x40-0x4F
    4, 4, 4, 4, 4, 4, 8, 4, 4, 4, 4, 4, 4, 4, 8, 4, // 0x50-0x5F
    4, 4, 4, 4, 4, 4, 8, 4, 4, 4, 4, 4, 4, 4, 8, 4, // 0x60-0x6F
    8, 8, 8, 8, 8, 8, 4, 8, 4, 4, 4, 4, 4, 4, 8, 4, // 0x70-0x7F
    4, 4, 4, 4, 4, 4, 8, 4, 4, 4, 4, 4, 4, 4, 8, 4, // 0x80-0x8F
    4, 4, 4, 4, 4, 4, 8, 4, 4, 4, 4, 4, 4, 4, 8, 4, // 0x90-0x9F
    4, 4, 4, 4, 4, 4, 8, 4, 4, 4, 4, 4, 4, 4, 8, 4, // 0xA0-0xAF
    4, 4, 4, 4, 4, 4, 8, 4, 4, 4, 4, 4, 4, 4, 8, 4, // 0xB0-0xBF
    20, 12, 16, 16, 24, 16, 8, 16, 20, 16, 16, 4, 24, 24, 8, 16, // 0xC0-0xCF
    20, 12, 16, 0, 24, 16, 8, 16, 20, 16, 16, 0, 24, 0, 8, 16, // 0xD0-0xDF
    12, 12, 8, 0, 0, 16, 8, 16, 16, 4, 16, 0, 0, 0, 8, 16, // 0xE0-0xEF
    12, 12, 8, 4, 0, 16, 8, 16, 12, 8, 16, 4, 0, 0, 8, 16, // 0xF0-0xFF
];

const CB_CYCLES: [u8; 256] = [
    8, 8, 8, 8, 8, 8, 16, 8, 8, 8, 8, 8, 8, 8, 16, 8, // 0x00-0x0F
    8, 8, 8, 8, 8, 8, 16, 8, 8, 8, 8, 8, 8, 8, 16, 8, // 0x10-0x1F
    8, 8, 8, 8, 8, 8, 16, 8, 8, 8, 8, 8, 8, 8, 16, 8, // 0x20-0x2F
    8, 8, 8, 8, 8, 8, 16, 8, 8, 8, 8, 8, 8, 8, 16, 8, // 0x30-0x3F
    8, 8, 8, 8, 8, 8, 12, 8, 8, 8, 8, 8, 8, 8, 12, 8, // 0x40-0x4F
    8, 8, 8, 8, 8, 8, 12, 8, 8, 8, 8, 8, 8, 8, 12, 8, // 0x50-0x5F
    8, 8, 8, 8, 8, 8, 12, 8, 8, 8, 8, 8, 8, 8, 12, 8, // 0x60-0x6F
    8, 8, 8, 8, 8, 8, 12, 8, 8, 8, 8, 8, 8, 8, 12, 8, // 0x70-0x7F
    8, 8, 8, 8, 8, 8, 16, 8, 8, 8, 8, 8, 8, 8, 16, 8, // 0x80-0x8F
    8, 8, 8, 8, 8, 8, 16, 8, 8, 8, 8, 8, 8, 8, 16, 8, // 0x90-0x9F
    8, 8, 8, 8, 8, 8, 16, 8, 8, 8, 8, 8, 8, 8, 16, 8, // 0xA0-0xAF
    8, 8, 8, 8, 8, 8, 16, 8, 8, 8, 8, 8, 8, 8, 16, 8, // 0xB0-0xBF
    8, 8, 8, 8, 8, 8, 16, 8, 8, 8, 8, 8, 8, 8, 16, 8, // 0xC0-0xCF
    8, 8, 8, 8, 8, 8, 16, 8, 8, 8, 8, 8, 8, 8, 16, 8, // 0xD0-0xDF
    8, 8, 8, 8, 8, 8, 16, 8, 8, 8, 8, 8, 8, 8, 16, 8, // 0xE0-0xEF
    8, 8, 8, 8, 8, 8, 16, 8, 8, 8, 8, 8, 8, 8, 16, 8, // 0xF0-0xFF
];

pub struct CPU {
    pub a: u8,
    pub f: u8, // F = Z,N,H,C
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,
    pub h: u8,
    pub l: u8,
    pub pc: u16,
    pub sp: u16,
    pub stopped: bool,
    pub halt: bool,
    pub halt_bug: bool,
    pub ime: bool,
    pub ime_pending: bool,
}

impl CPU {
    pub fn new() -> Self {
        CPU {
            a: 0x01,
            f: 0xB0,
            b: 0x00, // $00,
            c: 0x13,
            d: 0x00,
            e: 0xD8,    // $D8
            h: 0x01,    // $01
            l: 0x4D,    // $4D
            pc: 0x0100, // $0100
            sp: 0xFFFE, // $FFFE
            stopped: false,
            halt: false,
            halt_bug: false,
            ime: false,
            ime_pending: false,
        }
    }

    fn set_z(&mut self, val: bool) {
        if val {
            self.f |= 0x80;
        } else {
            self.f &= !0x80;
        }
    }
    fn set_n(&mut self, val: bool) {
        if val {
            self.f |= 0x40;
        } else {
            self.f &= !0x40;
        }
    }
    fn set_h(&mut self, val: bool) {
        if val {
            self.f |= 0x20;
        } else {
            self.f &= !0x20;
        }
    }
    fn set_c(&mut self, val: bool) {
        if val {
            self.f |= 0x10;
        } else {
            self.f &= !0x10;
        }
    }

    fn get_z(&self) -> bool {
        self.f & 0x80 != 0
    }
    fn get_n(&self) -> bool {
        self.f & 0x40 != 0
    }
    fn get_h(&self) -> bool {
        self.f & 0x20 != 0
    }
    fn get_c(&self) -> bool {
        self.f & 0x10 != 0
    }

    fn read16bytes(&self, bus: &Bus, addr: u16) -> u16 {
        let low = bus.read(addr) as u16;
        let high = bus.read(addr.wrapping_add(1)) as u16;
        (high << 8) | low
    }

    fn write16bytes(&mut self, bus: &mut Bus, addr: u16, value: u16) {
        bus.write(addr, (value & 0xFF) as u8);
        bus.write(addr.wrapping_add(1), (value >> 8) as u8);
    }

    fn split_u16(value: u16) -> (u8, u8) {
        (((value >> 8) as u8), (value & 0xFF) as u8)
    }
    fn combine_u8(x: u8, y: u8) -> u16 {
        ((x as u16) << 8) | (y as u16)
    }

    fn set_hl(&mut self, val: u16) {
        let (h, l) = Self::split_u16(val);
        self.h = h;
        self.l = l;
    }
    fn set_bc(&mut self, val: u16) {
        let (b, c) = Self::split_u16(val);
        self.b = b;
        self.c = c;
    }
    fn set_af(&mut self, val: u16) {
        let (a, f) = Self::split_u16(val);
        self.a = a;
        self.f = f;
    }
    fn set_de(&mut self, val: u16) {
        let (d, e) = Self::split_u16(val);
        self.d = d;
        self.e = e;
    }

    fn get_hl(&self) -> u16 {
        Self::combine_u8(self.h, self.l)
    }
    fn get_bc(&self) -> u16 {
        Self::combine_u8(self.b, self.c)
    }
    fn get_af(&self) -> u16 {
        Self::combine_u8(self.a, self.f)
    }
    fn get_de(&self) -> u16 {
        Self::combine_u8(self.d, self.e)
    }
    fn push_bc(&mut self, bus: &mut Bus) {
        self.push_to_stack(self.get_bc(), bus);
    }
    fn push_af(&mut self, bus: &mut Bus) {
        self.push_to_stack(self.get_af(), bus);
    }
    fn push_hl(&mut self, bus: &mut Bus) {
        self.push_to_stack(self.get_hl(), bus);
    }
    fn push_de(&mut self, bus: &mut Bus) {
        self.push_to_stack(self.get_de(), bus);
    }
    fn push_to_stack(&mut self, reg: u16, bus: &mut Bus) {
        self.sp = self.sp.wrapping_sub(2);
        self.write16bytes(bus, self.sp, reg);
    }

    fn pop_hl(&mut self, bus: &mut Bus) {
        let stack = self.pop_from_stack(bus);
        self.set_hl(stack);
    }
    fn pop_bc(&mut self, bus: &mut Bus) {
        let stack = self.pop_from_stack(bus);
        self.set_bc(stack);
    }
    fn pop_de(&mut self, bus: &mut Bus) {
        let stack = self.pop_from_stack(bus);
        self.set_de(stack);
    }
    fn pop_af(&mut self, bus: &mut Bus) {
        let value = self.pop_from_stack(bus);
        self.set_af(value);
        self.f = (value & 0xF0) as u8;
    }

    fn pop_from_stack(&mut self, bus: &mut Bus) -> u16 {
        let stack_val: u16 = self.read16bytes(bus, self.sp);
        self.sp = self.sp.wrapping_add(2);
        stack_val
    }

    fn load_16_to(&mut self, bus: &mut Bus, addr: u16, setter: fn(&mut Self, u16)) -> u16 {
        let n16 = self.read16bytes(bus, addr);
        setter(self, n16);
        addr.wrapping_add(2)
    }

    fn and(&mut self, val: u8) {
        self.a = self.a & val;
        self.set_z(self.a == 0);
        self.set_n(false);
        self.set_h(true);
        self.set_c(false);
    }

    fn xor(&mut self, val: u8) {
        self.a = self.a ^ val;
        self.set_z(self.a == 0);
        self.set_n(false);
        self.set_h(false);
        self.set_c(false);
    }

    fn or(&mut self, val: u8) {
        self.a = self.a | val;
        self.set_z(self.a == 0);
        self.set_n(false);
        self.set_h(false);
        self.set_c(false);
    }

    fn cp(&mut self, val: u8) {
        self.set_z(self.a.wrapping_sub(val) == 0);
        self.set_n(true);
        self.set_h((self.a & 0xF) < (val & 0xF));
        self.set_c(self.a < val);
    }

    fn increment(&mut self, val: u8) -> u8 {
        let h = (val & 0x0F) == 0x0F;
        let result = val.wrapping_add(1);
        self.set_z(result == 0);
        self.set_n(false);
        self.set_h(h);
        result
    }

    fn decrement(&mut self, val: u8) -> u8 {
        let h = (val & 0x0F) == 0x00;
        let result = val.wrapping_sub(1);
        self.set_z(result == 0);
        self.set_n(true);
        self.set_h(h);
        result
    }

    fn jump_relative_if(&self, bus: &Bus, addr: u16, condition: bool) -> (u16, u8) {
        let e8 = bus.read(addr) as i8;
        let mut new_addr = addr.wrapping_add(1);
        let mut cycles = 8;
        if condition {
            new_addr = new_addr.wrapping_add_signed(e8 as i16);
            cycles = 12;
        }
        (new_addr, cycles)
    }

    fn jump_if(&self, bus: &Bus, addr: u16, condition: bool) -> (u16, u8) {
        // JP a16 3  16 - - - -
        let a16 = self.read16bytes(bus, addr);
        let mut new_addr = addr.wrapping_add(2);
        let mut cycles = 12;
        if condition {
            new_addr = a16;
            cycles = 16;
        }
        (new_addr, cycles)
    }

    fn rst(&mut self, bus: &mut Bus, curr: u16, dest: u16) -> u16 {
        self.sp = self.sp.wrapping_sub(2);
        self.write16bytes(bus, self.sp, curr);
        dest
    }

    fn call_if(&mut self, condition: bool, bus: &mut Bus, addr: u16) -> (u16, u8) {
        let ret = addr.wrapping_add(2); // Adresse de retour
        // println!("CALL RET:0x{:04X} ADDR: 0x{:04X}", ret, addr);
        if condition {
            let n16 = self.read16bytes(bus, addr); // Adresse cible (du CALL)
            self.sp = self.sp.wrapping_sub(2); // Décrémenter SP pour empiler sur la stack
            self.write16bytes(bus, self.sp, ret); // Ecrire l'adresse de retour sur la stack
            // println!("CALL n16: 0x{:04X} SP: 0x{:04X}", n16, self.sp);
            (n16, 24) // Aller a l'adresse du CALL
        } else {
            (ret, 12)
        }
    }

    fn return_if(&mut self, condition: bool, bus: &mut Bus, current_pc: u16) -> (u16, u8) {
        if condition {
            // return from fonction call
            let n16 = self.read16bytes(bus, self.sp);
            self.sp = self.sp.wrapping_add(2);
            (n16, 20)
        } else {
            (current_pc, 8)
        }
    }

    fn add_to_a(&mut self, val: u8, carry: u8) {
        self.set_n(false);
        self.set_h((self.a & 0xF) + (val & 0xF) + (carry & 0xF) > 0xF);
        self.set_c((self.a as u16) + (val as u16) + (carry as u16) > 0xFF);
        self.a = self.a.wrapping_add(val).wrapping_add(carry);
        self.set_z(self.a == 0);
    }

    fn add_16b_registers(&mut self, reg1: u16, reg2: u16, setter: fn(&mut Self, u16)) {
        self.set_n(false);
        self.set_h((reg1 & 0xFFF) + (reg2 & 0xFFF) > 0xFFF);
        self.set_c((reg1 as u32) + (reg2 as u32) > 0xFFFF);
        setter(self, reg1.wrapping_add(reg2));
    }

    fn sub_from_a(&mut self, val: u8, carry: u8) {
        self.set_n(true);
        // SUB — half-carry if lower nibble would borrow
        self.set_h((self.a & 0xF) < (val & 0xF) + carry);
        // SUB — carry if it would borrow
        self.set_c((self.a as u16) < (val as u16) + (carry as u16));
        self.a = self.a.wrapping_sub(val).wrapping_sub(carry);
        self.set_z(self.a == 0);
    }

    fn rotate_left(value: u8) -> u8 {
        value << 1 | value >> 7
    }
    fn rotate_right(value: u8) -> u8 {
        value >> 1 | value << 7
    }

    fn rlca(&mut self) {
        self.set_c((self.a & 0x80) != 0);
        self.set_z(false);
        self.set_n(false);
        self.set_h(false);
        self.a = CPU::rotate_left(self.a);
    }

    fn rrca(&mut self) {
        self.set_c((self.a & 0x01) != 0);
        self.set_z(false);
        self.set_n(false);
        self.set_h(false);
        self.a = CPU::rotate_right(self.a);
    }

    fn rra(&mut self) {
        let old_carry = self.get_c() as u8;
        self.set_c((self.a & 0x01) != 0);
        self.a = self.a >> 1 | (old_carry << 7);
        self.set_z(false);
        self.set_n(false);
        self.set_h(false);
    }
    fn rla(&mut self) {
        let old_carry = self.get_c() as u8;
        self.set_c((self.a & 0x80) != 0);
        self.a = self.a << 1 | old_carry;
        self.set_z(false);
        self.set_n(false);
        self.set_h(false);
    }

    pub fn handle_interrupts(&mut self, bus: &mut Bus) {
        if self.ime == false {
            return;
        }
        let ie = bus.get_ie();
        let iflag = bus.get_io().get_if();

        let triggered = ie & iflag;

        if triggered == 0 {
            return;
        }

        let bit = triggered.trailing_zeros() as u8;
        // Clear that bit in IF — mark it as handled
        let mask = 1 << bit;
        bus.clear_if(mask);
        self.ime = false;
        // Push current PC onto stack
        self.sp = self.sp.wrapping_sub(2);
        self.write16bytes(bus, self.sp, self.pc);

        // Jump to the corresponding vector address
        match bit {
            0 => self.pc = 0x0040, // VBlank
            1 => self.pc = 0x0048, // LCD STAT
            2 => self.pc = 0x0050, // Timer
            3 => self.pc = 0x0058, // Serial
            4 => self.pc = 0x0060, // Joypad
            _ => std::panic!("Unexpected interrupt {:02x}", bit),
        }
    }

    // pub fn execute(&mut self, bus: &mut Bus) -> bool {
    pub fn execute(&mut self, bus: &mut Bus) -> Option<u8> {
        // if self.sp < 0xDFF0 {
        //     println!("STACK DEEP: SP=0x{:04X} PC=0x{:04X}", self.sp, self.pc);
        // }

        if self.ime_pending {
            self.ime = true;
            self.ime_pending = false;
        }

        let opcode: u8 = bus.read(self.pc);
        let mut cycles: u8 = CYCLES[opcode as usize];

        // println!(
        //     "EXECUTE PC: 0x{:04X} OP: 0x{:02X} SP: 0x{:04X}",
        //     self.pc, opcode, self.sp
        // );
        //
        let mut next_pc: u16 = if self.halt_bug {
            println!("HALT BUG  PC: {:02x}", self.pc);
            self.halt_bug = false;
            self.pc
        } else {
            self.pc.wrapping_add(1)
        };

        match opcode {
            0x00 => {} // NOP 1  4
            0x10 => {
                println!("STOPPING CPU with opcode 0x10");
                self.stopped = true; // STOP n8 2  4
            }
            0x01 => next_pc = self.load_16_to(bus, next_pc, Self::set_bc),
            0x11 => next_pc = self.load_16_to(bus, next_pc, Self::set_de),
            0x21 => next_pc = self.load_16_to(bus, next_pc, Self::set_hl),
            0x31 => next_pc = self.load_16_to(bus, next_pc, |this, val| this.sp = val),

            0x02 => bus.write(self.get_bc(), self.a),
            0x12 => bus.write(self.get_de(), self.a),
            0x22 => {
                let hl = self.get_hl();
                bus.write(hl, self.a);
                self.set_hl(hl.wrapping_add(1));
            }
            0x32 => {
                let hl = self.get_hl();
                bus.write(hl, self.a);
                self.set_hl(hl.wrapping_sub(1));
            }

            // INC
            0x03 => self.set_bc(self.get_bc().wrapping_add(1)),
            0x13 => self.set_de(self.get_de().wrapping_add(1)),
            0x23 => self.set_hl(self.get_hl().wrapping_add(1)),
            0x33 => self.sp = self.sp.wrapping_add(1), // INC SP 1  8

            0x04 => self.b = self.increment(self.b), // INC B 1  4 Z 0 H -
            0x14 => self.d = self.increment(self.d), // INC D 1  4 Z 0 H -
            0x24 => self.h = self.increment(self.h), // INC D 1  4 Z 0 H -
            0x34 => {
                // INC [HL] 1  12 Z 0 H -
                let mut val = bus.read(self.get_hl());
                val = self.increment(val);
                bus.write(self.get_hl(), val);
            }
            0x0C => self.c = self.increment(self.c), // INC C 1  4 Z 0 H -
            0x1C => self.e = self.increment(self.e), // INC E 1  4
            0x2C => self.l = self.increment(self.l), // INC L 1  4
            0x3C => self.a = self.increment(self.a), // INC A 1  4

            // DEC
            0x05 => self.b = self.decrement(self.b),
            0x15 => self.d = self.decrement(self.d),
            0x25 => self.h = self.decrement(self.h),
            0x35 => {
                // DEC [HL] 1  12 Z 1 H -
                let mut val = bus.read(self.get_hl());
                val = self.decrement(val);
                bus.write(self.get_hl(), val);
            }

            0x0B => self.set_bc(self.get_bc().wrapping_sub(1)),
            0x1B => self.set_de(self.get_de().wrapping_sub(1)),
            0x2B => self.set_hl(self.get_hl().wrapping_sub(1)),
            0x3B => self.sp = self.sp.wrapping_sub(1),

            0x0D => self.c = self.decrement(self.c),
            0x1D => self.e = self.decrement(self.e),
            0x2D => self.l = self.decrement(self.l),
            0x3D => self.a = self.decrement(self.a),

            // LD
            0x06 => {
                self.b = bus.read(next_pc);
                next_pc = next_pc.wrapping_add(1);
            }
            0x08 => {
                let a16 = self.read16bytes(bus, next_pc);
                self.write16bytes(bus, a16, self.sp);
                next_pc = next_pc.wrapping_add(2);
            }
            0x16 => {
                self.d = bus.read(next_pc);
                next_pc = next_pc.wrapping_add(1);
            }
            0x26 => {
                self.h = bus.read(next_pc);
                next_pc = next_pc.wrapping_add(1);
            }
            0x36 => {
                bus.write(self.get_hl(), bus.read(next_pc));
                next_pc = next_pc.wrapping_add(1);
            }

            0x0E => {
                self.c = bus.read(next_pc);
                next_pc = next_pc.wrapping_add(1);
            }
            0x1E => {
                self.e = bus.read(next_pc);
                next_pc = next_pc.wrapping_add(1);
            }
            0x2E => {
                self.l = bus.read(next_pc);
                next_pc = next_pc.wrapping_add(1);
            }
            0x3E => {
                self.a = bus.read(next_pc);
                next_pc = next_pc.wrapping_add(1);
            }

            0xEA => {
                let addr = self.read16bytes(bus, next_pc);
                bus.write(addr, self.a);
                next_pc = next_pc.wrapping_add(2);
            }
            0xFA => {
                let addr = self.read16bytes(bus, next_pc);
                self.a = bus.read(addr);
                next_pc = next_pc.wrapping_add(2);
            }
            0xF8 => {
                self.set_z(false);
                self.set_n(false);

                let byte = bus.read(next_pc);
                let s8 = byte as i8;
                self.set_h((self.sp & 0xF) + (byte as u16 & 0xF) > 0xF);
                self.set_c((self.sp & 0xFF) + (byte as u16) > 0xFF);
                self.set_hl(self.sp.wrapping_add_signed(s8 as i16));
                next_pc = next_pc.wrapping_add(1);
            }
            0xF9 => self.sp = self.get_hl(),

            0x0A => self.a = bus.read(self.get_bc()),
            0x1A => self.a = bus.read(self.get_de()),
            0x2A => {
                let hl: u16 = self.get_hl();
                self.a = bus.read(hl);
                self.set_hl(hl.wrapping_add(1));
            }
            0x3A => {
                let hl: u16 = self.get_hl();
                self.a = bus.read(hl);
                self.set_hl(hl.wrapping_sub(1));
            }

            0x40 => {}               //self.b = self.b, // LD B, B 1  4
            0x41 => self.b = self.c, // LD B, C 1  4
            0x42 => self.b = self.d, // LD B, D 1  4
            0x43 => self.b = self.e, // LD B, E 1  4
            0x44 => self.b = self.h,
            0x45 => self.b = self.l,
            0x46 => self.b = bus.read(self.get_hl()),
            0x47 => self.b = self.a, // LD B, A 1  4

            0x48 => self.c = self.b,
            0x49 => {} // self.c = self.c,
            0x4A => self.c = self.d,
            0x4B => self.c = self.e,
            0x4C => self.c = self.h,
            0x4D => self.c = self.l,
            0x4E => self.c = bus.read(self.get_hl()),
            0x4F => self.c = self.a,

            0x50 => self.d = self.b,
            0x51 => self.d = self.c,
            0x52 => {} // self.d = self.d,
            0x53 => self.d = self.e,
            0x54 => self.d = self.h,
            0x55 => self.d = self.l,
            0x56 => self.d = bus.read(self.get_hl()),
            0x57 => self.d = self.a,

            0x58 => self.e = self.b,
            0x59 => self.e = self.c,
            0x5A => self.e = self.d,
            0x5B => {} // self.e = self.e,
            0x5C => self.e = self.h,
            0x5D => self.e = self.l,
            0x5E => self.e = bus.read(self.get_hl()),
            0x5F => self.e = self.a,

            0x60 => self.h = self.b,
            0x61 => self.h = self.c,
            0x62 => self.h = self.d,
            0x63 => self.h = self.e,
            0x64 => {} // self.h = self.h,
            0x65 => self.h = self.l,
            0x66 => self.h = bus.read(self.get_hl()),
            0x67 => self.h = self.a,

            0x68 => self.l = self.b,
            0x69 => self.l = self.c,
            0x6A => self.l = self.d,
            0x6B => self.l = self.e,
            0x6C => self.l = self.h,
            0x6D => {} // self.l = self.l,
            0x6E => self.l = bus.read(self.get_hl()),
            0x6F => self.l = self.a,

            0x70 => bus.write(self.get_hl(), self.b),
            0x71 => bus.write(self.get_hl(), self.c),
            0x72 => bus.write(self.get_hl(), self.d),
            0x73 => bus.write(self.get_hl(), self.e),
            0x74 => bus.write(self.get_hl(), self.h),
            0x75 => bus.write(self.get_hl(), self.l),
            0x76 => {
                let ie = bus.get_ie();
                let if_flag = bus.get_io().get_if();
                // println!(
                //     "0x76 HALT → IME: {} IE: 0x{:02X} IF: 0x{:02X}",
                //     self.ime, ie, if_flag
                // );
                if !self.ime && (ie & if_flag) != 0 {
                    println!(
                        "HALT BUG: F={:02X} PC={:04X} A={:02X} B={:02X} C={:02X} D={:02X} E={:02X}",
                        self.f, self.pc, self.a, self.b, self.c, self.d, self.e
                    );
                    println!(
                        "HALT BUG: IE={:02X} IF={:02X} F={:02X} PC={:04X}",
                        ie, if_flag, self.f, self.pc
                    );

                    self.halt_bug = true;
                }

                if !self.ime && (ie & if_flag) != 0 {
                    // HALT BUG — don't halt, just corrupt next fetch
                    self.halt_bug = true;
                    // println!("HALT BUG TRIGGERED");
                } else {
                    self.halt = true;
                }
            }
            0x77 => bus.write(self.get_hl(), self.a),

            // LD (HL),B
            0x78 => self.a = self.b,
            0x79 => self.a = self.c,
            0x7A => self.a = self.d,
            0x7B => self.a = self.e,
            0x7C => self.a = self.h,
            0x7D => self.a = self.l,
            0x7E => self.a = bus.read(self.get_hl()),
            0x7F => {} // self.a = self.a,

            // Jumps relative
            0x18 => (next_pc, _) = self.jump_relative_if(bus, next_pc, true),
            0x20 => (next_pc, cycles) = self.jump_relative_if(bus, next_pc, !self.get_z()),
            0x30 => (next_pc, cycles) = self.jump_relative_if(bus, next_pc, !self.get_c()),
            0x28 => (next_pc, cycles) = self.jump_relative_if(bus, next_pc, self.get_z()),
            0x38 => (next_pc, cycles) = self.jump_relative_if(bus, next_pc, self.get_c()),

            // Jump
            0xC3 => (next_pc, _) = self.jump_if(bus, next_pc, true),
            0xC2 => (next_pc, cycles) = self.jump_if(bus, next_pc, !self.get_z()),
            0xD2 => (next_pc, cycles) = self.jump_if(bus, next_pc, !self.get_c()),
            0xCA => (next_pc, cycles) = self.jump_if(bus, next_pc, self.get_z()),
            0xDA => (next_pc, cycles) = self.jump_if(bus, next_pc, self.get_c()),
            0xE9 => next_pc = self.get_hl(),

            0xF3 => self.ime = false,
            0xFB => self.ime_pending = true,

            // CALL
            0xC4 => (next_pc, cycles) = self.call_if(!self.get_z(), bus, next_pc),
            0xD4 => (next_pc, cycles) = self.call_if(!self.get_c(), bus, next_pc),
            0xCC => (next_pc, cycles) = self.call_if(self.get_z(), bus, next_pc),
            0xDC => (next_pc, cycles) = self.call_if(self.get_c(), bus, next_pc),
            0xCD => (next_pc, _) = self.call_if(true, bus, next_pc),

            // RET
            0xC0 => (next_pc, cycles) = self.return_if(!self.get_z(), bus, next_pc),
            0xD0 => (next_pc, cycles) = self.return_if(!self.get_c(), bus, next_pc),
            0xC8 => (next_pc, cycles) = self.return_if(self.get_z(), bus, next_pc),
            0xD8 => (next_pc, cycles) = self.return_if(self.get_c(), bus, next_pc),
            0xD9 => {
                (next_pc, _) = self.return_if(true, bus, next_pc);
                self.ime = true;
            }
            0xC9 => (next_pc, _) = self.return_if(true, bus, next_pc),

            // RST
            0xC7 => next_pc = self.rst(bus, next_pc, 0x00),
            0xD7 => next_pc = self.rst(bus, next_pc, 0x10),
            0xE7 => next_pc = self.rst(bus, next_pc, 0x20),
            0xF7 => next_pc = self.rst(bus, next_pc, 0x30),
            0xCF => next_pc = self.rst(bus, next_pc, 0x08),
            0xDF => next_pc = self.rst(bus, next_pc, 0x18),
            0xEF => next_pc = self.rst(bus, next_pc, 0x28),
            0xFF => next_pc = self.rst(bus, next_pc, 0x38),

            0xC1 => self.pop_bc(bus),
            0xD1 => self.pop_de(bus),
            0xE1 => self.pop_hl(bus),
            0xF1 => self.pop_af(bus),

            0xC5 => self.push_bc(bus),
            0xD5 => self.push_de(bus),
            0xE5 => self.push_hl(bus),
            0xF5 => self.push_af(bus),

            0xA0 => self.and(self.b),
            0xA1 => self.and(self.c),
            0xA2 => self.and(self.d),
            0xA3 => self.and(self.e),
            0xA4 => self.and(self.h),
            0xA5 => self.and(self.l),
            0xA6 => self.and(bus.read(self.get_hl())),
            0xA7 => self.and(self.a),
            0xE6 => {
                self.and(bus.read(next_pc));
                next_pc = next_pc.wrapping_add(1);
            }

            0xA8 => self.xor(self.b),
            0xA9 => self.xor(self.c),
            0xAA => self.xor(self.d),
            0xAB => self.xor(self.e),
            0xAC => self.xor(self.h),
            0xAD => self.xor(self.l),
            0xAE => self.xor(bus.read(self.get_hl())),
            0xAF => self.xor(self.a),
            0xEE => {
                self.xor(bus.read(next_pc));
                next_pc = next_pc.wrapping_add(1);
            }

            0xB0 => self.or(self.b),
            0xB1 => self.or(self.c),
            0xB2 => self.or(self.d),
            0xB3 => self.or(self.e),
            0xB4 => self.or(self.h),
            0xB5 => self.or(self.l),
            0xB6 => self.or(bus.read(self.get_hl())),
            0xB7 => self.or(self.a),
            0xF6 => {
                self.or(bus.read(next_pc));
                next_pc = next_pc.wrapping_add(1);
            }

            // Load High
            0xE0 => {
                let a8 = bus.read(next_pc) as u16;
                let dest_addr = 0xFF00 | a8;
                bus.write(dest_addr, self.a);
                next_pc = next_pc.wrapping_add(1);
            }
            0xF0 => {
                let a8 = bus.read(next_pc) as u16;
                let src_addr = 0xFF00 | a8;
                self.a = bus.read(src_addr);
                next_pc = next_pc.wrapping_add(1);
            }
            0xE2 => {
                let src_addr = 0xFF00 | self.c as u16;
                bus.write(src_addr, self.a);
            }
            0xF2 => {
                let src_addr = 0xFF00 | self.c as u16;
                self.a = bus.read(src_addr);
            }

            // CP
            0xB8 => self.cp(self.b),
            0xB9 => self.cp(self.c),
            0xBA => self.cp(self.d),
            0xBB => self.cp(self.e),
            0xBC => self.cp(self.h),
            0xBD => self.cp(self.l),
            0xBE => self.cp(bus.read(self.get_hl())),
            0xBF => self.cp(self.a),
            0xFE => {
                let n8 = bus.read(next_pc);
                next_pc = next_pc.wrapping_add(1);
                self.cp(n8);
            }

            // ADD
            0x80 => self.add_to_a(self.b, 0),
            0x81 => self.add_to_a(self.c, 0),
            0x82 => self.add_to_a(self.d, 0),
            0x83 => self.add_to_a(self.e, 0),
            0x84 => self.add_to_a(self.h, 0),
            0x85 => self.add_to_a(self.l, 0),
            0x86 => self.add_to_a(bus.read(self.get_hl()), 0),
            0x87 => self.add_to_a(self.a, 0),
            0xC6 => {
                let n8 = bus.read(next_pc);
                self.add_to_a(n8, 0);
                next_pc = next_pc.wrapping_add(1);
            }

            0x09 => self.add_16b_registers(self.get_hl(), self.get_bc(), Self::set_hl),
            0x19 => self.add_16b_registers(self.get_hl(), self.get_de(), Self::set_hl),
            0x29 => self.add_16b_registers(self.get_hl(), self.get_hl(), Self::set_hl),
            0x39 => self.add_16b_registers(self.get_hl(), self.sp, Self::set_hl),

            0xE8 => {
                let byte = bus.read(next_pc);
                let s8 = byte as i8;
                let u8 = byte as u16;
                next_pc = next_pc.wrapping_add(1);
                self.set_z(false);
                self.set_n(false);
                self.set_h((self.sp & 0xF) + (u8 & 0xF) > 0xF);
                self.set_c((self.sp & 0xFF) + (u8 & 0xFF) > 0xFF);
                self.sp = self.sp.wrapping_add_signed(s8 as i16);
            }

            // ADC
            0x88 => self.add_to_a(self.b, self.get_c() as u8),
            0x89 => self.add_to_a(self.c, self.get_c() as u8),
            0x8A => self.add_to_a(self.d, self.get_c() as u8),
            0x8B => self.add_to_a(self.e, self.get_c() as u8),
            0x8C => self.add_to_a(self.h, self.get_c() as u8),
            0x8D => self.add_to_a(self.l, self.get_c() as u8),
            0x8E => self.add_to_a(bus.read(self.get_hl()), self.get_c() as u8),
            0x8F => self.add_to_a(self.a, self.get_c() as u8),
            0xCE => {
                let n8 = bus.read(next_pc);
                self.add_to_a(n8, self.get_c() as u8);
                next_pc = next_pc.wrapping_add(1);
            }

            //SUB
            0x90 => self.sub_from_a(self.b, 0),
            0x91 => self.sub_from_a(self.c, 0),
            0x92 => self.sub_from_a(self.d, 0),
            0x93 => self.sub_from_a(self.e, 0),
            0x94 => self.sub_from_a(self.h, 0),
            0x95 => self.sub_from_a(self.l, 0),
            0x96 => self.sub_from_a(bus.read(self.get_hl()), 0),
            0x97 => self.sub_from_a(self.a, 0),
            0xD6 => {
                let n8 = bus.read(next_pc);
                self.sub_from_a(n8, 0);
                next_pc = next_pc.wrapping_add(1);
            }

            // SBC
            0x98 => self.sub_from_a(self.b, self.get_c() as u8),
            0x99 => self.sub_from_a(self.c, self.get_c() as u8),
            0x9A => self.sub_from_a(self.d, self.get_c() as u8),
            0x9B => self.sub_from_a(self.e, self.get_c() as u8),
            0x9C => self.sub_from_a(self.h, self.get_c() as u8),
            0x9D => self.sub_from_a(self.l, self.get_c() as u8),
            0x9E => self.sub_from_a(bus.read(self.get_hl()), self.get_c() as u8),
            0x9F => self.sub_from_a(self.a, self.get_c() as u8),
            0xDE => {
                let n8 = bus.read(next_pc);
                self.sub_from_a(n8, self.get_c() as u8);
                next_pc = next_pc.wrapping_add(1);
            }

            0xCB => return self.execute_cb(bus, next_pc),

            // Rotate
            0x07 => self.rlca(),
            0x17 => self.rla(),
            0x0F => self.rrca(),
            0x1F => self.rra(),

            // CPL
            0x2F => {
                self.a = !self.a;
                self.set_n(true);
                self.set_h(true);
            }
            // CCF
            0x3F => {
                self.set_c(!self.get_c());
                self.set_n(false);
                self.set_h(false);
            }
            // SCF
            0x37 => {
                self.set_c(true);
                self.set_n(false);
                self.set_h(false);
            }

            // DAA
            0x27 => {
                let mut adjustment = 0;
                if self.get_n() {
                    if self.get_h() {
                        adjustment = adjustment + 0x06;
                    }
                    if self.get_c() {
                        adjustment = adjustment + 0x60;
                    }
                    self.a = self.a.wrapping_sub(adjustment);
                } else {
                    if self.get_h() || (self.a & 0xF) > 0x9 {
                        adjustment = adjustment + 0x06;
                    }
                    if self.get_c() || self.a > 0x99 {
                        adjustment = adjustment + 0x60;
                        self.set_c(true);
                    }
                    self.a = self.a.wrapping_add(adjustment);
                }
                self.set_h(false);
                self.set_z(self.a == 0);
            }

            _ => {
                println!(
                    "Unimplemented opcode: 0x{:02X} at PC: 0x{:04X}",
                    opcode, self.pc
                );
                return None;
            }
        }

        self.pc = next_pc;
        return Some(cycles);
    }

    fn sla(&mut self, reg: u8, bus: &mut Bus) {
        self.shift_op(bus, reg, |r, _| (r << 1, (r & 0b10000000 != 0)));
    }

    fn sra(&mut self, reg: u8, bus: &mut Bus) {
        self.shift_op(bus, reg, |r, _| {
            (r >> 1 | r & 0b10000000, (r & 0b00000001 != 0))
        });
    }

    fn srl(&mut self, reg: u8, bus: &mut Bus) {
        self.shift_op(bus, reg, |r, _| (r >> 1, (r & 0b00000001 != 0)));
    }

    fn rl(&mut self, reg: u8, bus: &mut Bus) {
        self.shift_op(bus, reg, |r, c| ((r << 1 | c as u8), (r & 0b10000000 != 0)));
    }

    fn rr(&mut self, reg: u8, bus: &mut Bus) {
        self.shift_op(bus, reg, |r, c| {
            (r >> 1 | (c as u8) << 7, (r & 0b00000001 != 0))
        });
    }

    fn rlc(&mut self, reg: u8, bus: &mut Bus) {
        self.shift_op(bus, reg, |r, _| {
            (CPU::rotate_left(r), (r & 0b10000000 != 0))
        });
    }

    fn rrc(&mut self, reg: u8, bus: &mut Bus) {
        self.shift_op(bus, reg, |r, _| {
            (CPU::rotate_right(r), (r & 0b00000001 != 0))
        });
    }

    fn shift_op(&mut self, bus: &mut Bus, reg: u8, op: fn(u8, bool) -> (u8, bool)) {
        let curr_reg = self.get_register(reg, bus);
        let (new_reg, new_c) = op(curr_reg, self.get_c());
        self.set_c(new_c);
        self.set_n(false);
        self.set_h(false);
        self.set_z(new_reg == 0);
        self.set_register(reg, new_reg, bus);
    }

    fn swap(&mut self, reg: u8, bus: &mut Bus) {
        let curr_reg = self.get_register(reg, bus);
        let new_reg = curr_reg << 4 | curr_reg >> 4;
        self.set_c(false);
        self.set_n(false);
        self.set_h(false);
        self.set_z(new_reg == 0);
        self.set_register(reg, new_reg, bus);
    }

    fn get_register(&self, reg: u8, bus: &Bus) -> u8 {
        match reg {
            0 => self.b,
            1 => self.c,
            2 => self.d,
            3 => self.e,
            4 => self.h,
            5 => self.l,
            6 => bus.read(self.get_hl()),
            7 => self.a,
            _ => panic!("Invalid register index: {}", reg),
        }
    }

    fn bit(&mut self, bit_index: u8, reg: u8, bus: &Bus) {
        let curr_reg = self.get_register(reg, bus);
        let bit = (curr_reg >> bit_index) & 0b00000001;
        self.set_n(false);
        self.set_h(true);
        self.set_z(bit == 0);
    }

    fn reset_bit(&mut self, bit_index: u8, reg: u8, bus: &mut Bus) {
        let curr_reg = self.get_register(reg, bus);
        let bit_mask = 1 << bit_index;
        let new_reg = curr_reg & !bit_mask;
        self.set_register(reg, new_reg, bus);
    }

    fn set_bit(&mut self, bit_index: u8, reg: u8, bus: &mut Bus) {
        let curr_reg = self.get_register(reg, bus);
        let bit_mask = 1 << bit_index;
        let new_reg = curr_reg | bit_mask;
        self.set_register(reg, new_reg, bus);
    }

    fn set_register(&mut self, reg: u8, value: u8, bus: &mut Bus) {
        match reg {
            0 => self.b = value,
            1 => self.c = value,
            2 => self.d = value,
            3 => self.e = value,
            4 => self.h = value,
            5 => self.l = value,
            6 => bus.write(self.get_hl(), value),
            7 => self.a = value,
            _ => panic!("Invalid register index: {}", reg),
        }
    }
    fn execute_cb(&mut self, bus: &mut Bus, current_pc: u16) -> Option<u8> {
        let opcode = bus.read(current_pc);
        let next_pc = current_pc.wrapping_add(1);

        let category: u8 = opcode >> 6; // 1100 0000 -> 0000 0011
        let subcategory: u8 = opcode >> 3 & 0b00000111; // 1111 1000 -> 0001 1111 -> 00000111
        let operand: u8 = opcode & 0b00000111; // 0000 0111 

        // println!("category: {:02X}", category);
        // println!("subcategory: {:02X}", subcategory);
        // println!("operand {:02X}", operand);

        match category {
            0 => match subcategory {
                0 => self.rlc(operand, bus),
                1 => self.rrc(operand, bus),
                2 => self.rl(operand, bus),
                3 => self.rr(operand, bus),
                4 => self.sla(operand, bus),
                5 => self.sra(operand, bus),
                6 => self.swap(operand, bus),
                7 => self.srl(operand, bus),
                _ => {}
            },
            1 => self.bit(subcategory, operand, bus),
            2 => self.reset_bit(subcategory, operand, bus),
            3 => self.set_bit(subcategory, operand, bus),

            _ => {
                println!(
                    "Unimplemented opcode: 0xCB{:02X} at PC: 0x{:04X}",
                    opcode, self.pc
                );
                return None;
            }
        }

        // println!(" Going to addr: {:#x}", next_pc);
        self.pc = next_pc;
        return Some(CB_CYCLES[opcode as usize]);
    }
}
