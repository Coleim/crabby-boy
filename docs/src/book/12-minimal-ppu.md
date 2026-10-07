# 12. Les scanlines 101 & un PPU minimal

C'est le chapitre vers lequel tout ce livre a construit son chemin (et,
honnêtement, là où un nouveau guide complémentaire prend le relais par la
suite — voir la Partie IX). Ce commit (`624949c`) donne à la structure
`PPU` du Chapitre 7 son premier véritable comportement : compter le temps
et déclencher une interruption (interrupt). Toujours **pas de pixels pour
l'instant** — c'est délibéré, et expliqué ci-dessous.

## Un rappel : comment le véritable PPU passe son temps

(Si cela semble nouveau, le Chapitre 0 a introduit le vocabulaire — PPU,
VRAM — en un coup d'œil ; voici maintenant spécifiquement le modèle de
timing.)

L'écran fait 160×144 pixels, dessiné une ligne à la fois, 60 fois par
seconde. Une trame (frame) complète prend exactement **70 224 "dots"**
(l'unité d'horloge propre au PPU, 4 dots par cycle M du CPU) — répartis
en **154 scanlines × 456 dots chacune**. Sur ces 154 scanlines, seules les
144 premières sont réellement visibles ; les 10 restantes forment une
pause appelée **VBlank** (blanc vertical), utilisée historiquement pour
laisser aux anciens écrans CRT le temps de ramener physiquement leur
faisceau d'électrons dans le coin supérieur gauche, et utilisée par les
jeux Game Boy comme une fenêtre sûre pour mettre à jour les données
graphiques sans déchirement d'image.

## Compter les dots, un tick à la fois

```rust
// src/hardware/ppu.rs
pub struct PPU {
    lcd_control: u8,
    scy: u8,
    scx: u8,
    lcd_y_coord: u8, // FF44 — LY
    bgp: u8,
    dots: u16,       // T-cycle counter
}

pub fn tick(&mut self) -> bool {
    self.dots += 4;
    if self.dots >= 456 {
        self.dots -= 456;
        self.lcd_y_coord += 1;
        if self.lcd_y_coord >= 154 {
            self.lcd_y_coord = 0;
        }
        if self.lcd_y_coord == 144 {
            return true;
        }
    }
    false
}
```

Comparez cela directement au `tick` du Timer au Chapitre 9 — même forme,
même idée : un compteur interne avance à chaque appel de cette fonction,
et lorsqu'il franchit un seuil (456 dots), quelque chose d'observable se
produit (ici : `LY`, le registre de scanline courante du Chapitre 7,
s'incrémente). Quand `LY` atteint 144, c'est exactement le moment où
VBlank commence — et `tick` retourne `true` pour le signaler, exactement
une fois par trame (frame).

`self.dots += 4` correspond à la manière dont cette fonction est appelée :
une fois par cycle M du CPU, chacun valant 4 dots — la même granularité de
tick par accès que celle construite au Chapitre 10 pour le timer,
pilotant désormais aussi le PPU.

## Câbler l'interruption VBlank

```rust
// src/bus/iobridge.rs
pub fn tick(&mut self) {
    if self.timer.tick() {
        self.interrupt_flag |= 0b0000_0100; // Timer interrupt
    }
    if self.ppu.tick() {
        self.interrupt_flag |= 0b0000_0001; // VBlank interrupt
    }
    self.audio.tick();
}
```

Exactement le même schéma que l'interruption du Timer au Chapitre 9 :
`tick()` retourne `true` précisément quand quelque chose méritant une
interruption s'est produit, et l'appelant fait un OU logique sur le bit
correspondant dans `interrupt_flag` (`IF`, `0xFF0F`). VBlank est le bit
d'interruption 0 — la toute première et, sur le vrai hardware, de loin la
plus utilisée, car c'est ainsi que les jeux savent qu'il est sûr de
commencer les mises à jour graphiques de la trame suivante.

## Deux petites corrections de justesse `IF`/`IE`, en passant

```rust
0xFF0F => self.interrupt_flag | 0b1110_000, // on read: top 3 bits always read as 1
```
```rust
0xFF0F => self.interrupt_flag = val & 0b0001_1111, // on write: only the low 5 bits are real
```

`IF` n'est en termes matériels qu'un registre de 5 bits (un bit par type
d'interruption : VBlank, STAT, Timer, Serial, Joypad) — les 3 bits de
poids fort n'existent pas réellement en stockage et se lisent toujours
comme `1`. Maintenant qu'une *seconde* véritable source d'interruption
(VBlank) existe aux côtés du Timer, ces détails commencent réellement à
compter pour les ROMs de test qui inspectent `IF` précisément, d'où leur
resserrement ici.

## Pourquoi pas encore de pixels — et c'est tout à fait normal

Il serait raisonnable de s'attendre à ce que "le chapitre PPU" se termine
avec quelque chose affiché à l'écran. Ce n'est pas le cas ici, et c'est un
aperçu honnête et intentionnel de la façon dont le projet a réellement été
construit : obtenir d'abord le squelette du *timing* correct et
démontrablement juste (dots → scanlines → VBlank → interruption), *avant*
de consacrer des efforts au décodage des tuiles, au défilement et aux
palettes. Un `PPU` qui incrémente correctement `LY` et déclenche `VBlank`
au bon moment suffit déjà à :

- Permettre aux jeux de progresser au-delà des séquences de démarrage qui
  attendent VBlank avant de continuer.
- Donner à `halt_bug.gb` (Chapitre 5) et à `interrupt_time.gb` une
  véritable source d'interruption non liée au Timer à tester, au lieu des
  seules conditions synthétiques utilisées jusqu'ici.
- Fournir une base stable (`dots`, `lcd_y_coord`, la disposition des
  registres) sur laquelle les chapitres suivants construiront un véritable
  rendu, plutôt que de tout réécrire depuis zéro.

## Ce que nous avons maintenant

- Un `PPU` qui suit précisément les scanlines et les dots, correspondant
  au véritable modèle de timing de 70 224 dots par trame.
- Une interruption VBlank fonctionnelle, se déclenchant exactement une
  fois par trame au bon moment.
- Un masquage de lecture/écriture légèrement plus précis des registres
  `IE`/`IF`.

## Ce qui manque encore

- Pas encore de suivi du mode STAT (`0xFF41`) — le vrai hardware expose
  lequel des 4 modes du PPU (balayage OAM / dessin / HBlank / VBlank) est
  actuellement actif, et ceci n'est pas du tout modélisé pour l'instant.
- Aucun accès VRAM ou OAM depuis le PPU — il ne peut pas lire les données
  de tuiles, car il ne regarde en rien `vram`/`oam` (toujours détenus par
  `Bus`).
- **Pas de pixels, pas de tampon d'image (framebuffer), pas de fenêtre
  (window), pas de sprites.** Tout cela est délibérément reporté — la
  Partie IX revient une dernière fois sur le PPU à la fin de ce livre, et
  le véritable travail de rendu de pixels (le mécanisme de pixel FIFO/
  fetcher) fait l'objet du guide séparé
  [Guide de rendu de l'arrière-plan du PPU](../ppu-background.md).
