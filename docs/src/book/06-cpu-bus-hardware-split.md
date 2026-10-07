# 6. Séparer en CPU / Bus / Hardware

Après l'effort sur les opcodes (Partie II), le projet avait tout entassé
sous `src/cpu/` : le CPU lui-même, le bus, l'en-tête de la cartouche, le
port série, un timer, et même les débuts de l'audio. Cela cesse de passer
à l'échelle dès lors que davantage de composants hardware (manette
(joypad), PPU, davantage de canaux APU...) sont sur le point de
rejoindre le projet. Le commit de ce chapitre (`db7f453`) est une pure
réorganisation — presque aucun nouveau comportement, juste une forme bien
meilleure pour tout ce qui suit.

## La nouvelle disposition des modules

```text
src/
  cpu/        — the CPU itself (registers, opcodes, header parsing)
  bus/        — the memory bus, and a new IOBridge
  hardware/   — timer, serial, APU, (soon: joypad, PPU)
```

L'idée directrice : `cpu/` ne devrait connaître que l'*exécution des
instructions*. Tout ce qu'une instruction pourrait lire ou écrire —
mémoire, registres d'E/S, timers, son — appartient plutôt à `bus/` et
`hardware/`. C'est une forme très courante pour les émulateurs en
général : un module qui est « la chose qui exécute le code », et une
couche séparée qui modélise « tout ce que le code peut observer ou
affecter ».

## Un nouveau concept : `IOBridge`

Le Chapitre 4 a déjà introduit la carte mémoire et l'idée que
`0xFF00`–`0xFF7F` est un bloc de **registres d'E/S**, et non de la
mémoire réelle. Jusqu'à présent, `Bus` gérait directement quelques-unes
de ces adresses (juste le port série). Au fur et à mesure que davantage
de composants hardware ont besoin de leur propre tranche de cette plage
d'adresses, entasser toute leur logique directement dans `Bus::read`/
`Bus::write` rendrait `Bus` énorme et étroitement couplé à chaque
périphérique. Une nouvelle structure apparaît donc spécifiquement pour
posséder *seulement* cette région :

```rust
// src/bus/iobridge.rs
pub struct IOBridge {
    serial: Serial,
    timer: Timer,       // $FF04-$FF07 — Timer and divider
    interrupt_flag: u8, // $FF0F — IF: Interrupt flag
    audio: APU,         // $FF10-$FF26 — Audio
}

impl IOBridge {
    pub fn tick(&mut self, cycles: u8) {
        if self.timer.tick(cycles) {
            self.interrupt_flag |= 0b0000_0100;
        }
    }

    pub fn read(&self, addr: u16) -> u8 {
        match addr {
            0xFF01..=0xFF02 => self.serial.read(addr),
            0xFF04..=0xFF07 => self.timer.read(addr),
            0xFF0F => self.interrupt_flag,
            0xFF10..=0xFF26 => self.audio.read(addr),
            0xFF40 => 0x91, // LCDC — hardcoded placeholder for now
            0xFF41 => 0x85, // STAT — hardcoded placeholder for now
            0xFF44 => 0x00, // LY — hardcoded placeholder for now
            // ...
            _ => std::panic!("[IOREG] READ NOT IMPLEMENTED FOR ADDR: {:02X}", addr),
        }
    }

    pub fn write(&mut self, addr: u16, val: u8) {
        match addr {
            0xFF01..=0xFF02 => self.serial.write(addr, val),
            0xFF04..=0xFF07 => self.timer.write(addr, val),
            0xFF0F => self.interrupt_flag = val,
            0xFF10..=0xFF26 => self.audio.write(addr, val),
            0xFF40..=0xFF4B => {} // PPU registers — silently ignored for now
            _ => {}
        }
    }
}
```

Remarquez le schéma qui se dessine : `IOBridge` est lui-même un petit
« mini-bus » — il possède plusieurs périphériques hardware et route
chaque adresse d'E/S vers celui d'entre eux qui la possède réellement.
`Bus` (Chapitre 4) va déléguer toute la plage `0xFF00..=0xFF7F` à
`IOBridge::read`/`write`, de la même façon que le CPU délègue
`0x8000..=0x9FFF` à `Bus`. Ce schéma de « déléguer à un sous-composant
responsable d'une plage d'adresses » se répète à chaque niveau de cet
émulateur, jusqu'en bas.

## Valeurs de registres codées en dur : un bouchon (placeholder) honnête

Remarquez que `0xFF40 => 0x91` (LCDC) et `0xFF44 => 0x00` (LY) sont
simplement des constantes, non soutenues par un véritable état de PPU
pour l'instant — il n'existe aucune structure de PPU du tout à ce point
précis de l'historique. De nombreux jeux lisent ces registres juste pour
vérifier « l'écran est-il dans un état sûr pour mettre à jour les
graphismes », et une constante à l'allure plausible suffit souvent à
laisser la séquence de démarrage d'un jeu se poursuivre, bien avant
qu'une véritable puce graphique n'existe pour la soutenir. C'est une
technique générale utile lors de l'amorçage d'un émulateur : simuler
(faker) d'abord un registre avec une valeur fixe raisonnable, le
remplacer plus tard par la vraie chose (la Partie V entame ce
remplacement pour LCDC/STAT/LY).

## Ce que nous avons maintenant

- Trois préoccupations clairement séparées : `cpu/`, `bus/`, `hardware/`.
- `IOBridge`, une nouvelle couche de routage spécifiquement pour les
  registres d'E/S, déjà reliée au timer, au port série, et à la première
  ébauche d'APU.
- Des valeurs de registres liées au PPU en guise de bouchons, suffisantes
  pour maintenir les jeux en fonctionnement sans qu'un véritable PPU
  n'existe encore.

## Ce qui manque encore

- Pas de `Joypad`, pas encore de véritable structure `PPU` — ceux-ci
  arrivent au chapitre suivant.
- `IOBridge::tick` ne fait avancer que le timer pour l'instant ; rien
  d'autre n'avance (tick) en parallèle du CPU pour le moment.
- Les lectures non mappées déclenchent encore carrément un `panic!` au
  lieu de renvoyer une valeur par défaut sûre — acceptable pour
  l'instant, puisque chaque ROM contre laquelle nous testons se comporte
  bien, mais à garder en tête comme une aspérité à corriger.
