# 15. Architecture de l'APU & le module `audio/`

Avec le CPU, le bus, le timer, et un premier PPU tous en état raisonnable,
cette partie de l'histoire du projet (`7d79a24` et au-delà) se tourne
vers le son. Le son et le graphisme sont des sous-systèmes véritablement
indépendants — c'est pourquoi, dans l'histoire réelle du projet, le
travail sur l'APU a lieu *après* le PPU minimal (Chapitre 12) plutôt
qu'après un PPU complet : rien dans la génération audio ne dépend du fait
que des pixels soient dessinés.

## Un module `audio/` dédié

```text
src/
  audio/
    apu.rs           — the APU itself: 4 channels, mixing, frame sequencer
    audio_buffer.rs  — a ring buffer handing samples to the OS
    audio_output.rs  — talks to the actual sound card, via cpal
    channel.rs       — shared logic for channels 1 and 2 (square waves)
    wave_channel.rs  — channel 3 (custom waveform)
    noise_channel.rs — channel 4 (noise)
```

`hardware/apu.rs` (Chapitre 6) migre vers son propre module de premier
niveau, avec de la place pour que tout ce qui concerne le son vive
ensemble, à l'image de la façon dont `cpu/`, `bus/` et `hardware/` avaient
été séparés plus tôt (Chapitre 6).

## Quatre canaux, un mixeur

```rust
// src/audio/apu.rs
pub struct APU {
    audio_buffer: Option<Arc<Mutex<AudioBuffer>>>,
    div_apu_counter: u16,
    frame_seq_step: u8,
    tick_counter: f64,
    tick_per_sample: f64,
    is_on: bool,
    channel1: Channel,     // square wave + sweep
    channel2: Channel,     // square wave
    channel3: WaveChannel, // custom waveform
    channel4: NoiseChannel,
    nr50: u8, // master volume
    nr51: u8, // stereo panning
}
```

Correspondant exactement à la brève mention du Chapitre 0 : 4 générateurs
de son indépendants fonctionnant en parallèle, combinés en un signal
final. Les canaux 1 et 2 partagent une structure `Channel` (le canal 2
n'est simplement le canal 1 sans la fonctionnalité de sweep) — réutiliser
la même structure évite de dupliquer deux fois la logique de duty cycle
et d'enveloppe (envelope).

## Le séquenceur de trame : une horloge partagée pour toutes les fonctionnalités "lentes" des canaux

Chaque canal possède sa propre horloge rapide de génération audio
(produisant la forme d'onde réelle), mais plusieurs fonctionnalités plus
*lentes* — l'enveloppe (fondu du volume), le sweep (glissement de
fréquence, canal 1 uniquement), et les compteurs de longueur
(mise en sourdine automatique d'un canal après une durée définie) — sont
toutes pilotées par une seule horloge partagée à 512 Hz, dérivée de
l'horloge principale du CPU :

```rust
pub fn tick(&mut self) {
    for _ in 0..4 {
        self.div_apu_counter += 1;
        if self.div_apu_counter == 8192 {
            self.div_apu_counter = 0;
            self.frame_seq_step = (self.frame_seq_step + 1) & 0x07;
            if self.is_on {
                match self.frame_seq_step {
                    0 | 2 | 4 | 6 => self.clock_length_all(), // 256 Hz
                    _ => {}
                }
                if self.frame_seq_step == 7 {
                    self.clock_envelope_all(); // 64 Hz
                }
                if self.frame_seq_step == 2 || self.frame_seq_step == 6 {
                    self.clock_sweep(); // 128 Hz
                }
            }
        }
        if !self.is_on { continue; }
        self.channel1.tick();
        self.channel2.tick();
        // ...
    }
}
```

`div_apu_counter` atteignant 8192 cycles CPU correspond exactement à
512 Hz (4 194 304 Hz ÷ 8192 = 512). `frame_seq_step` parcourt ensuite 8
étapes (0-7), et différentes fonctionnalités sont cadencées à des étapes
différentes — les compteurs de longueur une étape sur deux (256 Hz), le
sweep tous les 4 pas (128 Hz), l'enveloppe une fois par cycle complet
(64 Hz). Ce "séquenceur de trame" unique et partagé est exactement la
manière dont le vrai hardware Game Boy cadence ces fonctionnalités, et
vaut la peine d'être retenu comme un concept nommé si vous lisez d'autres
documentations matérielles.

## Les duty cycles : ce qui donne une forme à une "onde carrée"

```rust
const DUTY_TABLE: [[u8; 8]; 4] = [
    [0, 0, 0, 0, 0, 0, 0, 1], // 00 → 12.5%
    [1, 0, 0, 0, 0, 0, 0, 1], // 01 → 25%
    [1, 0, 0, 0, 0, 1, 1, 1], // 10 → 50%
    [0, 1, 1, 1, 1, 1, 1, 0], // 11 → 75%
];
```

Une "onde carrée" n'est pas qu'une seule forme fixe — c'est un motif
répétitif de valeurs haute/basse, et *quelle fraction de chaque cycle est
"haute"* (le **duty cycle**) change son timbre, même à une même hauteur
de son (pitch). Ces 4 lignes sont les 4 motifs fixes que le vrai hardware
prend en charge, chacun étant simplement une séquence fixe de 8 bits
répétée encore et encore à la fréquence courante du canal. Les canaux 1
et 2 (Chapitre 16/17) utilisent tous deux cette table ; le canal 3
(Chapitre 18) lit à la place une forme d'onde arbitraire fournie par le
jeu ; le canal 4 (Chapitre 19) utilise un bruit pseudo-aléatoire au lieu
de toute forme répétitive.

## Faire sortir les échantillons vers de vrais haut-parleurs : `cpal`

```rust
// src/audio/audio_output.rs
pub struct AudioOutput {
    _stream: cpal::Stream, // must stay alive, or audio stops
}
```

C'est le seul endroit de tout le projet, jusqu'ici, qui dépend d'une
crate externe pour quelque chose de spécifique à l'OS : parler
réellement à la carte son. [`cpal`](https://docs.rs/cpal/latest/cpal/)
est une crate d'entrée/sortie audio multiplateforme — elle trouve un
périphérique de sortie, choisit un format/une fréquence d'échantillonnage
pris en charge, et fournit un callback auquel on demande de nouveaux
échantillons chaque fois que l'OS en a besoin davantage. L'APU lui-même
n'a aucune idée que `cpal` existe : il se contente d'écrire les
échantillons terminés dans un `AudioBuffer` (un tampon circulaire
(ring buffer) thread-safe), et le callback d'`AudioOutput` lit depuis ce
même tampon chaque fois que la carte son en demande davantage. Cette
séparation est importante : l'APU tourne au rythme de la vitesse
d'émulation du CPU, tandis que la carte son exige des échantillons selon
son propre calendrier — le tampon circulaire est ce qui permet à ces deux
mondes de timing différents de coexister en toute sécurité entre les
threads (`Arc<Mutex<AudioBuffer>>`).

## Ce que nous avons maintenant

- Un module `audio/` dédié, proprement séparé du CPU/bus/hardware.
- Le séquenceur de trame de l'APU, cadençant correctement les
  fonctionnalités d'enveloppe/sweep/longueur à leurs fréquences
  matérielles réelles.
- Une première véritable connexion à un vrai matériel audio via `cpal`.

## Ce qui manque encore

- Aucun canal individuel n'est encore entièrement implémenté — ce
  chapitre est l'échafaudage partagé ; les Chapitres 16 à 19 couvrent
  chaque canal à tour de rôle.
- Aucune véritable génération/mixage d'échantillons ne se produit encore
  d'une manière que vous pourriez réellement écouter.
