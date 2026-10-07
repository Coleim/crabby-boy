# 4. Étoffer le jeu d'instructions

Ce chapitre couvre la partie la plus longue et la plus répétitive de tout
le projet : transformer une poignée d'opcodes en (presque) la totalité
des 512 qui existent (256 opcodes « principaux », plus 256 autres
derrière un préfixe spécial `0xCB`). Il s'étend sur une longue séquence
de commits (`ab86d28` → `dc9a258`). Plutôt que de parcourir chaque opcode
un par un (c'est à cela que servent les
[tables d'opcodes](https://gbdev.io/gb-opcodes/optables/)), ce chapitre
se concentre sur la poignée d'*idées* qui, une fois comprises, rendent
chaque opcode individuel simple à aborder.

## Une première réorganisation à mi-parcours : naissance du `Bus`

En plein ajout d'opcodes, les instructions ont commencé à avoir besoin de
vraies régions mémoire (VRAM, WRAM, OAM, HRAM) au lieu d'un seul tableau
plat, donc une structure `Bus` apparaît (`a6f2e4e`), déjà annotée avec la
carte mémoire complète du Chapitre 0 :

```rust
// src/cpu/bus.rs
pub struct Bus {
    rom: Vec<u8>,
    vram: [u8; 0x2000],
    wram: [u8; 0x2000],
    oam: [u8; 0x2000],
    serial: Serial,
    hram: [u8; 0x2000],
    ie: u8,
}

impl Bus {
    pub fn read(&self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x3FFF => self.rom[addr as usize],
            0x4000..=0x7FFF => self.rom[addr as usize], // no banking yet
            0x8000..=0x9FFF => self.vram[(addr - 0x8000) as usize],
            0xC000..=0xDFFF => self.wram[(addr - 0xC000) as usize],
            0xFE00..=0xFE9F => self.oam[(addr - 0xFE00) as usize],
            0xFF01..=0xFF02 => self.serial.read(addr),
            0xFF80..=0xFFFE => self.hram[(addr - 0xFF80) as usize],
            0xFFFF => self.ie,
            _ => { println!("[BUS] Not mapped addressed {}", addr); 0 }
        }
    }
    // ...
}
```

Le `match` de Rust sur une plage (`0x8000..=0x9FFF => ...`) fait
exactement le travail de la table de carte mémoire du Chapitre 0 — chaque
branche est une ligne de cette table. À partir d'ici, `CPU::execute`
prend un `&mut Bus` au lieu d'une tranche brute `&mut [u8]`, et chaque
accès mémoire passe par `bus.read`/`bus.write` plutôt que par l'indexation
directe d'un tableau. Ce changement unique est ce qui rend possibles les
chapitres suivants (le banking de ROM, les registres d'E/S, la VRAM,
l'OAM ont tous besoin d'un comportement *différent* selon la plage
d'adresses, pas simplement d'un tableau de stockage différent).

## Idée 1 : les paires de registres ne sont que deux registres collés ensemble

Plusieurs instructions traitent `B`+`C`, `D`+`E`, ou `H`+`L` comme une
seule valeur 16 bits (par exemple `LD BC, nn`, `INC HL`). Il n'y a aucun
nouveau concept hardware ici — c'est exactement le même tour de
combinaison little-endian que celui de `read16bytes` au Chapitre 3,
simplement exposé sous forme de paires de getter/setter pratiques :

```rust
fn get_bc(&self) -> u16 {
    (self.b as u16) << 8 | self.c as u16
}
fn set_bc(&mut self, value: u16) {
    self.b = (value >> 8) as u8;
    self.c = value as u8; // truncates to the low byte
}
```

## Idée 2 : les flags, accessibles via des aides nommées plutôt que des opérations binaires brutes

Écrire `self.f |= 0x80` partout (Chapitre 3) devient vite source
d'erreurs. À ce stade de l'avancement, l'accès aux flags devient de
petites méthodes d'aide — `get_z`/`set_z`, `get_n`/`set_n`, `get_h`/
`set_h`, `get_c`/`set_c` — afin que le corps des instructions se lise
comme la spécification, et non comme de la manipulation de bits :

```rust
self.set_h(false);
self.set_z(self.a == 0);
```

Si vous implémentez votre propre CPU, écrivez ces aides *avant* d'en
avoir besoin à la vingtième instruction — cela se rentabilise presque
immédiatement.

## Idée 3 : le préfixe `0xCB` est une seconde table d'opcodes

Si l'octet à `PC` est `0xCB`, cet octet ne représente pas une instruction
à lui seul — il signifie « l'octet *suivant* sélectionne dans une table
complètement séparée de 256 instructions de manipulation de bits »
(rotations, décalages, et opérations de test/positionnement/effacement
bit à bit). Plutôt que d'écrire 256 branches `match` de plus à la main,
l'octet préfixé par CB est en réalité décodé en le divisant en 3 champs
de bits, car les concepteurs de ce CPU ont délibérément disposé l'octet
d'opcode de cette façon :

```rust
let category: u8 = opcode >> 6;              // top 2 bits:   which family (rotate/shift, BIT, RES, SET)
let subcategory: u8 = opcode >> 3 & 0b0000_0111; // middle 3 bits: which operation, or which bit number for BIT/RES/SET
let operand: u8 = opcode & 0b0000_0111;         // bottom 3 bits: which register (or (HL))

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
    1 => self.bit(subcategory, operand, bus), // BIT b, r
    2 => { /* RES b, r */ }
    3 => { /* SET b, r */ }
    _ => {}
}
```

C'est une bonne leçon générale, pas seulement propre à la Game Boy :
quand la disposition binaire d'une spécification semble étrangement bien
ordonnée (exactement 2 + 3 + 3 bits, correspondant à « 8 opérations × 8
registres », ou « 4 catégories × 8 numéros de bits × 8 registres »),
c'est presque toujours intentionnel, et décoder par décalages et masques
est préférable à écrire les 256 cas à la main.

## Idée 4 : `DAA` — l'ajustement décimal, une bizarrerie de la Game Boy à voir au moins une fois

La plupart des instructions sont intuitives une fois que l'on connaît les
concepts du CPU. `DAA` (« Decimal Adjust Accumulator ») est la seule
instruction de tout le jeu qui ressemble à de la magie noire la première
fois qu'on la lit :

```rust
0x27 => {
    // DAA — see https://rgbds.gbdev.io/docs/v1.0.0/gbz80.7#DAA
    let mut adjustment = 0;
    if self.get_n() {
        if self.get_h() { adjustment += 0x06; }
        if self.get_c() { adjustment += 0x60; }
        self.a = self.a.wrapping_sub(adjustment);
    } else {
        if self.get_h() || (self.a & 0xF) > 0x9 { adjustment += 0x06; }
        if self.get_c() || self.a > 0x99 {
            adjustment += 0x60;
            self.set_c(true);
        }
        self.a = self.a.wrapping_add(adjustment);
    }
    self.set_h(false);
    self.set_z(self.a == 0);
}
```

Le contexte qui donne du sens à tout ça : `DAA` existe pour faire en
sorte que l'addition binaire *se comporte comme* une addition décimale,
pour les programmes qui stockent des nombres en « BCD compacté » (binaire
codé décimal — chaque nibble d'un octet représente un chiffre décimal,
0-9, au lieu que l'octet représente un seul nombre binaire combiné
0-255). Après un `ADD`/`SUB` sur des valeurs encodées en BCD, `DAA`
corrige le résultat pour que chaque nibble revienne dans la plage valide
0-9, en utilisant les flags `H` et `C` positionnés par l'instruction
*précédente* pour savoir si un nibble ou un octet a « débordé ». Vous
n'aurez presque certainement jamais besoin de comprendre cela plus en
profondeur que « copier l'algorithme correctement, le tester avec la ROM
de test de Blargg, passer à autre chose » — ce qui est une approche
parfaitement valable pour des détails aussi pointus du hardware.

## Idée 5 : vous sous-implémenterez des opcodes d'abord, et c'est normal

Tout au long de cet effort, la branche générique `_ => {}` (vue depuis le
Chapitre 3) reste en place, et des opcodes individuels sont commentés
puis réintégrés au fur et à mesure que des bugs sont trouvés :

```rust
// 0x38 => {
//     println!("SRL B 2  8 Z 0 0 C")
// }
_ => {
    println!("Unimplemented opcode: 0xCB{:02X} at PC: 0x{:04X}", opcode, current_pc);
}
```

C'est une façon tout à fait normale de construire un cœur de CPU :
implémenter une instruction, la tester, découvrir qu'elle était
subtilement incorrecte (mauvais flag, mauvais nombre de cycles, oubli
d'avancer `PC`), la commenter pendant que vous corrigez la fonction
d'aide sous-jacente, puis la réactiver. Ne visez pas une passe unique
parfaite sur les 256+256 opcodes — visez une boucle d'implémenter →
tester → corriger.

## Comment savoir si c'est correct ? Le `cpu_instrs.gb` de Blargg

Tous les commits de ce chapitre sont validés de la même façon : en
exécutant une ROM de test communautaire bien connue,
[`cpu_instrs.gb` de Blargg](https://github.com/retrio/gb-test-roms)
(déjà présente dans `tests/cpu_instrs/`, divisée en 11 sous-tests :
`01-special.gb`, `02-interrupts.gb`, et ainsi de suite). Cette ROM
exerce les instructions du CPU et rapporte **PASS** ou **FAIL** sous
forme de texte, en écrivant des caractères via le **port série** de la
Game Boy — un simple lien de communication qui, sur le hardware réel,
aurait servi à parler à un câble reliant deux consoles, et que les
auteurs de ROMs de test ont depuis longtemps détourné comme moyen
pratique d'imprimer du texte de débogage depuis une ROM en cours
d'exécution sans aucun écran requis. Nous capturons cette sortie série et
l'affichons, bien avant d'avoir un écran pour afficher un vrai message
PASS/FAIL. Le Chapitre 11 transforme cela en une véritable suite de tests
automatisée.

## Ce que nous avons maintenant

- Une structure `Bus`, qui route les lectures vers la bonne région
  mémoire.
- Des paires de registres (`BC`/`DE`/`HL`) et des méthodes d'aide pour les
  flags.
- Presque toute la table d'opcodes principale, plus la table préfixée par
  `0xCB` via un décodage par champs de bits.
- Un moyen de valider la justesse par rapport à une véritable ROM de test
  communautaire via la sortie série.

## Ce qui manque encore

- Les catégories `0xCB` `RES`/`SET` sont des « stubs » (des coquilles
  vides) (voir les branches vides `2 => {}` / `3 => {}` ci-dessus) à ce
  point précis de l'historique — elles sont remplies peu après.
- `HALT` existe comme opcode nommé mais son fameux bug hardware n'est
  pas encore géré — c'est le Chapitre 5, juste après.
- Pas encore d'interruptions, même si `02-interrupts.gb` est déjà présent
  dans le dossier de test, en attente.
- Toujours pas de registres d'E/S, de timer, ni de PPU — le CPU peut
  calculer, mais il n'y a encore rien de significatif pour lui à
  contrôler.
