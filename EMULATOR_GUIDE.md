# Game Boy Emulator Development Guide

## 📊 Current Progress

**✅ Completed:**
- ROM loading from files (Tetris.gb, cpu_instrs.gb)
- Cartridge header parsing (reading metadata from bytes 0x0100-0x014F)
- Header validation (Nintendo logo check)
- Basic memory structure (65,536 bytes)
- Mapping tables for cartridge types, licensees, and sizes

**Current Stage:** ROM parsing phase - successfully reading Game Boy cartridge data!

---

## 🎯 Game Boy Emulator Development Roadmap

### **Phase 1: CPU Core** ⚡ (Your Next Steps!)

The CPU is the heart of the emulator. The Game Boy uses a modified Intel 8080/Zilog Z80 called the **Sharp LR35902**.

#### Step 1.1: CPU Registers

Create a CPU struct with these registers:

**8-bit registers:**
- `A` - Accumulator (main math register)
- `B, C, D, E, H, L` - General purpose registers
- `F` - Flags register (stores operation results)

**16-bit registers:**
- `PC` - Program Counter (address of next instruction)
- `SP` - Stack Pointer (address of top of stack)

**Combined 16-bit pairs:**
- `AF, BC, DE, HL` - Can be used as 8-bit or 16-bit

**Flag bits in F register (bit positions):**
- `Z` (bit 7) - Zero flag: Set when result is 0
- `N` (bit 6) - Subtract flag: Set when last operation was subtraction
- `H` (bit 5) - Half-Carry flag: Set when carry from bit 3 to 4
- `C` (bit 4) - Carry flag: Set when carry/borrow occurs

```rust
// Example structure (don't copy - learn!)
struct CPU {
    a: u8,
    b: u8,
    c: u8,
    // ... other registers
    pc: u16,
    sp: u16,
}
```

#### Step 1.2: Instruction Decoding

The CPU has **512 total opcodes:**
- 256 main opcodes (0x00-0xFF)
- 256 prefixed opcodes (0xCB prefix)

Each opcode is **1-3 bytes long:**
- 1 byte: `NOP` (0x00)
- 2 bytes: `LD A, n` (0x3E, value)
- 3 bytes: `JP nn` (0xC3, low byte, high byte)

**Start with these simple instructions:**
- `0x00` - NOP (do nothing)
- `0x3E` - LD A, n (load immediate into A)
- `0x06` - LD B, n (load immediate into B)
- `0x04` - INC B (increment B)
- `0x05` - DEC B (decrement B)
- `0x80` - ADD A, B (add B to A)
- `0x90` - SUB B (subtract B from A)

**Implementation approach:**
```rust
// Pseudocode - understand the pattern
fn execute(&mut self, opcode: u8) {
    match opcode {
        0x00 => { /* NOP */ },
        0x3E => { 
            let value = self.read_next_byte();
            self.a = value;
        },
        // ... more opcodes
    }
}
```

#### Step 1.3: Fetch-Decode-Execute Loop

This is the CPU's main cycle:

1. **Fetch:** Read the byte at PC (program counter)
2. **Decode:** Determine which instruction it is
3. **Execute:** Perform the operation
4. **Update:** Increment PC, track cycles taken

```rust
// Conceptual loop
loop {
    let opcode = memory.read_byte(cpu.pc);  // Fetch
    let instruction = decode(opcode);        // Decode
    cpu.execute(instruction);                // Execute
    cpu.pc += instruction.length;            // Update PC
    cycles += instruction.cycles;            // Track timing
}
```

**Learning Resources:**
- [Pan Docs - CPU Registers](https://gbdev.io/pandocs/CPU_Registers_and_Flags.html)
- [Game Boy CPU Manual](https://gbdev.io/gb-opcodes/)
- [Instruction Set Table](https://www.pastraiser.com/cpu/gameboy/gameboy_opcodes.html)

---

### **Phase 2: Memory Management** 💾 (Expand your memory.rs)

#### Step 2.1: Memory Map Implementation

The Game Boy has a **64KB address space** (0x0000-0xFFFF) divided into regions:

| Address Range | Size | Description |
|---------------|------|-------------|
| 0x0000-0x3FFF | 16KB | ROM Bank 0 (fixed, from cartridge) |
| 0x4000-0x7FFF | 16KB | ROM Bank 1-N (switchable via MBC) |
| 0x8000-0x9FFF | 8KB  | Video RAM (VRAM) - tile data |
| 0xA000-0xBFFF | 8KB  | External RAM (cartridge RAM, if present) |
| 0xC000-0xDFFF | 8KB  | Work RAM (WRAM) |
| 0xE000-0xFDFF | ~8KB | Echo RAM (mirror of 0xC000-0xDDFF) |
| 0xFE00-0xFE9F | 160B | OAM (Object Attribute Memory - sprites) |
| 0xFEA0-0xFEFF | 96B  | Unusable (reads as 0x00) |
| 0xFF00-0xFF7F | 128B | I/O Registers (hardware control) |
| 0xFF80-0xFFFE | 127B | High RAM (HRAM) - fast access |
| 0xFFFF        | 1B   | Interrupt Enable Register |

#### Step 2.2: Read/Write Functions

Implement proper memory access:

```rust
// Conceptual approach
impl Memory {
    pub fn read_byte(&self, address: u16) -> u8 {
        match address {
            0x0000..=0x7FFF => /* ROM */,
            0x8000..=0x9FFF => /* VRAM */,
            0xA000..=0xBFFF => /* External RAM */,
            0xC000..=0xDFFF => /* Work RAM */,
            0xE000..=0xFDFF => /* Echo RAM - mirror 0xC000 */,
            0xFE00..=0xFE9F => /* OAM */,
            0xFF00..=0xFF7F => /* I/O Registers */,
            0xFF80..=0xFFFE => /* HRAM */,
            0xFFFF          => /* Interrupt Enable */,
            _ => 0x00
        }
    }
    
    pub fn write_byte(&mut self, address: u16, value: u8) {
        // Similar structure, but some areas are read-only!
    }
}
```

**Important Behaviors:**
- **ROM (0x0000-0x7FFF):** Read-only, writes go to MBC control
- **Echo RAM (0xE000-0xFDFF):** Mirrors 0xC000-0xDDFF
- **I/O Registers:** Reading/writing triggers hardware behavior
- **Unusable area (0xFEA0-0xFEFF):** Returns 0x00

#### Step 2.3: Important I/O Registers

| Address | Name | Purpose |
|---------|------|---------|
| 0xFF00  | P1/JOYP | Joypad input |
| 0xFF01  | SB | Serial transfer data |
| 0xFF02  | SC | Serial transfer control |
| 0xFF04  | DIV | Divider register |
| 0xFF05  | TIMA | Timer counter |
| 0xFF06  | TMA | Timer modulo |
| 0xFF07  | TAC | Timer control |
| 0xFF0F  | IF | Interrupt flags |
| 0xFF40  | LCDC | LCD control |
| 0xFF41  | STAT | LCD status |
| 0xFF42  | SCY | Scroll Y |
| 0xFF43  | SCX | Scroll X |
| 0xFF44  | LY | LCD Y coordinate |
| 0xFF47  | BGP | Background palette |

**Key Concept:** Memory isn't just storage - reading/writing certain addresses controls the entire Game Boy!

---

### **Phase 3: Basic Testing** 🧪

Before implementing graphics, test your CPU with test ROMs.

#### Step 3.1: Test ROMs

You already have **cpu_instrs.gb** - this is Blargg's CPU instruction test suite!

**Blargg's CPU Tests:**
- `01-special.gb` - Special instructions
- `02-interrupts.gb` - Interrupt handling
- `03-op sp,hl.gb` - Stack pointer operations
- `04-op r,imm.gb` - Register/immediate operations
- `05-op rp.gb` - Register pair operations
- `06-ld r,r.gb` - Load register to register
- `07-jr,jp,call,ret,rst.gb` - Jumps and calls
- `08-misc instrs.gb` - Miscellaneous
- `09-op r,r.gb` - Register to register operations
- `10-bit ops.gb` - Bit operations
- `11-op a,(hl).gb` - Accumulator operations

**How Tests Work:**
1. Execute test ROM
2. Test writes results to **serial port** (0xFF01-0xFF02)
3. Read serial output to see pass/fail

**Serial Port Output:**
```rust
// When 0xFF02 is written with 0x81, read 0xFF01
if address == 0xFF02 && value == 0x81 {
    let character = self.read_byte(0xFF01);
    print!("{}", character as char);  // Output test result
}
```

#### Step 3.2: Logging & Debugging

Essential for development:

**Log each instruction:**
```
PC: 0x0100 | Opcode: 0x3E | Instruction: LD A, 0x42 | A: 0x00 -> 0x42
PC: 0x0102 | Opcode: 0x06 | Instruction: LD B, 0x13 | B: 0x00 -> 0x13
```

**Compare with known emulators:**
- Run the same ROM in a reference emulator
- Log register states after each instruction
- Find where your emulator diverges

**Debugging Tools:**
- Step-by-step execution
- Breakpoints at specific addresses
- Memory viewer
- Register inspector

---

### **Phase 4: Graphics (PPU - Pixel Processing Unit)** 🎨

**THIS IS WHERE YOU SEE VISUALS!**

#### Step 4.1: Understanding the PPU

**Display specs:**
- **Resolution:** 160×144 pixels
- **Refresh rate:** ~59.7 Hz (about 60 FPS)
- **Colors:** 4 shades of gray (or green on original)

**PPU renders in scanlines** (one horizontal line at a time):
- 144 visible scanlines (0-143)
- 10 V-Blank scanlines (144-153)
- Each scanline takes 456 CPU cycles

**PPU Modes (changes per scanline):**
1. **Mode 2 - OAM Search:** 80 cycles - scanning for sprites on this line
2. **Mode 3 - Pixel Transfer:** 172+ cycles - drawing pixels
3. **Mode 0 - H-Blank:** Remaining cycles - horizontal blank
4. **Mode 1 - V-Blank:** 4560 cycles - vertical blank (after line 143)

**Current scanline:** Read from LY register (0xFF44)

#### Step 4.2: Tile System

Game Boy graphics use **8×8 pixel tiles**.

**Tile Data:**
- Stored in VRAM (0x8000-0x97FF)
- Each tile is **16 bytes**
- Each pixel is **2 bits** (4 possible colors)

**Tile encoding:**
```
Tile bytes:  [byte0] [byte1] [byte2] [byte3] ... (16 bytes total)
             └─ Row 0 ─┘ └─ Row 1 ─┘

Each row (8 pixels) = 2 bytes:
Byte 0: Low bits  (76543210)
Byte 1: High bits (76543210)

Pixel color = (bit from byte1 << 1) | bit from byte0
Example:
  Byte 0: 0b11100000
  Byte 1: 0b10100000
  
  Pixel 0: (1 << 1) | 1 = 3 (darkest)
  Pixel 1: (0 << 1) | 1 = 1 (light)
  Pixel 2: (1 << 1) | 1 = 3 (darkest)
```

**Tile Maps:**
- Background map: 32×32 tiles (256×256 pixels)
- Two tile maps in VRAM:
  - 0x9800-0x9BFF (Map 0)
  - 0x9C00-0x9FFF (Map 1)
- Each byte in map = tile index to render

#### Step 4.3: Rendering Pipeline

**Step-by-step rendering process:**

1. **Read LCDC (0xFF40)** - LCD control register
   - Bit 7: LCD on/off
   - Bit 4: Background tile data area
   - Bit 3: Background tile map area
   - Bit 1: Sprites enabled
   - Bit 0: Background enabled

2. **For each scanline (0-143):**
   - Determine which tile row to render
   - For each pixel (0-159):
     - Calculate which tile to use
     - Get pixel color from tile data
     - Apply palette (0xFF47 - BGP register)
     - Draw to framebuffer

3. **Apply palettes:**
   - BGP (0xFF47): Background palette
   - 4 colors, 2 bits each: `[color3][color2][color1][color0]`
   - Example: 0b11100100 = `[11][10][01][00]`
     - 00 (lightest) → white
     - 01 → light gray
     - 10 → dark gray
     - 11 (darkest) → black

4. **Render sprites (if enabled):**
   - OAM (0xFE00-0xFE9F) contains 40 sprite entries
   - Each sprite = 4 bytes (Y, X, tile, attributes)
   - Draw sprites on top of background

5. **Display framebuffer to screen**

#### Step 4.4: Graphics Library Setup

**Recommended Rust crates:**

**Option 1: minifb** (simplest)
```toml
[dependencies]
minifb = "0.25"
```
- Easy to use
- Just need a pixel buffer
- Good for learning

**Option 2: pixels** (modern)
```toml
[dependencies]
pixels = "0.13"
winit = "0.29"
```
- More features
- Better performance
- Active development

**Option 3: SDL2** (traditional)
```toml
[dependencies]
sdl2 = "0.37"
```
- Industry standard
- More complex setup
- Lots of features

**Basic rendering concept:**
```rust
// Create a 160x144 pixel buffer
let mut screen: [u32; 160 * 144] = [0; 160 * 144];

// In your PPU, set pixels:
screen[y * 160 + x] = color;  // color as RGB

// Display to window each frame
window.update_with_buffer(&screen);
```

#### Step 4.5: First Pixels Tutorial

**Minimal PPU for first visuals:**

1. **Initialize screen buffer** (160×144 pixels)
2. **In your CPU loop, track cycles**
3. **Every 456 cycles, increment scanline (LY)**
4. **For each scanline, render background:**
   ```
   - Read SCY, SCX (scroll position)
   - For each pixel x (0-159):
     - Calculate tile coordinates
     - Get tile from tile map
     - Get pixel from tile data
     - Apply palette
     - Set pixel in framebuffer
   ```
5. **After 144 scanlines, trigger V-Blank interrupt**
6. **Render 10 more blank scanlines**
7. **Display framebuffer to window**
8. **Reset to scanline 0**

**You'll see:** Static background tiles! Possibly garbage at first, then recognizable patterns as you fix bugs.

---

### **Phase 5: Timing** ⏱️

Game Boy runs at **4.194304 MHz** (4,194,304 cycles per second).

#### Basic Timing:
- **CPU clock:** 4.194304 MHz
- **One frame:** 70,224 cycles (456 × 154 scanlines)
- **Frame rate:** ~59.7 Hz
- **Cycles per second:** 4,194,304
- **Frames per second:** 4,194,304 ÷ 70,224 ≈ 59.7

#### Implementation:
```rust
// Track cycles
let mut total_cycles = 0;
const CYCLES_PER_FRAME: u32 = 70224;

loop {
    let cycles = cpu.execute_instruction();
    total_cycles += cycles;
    
    ppu.step(cycles);  // Update PPU
    
    if total_cycles >= CYCLES_PER_FRAME {
        render_frame();
        total_cycles -= CYCLES_PER_FRAME;
        
        // Sleep to maintain 60 FPS
        sleep(16.7ms);
    }
}
```

#### Instruction Timing:
Each instruction takes a specific number of cycles:
- `NOP`: 4 cycles
- `LD A, n`: 8 cycles
- `JP nn`: 16 cycles
- `CALL nn`: 24 cycles

**Include timing in your instruction implementation!**

---

### **Phase 6: Input** 🎮

#### Step 6.1: Joypad Register (0xFF00)

The Game Boy has **8 buttons:**
- **D-Pad:** Up, Down, Left, Right
- **Buttons:** A, B, Start, Select

**Joypad register (P1/JOYP at 0xFF00):**
```
Bit 7: Not used
Bit 6: Not used
Bit 5: P15 - Select button keys (0=select)
Bit 4: P14 - Select direction keys (0=select)
Bit 3: P13 - Input Down  or Start    (0=pressed)
Bit 2: P12 - Input Up    or Select   (0=pressed)
Bit 1: P11 - Input Left  or B        (0=pressed)
Bit 0: P10 - Input Right or A        (0=pressed)
```

**How it works:**
1. Game writes to 0xFF00 to select button group (bit 4 or 5 = 0)
2. Game reads from 0xFF00 to get button states
3. 0 = pressed, 1 = not pressed

#### Step 6.2: Input Handling

```rust
// When game reads 0xFF00
match self.joypad_register {
    // Direction keys selected (bit 4 = 0)
    0x10 => {
        let mut result = 0xCF;  // Base value
        if right_pressed { result &= !(1 << 0); }
        if left_pressed  { result &= !(1 << 1); }
        if up_pressed    { result &= !(1 << 2); }
        if down_pressed  { result &= !(1 << 3); }
        return result;
    },
    // Button keys selected (bit 5 = 0)
    0x20 => {
        let mut result = 0xDF;  // Base value
        if a_pressed      { result &= !(1 << 0); }
        if b_pressed      { result &= !(1 << 1); }
        if select_pressed { result &= !(1 << 2); }
        if start_pressed  { result &= !(1 << 3); }
        return result;
    },
}
```

**Map keyboard to Game Boy:**
- Arrow keys → D-Pad
- Z/X → A/B
- Enter → Start
- Shift → Select

**Joypad Interrupt:**
When a button is pressed, can trigger interrupt at 0x60.

---

### **Phase 7: Interrupts** ⚡

The Game Boy has **5 interrupt types:**

| Interrupt | Bit | Address | Trigger |
|-----------|-----|---------|---------|
| V-Blank   | 0   | 0x0040  | After frame drawn (line 144) |
| LCD STAT  | 1   | 0x0048  | Various LCD conditions |
| Timer     | 2   | 0x0050  | Timer overflow |
| Serial    | 3   | 0x0058  | Serial transfer complete |
| Joypad    | 4   | 0x0060  | Button pressed |

#### Interrupt Registers:
- **IE (0xFFFF):** Interrupt Enable - which interrupts are enabled
- **IF (0xFF0F):** Interrupt Flag - which interrupts are pending
- **IME:** Interrupt Master Enable - global interrupt switch (not a register, CPU flag)

#### How Interrupts Work:

1. **Interrupt condition occurs** (e.g., V-Blank starts)
2. **Set corresponding bit in IF (0xFF0F)**
3. **If interrupt is enabled in IE (0xFFFF) AND IME is set:**
   - Push PC onto stack
   - Set PC to interrupt vector address
   - Clear IME
   - Execute interrupt handler code
4. **Handler ends with RETI instruction:**
   - Pop PC from stack
   - Set IME
   - Continue normal execution

**Most important:** V-Blank interrupt! Games use this to update graphics between frames.

**Implementation:**
```rust
fn check_interrupts(&mut self) {
    if !self.ime { return; }  // Master enable off
    
    let ie = self.memory.read_byte(0xFFFF);
    let if_reg = self.memory.read_byte(0xFF0F);
    let triggered = ie & if_reg;
    
    if triggered == 0 { return; }
    
    // Check each interrupt (priority order)
    for i in 0..5 {
        if (triggered & (1 << i)) != 0 {
            self.handle_interrupt(i);
            break;  // Only one per check
        }
    }
}

fn handle_interrupt(&mut self, interrupt: u8) {
    self.ime = false;  // Disable interrupts
    
    // Clear flag
    let if_reg = self.memory.read_byte(0xFF0F);
    self.memory.write_byte(0xFF0F, if_reg & !(1 << interrupt));
    
    // Push PC to stack
    self.push_stack(self.pc);
    
    // Jump to handler
    self.pc = match interrupt {
        0 => 0x0040,  // V-Blank
        1 => 0x0048,  // LCD STAT
        2 => 0x0050,  // Timer
        3 => 0x0058,  // Serial
        4 => 0x0060,  // Joypad
        _ => panic!("Invalid interrupt"),
    };
}
```

**Instructions related to interrupts:**
- `EI` (0xFB): Enable interrupts (set IME)
- `DI` (0xF3): Disable interrupts (clear IME)
- `RETI` (0xD9): Return from interrupt
- `HALT` (0x76): Wait for interrupt

---

### **Phase 8: Timers** ⏲️

The Game Boy has **4 timer registers:**

| Register | Address | Purpose |
|----------|---------|---------|
| DIV      | 0xFF04  | Divider Register (increments at 16384 Hz) |
| TIMA     | 0xFF05  | Timer Counter (configurable speed) |
| TMA      | 0xFF06  | Timer Modulo (reload value for TIMA) |
| TAC      | 0xFF07  | Timer Control (enable/speed) |

#### DIV Register (0xFF04):
- Increments at 16384 Hz (every 256 CPU cycles)
- **Any write resets it to 0**
- Commonly used for random number generation

#### TIMA Register (0xFF05):
- Increments at speed set by TAC
- When it overflows (0xFF → 0x00):
  - Reloads with value from TMA
  - Triggers Timer interrupt

#### TAC Register (0xFF07):
```
Bit 2: Timer enable (1=on, 0=off)
Bit 1-0: Speed
  00: 4096 Hz   (1024 CPU cycles)
  01: 262144 Hz (16 CPU cycles)
  10: 65536 Hz  (64 CPU cycles)
  11: 16384 Hz  (256 CPU cycles)
```

#### Implementation:
```rust
struct Timer {
    div_counter: u16,   // Internal counter for DIV
    tima_counter: u16,  // Internal counter for TIMA
}

fn update_timers(&mut self, cycles: u8) {
    // Update DIV (increments every 256 cycles)
    self.div_counter += cycles as u16;
    if self.div_counter >= 256 {
        self.div_counter -= 256;
        let div = self.memory.read_byte(0xFF04);
        self.memory.write_byte(0xFF04, div.wrapping_add(1));
    }
    
    // Update TIMA (if enabled)
    let tac = self.memory.read_byte(0xFF07);
    if (tac & 0x04) != 0 {  // Timer enabled
        let frequency = match tac & 0x03 {
            0 => 1024,
            1 => 16,
            2 => 64,
            3 => 256,
            _ => unreachable!(),
        };
        
        self.tima_counter += cycles as u16;
        while self.tima_counter >= frequency {
            self.tima_counter -= frequency;
            
            let tima = self.memory.read_byte(0xFF05);
            if tima == 0xFF {
                // Overflow - reload and interrupt
                let tma = self.memory.read_byte(0xFF06);
                self.memory.write_byte(0xFF05, tma);
                
                // Request timer interrupt
                let if_reg = self.memory.read_byte(0xFF0F);
                self.memory.write_byte(0xFF0F, if_reg | 0x04);
            } else {
                self.memory.write_byte(0xFF05, tima + 1);
            }
        }
    }
}
```

---

### **Phase 9: Memory Bank Controllers (MBC)** 🗄️

Larger games need **bank switching** because they exceed 32KB ROM limit.

#### MBC Types (you parsed this in header!):

**MBC1** (Most common):
- Up to 2MB ROM (125 banks of 16KB)
- Up to 32KB RAM (4 banks of 8KB)
- Two banking modes

**MBC3** (with RTC):
- Up to 2MB ROM
- Up to 32KB RAM
- Real-Time Clock (RTC) support

**MBC5** (larger games):
- Up to 8MB ROM
- Up to 128KB RAM

#### How Banking Works:

**ROM Banking:**
- 0x0000-0x3FFF: Bank 0 (always accessible)
- 0x4000-0x7FFF: Banks 1-N (switchable)

**Writing to ROM area changes banks:**
```rust
fn write_byte(&mut self, address: u16, value: u8) {
    match address {
        // MBC1 example
        0x0000..=0x1FFF => {
            // RAM enable/disable
            self.ram_enabled = (value & 0x0F) == 0x0A;
        },
        0x2000..=0x3FFF => {
            // ROM bank number (lower 5 bits)
            self.rom_bank = (value & 0x1F) as usize;
            if self.rom_bank == 0 {
                self.rom_bank = 1;  // Can't select bank 0
            }
        },
        0x4000..=0x5FFF => {
            // RAM bank or upper ROM bank bits
            // ...
        },
        // ... more control registers
    }
}
```

**Reading from ROM:**
```rust
fn read_byte(&self, address: u16) -> u8 {
    match address {
        0x0000..=0x3FFF => {
            // Bank 0 - always first 16KB
            self.rom_data[address as usize]
        },
        0x4000..=0x7FFF => {
            // Banked ROM
            let offset = (self.rom_bank * 0x4000) + ((address - 0x4000) as usize);
            self.rom_data[offset]
        },
        // ...
    }
}
```

**Why important:** Without MBC, you can only play simple games. MBC unlocks Pokemon, Zelda, etc!

---

### **Phase 10: Sound (APU - Audio Processing Unit)** 🔊

**Note:** Sound is complex and can be added later. It's not required for visual gameplay!

#### Sound Channels:

**Channel 1 (0xFF10-0xFF14):** Square wave with sweep
- Adjustable frequency, volume envelope, sweep

**Channel 2 (0xFF16-0xFF19):** Square wave
- Similar to channel 1, no sweep

**Channel 3 (0xFF1A-0xFF1E):** Custom wave
- 32 4-bit samples in wave RAM (0xFF30-0xFF3F)

**Channel 4 (0xFF20-0xFF23):** Noise
- For percussion and sound effects

**Master Control:**
- 0xFF24: Master volume
- 0xFF25: Sound panning (left/right)
- 0xFF26: Sound on/off

#### Basic Implementation:
1. Track each channel's frequency, volume, duty cycle
2. Generate samples at 44100 Hz
3. Mix channels together
4. Output to audio device (use `cpal` or `rodio` crate)

**Skip this initially!** Get video and input working first.

---

### **Phase 11: Additional Features** ✨

#### Save States:
Serialize entire emulator state to file:
```rust
struct SaveState {
    cpu_registers: CPUState,
    memory: [u8; 0x10000],
    // ... all emulator state
}
```

#### Battery Saves:
Games with RAM need to persist data:
- When game writes to cartridge RAM (0xA000-0xBFFF)
- Save RAM to file on exit
- Load RAM file on startup

#### Debugger Features:
- **Breakpoints:** Pause at specific PC
- **Step execution:** Execute one instruction
- **Memory viewer:** Inspect memory regions
- **Disassembler:** Show instruction at address
- **Register viewer:** Track all registers

#### Game Boy Color Support:
Extended features (after original GB works):
- Double CPU speed mode
- More VRAM banks
- Color palettes
- More sprites per line

---

## 🚀 Recommended Learning Path

### **Week 1-2: CPU Foundation** ⚙️

**Goals:**
- [ ] Create CPU struct with all registers
- [ ] Implement register read/write functions
- [ ] Implement flag manipulation (Z, N, H, C)
- [ ] Write unit tests for flag operations

**Instructions to implement (start with 20-30):**
- `0x00` NOP
- `0x3E` LD A, n
- `0x06, 0x0E, 0x16, 0x1E, 0x26, 0x2E` LD r, n (all registers)
- `0x04, 0x0C, 0x14, 0x1C, 0x24, 0x2C, 0x3C` INC r
- `0x05, 0x0D, 0x15, 0x1D, 0x25, 0x2D, 0x3D` DEC r
- `0x80-0x87` ADD A, r
- `0x90-0x97` SUB r
- `0xC3` JP nn
- `0x18` JR n

**Testing:**
- Write unit tests for each instruction
- Test flag behavior carefully

---

### **Week 3: Memory & Main Loop** 💾

**Goals:**
- [ ] Expand memory.rs with proper read/write
- [ ] Implement memory regions (ROM, RAM, VRAM, etc.)
- [ ] Create fetch-decode-execute loop
- [ ] Connect CPU to memory

**Test:**
- Load a simple ROM
- Execute a few instructions
- Log PC and register values

---

### **Week 4-5: Complete Instruction Set** 📚

**Goals:**
- [ ] Implement all 256 main opcodes
- [ ] Implement 0xCB prefixed opcodes
- [ ] Include proper cycle timing
- [ ] Handle stack operations (PUSH, POP, CALL, RET)

**Instruction categories:**
- **8-bit loads:** LD r, r / LD r, n / LD r, (HL)
- **16-bit loads:** LD rr, nn / PUSH / POP
- **8-bit ALU:** ADD, ADC, SUB, SBC, AND, OR, XOR, CP
- **16-bit ALU:** ADD HL, rr / INC rr / DEC rr
- **Bit operations:** BIT, SET, RES (0xCB prefix)
- **Jumps:** JP, JR, CALL, RET, RST
- **Shifts/Rotates:** RLC, RRC, RL, RR, SLA, SRA, SRL

**Testing:**
- Run Blargg's cpu_instrs.gb
- Start with test 01
- Fix bugs until tests pass

**Expected difficulty:** This is the most tedious part! But essential.

---

### **Week 6-7: Graphics - First Pixels!** 🎨

**Goals:**
- [ ] Set up graphics library (minifb recommended)
- [ ] Create screen buffer (160×144)
- [ ] Implement basic PPU timing (scanlines)
- [ ] Render background tiles
- [ ] Apply palettes

**Milestone:** **SEE YOUR FIRST PIXELS!** 🎉

**Implementation order:**
1. Create 160×144 framebuffer
2. Every 456 cycles = 1 scanline
3. Update LY register (0xFF44)
4. For scanlines 0-143, render background:
   - Read tile map
   - Read tile data
   - Apply palette
5. Trigger V-Blank at line 144
6. Display buffer to screen

**Test:**
- Load Tetris.gb
- You should see background graphics (even if incorrect)
- Debug until recognizable

---

### **Week 8: Complete Graphics** 🖼️

**Goals:**
- [ ] Implement sprite rendering (OAM)
- [ ] Handle sprite attributes (flip, priority, palette)
- [ ] Implement sprite limits (10 per line)
- [ ] Fix graphical bugs

**Test:**
- Tetris pieces should appear
- Moving objects should render

---

### **Week 9: Input & Timing** 🎮

**Goals:**
- [ ] Implement joypad register (0xFF00)
- [ ] Map keyboard to buttons
- [ ] Trigger joypad interrupts
- [ ] Implement frame timing (60 FPS)

**Milestone:** **PLAYABLE TETRIS!** 🎉🎉🎉

---

### **Week 10: Interrupts & Timers** ⚡

**Goals:**
- [ ] Implement interrupt system
- [ ] V-Blank interrupt (most important)
- [ ] Timer registers and interrupts
- [ ] HALT instruction

**Test:**
- More games should boot
- cpu_instrs.gb test 02 should pass

---

### **Week 11-12: Memory Bank Controllers** 🗄️

**Goals:**
- [ ] Implement MBC1
- [ ] Implement MBC3
- [ ] Implement MBC5
- [ ] Save/load cartridge RAM

**Milestone:** **PLAY POKEMON!** 🎉🎉🎉

---

### **Beyond: Polish & Features** ✨

- [ ] Implement sound (APU)
- [ ] Add save states
- [ ] Create debugger UI
- [ ] Game Boy Color support
- [ ] Speed controls (fast-forward, slow-mo)
- [ ] Screenshots
- [ ] Game Genie / cheat codes

---

## 📚 Essential Resources

### Documentation:
- **[Pan Docs](https://gbdev.io/pandocs/)** - THE Game Boy technical reference (bookmark this!)
- **[GB Opcodes Table](https://gbdev.io/gb-opcodes/)** - Complete instruction set with timing
- **[GBEDG](https://hacktix.github.io/GBEDG/)** - Emulator Development Guide
- **[Game Boy CPU Manual](https://www.pastraiser.com/cpu/gameboy/gameboy_opcodes.html)** - Detailed opcode reference

### Test ROMs:
- **[Blargg's Test ROMs](https://github.com/retrio/gb-test-roms)** - CPU instruction tests
- **[Mooneye Test Suite](https://github.com/Gekkio/mooneye-test-suite)** - Hardware tests
- **[dmg-acid2](https://github.com/mattcurrie/dmg-acid2)** - PPU rendering test

### Communities:
- **[r/EmuDev](https://www.reddit.com/r/EmuDev/)** - Emulator development community
- **[GBDev Discord](https://discord.gg/gbdev)** - Game Boy development/emulation chat
- **[Emudev.de Forums](https://forums.nesdev.org/)** - Technical discussions

### Reference Emulators (to study):
- **[SameBoy](https://github.com/LIJI32/SameBoy)** - Highly accurate, good reference
- **[Gambatte](https://github.com/sinamas/gambatte)** - Accurate and well-documented
- **[mooneye-gb](https://github.com/Gekkio/mooneye-gb)** - Rust implementation!

### Tutorials:
- **[Codeslinger's GB Emulator Tutorial](http://www.codeslinger.co.uk/pages/projects/gameboy.html)** - Step-by-step guide
- **[gbdev.io Resources](https://gbdev.io/resources.html)** - Curated learning materials

---

## 💡 Development Tips

### General Advice:
1. **Start small, build up** - Don't try to implement everything at once
2. **Test constantly** - Use test ROMs after implementing each feature
3. **Log everything** - Debug output is your best friend
4. **Compare with references** - Check your emulator against known-good ones
5. **Be patient with CPU** - Instruction implementation is tedious but necessary
6. **Celebrate milestones** - First instruction? First pixel? First game? Party time!
7. **Don't optimize early** - Get it working first, fast second

### Debugging Strategies:
- **Instruction logging:** Log every opcode executed with registers
- **Memory dumps:** Save memory state at specific points
- **Breakpoints:** Pause at specific PC values
- **Register comparison:** Compare with reference emulator
- **Step-by-step execution:** Execute one instruction at a time

### Common Pitfalls:
- **Flag calculation errors** - Half-carry and carry flags are tricky
- **Endianness** - Game Boy is little-endian (LSB first)
- **Memory mirroring** - Echo RAM must mirror work RAM
- **Timing issues** - Instructions take different cycle counts
- **Off-by-one errors** - Be careful with address ranges

### Performance Tips (for later):
- Use lookup tables for instruction decoding
- Cache tile data conversions
- Optimize hot paths (PPU rendering)
- Profile before optimizing

### Recommended Rust Patterns:
```rust
// Use match for opcode decoding
match opcode {
    0x00 => self.nop(),
    0x3E => self.ld_a_n(),
    // ...
}

// Separate concerns
struct GameBoy {
    cpu: CPU,
    memory: Memory,
    ppu: PPU,
    timer: Timer,
}

// Use traits for memory access
trait MemoryBus {
    fn read_byte(&self, addr: u16) -> u8;
    fn write_byte(&mut self, addr: u16, value: u8);
}
```

---

## 🎯 Success Metrics

### Milestone Checklist:

**🥉 Bronze Level:**
- [ ] CPU executes basic instructions
- [ ] Blargg test 01 passes
- [ ] Can run simple ROMs

**🥈 Silver Level:**
- [ ] All CPU instructions implemented
- [ ] Background rendering works
- [ ] Can see graphics (even if buggy)

**🥇 Gold Level:**
- [ ] Sprites rendering
- [ ] Input working
- [ ] Tetris is playable
- [ ] V-Blank interrupts working

**💎 Platinum Level:**
- [ ] All Blargg tests pass
- [ ] MBC support
- [ ] Pokemon is playable
- [ ] Audio implemented

**🏆 Master Level:**
- [ ] Passes Mooneye tests
- [ ] Game Boy Color support
- [ ] Save states
- [ ] Debugger UI

---

## 🎬 Final Thoughts

Building a Game Boy emulator is a **fantastic learning experience**! You'll gain deep understanding of:
- Computer architecture
- CPU design and instruction sets
- Memory management
- Graphics rendering
- Timing and synchronization
- Low-level programming

**The journey ahead:**
- **Weeks 1-5:** No visuals, but building the foundation
- **Week 6:** FIRST PIXELS! This is when it gets exciting
- **Week 9:** FIRST PLAYABLE GAME! The payoff for all your work
- **Week 12+:** Playing Pokemon, showing off to friends!

**Remember:** Every emulator developer struggled with the same bugs you'll face. The community is helpful, resources are abundant, and the satisfaction of playing a game on YOUR emulator is incredible.

**You've already started strong** with ROM parsing and header validation. The CPU is next, and while it's tedious, it's the key to everything else.

**Good luck, and happy emulating!** 🦀🎮✨

---

## Quick Reference: Next Immediate Steps

1. **Create `src/cpu.rs`:**
   - Define CPU struct with registers
   - Implement register getters/setters
   - Implement flag manipulation

2. **Implement first instruction:**
   - Start with `0x00` NOP (easiest)
   - Then `0x3E` LD A, n
   - Test thoroughly

3. **Create main loop:**
   - Fetch opcode from memory
   - Match and execute
   - Log output

4. **Test:**
   - Load cpu_instrs.gb
   - Watch first few instructions execute
   - Fix bugs

**When stuck:** Check Pan Docs, ask on r/EmuDev, compare with reference emulators.

**When successful:** Celebrate! Then move to next instruction.

You've got this! 🚀
