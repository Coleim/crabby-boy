# 18. Canal 3 — Onde personnalisée

Le canal 3 (`16b3864`) est le premier canal qui a réellement besoin de sa propre
structure : au lieu d'un duty cycle fixe à 4 formes (chapitre 16), il relit en
boucle une forme d'onde (wave) arbitraire à 32 échantillons fournie par le jeu, à une
hauteur et un volume contrôlables.

## `WaveChannel`

```rust
// src/audio/wave_channel.rs
#[derive(Default)]
pub struct WaveChannel {
    pub dac_enabled: bool,
    pub enabled: bool,
    pub initial_len_timer: u16,
    pub len_timer: u16,
    pub length_enabled: bool,
    pub sample_countdown: u16,
    pub volume_level: u8,
    pub period: u16,
    pub wave_ram: [u8; 16],
    pub wave_index: u8, // 0..31 (32 nibbles total)
    pub sample_buffer: u8,
    pub divider_phase: bool,
    pub wave_form_just_read: bool,
    pub wave_access_window: u8,
}
```

## Wave RAM : 16 octets contenant 32 échantillons

`wave_ram` fait 16 octets, mais la forme d'onde elle-même a 32 échantillons
(`wave_index` va de 0 à 31) — chaque octet compresse **deux échantillons de 4 bits**
(un « nibble » chacun), la même idée de « compresser deux petites valeurs dans un
octet » que vous avez déjà rencontrée plusieurs fois (les flags sur 2 bits dans `F`,
chapitre 3 ; les indices de couleur sur 2 bits dans les tuiles VRAM, dans le guide PPU
associé). Un jeu écrit la forme d'onde à 32 échantillons de son choix dans cette
région (`0xFF30`-`0xFF3F` dans la carte mémoire réelle) avant de lire le canal 3 —
c'est ce qui fait que le canal 3 sonne différemment d'un jeu à l'autre, contrairement
aux canaux 1/2/4 dont le caractère sonore est fixé par le matériel.

## Un drapeau d'activation/désactivation dédié : le DAC

```rust
pub fn write_nr0(&mut self, val: u8) {
    self.dac_enabled = val & 0b1000_0000 != 0;
    if !self.dac_enabled {
        self.enabled = false;
    }
}
```

Cela introduit une distinction qu'il vaut la peine de comprendre une bonne fois pour
toutes, car elle revient pour chaque canal (à nouveau perceptible dans le canal de bruit
(noise) du chapitre 19) : un canal a à la fois un drapeau **enabled** (est-il actuellement
en train de jouer activement, par exemple son compteur de longueur n'est-il pas encore
écoulé) et un concept distinct de **DAC enabled** (son convertisseur numérique-analogique
interne est-il même allumé). Éteindre le DAC coupe immédiatement le son du canal
indépendamment de tout le reste — c'est un interrupteur plus fondamental que le
compteur de longueur ou l'enveloppe.

## Disposition des registres : la période répartie sur deux registres

```rust
pub fn write_nr3(&mut self, val: u8) {
    self.period = (self.period & 0b111_0000_0000) | val as u16;
}

pub fn write_nr4(&mut self, val: u8, length_clock_on_write: bool) {
    // ...
    self.period = (self.period & 0b000_1111_1111) | ((val as u16 & 0b111) << 8);
    self.length_enabled = val & 0b0100_0000 != 0;
    // ...
}
```

La hauteur du canal (« period ») est une valeur de 11 bits, trop large pour un
registre de 8 bits, donc elle est séparée : `NR3` contient les 8 bits de poids faible,
`NR4` les 3 bits de poids fort (plus des flags sans rapport comme `length_enabled`
compressés dans ses autres bits). Chaque écriture combine le nouvel octet avec un
masque préservant l'*autre* moitié de la période que ce registre particulier ne
possède pas (`self.period & 0b111_0000_0000` conserve les bits de poids fort inchangés
tout en remplaçant ceux de poids faible, et vice-versa). Ce schéma « répartir une
valeur logique sur deux registres adressables, avec des masques pour éviter d'écraser
l'autre moitié » est courant parmi les registres d'E/S de la Game Boy — vous voudrez
le reconnaître au premier coup d'œil.

## La bizarrerie du compteur de longueur du DMG, écrite en code

```rust
// DMG quirk: enabling length can immediately clock it depending on frame
// phase.
if !was_length_enabled && self.length_enabled && length_clock_on_write && self.len_timer > 0 {
    self.len_timer = self.len_timer.saturating_sub(1);
    // ...
}
```

Un autre exemple du thème « ça ressemble à un bug, mais c'est en réalité un
comportement matériel documenté » du chapitre 16 : sur le matériel DMG original,
activer le bit d'activation de longueur à exactement le mauvais moment dans le
cycle du séquenceur de trame provoque une décrémentation immédiate et
supplémentaire du compteur de longueur, en conséquence de la manière dont
l'horloge du compteur de longueur et la vérification du bit d'activation
interagissent dans le circuit réel. `length_clock_on_write` est la façon dont
l'APU communique « sommes-nous actuellement à l'un de ces moments précis » à
cette méthode.

## Ce que nous avons maintenant

- Le canal 3 relisant une forme d'onde arbitraire fournie par le jeu, à une
  hauteur/volume contrôlables.
- Une compréhension plus claire de DAC activé vs. canal activé comme deux
  concepts distincts.
- Les schémas de registre réparti et de compteur de longueur capricieux, qui
  réapparaissent tous deux dans le canal 4.

## Ce qu'il manque encore

- Le canal 4 (bruit) reste à venir, au prochain chapitre.
- Rien ici n'est encore vérifié par rapport à la suite de tests `dmg_sound` de
  Blargg — ce sera le chapitre 20, une fois que les 4 canaux existeront.
