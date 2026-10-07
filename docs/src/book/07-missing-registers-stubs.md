# 7. Compléter les registres manquants

Avec `IOBridge` en place (Chapitre 6), il est désormais facile d'ajouter un
vrai stockage derrière des registres qui étaient auparavant des constantes
codées en dur ou carrément absents. Le commit de ce chapitre (`5379233`)
fait exactement cela pour le joypad et — fait notable — plante la toute
première structure `PPU`, même si les vrais graphismes sont encore loin.

## `Joypad` : le stockage d'abord, les vrais boutons plus tard

```rust
// src/hardware/joypad.rs
pub struct Joypad {
    p1: u8,
}

impl Joypad {
    pub fn new() -> Self {
        Joypad { p1: 0 }
    }
    pub fn read(&self) -> u8 {
        self.p1
    }
    pub fn write(&mut self, val: u8) {
        self.p1 = val;
    }
}
```

`P1` (registre `0xFF00`) est le vrai nom du registre du joypad dans la
documentation Game Boy. À ce stade, c'est juste un simple octet sans
logique de bouton réelle attachée — les jeux peuvent y écrire et relire ce
qu'ils y ont écrit, ce qui suffit à les empêcher de rester bloqués, sans
pour autant signaler la moindre pression de bouton. La gestion réelle des
boutons, y compris le protocole un peu particulier de « sélectionner quel
groupe de 4 boutons on interroge », c'est le Chapitre 21.

## `PPU` : une structure qui existe, mais ne fait que stocker des nombres

```rust
// src/hardware/ppu.rs
pub struct PPU {
    lcd_control: u8, // FF40 — LCDC: LCD control
    scy: u8,         // FF42–FF43 — SCY, SCX
    scx: u8,
    lcd_y_coord: u8, // FF44 — LY: LCD Y coordinate [read-only]
    bgp: u8,         // FF47 — BGP
}

impl PPU {
    pub fn read(&self, addr: u16) -> u8 {
        match addr {
            0xFF40 => self.lcd_control,
            0xFF42 => self.scy,
            0xFF43 => self.scx,
            0xFF44 => self.lcd_y_coord,
            0xFF47 => self.bgp,
            _ => { println!("[PPU] READ NOT IMPLEMENTED FOR ADDR: {:02X}", addr); 0x00 }
        }
    }
    pub fn write(&mut self, addr: u16, val: u8) {
        match addr {
            0xFF40 => self.lcd_control = val,
            0xFF42 => self.scy = val,
            0xFF43 => self.scx = val,
            0xFF44 => self.lcd_y_coord = val,
            0xFF47 => self.bgp = val,
            _ => { /* ... */ }
        }
    }
}
```

C'est une étape délibérément petite et honnête : cinq registres du PPU que
vous avez déjà rencontrés conceptuellement au Chapitre 0 (`LCDC` — le
registre de contrôle maître « quoi dessiner et comment » ; `SCY`/`SCX` —
position de défilement de l'arrière-plan ; `LY` — le compteur de « ligne
de balayage actuelle » du PPU ; `BGP` — la palette de l'arrière-plan),
chacun n'étant qu'un octet qui se souvient de la dernière valeur écrite.
Rien n'*utilise* encore ces valeurs pour dessiner quoi que ce soit — il n'y
a ni timing, ni accès à la VRAM, ni comptage de lignes de balayage. Mais
une vraie structure existe maintenant pour construire dessus, ce qui
compte plus qu'il n'y paraît : le Chapitre 12 (le vrai PPU minimal) étend
*cette structure exacte* plutôt que de repartir de zéro.

## Connecter les deux à `IOBridge`

```rust
pub struct IOBridge {
    joypad: Joypad,
    serial: Serial,
    timer: Timer,
    interrupt_flag: u8,
    audio: APU,
    ppu: PPU,     // $FF40–$FF4B
    key1_spd: u8, // $FF4D — CGB-only, harmless to store anyway
}
```

```rust
pub fn read(&self, addr: u16) -> u8 {
    match addr {
        0xFF00 => self.joypad.read(),
        0xFF01..=0xFF02 => self.serial.read(addr),
        0xFF04..=0xFF07 => self.timer.read(addr),
        0xFF0F => self.interrupt_flag,
        0xFF10..=0xFF26 => self.audio.read(addr),
        0xFF40..=0xFF4B => self.ppu.read(addr),
        0xFF4D => self.key1_spd,
        _ => { println!("[IOREG] READ NOT IMPLEMENTED FOR ADDR: {:02X}", addr); 0x00 }
    }
}
```

Comparez ceci à la version du Chapitre 6 : la constante codée en dur
`0xFF40 => 0x91` a disparu, remplacée par `self.ppu.read(addr)` — une
vraie délégation vers un composant réel (même s'il reste encore
essentiellement vide). Remarquez également que le comportement de repli
pour les adresses non mappées est passé d'un `panic!` (Chapitre 6) à
l'affichage d'un avertissement avec retour de `0x00` — une petite
amélioration de robustesse mais significative : un jeu accédant à une
adresse que nous n'avons pas encore implémentée ne fait plus planter tout
l'émulateur, il reçoit simplement un zéro (peut-être erroné) et une ligne
de journal (log) à examiner plus tard.

## Ce que nous avons maintenant

- Une structure `PPU` et une structure `Joypad` réelles (quoique
  minimales), toutes deux connectées via `IOBridge`.
- Cinq registres PPU nommés avec un vrai stockage derrière.
- Un comportement de repli plus tolérant pour les adresses non encore
  implémentées.

## Ce qui manque encore

- Le `PPU` n'a encore aucun comportement — pas de timing, pas de
  progression des lignes de balayage, pas de déclenchement d'interruption
  (interrupt), pas de pixels. Le Chapitre 12 est l'endroit où il commence
  à réellement faire quelque chose.
- Le `Joypad` n'a encore aucune notion d'état réel des boutons — juste un
  octet « renvoyer ce qui a été écrit ».
- `STAT` (`0xFF41`, statut du LCD) n'est pas encore listé dans le
  match read/write propre au PPU — il tombe pour l'instant dans le chemin
  d'avertissement générique de `IOBridge`.
