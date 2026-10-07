# 16. Canal 1 — Onde carrée avec sweep

Le canal 1 est le premier canal à recevoir un comportement réel (`a8e1bcf`, `be5f728`,
`ff84e29`, `36dd81a`), et c'est le plus complet des deux canaux
d'onde carrée : en plus du duty cycle et de l'enveloppe (envelope) que possède chaque canal
carré, le canal 1 est le seul à supporter aussi le **sweep de fréquence** — un
glissement de hauteur fluide, le classique effet sonore laser « pew! ».

## La structure partagée `Channel`

```rust
// src/audio/channel.rs
#[derive(Default)]
pub struct Channel {
    pub duty_cycle: u8,
    pub duty_pos: u8,
    pub length_timer: u8,
    pub initial_length_timer: u8,
    pub length_enabled: bool,
    pub env_timer: u8,
    pub env_dir: u8,  // 0 = down, 1 = up
    pub env_pace: u8, // 0..7, envelope speed
    pub period: u16,
    pub volume: u8,
    pub initial_volume: u8,
    pub freq_timer: u32,
    pub enabled: bool,
    // sweep (channel 1 only — channel 2 just never uses these fields)
    pub sweep_pace: u8,
    pub sweep_substraction: bool,
    pub sweep_step: u8,
    pub sweep_timer: u8,
    pub sweep_enabled: bool,
    pub sweep_shadow_period: u16,
    pub sweep_negate_used: bool,
}
```

Réutiliser une seule structure pour les deux canaux carrés (le canal 2 ne
déclenche simplement jamais les champs liés au sweep) évite de dupliquer la
logique du duty cycle et de l'enveloppe — un compromis raisonnable entre « quelques
champs inutilisés sur le canal 2 » et « deux structures quasi identiques à maintenir
synchronisées. »

## Parser les registres NRx

Chaque canal est contrôlé par une petite poignée de registres dédiés
(nommés `NR1x` pour le canal 1, `NR2x` pour le canal 2, etc. dans la
documentation Game Boy). Les parser revient surtout à extraire des champs de bits,
la même compétence que pour le décodage des opcodes préfixés `0xCB` du chapitre 4 :

```rust
pub fn write_nr1(&mut self, val: u8) {
    self.duty_cycle = (val & 0b1100_0000) >> 6;
    self.initial_length_timer = 64 - (val & 0b0011_1111);
    self.length_timer = self.initial_length_timer;
}

pub fn write_nr2(&mut self, val: u8) {
    self.initial_volume = (val & 0b1111_0000) >> 4;
    self.env_dir = (val & 0b0000_1000) >> 3;
    self.env_pace = val & 0b0000_0111;
}

pub fn write_sweep(&mut self, val: u8) {
    self.sweep_pace = (val & 0b0111_0000) >> 4;
    let old_sub = self.sweep_substraction;
    self.sweep_substraction = val & 0b0000_1000 != 0;
    self.sweep_step = val & 0b0000_0111;

    // CH1 quirk: leaving "subtract" mode right after a subtract-mode
    // calculation was used silently disables the whole channel.
    if old_sub && !self.sweep_substraction && self.sweep_negate_used {
        self.enabled = false;
        self.sweep_enabled = false;
    }
}
```

Cette dernière bizarrerie dans `write_sweep` est un bon exemple de quelque chose
que l'on rencontre constamment en écrivant un émulateur fidèle : des cas limites
(edge cases) matériels documentés et délibérés qui ressemblent à des bugs, vérifiés
par rapport à des ROMs de test (chapitre 20) plutôt que dérivés de premiers principes.
Personne ne conçoit volontairement, d'un point de vue produit, une fonctionnalité où
« changer un réglage pour revenir à l'état précédent peut couper silencieusement tout le
canal » — mais c'est exactement ce que fait le vrai silicium, les jeux peuvent s'appuyer
dessus (intentionnellement ou non), et un émulateur précis doit le reproduire.

## Sweep : calculer la prochaine fréquence

```rust
pub fn sweep_next_period_and_overflow(&self) -> (u16, bool) {
    let delta = self.sweep_shadow_period >> self.sweep_step;
    if self.sweep_substraction {
        (self.sweep_shadow_period.saturating_sub(delta), false)
    } else {
        let next = self.sweep_shadow_period as u32 + delta as u32;
        (next as u16, next > 0x7FF)
    }
}
```

À chaque étape de sweep (cadencée par le séquenceur de trame (frame) du chapitre 15, à 128
Hz), la fréquence du canal (« period », ici) monte ou descend d'une
fraction de sa valeur actuelle (`>> sweep_step` — une valeur de décalage plus grande
signifie un changement fractionnel plus petit, donc un sweep plus lent). `0x7FF`
(2047) est la plus grande valeur que le registre de période peut contenir ; dépasser
cette valeur par sweep signifie que la fréquence est sortie de la plage représentable, ce qui
désactive silencieusement le canal (`calculate_new_period(sweep) > 2047` dans
les notes de conception initiales de `APU.MD.md`, implémenté ici comme le
booléen `overflow` que cette fonction renvoie).

## Ce que nous avons maintenant

- Le parsing des registres du canal 1 : duty cycle, longueur, enveloppe et sweep.
- Un véritable calcul de sweep, incluant les bizarreries « dépassement désactive le
  canal » et « changement de mode désactive le canal ».
- La structure `Channel` partagée, prête à être réutilisée telle quelle pour le canal 2.

## Ce qu'il manque encore

- Le canal 2 lui-même n'est pas encore câblé (prochain chapitre — il réutilise
  tout ce qui a été construit ici).
- Aucune vérification audible pour l'instant au-delà de l'écoute informelle (`be5f728`,
  « BIP sound », était littéralement le premier bip audible) — la validation
  formelle par ROMs de test fait l'objet du chapitre 20.
