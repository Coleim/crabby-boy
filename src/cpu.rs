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
    // F = Z,N,H,C
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
        }
    }

    fn read16bytes(&mut self, mem: &[u8], pc: u16) -> u16 {
        let low = mem[(pc) as usize] as u16;
        let high = mem[(pc + 1) as usize] as u16;
        (high << 8) | low
    }

    pub fn execute(&mut self, mem: &mut [u8]) -> bool {
        let opcode = mem[self.pc as usize];
        println!("Parsing OP CODE: {:#X}", opcode);
        let mut next_pc: u16 = self.pc + 1;
        match opcode {
            0x00 => {
                println!("NOOP")
            }
            0xC3 => {
                println!("JP nn");
                println!(
                    "nn: {:#x} {:#x}",
                    mem[next_pc as usize],
                    mem[(next_pc + 1) as usize]
                );
                let addr = self.read16bytes(mem, next_pc);
                next_pc = addr;
            }
            0x31 => {
                println!("LD SP, n16 3  12");
                self.sp = self.read16bytes(mem, next_pc);
                next_pc += 2;
            }
            0xEA => {
                println!("LD [a16], A 3  16");
                let addr = self.read16bytes(mem, next_pc);
                mem[addr as usize] = self.a; //TODO: Replace with bus.write(addr, self.a); later
                //TODO handle memory bus
            }
            0xF3 => {
                println!("DI"); // IME = Interrupt Master Enable = 0 
                //TODO
            }
            0x3E => {
                println!("LD A, n8 2  8");
                let val: u8 = mem[next_pc as usize];
                self.a = val;
                next_pc += 1;
            }
            0xD6 => {
                println!("SUB A, n8 2  8 Z 1 H C");
                let val: u8 = mem[next_pc as usize];
                let a = self.a;
                self.a = a.wrapping_sub(val);
                self.f = 0; // 0000 0000
                if self.a == 0 {
                    self.f |= 0x80;
                } // 0000 0000 | 1000 0000
                self.f |= 0x40;

                // H = did borrow happen from bit 4?
                // H flag (half borrow)
                if (a & 0xF) < (val & 0xF) {
                    self.f |= 0x20;
                }

                // C = did borrow happen from bit 8?
                // C flag (full borrow)
                if a < val {
                    self.f |= 0x10;
                }
                next_pc += 1;
            }
            _ => {
                println!("Something else");
                return false;
            }
        }
        println!(" Going to addr: {:#x}", next_pc);
        self.pc = next_pc;
        return true;
    }
}
