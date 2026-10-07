# 19. Canal 4 — Bruit

Le canal 4 (`9e6164c`) est le cas particulier : il n'a ni hauteur, ni
forme d'onde, ni duty cycle. Il produit un bruit (noise) de type bourdonnement/sifflement, en
basculant rapidement entre deux niveaux de volume selon un motif qui *semble*
aléatoire mais est en réalité totalement déterministe — généré par quelque chose
appelé un **registre à décalage à rétroaction linéaire (Linear Feedback Shift
Register, LFSR)**.

## Un registre de 15 bits qui « se décale » vers un pseudo-hasard

```rust
// src/audio/noise_channel.rs
#[derive(Default)]
pub struct NoiseChannel {
    pub enabled: bool,
    pub freq_timer: u32,
    // ... length/envelope fields, same shape as Channel (Chapter 16)
    pub clock_shift: u8,
    pub clock_divider: u8,
    pub short_mode: bool,
    pub lfsr: u16,
}
```

Un LFSR est simplement un nombre (`lfsr`, ici large de 16 bits bien que seuls 15
soient réellement utilisés) qui, de temps en temps, calcule un nouveau bit à partir
de ses bits *actuels*, décale tout d'une position, et insère ce nouveau bit en
haut. Répéter cette opération produit une longue séquence de bits qui réussit les
tests sommaires d'aléatoire (elle ne se répète pas de façon évidente avant très
longtemps), tout en restant 100 % reproductible étant donné la même valeur de départ
et la même séquence d'opérations — exactement ce que l'on veut dans un émulateur
déterministe : le canal de bruit « aléatoire » doit quand même sonner bit pour bit
identique au matériel réel à chaque fois, y compris exactement la même séquence
« aléatoire », puisque tout émulateur précis (et le matériel réel !) n'est que
pseudo-aléatoire.

## L'étape réelle de décalage et de rétroaction

```rust
pub fn tick(&mut self) {
    self.freq_timer = self.freq_timer.saturating_sub(1);
    if self.freq_timer == 0 {
        let divider = if self.clock_divider == 0 { 8 } else { self.clock_divider as u32 * 16 };
        self.freq_timer = divider << self.clock_shift;

        // bit 0 XNOR bit 1 (1 if they're equal, 0 otherwise) feeds back into bit 15
        let bit0 = self.lfsr & 0x01;
        let bit1 = (self.lfsr >> 1) & 0x01;
        let res = if bit0 == bit1 { 1 } else { 0 };
        self.lfsr = (self.lfsr & 0b0111_1111_1111_1111) | (res << 15);

        // "short mode": also copy that same feedback bit into bit 7
        if self.short_mode {
            self.lfsr = (self.lfsr & 0b1111_1111_0111_1111) | (res << 7);
        }

        // shift everything right by one; the new bit 0 selects the output
        self.lfsr = self.lfsr >> 1;
    }
}
```

Étape par étape : prendre les deux bits de poids faible de la valeur `lfsr`
actuelle, les comparer (`1` s'ils sont égaux, `0` s'ils diffèrent — cette
comparaison s'appelle XNOR), et réinjecter ce bit de résultat unique en *haut*
du registre (bit 15), avant de décaler tout l'ensemble d'une position vers la
droite. La sortie audio réelle du canal, à tout instant, est simplement « le
bit 0 de `lfsr` est-il à 1 » — un seul bit qui bascule rapidement et de façon
pseudo-aléatoire, ce qui est exactement à quoi ressemble du bruit.

Le « mode court » (bit 3 de `NR43`) copie en plus ce même bit de rétroaction
dans le bit 7, ce qui raccourcit la longueur de cycle effective du LFSR,
produisant une texture sensiblement différente, plus « métallique » que le
« mode long » par défaut — un véritable exemple d'un seul bit de configuration
changeant significativement le timbre d'un canal, et pas seulement sa hauteur
ou son volume.

## Pourquoi le calcul de fréquence diffère des canaux 1 à 3

```rust
let divider = if self.clock_divider == 0 { 8 } else { self.clock_divider as u32 * 16 };
self.freq_timer = divider << self.clock_shift;
```

Les canaux carrés et à onde (chapitres 16-18) tirent leur hauteur d'une seule
valeur « period » linéaire. Le bruit, lui, dérive son taux de cadence à partir
de deux champs distincts multipliés entre eux d'une manière un peu inhabituelle :
une petite règle de type table de diviseurs de base (`clock_divider == 0` est
traité comme un cas spécial signifiant 8, sinon c'est `clock_divider * 16`),
puis décalée vers la gauche de `clock_shift`. Ce n'est pas quelque chose que
l'on dérive de premiers principes — c'est un format de registre documenté que
l'on implémente et vérifie, dans le même esprit que `DAA` au chapitre 4.

## Ce que nous avons maintenant

- Les 4 canaux de l'APU existent désormais : deux ondes carrées (1 avec sweep, 2
  sans), une forme d'onde personnalisée, et un générateur de bruit basé sur LFSR.
- Un générateur de bruit pseudo-aléatoire déterministe et fonctionnel, reproduisant
  bit pour bit le comportement exact du matériel réel.

## Ce qu'il manque encore

- Aucun des 4 canaux n'a encore été validé par rapport aux ROMs de test audio
  dédiées de Blargg — ce sera le tout prochain chapitre, et c'est là qu'un
  certain nombre des « bizarreries » des chapitres 16-19 sont réellement
  détectées et corrigées, pas seulement implémentées de façon spéculative.
