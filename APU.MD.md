# APU — Game Boy Audio Implementation Guide

**Objectif :** Entendre le beep de fin des tests Blargg (Channel 1, square wave).

**Référence :** https://gbdev.io/pandocs/Audio.html

---

## Architecture

```
┌──────────────────────────────────────────────────────────────┐
│                         Émulateur                            │
│                                                              │
│   CPU tick → IOBridge tick → APU tick                        │
│                                      │                       │
│                                      ▼                       │
│                              ┌──────────────┐                │
│                              │  Channel 1   │                │
│                              │ (square wave)│                │
│                              └──────┬───────┘                │
│                                     │ sample (0-15)          │
│                                     ▼                        │
│                              ┌──────────────┐                │
│                              │    Mixer     │                │
│                              │  NR50/NR51   │                │
│                              └──────┬───────┘                │
│                                     │ f32 (-1.0 à 1.0)       │
│                                     ▼                        │
│                              ┌──────────────┐                │
│                              │  Ring Buffer │◄── Arc<Mutex>  │
│                              └──────┬───────┘                │
└─────────────────────────────────────┼────────────────────────┘
                                      │ pop()
                                      ▼
                              ┌──────────────┐
                              │  cpal thread │
                              │  (44100 Hz)  │
                              │  → speakers  │
                              └──────────────┘
```

---

## Thread audio & partage du buffer

### Le thread audio n'est pas un thread que tu spawn toi-même

C'est **cpal** qui crée le thread en interne quand tu appelles `build_output_stream()`. Tu donnes un callback (closure), cpal l'appelle en boucle depuis son propre thread pour remplir le buffer de la carte son.

### Comment partager le buffer

```
emulator.rs / run()
   │
   ├── audio_buffer = Arc::new(Mutex::new(AudioBuffer::new(8192)))
   │        │                                    │
   │        ▼ (.clone())                         ▼ (original ou .clone())
   │   AudioOutput::new(buffer)           bus.set_audio_buffer(buffer)
   │        │                                    │
   │        ▼                                    ▼
   │   cpal callback (pop)               IOBridge → APU (push)
```

- `Arc` = plusieurs propriétaires (émulateur + thread audio)
- `Mutex` = un seul accède au buffer à la fois (pas de data race)
- `.clone()` sur un Arc ne copie pas les données, juste le pointeur
---

audio

display  ( PPU )

graphics ( UI )

commands ( A/B start)


## Step 3 — Channel 1 (Square Wave)

**Doc :** https://gbdev.io/pandocs/Audio_Registers.html#ff11--nr11-channel-1-length-timer--duty-cycle

### Registres

| Addr | Nom | Bits | Rôle |
|------|-----|------|------|
| FF11 | NR11 | `DDLL LLLL` | D=duty(2 bits), L=length timer(6 bits) |
| FF12 | NR12 | `VVVV DAAA` | V=volume initial, D=direction(1=up), A=pace |
| FF13 | NR13 | `PPPP PPPP` | Period low 8 bits |
| FF14 | NR14 | `TL-- -PPP` | T=trigger, L=length enable, P=period high 3 bits |

### Duty table

```rust
const DUTY_TABLE: [[u8; 8]; 4] = [
    [0, 0, 0, 0, 0, 0, 0, 1],  // 12.5%
    [1, 0, 0, 0, 0, 0, 0, 1],  // 25%
    [1, 1, 1, 1, 0, 0, 0, 0],  // 50%  ← Blargg
    [0, 1, 1, 1, 1, 1, 1, 0],  // 75%
];
```
### Pseudo-code du tick (appelé chaque 1 T-cycle, ou par groupe de 4)

```rust
fn tick_channel1(&mut self) {
    // Le freq_timer décrémente de 1 chaque T-cycle
    // Quand il atteint 0 :
    //   - Recharger : freq_timer = (2048 - period) * 4
    //   - Avancer : duty_pos = (duty_pos + 1) % 8

    self.freq_timer -= 1;
    if self.freq_timer == 0 {
        self.freq_timer = (2048 - self.period as u32) * 4;
        self.duty_pos = (self.duty_pos + 1) % 8;
    }
}

fn get_sample(&self) -> u8 {
    // Retourne 0 ou volume selon la position dans le duty
    if !self.enabled {
        return 0;
    }
    let duty = DUTY_TABLE[self.duty_cycle as usize][self.duty_pos as usize];
    duty * self.volume
}
```

### Trigger (quand NR14 bit 7 est écrit à 1)

```rust
fn trigger(&mut self) {
    self.enabled = true;
    self.freq_timer = (2048 - self.period as u32) * 4;
    self.volume = self.volume_initial;  // depuis NR12 bits 4-7
    self.duty_pos = 0;
    // (Plus tard : reset length counter, reset envelope timer)
}
```

---

## Step 4 — Downsampling et push

**Dans `APU::tick()` :** (appelé 4 T-cycles à la fois depuis IOBridge)

```rust
const CPU_CLOCK: u32 = 4_194_304;
const SAMPLE_RATE: u32 = 44_100;
const CYCLES_PER_SAMPLE: u32 = CPU_CLOCK / SAMPLE_RATE;  // ≈ 95

fn tick(&mut self) {
    // 1. Tick le channel (4 T-cycles)
    for _ in 0..4 {
        self.tick_channel1();
    }

    // 2. Downsampling : accumuler et push un sample tous les ~95 T-cycles
    self.sample_counter += 4;
    if self.sample_counter >= CYCLES_PER_SAMPLE {
        self.sample_counter -= CYCLES_PER_SAMPLE;

        let raw_sample = self.get_sample();  // 0-15

        // 3. Convertir en f32 [-1.0, 1.0]
        let normalized = (raw_sample as f32 / 15.0) * 2.0 - 1.0;
        // Si volume=0 ou disabled → normalized sera -1.0, c'est OK (silence = 0.0)
        let output = if self.enabled && self.volume > 0 {
            normalized
        } else {
            0.0
        };

        // 4. Push dans le ring buffer
        if let Some(ref buffer) = self.audio_buffer {
            buffer.lock().unwrap().push(output);
        }
    }
}
```

---

## Step 5 — Brancher dans l'émulateur

**Dans `CrabbyBoy` :**

```rust
pub struct CrabbyBoy {
    audio_output: Option<AudioOutput>,
}

pub fn run(&mut self, file_path: &str) -> Result<(), String> {
    // 1. Créer le buffer partagé
    let audio_buffer = Arc::new(Mutex::new(AudioBuffer::new(8192)));

    // 2. Démarrer le stream audio (stocké dans self pour rester vivant)
    self.audio_output = Some(AudioOutput::new(audio_buffer.clone()));

    // 3. Passer le buffer à l'APU via le bus
    bus.set_audio_buffer(audio_buffer);

    // 4. La boucle principale tourne normalement — l'APU push à chaque tick
}
```

---

## Step 6 — Registres (write dans IOBridge)

Quand le CPU écrit dans les registres audio :

```rust
// Dans apu.write(addr, val) :
match addr {
    0xFF11 => {
        self.duty_cycle = (val >> 6) & 0x03;
        self.length_timer = val & 0x3F;
    }
    0xFF12 => {
        self.volume_initial = (val >> 4) & 0x0F;
        self.envelope_direction = (val >> 3) & 0x01;  // 1=up, 0=down
        self.envelope_pace = val & 0x07;
        // Si top 5 bits = 0 → DAC off → channel disabled
        if val & 0xF8 == 0 { self.enabled = false; }
    }
    0xFF13 => {
        self.period = (self.period & 0x700) | val as u16;
    }
    0xFF14 => {
        self.period = (self.period & 0xFF) | ((val as u16 & 0x07) << 8);
        self.length_enable = (val & 0x40) != 0;
        if val & 0x80 != 0 {
            self.trigger();  // ← c'est ici que le son démarre
        }
    }
    0xFF24 => self.nr50 = val,  // master volume
    0xFF25 => self.nr51 = val,  // panning
    0xFF26 => {
        // Bit 7 = master enable. Si off → reset tout
        self.master_enable = (val & 0x80) != 0;
        if !self.master_enable { self.reset(); }
    }
    _ => {}
}
```

---

## Ce que Blargg écrit (pour tester)

```
FF26 ← 0x80  (APU ON)
FF24 ← 0x77  (volume max L+R)
FF25 ← 0xFF  (tous channels → L+R)
FF11 ← 0x80  (duty 50%, length 0)
FF12 ← 0xF3  (vol=15, down, pace=3)
FF13 ← 0x83  (period low)
FF14 ← 0x87  (trigger=1, length_enable=1, period high=7)
→ period = 0x783 = 1923
→ freq = 131072 / (2048 - 1923) = 1048 Hz
```

---

## Pour plus tard (pas nécessaire pour le beep)

- **Volume envelope** : toutes les N frames (pace), volume ±1. Donne le fade-out du beep.
- **Length counter** : après 64-L frames, le channel s'éteint automatiquement.
- **Frame sequencer** : clock à 512 Hz qui drive envelope + length. Doc : https://gbdev.io/pandocs/Audio_details.html#frame-sequencer
- **Sweep (CH1 only)** : modifie la fréquence au fil du temps.
- **Channel 2** : identique à CH1 sans sweep.
- **Channel 3** : wave (4-bit samples depuis wave RAM FF30-FF3F).
- **Channel 4** : noise (LFSR).

---

## Liens utiles

- Pan Docs Audio : https://gbdev.io/pandocs/Audio.html
- Pan Docs Registres : https://gbdev.io/pandocs/Audio_Registers.html
- Pan Docs Détails : https://gbdev.io/pandocs/Audio_details.html
- cpal docs : https://docs.rs/cpal/latest/cpal/
- Nightshade's APU (référence claire) : https://nightshade256.github.io/2021/03/27/gb-sound-emulation.html
