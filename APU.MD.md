# APU — Comprendre et implémenter le son Game Boy

## Objectif final

1. **Tetris joue sa musique** (channels 1, 2, 3, 4)
2. **Blargg dmg_sound** passe (01-registers → 04-sweep)


## Vue d'ensemble : Comment le Game Boy génère du son

---

## Pseudo-code des clocks APU



### clock_sweep
Uniquement pour CH1. Si sweep.enabled et sweep.pace != 0, décrémente sweep.timer. Quand il atteint 0, recharge, puis calcule la nouvelle fréquence. Si overflow, désactive le channel.

```rust
fn clock_sweep(&mut self) {
    let sweep = &mut self.ch1_sweep;
    if sweep.timer > 0 { sweep.timer -= 1; }
    if sweep.timer == 0 {
        sweep.timer = if sweep.pace != 0 { sweep.pace } else { 8 };
        if sweep.enabled && sweep.pace != 0 {
            let new_period = calculate_new_period(sweep);
            if new_period <= 2047 && sweep.step != 0 {
                self.ch1.period = new_period;
                sweep.shadow_period = new_period;
                // Re-vérifier overflow
                if calculate_new_period(sweep) > 2047 {
                    self.ch1.enabled = false;
                }
            } else if new_period > 2047 {
                self.ch1.enabled = false;
            }
        }
    }
}
```

---

Le Game Boy est une console de 1989. Pas de MP3, pas de streaming audio.
Le son est **synthétisé en temps réel** par un chip dédié : l'**APU** (Audio Processing Unit).

L'APU contient **4 générateurs de son** (channels) qui tournent en parallèle :

```
┌─────────────────────────────────────────────────────┐
│                      APU                             │
│                                                     │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌────────────┐ │
│  │ Channel 1│  │ Channel 2│  │ Channel 3│  │ Channel 4  │ │
│  │ Square+  │  │ Square   │  │ Wave     │  │ Noise      │ │
│  │ Sweep    │  │          │  │          │  │            │ │
│  └────┬─────┘  └────┬─────┘  └────┬─────┘  └─────┬──────┘ │
│       │              │              │              │        │
│       └──────┬───────┴──────┬───────┘              │        │
│              │              │                      │        │
│              ▼              ▼                      ▼        │
│         ┌────────────────────────────────────────────┐     │
│         │              Mixer (NR50/NR51)              │     │
│         │   Panning gauche/droite + Master volume    │     │
│         └────────────────────┬───────────────────────┘     │
│                              │                             │
└──────────────────────────────┼─────────────────────────────┘
                               │ signal audio
                               ▼
                          🔊 Haut-parleur
```

**Analogie musicale :**
- Channel 1 & 2 = deux synthétiseurs qui jouent des notes (mélodie, harmonie)
- Channel 3 = un sampler basique (basse, effets)
- Channel 4 = une boîte à rythmes (percussions)

Le jeu (la ROM) contrôle l'APU en écrivant dans des **registres mémoire** (0xFF10-0xFF3F).
Par exemple, pour jouer un Do sur le Channel 1, Tetris écrit la fréquence correspondante dans les registres FF13/FF14.

**Doc :** https://gbdev.io/pandocs/Audio.html

---

## Ce que tu as déjà implémenté

- [x] Channel 1 : square wave basique (duty + period + trigger)
- [x] Ring buffer + cpal output
- [x] Downsampling (tick_counter)
- [x] NR50/NR51/NR52 registers stockés

Ton Channel 1 actuel fait : trigger → boucle sur le duty pattern → produit du son.
Mais le son ne s'arrête jamais, ne change pas de volume, pas d'effet de fréquence.
C'est comme un synthé bloqué sur une note sans release ni vibrato.

---

## Étape 1 : Le Frame Sequencer

### C'est quoi ?

Le Frame Sequencer est un **métronome interne** de l'APU. C'est un simple compteur qui génère des "clocks" à intervalles réguliers pour piloter les composants lents de l'APU.

### Pourquoi ça existe ?

Le CPU du Game Boy tourne à ~4 MHz. Mais certains effets audio n'ont pas besoin d'être mis à jour aussi vite :
- L'enveloppe de volume change **64 fois par seconde** (64 Hz)
- Le compteur de durée avance **256 fois par seconde** (256 Hz)
- Le sweep de fréquence change **128 fois par seconde** (128 Hz)

Au lieu d'avoir 3 timers séparés, le hardware utilise UN seul compteur à **512 Hz** (= un tick tous les 8192 T-cycles) qui distribue les clocks selon un pattern fixe sur 8 steps :

```
                    Frame Sequencer (512 Hz)
                    ========================

        ┌─────────────────────────────────────────────┐
        │  Timer interne : compte les T-cycles        │
        │  Tous les 8192 T-cycles → avance d'un step  │
        └──────────────────────┬──────────────────────┘
                               │
            Step:  0   1   2   3   4   5   6   7
                   │       │       │       │   │
  Length (256Hz):  ✓       ✓       ✓       ✓
  Sweep (128Hz):          ✓               ✓
  Envelope(64Hz):                              ✓

  Puis ça boucle : 0, 1, 2, 3, 4, 5, 6, 7, 0, 1, 2...
```

**Analogie :** C'est comme un chef d'orchestre qui bat la mesure. À chaque temps, il pointe vers un pupitre différent : "violons, jouez ! ... cuivres, jouez !"

### D'où ça vient ?

C'est un choix de design hardware de Sharp (le fabricant du chip). Un seul diviseur de fréquence = moins de transistors = moins cher. Le DIV register du timer (0xFF04) est en fait connecté au bit 4, ce qui génère les 512 Hz. Sur le vrai hardware, c'est littéralement le même compteur.

### Comment l'implémenter

Tu as déjà `cycle_count` dans ton APU — pas besoin d'une struct séparée.
Il suffit de vérifier quand `cycle_count` atteint un multiple de 8192 :

```rust
pub fn tick(&mut self) {
    self.cycle_count += 4;

    // Frame Sequencer : tous les 8192 T-cycles, on clock les composants lents
    if self.cycle_count % 8192 == 0 {
        let step = ((self.cycle_count / 8192) % 8) as u8;
        match step {
            0 | 2 | 4 | 6 => self.clock_length_all(),   // 256 Hz
            _ => {}
        }
        if step == 7 {
            self.clock_envelope_all();                    // 64 Hz
        }
        if step == 2 || step == 6 {
            self.clock_sweep();                           // 128 Hz
        }
    }

    self.tick_channel1();
    // ... downsampling avec tick_counter ...
}
```

`cycle_count` fait déjà le boulot. Pas besoin d'une struct `FrameSequencer` en plus.

### Que font ces fonctions ?

Ces trois fonctions sont les "effets lents" que le frame sequencer cadence.
Elles sont détaillées aux étapes suivantes, mais en résumé :

| Fonction | Ce qu'elle fait | Fréquence | Détails |
|----------|----------------|-----------|---------|
| `clock_length_all()` | Décrémente le compteur de durée de chaque channel. Quand il atteint 0 → channel OFF. | 256 Hz | Étape 2 |
| `clock_envelope_all()` | Fait monter ou descendre le volume de chaque channel (ch1, ch2, ch4) d'un cran. | 64 Hz | Étape 3 |
| `clock_sweep()` | Modifie la fréquence du channel 1 (glissando haut/bas). | 128 Hz | Étape 4 |

Important :
- `clock_length_all()` décrémente bien `length_timer`.
- `clock_envelope_all()` ne décrémente PAS `env_dir` ni `env_pace`.
- `env_dir` et `env_pace` sont des paramètres configurés par la ROM (NRx2).
- Ce qui est décrémenté est `env_timer` (timer interne), puis on ajuste `volume` selon `env_dir`.

**Doc :** https://gbdev.io/pandocs/Audio_details.html#div-apu

---

## Structure recommandée des variables

Tu as demandé "toutes les vars qu'il faut dans un channel". En pratique, il faut 3 structs de channel (pulse, wave, noise), car les besoins ne sont pas les mêmes.

### 1) Pulse channel (CH1 / CH2)

CH1 et CH2 partagent la même base. CH1 ajoute la partie sweep.

```rust
#[derive(Default, Clone, Copy)]
pub struct PulseChannel {
    // Run state
    pub enabled: bool,
    pub dac_enabled: bool,

    // Waveform generation
    pub duty_cycle: u8,      // 0..3 (NRx1 bits 7-6)
    pub duty_pos: u8,        // 0..7
    pub period: u16,         // 11 bits (NRx3 + NRx4 bits 0-2)
    pub freq_timer: u32,     // countdown interne

    // Length unit
    pub length_timer: u8,    // 0..64
    pub length_enabled: bool,// NRx4 bit 6

    // Envelope unit
    pub volume: u8,          // volume courant 0..15
    pub initial_volume: u8,  // NRx2 bits 7-4
    pub env_dir: u8,         // NRx2 bit 3 (0=down,1=up)
    pub env_pace: u8,        // NRx2 bits 2-0
    pub env_timer: u8,       // countdown interne de l'envelope
}
```

### 2) Sweep unit (CH1 uniquement)

```rust
#[derive(Default, Clone, Copy)]
pub struct SweepUnit {
    pub pace: u8,            // NR10 bits 6-4
    pub direction: u8,       // NR10 bit 3 (0=add,1=sub)
    pub step: u8,            // NR10 bits 2-0

    pub enabled: bool,
    pub timer: u8,           // countdown interne
    pub shadow_period: u16,  // copie de period au trigger
}
```

### 3) Wave channel (CH3)

```rust
#[derive(Clone)]
pub struct WaveChannel {
    pub enabled: bool,
    pub dac_enabled: bool,   // NR30 bit 7

    pub period: u16,
    pub freq_timer: u32,

    pub length_timer: u16,   // 0..256
    pub length_enabled: bool,

    pub output_level: u8,    // NR32 bits 6-5 (0..3)
    pub wave_pos: u8,        // 0..31
    pub sample_buffer: u8,   // dernier sample lu (4-bit)
    pub wave_ram: [u8; 16],  // FF30..FF3F
}
```

### 4) Noise channel (CH4)

```rust
#[derive(Default, Clone, Copy)]
pub struct NoiseChannel {
    pub enabled: bool,
    pub dac_enabled: bool,

    // Length
    pub length_timer: u8,    // 0..64
    pub length_enabled: bool,

    // Envelope
    pub volume: u8,
    pub initial_volume: u8,
    pub env_dir: u8,
    pub env_pace: u8,
    pub env_timer: u8,

    // Noise generator
    pub clock_shift: u8,     // NR43 bits 7-4
    pub width_mode: bool,    // NR43 bit 3 (false=15bit,true=7bit)
    pub divisor_code: u8,    // NR43 bits 2-0
    pub freq_timer: u32,
    pub lfsr: u16,           // registre pseudo-aléatoire
}
```

### 5) Exemple d'agrégation dans l'APU

```rust
pub struct APU {
    // Sortie audio
    audio_buffer: Option<Arc<Mutex<AudioBuffer>>>,

    // Timing global
    cycle_count: u32,        // base pour DIV-APU / frame sequencer
    tick_counter: f64,       // downsampling vers 44.1kHz

    // Channels
    ch1: PulseChannel,
    ch2: PulseChannel,
    ch1_sweep: SweepUnit,
    ch3: WaveChannel,
    ch4: NoiseChannel,

    // Mixer / power regs
    nr50: u8,
    nr51: u8,
    nr52: u8,
    power_on: bool,
}
```

Note : tu peux garder un nom simple `Channel` pour l'instant si tu implémentes d'abord CH1/CH2, mais dès que tu ajoutes CH3/CH4, séparer les structs rend le code plus lisible.

---

## Étape 2 : Le Length Counter (compteur de durée)

### C'est quoi ?

Un compte-à-rebours qui **éteint automatiquement** un channel après une durée précise.

### Pourquoi ça existe ?

Sans length counter, quand tu déclenches une note, elle joue **indéfiniment** jusqu'à ce que le programme la coupe manuellement. Le length counter permet au jeu de dire : "joue cette note pendant exactement X temps, puis arrête-toi tout seul."

C'est essentiel pour les **effets sonores** courts (bruitages de saut, de collision) et pour les **percussions** (un clap dure ~100ms, pas l'éternité).

### Comment ça marche

```
Le jeu écrit NR11 = 0b00_100000  (length data = 32)
   → length_timer = 64 - 32 = 32 "ticks" restants

Le jeu écrit NR14 avec bit 6 = 1  (length enable = ON)
   → le length counter est armé

Frame Sequencer step 0 (puis 2, 4, 6) :
   length_timer : 32 → 31 → 30 → ... → 1 → 0
                                              └→ channel OFF !
```

**Fréquence :** clocké à 256 Hz (steps 0, 2, 4, 6 = 4 fois sur 8 steps × 512/8 = 256)

**Durée max :**
- Ch1/Ch2/Ch4 : timer part de 1 à 64 → durée max = 64/256 = 0.25 seconde
- Ch3 : timer part de 1 à 256 → durée max = 256/256 = 1 seconde

### Implémentation

Ajouter au `Channel` :
```rust
length_enabled: bool,  // NRx4 bit 6
```

```rust
fn clock_length(ch: &mut Channel) {
    if !ch.length_enabled { return; }
    if ch.length_timer > 0 {
        ch.length_timer -= 1;
        if ch.length_timer == 0 {
            ch.enabled = false;  // coupe le son
        }
    }
}
```

**Trigger :** quand on trigger un channel, si `length_timer == 0`, on le remet à max (64 ou 256).

**Doc :** https://gbdev.io/pandocs/Audio_details.html#length-timer

---

## Étape 3 : Le Volume Envelope (enveloppe de volume)

### C'est quoi ?

Un mécanisme qui fait **monter ou descendre le volume automatiquement** au fil du temps.

### Pourquoi ça existe ?

Dans la vraie vie, quand tu frappes une touche de piano, le son est fort au début puis s'atténue. C'est l'"enveloppe" du son. Sans ça, toutes les notes sonneraient comme des bips robotiques constants (ce que tu as actuellement !).

L'enveloppe donne de la **vie** et du **naturel** au son :
- Volume qui descend = note qui meurt naturellement (piano, guitare)
- Volume qui monte = crescendo, effet de "swell"
- Pas d'enveloppe = orgue (son constant tant qu'on appuie)

```
Volume (0-15)
    │
 15 ┤ ████
    │     ████
    │         ████
    │             ████
    │                 ████
  0 ┤                     ████████████  (volume = 0 = silence)
    └──────────────────────────────────── Temps
         ↑
    Trigger (note ON)        Env pace = vitesse de descente
```

### Comment ça marche

**Registre NR12 (0xFF12) pour Channel 1 :**
```
Bits 7-4 : Volume initial (0-15)     ← volume au moment du trigger
Bit 3    : Direction (0=descend, 1=monte)
Bits 2-0 : Pace (0-7)                ← vitesse. 0 = pas d'enveloppe
```

L'enveloppe est clockée à **64 Hz** (step 7 du frame sequencer).
Mais elle n'avance pas à chaque clock ! Le **pace** divise encore :
- Pace = 1 → avance tous les 1/64s
- Pace = 3 → avance tous les 3/64s
- Pace = 7 → avance tous les 7/64s
- Pace = 0 → désactivée

On utilise un timer interne (`env_timer`) qui compte de pace → 0, puis reload.

### Implémentation

Ajouter au `Channel` :
```rust
env_timer: u8,  // countdown, reload = env_pace
```

```rust
fn clock_envelope(ch: &mut Channel) {
    if ch.env_pace == 0 { return; }  // pace 0 = envelope désactivée

    ch.env_timer = ch.env_timer.saturating_sub(1);
    if ch.env_timer == 0 {
        ch.env_timer = ch.env_pace;  // reload

        // Monter ou descendre le volume
        if ch.env_dir == 1 && ch.volume < 15 {
            ch.volume += 1;
        } else if ch.env_dir == 0 && ch.volume > 0 {
            ch.volume -= 1;
        }
    }
}
```

**Trigger :** quand on trigger, `volume = initial_volume` et `env_timer = env_pace`.

**Doc :** https://gbdev.io/pandocs/Audio_Registers.html#ff12--nr12-channel-1-volume--envelope

---

## Étape 4 : Le Sweep (balayage de fréquence) — Channel 1 uniquement

### C'est quoi ?

Un mécanisme qui **modifie automatiquement la fréquence** (= la hauteur de la note) au fil du temps.

### Pourquoi ça existe ?

Le sweep crée des effets de **glissando** :
- Fréquence qui monte = son type "laser" 📈 (pew pew !)
- Fréquence qui descend = son type "bombe qui tombe" 📉

C'est aussi utilisé pour simuler des instruments qui "bendent" les notes (guitare, cuivres).
**Seul Channel 1 en dispose** — c'est ce qui le rend spécial par rapport à Channel 2.

```
Fréquence (Hz)
    │
    │                    ╱╱╱  (sweep up)
    │                 ╱╱╱
    │              ╱╱╱
    │           ╱╱╱
    │  ████████╱        ← fréquence initiale
    │
    └──────────────────── Temps
       ↑
   Trigger
```

### Comment ça marche

**Registre NR10 (0xFF10) :**
```
Bits 6-4 : Pace (0-7)       ← vitesse du sweep. 0 = désactivé
Bit 3    : Direction (0=fréq monte, 1=fréq descend)
Bits 2-0 : Step (0-7)       ← amplitude du changement
```

Clocké à **128 Hz** (steps 2, 6 du frame sequencer).

**Formule :** À chaque clock sweep :
```
delta = shadow_period >> step
nouveau_period = old_period ± delta   (selon direction)
```

Si `nouveau_period > 2047` → **overflow** → channel désactivé (protection contre les fréquences trop hautes qui feraient du bruit).

**shadow_period** est une copie de la period au moment du trigger. C'est sur cette copie qu'on travaille, pas directement sur le registre.

### Implémentation

```rust
struct Sweep {
    pace: u8,
    direction: u8,     // 0=addition (freq monte), 1=soustraction (freq descend)
    step: u8,
    timer: u8,         // countdown interne
    shadow_period: u16,
    enabled: bool,
}

fn clock_sweep(ch1: &mut Channel, sweep: &mut Sweep) {
    if sweep.timer > 0 {
        sweep.timer -= 1;
    }
    if sweep.timer == 0 {
        // Reload timer (pace=0 traité comme 8)
        sweep.timer = if sweep.pace != 0 { sweep.pace } else { 8 };

        if sweep.enabled && sweep.pace != 0 {
            let new_period = calculate_new_period(sweep);
            if new_period <= 2047 && sweep.step != 0 {
                // Appliquer le nouveau period
                ch1.period = new_period;
                sweep.shadow_period = new_period;
                // Re-vérifier overflow (oui, 2 fois !)
                if calculate_new_period(sweep) > 2047 {
                    ch1.enabled = false;
                }
            } else if new_period > 2047 {
                ch1.enabled = false;  // overflow = channel OFF
            }
        }
    }
}

fn calculate_new_period(sweep: &Sweep) -> u16 {
    let delta = sweep.shadow_period >> sweep.step;
    if sweep.direction == 0 {
        sweep.shadow_period + delta  // addition = fréq monte
    } else {
        sweep.shadow_period.wrapping_sub(delta)  // soustraction
    }
}
```

**Trigger et sweep :**
```rust
fn trigger_sweep(ch1: &mut Channel, sweep: &mut Sweep) {
    sweep.shadow_period = ch1.period;
    sweep.timer = if sweep.pace != 0 { sweep.pace } else { 8 };
    sweep.enabled = sweep.pace != 0 || sweep.step != 0;
    // Vérification overflow immédiate si step != 0
    if sweep.step != 0 {
        if calculate_new_period(sweep) > 2047 {
            ch1.enabled = false;
        }
    }
}
```

**Doc :** https://gbdev.io/pandocs/Audio_Registers.html#ff10--nr10-channel-1-sweep

---

## Étape 5 : Channel 2 (deuxième voix square)

### C'est quoi ?

Un deuxième générateur d'onde carrée, **identique à Channel 1 mais sans sweep**.

### Pourquoi ça existe ?

Avec un seul channel, tu ne peux jouer qu'une note à la fois. Deux channels = tu peux jouer **mélodie + harmonie** simultanément. Dans Tetris :
- Channel 1 : mélodie principale (la ligne du haut)
- Channel 2 : contre-mélodie / harmonisation

### Comment l'implémenter

Tu as déjà la struct `Channel` qui fonctionne pour ch1. Il suffit de :
1. Créer un `channel2: Channel` dans l'APU
2. Router les registres 0xFF16-0xFF19 vers channel2
3. Appeler `tick_channel2()` et `get_ch2_sample()` exactement comme ch1
4. Pas de sweep.

**Registres :**
| Addr | Nom | Identique à |
|------|-----|-------------|
| 0xFF16 | NR21 | NR11 (duty + length) |
| 0xFF17 | NR22 | NR12 (volume envelope) |
| 0xFF18 | NR23 | NR13 (period low) |
| 0xFF19 | NR24 | NR14 (trigger + period high) |

**Doc :** https://gbdev.io/pandocs/Audio_Registers.html#sound-channel-2--pulse

---

## Étape 6 : Channel 3 (Wave — le sampler)

### C'est quoi ?

Un channel qui joue des **formes d'onde personnalisées** stockées dans une petite RAM de 16 octets (= 32 échantillons de 4 bits chacun).

### Pourquoi ça existe ?

Les ondes carrées (ch1/ch2) sonnent bien pour les mélodies aiguës, mais elles manquent de richesse pour les sons graves ou les timbres spéciaux. Le Wave channel permet au jeu de dessiner **n'importe quelle forme d'onde** :

```
Wave RAM (FF30-FF3F) : 16 octets = 32 nybbles (4 bits)

Chaque nybble = un échantillon entre 0 et 15 :

  15│    ██
    │  ██  ██
    │██      ██
    │          ██
   0│            ████████    ← exemple: onde triangle
    └────────────────────
     Positions 0  ...  31

Le channel lit ces 32 valeurs en boucle, à la vitesse
déterminée par le period register.
```

**Dans Tetris :** Channel 3 joue la **ligne de basse**. La Wave RAM contient une forme d'onde qui ressemble à une basse (un truc entre triangle et carré).

### Comment ça marche

La différence principale avec ch1/ch2 :
- **Pas de duty cycle** : on lit directement les samples depuis la Wave RAM
- **Le freq timer avance 2× plus vite** : `(2048 - period) * 2` au lieu de `* 4`
- **Le volume est un simple shift** (pas d'envelope) :
  - Output level 0 → mute (shift de 4 = résultat 0)
  - Output level 1 → 100% (shift de 0)
  - Output level 2 → 50% (shift de 1)
  - Output level 3 → 25% (shift de 2)

### Implémentation

```rust
struct WaveChannel {
    enabled: bool,
    dac_enabled: bool,       // NR30 bit 7
    length_timer: u16,       // 0-256 (plus grand que ch1/2!)
    length_enabled: bool,
    output_level: u8,        // 0-3 (NR32 bits 6-5)
    period: u16,
    freq_timer: u32,
    wave_pos: u8,            // 0-31, position dans les 32 samples
    wave_ram: [u8; 16],      // 16 octets = 32 nybbles
    sample_buffer: u8,       // dernier sample lu (4-bit, 0-15)
}

fn tick_wave(ch: &mut WaveChannel) {
    if !ch.enabled { return; }
    ch.freq_timer = ch.freq_timer.saturating_sub(1);
    if ch.freq_timer == 0 {
        ch.freq_timer = (2048 - ch.period as u32) * 2;  // ← *2, pas *4 !
        ch.wave_pos = (ch.wave_pos + 1) % 32;
        // Lire le sample à wave_pos
        let byte = ch.wave_ram[(ch.wave_pos / 2) as usize];
        ch.sample_buffer = if ch.wave_pos % 2 == 0 {
            (byte >> 4) & 0x0F   // nybble haut
        } else {
            byte & 0x0F          // nybble bas
        };
    }
}

fn get_wave_sample(ch: &WaveChannel) -> u8 {
    if !ch.enabled || !ch.dac_enabled { return 0; }
    let shift = match ch.output_level {
        0 => 4,  // mute
        1 => 0,  // 100%
        2 => 1,  // 50%
        3 => 2,  // 25%
        _ => 4,
    };
    ch.sample_buffer >> shift
}
```

**Registres :**
| Addr | Nom | Bits |
|------|-----|------|
| 0xFF1A | NR30 | `D--- ----` D=DAC on/off |
| 0xFF1B | NR31 | `LLLL LLLL` L=length (timer = 256 - L) |
| 0xFF1C | NR32 | `-VV- ----` V=output level (0-3) |
| 0xFF1D | NR33 | `PPPP PPPP` P=period low |
| 0xFF1E | NR34 | `TL-- -PPP` T=trigger, L=length enable, P=period high |
| 0xFF30-0xFF3F | Wave RAM | données brutes |

**Doc :** https://gbdev.io/pandocs/Audio_Registers.html#sound-channel-3--wave-output

---

## Étape 7 : Channel 4 (Noise — les percussions)

### C'est quoi ?

Un générateur de **bruit pseudo-aléatoire** qui crée des sons de percussions, explosions, vent, etc.

### Pourquoi ça existe ?

Les percussions (caisse claire, charleston, cymbale) sont des sons **non-tonaux** = pas de note précise, juste du bruit structuré. Le channel 4 produit ce type de son en utilisant un **LFSR** (Linear Feedback Shift Register).

### Le LFSR, c'est quoi ?

C'est un registre à décalage avec une boucle de rétroaction. À chaque tick, il produit un bit (0 ou 1) de manière pseudo-aléatoire :

```
LFSR 15 bits :  [14][13][12][11][10][9][8][7][6][5][4][3][2][1][0]
                  ↑                                            │  │
                  │                              XOR ──────────┘  │
                  │                               │               │
                  └───────────── résultat ─────────┘               └→ OUTPUT
                                                                      (0 ou 1)

À chaque tick :
  1. XOR bit 0 et bit 1
  2. Shift tout vers la droite
  3. Mettre le résultat du XOR dans bit 14
  4. Le bit qui sort (ancien bit 0) = le sample (inversé)
```

**Mode 7-bit :** En plus de mettre le XOR en bit 14, on le met aussi en bit 6. Ça donne un pattern plus court qui sonne plus "métallique" / "tonal" (utile pour certains effets).

**La vitesse du LFSR** détermine la "hauteur" du bruit :
- Rapide = bruit blanc (cymbale, souffle)
- Lent = grondement, roulement

### Comment la vitesse est contrôlée

**NR43 (0xFF22) :**
```
Bits 7-4 : Clock shift (s)     ← exposant
Bit 3    : Width mode (0=15bit, 1=7bit)
Bits 2-0 : Divisor code (r)    ← base

Freq timer = DIVISORS[r] << s
DIVISORS = [8, 16, 32, 48, 64, 80, 96, 112]
```

Plus le timer est long, plus le bruit est grave.

### Implémentation

```rust
struct NoiseChannel {
    enabled: bool,
    length_timer: u8,
    length_enabled: bool,
    // Envelope (identique à ch1/ch2)
    volume: u8,
    initial_volume: u8,
    env_dir: u8,
    env_pace: u8,
    env_timer: u8,
    // Noise spécifique
    clock_shift: u8,    // NR43 bits 7-4
    width_mode: bool,   // NR43 bit 3 (true = 7-bit)
    divisor_code: u8,   // NR43 bits 2-0
    freq_timer: u32,
    lfsr: u16,          // le registre à décalage (15 bits)
}

const DIVISORS: [u32; 8] = [8, 16, 32, 48, 64, 80, 96, 112];

fn tick_noise(ch: &mut NoiseChannel) {
    ch.freq_timer = ch.freq_timer.saturating_sub(1);
    if ch.freq_timer == 0 {
        ch.freq_timer = DIVISORS[ch.divisor_code as usize] << ch.clock_shift;

        // Feedback = XOR des bits 0 et 1
        let xor_bit = (ch.lfsr & 0b01) ^ ((ch.lfsr >> 1) & 0b01);
        // Shift right et injecter le feedback en bit 14
        ch.lfsr = (ch.lfsr >> 1) | (xor_bit << 14);
        // Mode 7-bit : aussi en bit 6
        if ch.width_mode {
            ch.lfsr = (ch.lfsr & !(1 << 6)) | (xor_bit << 6);
        }
    }
}

fn get_noise_sample(ch: &NoiseChannel) -> u8 {
    if !ch.enabled { return 0; }
    // Bit 0 INVERSÉ : 0 → volume, 1 → silence
    if ch.lfsr & 0x01 == 0 { ch.volume } else { 0 }
}
```

**Trigger noise :**
```rust
fn trigger_noise(ch: &mut NoiseChannel) {
    ch.enabled = true;
    ch.lfsr = 0x7FFF;  // tous les bits à 1
    ch.volume = ch.initial_volume;
    ch.env_timer = ch.env_pace;
    if ch.length_timer == 0 { ch.length_timer = 64; }
}
```

**Doc :** https://gbdev.io/pandocs/Audio_Registers.html#sound-channel-4--noise

---

## Étape 8 : Le Mixer (NR50 / NR51)

### C'est quoi ?

Le mixer combine les 4 channels en un signal stéréo final (gauche + droite).

### Pourquoi ça existe ?

Le Game Boy a un **haut-parleur mono** mais une **sortie casque stéréo**. Le mixer permet au jeu de placer chaque channel à gauche, à droite, ou des deux côtés. Ça donne de la **spatialisation** au son (dans Tetris : mélodie centrée, effets sur un côté).

### Comment ça marche

**NR51 (0xFF25) — Panning (quel channel va où) :**
```
    Bit 7  Bit 6  Bit 5  Bit 4  │  Bit 3  Bit 2  Bit 1  Bit 0
    CH4←   CH3←   CH2←   CH1←   │  CH4→   CH3→   CH2→   CH1→
    (left)                       │  (right)
```

**NR50 (0xFF24) — Volume master :**
```
    Bit 7    Bits 6-4      Bit 3    Bits 2-0
    VIN←     Volume left   VIN→     Volume right
    (ignoré)  (0-7)       (ignoré)   (0-7)
```

Volume left/right : valeur 0-7, la sortie est multipliée par `(volume + 1)`.

### Implémentation

```rust
fn mix_samples(&self) -> (f32, f32) {
    let samples = [
        self.get_ch1_sample() as f32,
        self.get_ch2_sample() as f32,
        self.get_ch3_sample() as f32,
        self.get_ch4_sample() as f32,
    ];

    let mut left = 0.0f32;
    let mut right = 0.0f32;

    for i in 0..4 {
        if self.nr51 & (0x10 << i) != 0 { left += samples[i]; }
        if self.nr51 & (0x01 << i) != 0 { right += samples[i]; }
    }

    let left_vol = ((self.nr50 >> 4) & 0x07) as f32 + 1.0;  // 1-8
    let right_vol = (self.nr50 & 0x07) as f32 + 1.0;         // 1-8

    left = left * left_vol / 8.0;
    right = right * right_vol / 8.0;

    // Normaliser : 4 channels × 15 max × 8 volume = range brute 0-480
    // On ramène à [-1.0, 1.0] pour cpal
    let normalize = 1.0 / (15.0 * 4.0);
    (left * normalize, right * normalize)
}

/// Pour un buffer mono (ton cas actuel) :
fn get_mono_sample(&self) -> f32 {
    let (left, right) = self.mix_samples();
    (left + right) / 2.0
}
```

**Doc :** https://gbdev.io/pandocs/Audio_Registers.html#ff24--nr50-master-volume--vin-panning

---

## Étape 9 : Le DAC (Digital-to-Analog Converter)

### C'est quoi ?

Chaque channel a un **DAC** qui convertit les valeurs numériques (0-15) en signal analogique. Le DAC peut être **allumé ou éteint** indépendamment du channel.

### Pourquoi ça existe ?

Sur le vrai hardware, quand le DAC est éteint, le channel est **physiquement déconnecté** de la sortie. Ça évite le bruit de fond. C'est aussi un moyen pour le jeu de couper un channel proprement.

La règle est simple : **si le DAC est off, le channel ne peut PAS être enabled.**
Même un trigger ne peut pas activer un channel dont le DAC est off.

### Comment savoir si le DAC est on/off ?

| Channel | DAC ON si... | Registre |
|---------|-------------|----------|
| 1 | NR12 bits 7-3 ≠ 0 | `val & 0xF8 != 0` |
| 2 | NR22 bits 7-3 ≠ 0 | `val & 0xF8 != 0` |
| 3 | NR30 bit 7 = 1 | `val & 0x80 != 0` |
| 4 | NR42 bits 7-3 ≠ 0 | `val & 0xF8 != 0` |

Pour ch1/ch2/ch4 : le DAC est ON si volume initial > 0 OU direction = up. Autrement dit, si les bits d'envelope ne sont pas tous à 0.

### Implémentation

```rust
fn dac_enabled_pulse(nr_x2: u8) -> bool {
    nr_x2 & 0xF8 != 0  // bits 7-3 pas tous à 0
}

// Quand on écrit NR12/NR22/NR42 :
fn write_nrx2(ch: &mut Channel, val: u8) {
    ch.initial_volume = (val >> 4) & 0x0F;
    ch.env_dir = (val >> 3) & 0x01;
    ch.env_pace = val & 0x07;
    // DAC check
    if val & 0xF8 == 0 {
        ch.enabled = false;  // DAC off → force channel off
    }
}
```

**Doc :** https://gbdev.io/pandocs/Audio_details.html#dacs

---

## Étape 10 : NR52 — Le master switch

### C'est quoi ?

Le registre **NR52 (0xFF26)** est l'interrupteur principal de l'APU. Il a aussi un rôle de **registre de statut** en lecture.

### Pourquoi ça existe ?

- **Power off (bit 7 = 0)** : coupe tout l'APU pour économiser la batterie. Reset tous les registres.
- **Statut (bits 0-3)** : permet au jeu de savoir quels channels sont actifs sans lire chaque registre individuel.

### Lecture de NR52

```
Bit 7 : Master ON/OFF (R/W)
Bit 6-4 : toujours 1 (OR mask 0x70)
Bit 3 : Channel 4 actif ? (read-only)
Bit 2 : Channel 3 actif ? (read-only)
Bit 1 : Channel 2 actif ? (read-only)
Bit 0 : Channel 1 actif ? (read-only)
```

### Écriture de NR52

**Seul le bit 7 est writable.** Écrire 0 au bit 7 :
- Tous les registres FF10-FF25 → reset à 0
- Tous les channels → disabled
- Les length timers sont **conservés** (quirk du hardware)
- La Wave RAM est **conservée**

### Écriture quand APU est OFF

Quand bit 7 = 0, **toute écriture dans FF10-FF25 est ignorée** (sauf NR41 length sur DMG, quirk).

### Implémentation

```rust
fn read_nr52(&self) -> u8 {
    let mut val = 0x70;  // bits 4-6 toujours 1
    if self.power_on { val |= 0x80; }
    if self.channel1.enabled { val |= 0x01; }
    if self.channel2.enabled { val |= 0x02; }
    if self.channel3.enabled { val |= 0x04; }
    if self.channel4.enabled { val |= 0x08; }
    val
}

fn write_nr52(&mut self, val: u8) {
    let new_power = val & 0x80 != 0;
    if self.power_on && !new_power {
        // Power OFF → reset tout sauf length timers et wave RAM
        self.reset_all_registers();
    }
    if !self.power_on && new_power {
        // Power ON → frame sequencer reset à step 0
        self.cycle_count = 0;
    }
    self.power_on = new_power;
}

fn write_register(&mut self, addr: u16, val: u8) {
    // Guard: si APU off, ignorer les écritures (sauf NR52 lui-même)
    if !self.power_on && addr != 0xFF26 {
        return;
    }
    // ... dispatch normal
}
```

**Doc :** https://gbdev.io/pandocs/Audio_Registers.html#ff26--nr52-audio-master-control

---

## Bonus : Les OR masks (crucial pour blargg 01-registers)

### C'est quoi ?

Quand tu **lis** un registre APU, certains bits retournent toujours 1, même s'ils ont été écrits à 0. C'est parce que ces bits sont "write-only" ou inutilisés, et le hardware les force à 1 en lecture.

### Pourquoi blargg en a besoin ?

Le test `01-registers` lit chaque registre et vérifie que les bits non-lisibles retournent 1. Si tu retournes la valeur brute écrite, le test échoue.

### Table des OR masks

| Addr | Registre | OR mask | Raison |
|------|----------|---------|--------|
| FF10 | NR10 | `0x80` | bit 7 inutilisé |
| FF11 | NR11 | `0x3F` | length (bits 0-5) non-lisible |
| FF12 | NR12 | `0x00` | tout lisible |
| FF13 | NR13 | `0xFF` | write-only (period low) |
| FF14 | NR14 | `0xBF` | period high + trigger non-lisibles |
| FF16 | NR21 | `0x3F` | comme NR11 |
| FF17 | NR22 | `0x00` | tout lisible |
| FF18 | NR23 | `0xFF` | write-only |
| FF19 | NR24 | `0xBF` | comme NR14 |
| FF1A | NR30 | `0x7F` | bits 0-6 non-lisibles |
| FF1B | NR31 | `0xFF` | write-only |
| FF1C | NR32 | `0x9F` | bits 0-4 et 7 non-lisibles |
| FF1D | NR33 | `0xFF` | write-only |
| FF1E | NR34 | `0xBF` | comme NR14 |
| FF20 | NR41 | `0xFF` | write-only |
| FF21 | NR42 | `0x00` | tout lisible |
| FF22 | NR43 | `0x00` | tout lisible |
| FF23 | NR44 | `0xBF` | comme NR14 |
| FF24 | NR50 | `0x00` | tout lisible |
| FF25 | NR51 | `0x00` | tout lisible |
| FF26 | NR52 | `0x70` | bits 4-6 toujours 1 |

**Implémentation :**
```rust
fn read(&self, addr: u16) -> u8 {
    let raw = self.get_raw_register(addr);
    let mask = self.get_or_mask(addr);
    raw | mask
}
```

**Doc :** https://gbdev.io/pandocs/Audio_Registers.html (colonne "Read mask" dans chaque registre)

---

## Ordre d'implémentation recommandé

```
1. Frame Sequencer                          ← fondation pour tout le reste
2. Volume Envelope (ch1)                    ← le bip fade out
3. Length counter (ch1)                     ← le bip s'arrête
4. Channel 2 (copier ch1 sans sweep)        ← Tetris mélodie complète
5. Channel 3 (wave)                         ← Tetris basse
6. Channel 4 (noise)                        ← Tetris percussions
7. Mixer (NR50/NR51)                        ← volume correct
8. Sweep (ch1)                              ← effets de fréquence
9. DAC + NR52 master on/off                 ← blargg 01
10. Read-back avec OR masks                  ← blargg 01
```

Après chaque étape, lance Tetris et écoute le résultat. Tu devrais entendre la musique s'enrichir progressivement.

---

## Blargg dmg_sound : ce que chaque test vérifie

| Test | Concept testé | Pré-requis |
|------|--------------|------------|
| 01-registers | R/W registres, OR masks, NR52 power off | Tout le register map + OR masks + NR52 |
| 02-len ctr | Length counter timing, edge cases | Frame sequencer + length |
| 03-trigger | Trigger behavior (4 channels) | Trigger correcte + DAC |
| 04-sweep | Sweep calculations, overflow | Sweep complet |
| 05-sweep details | Sweep negate mode quirk | Sweep très précis |
| 06+ | Edge cases timing | Implémentation très fidèle |

---

## Liens utiles

- **Pan Docs Audio (vue d'ensemble)** : https://gbdev.io/pandocs/Audio.html
- **Pan Docs Registres (détail par registre)** : https://gbdev.io/pandocs/Audio_Registers.html
- **Pan Docs Détails (frame seq, DAC, etc.)** : https://gbdev.io/pandocs/Audio_details.html
- **Nightshade's APU tutorial** : https://nightshade256.github.io/2021/03/27/gb-sound-emulation.html
- **GBEDG Audio** : https://hacktix.github.io/GBEDG/apu/
- **Explanation of LFSR** : https://en.wikipedia.org/wiki/Linear-feedback_shift_register
