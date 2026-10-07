# 9. Timers

Le timer est le premier périphérique matériel de ce livre qui doit suivre
le temps *indépendamment* de l'instruction en cours d'exécution — il doit
continuer d'avancer exactement de la bonne quantité, quel que soit
l'opcode qui vient de s'exécuter. Ce chapitre (`d51d64e`) introduit
également la boucle principale de l'émulateur qui relie pour la première
fois l'exécution du CPU et l'avancement (ticking) du hardware.

## Quatre registres, un compteur interne

```rust
// src/hardware/timer.rs
pub struct Timer {
    internal_div: u16, // the real 16-bit counter backing DIV
    tima: u8,          // FF05 — TIMA: Timer counter
    tma: u8,           // FF06 — TMA: Timer modulo
    tac: u8,           // FF07 — TAC: Timer control
}
```

- **DIV** (`0xFF04`) — un compteur à défilement libre, toujours en train
  d'avancer, utilisé par les jeux pour des choses comme la génération de
  nombres aléatoires. Sa lecture renvoie l'*octet de poids fort* d'un
  compteur interne 16 bits plus large (`internal_div`) ; y écrire
  n'importe quelle valeur réinitialise ce compteur interne à zéro.
- **TIMA** (`0xFF05`) — un compteur qui s'incrémente à une fréquence
  configurable, et déclenche l'interruption (interrupt) du timer lors d'un
  débordement (overflow) au-delà de `0xFF`.
- **TMA** (`0xFF06`) — la valeur avec laquelle `TIMA` est rechargé après le
  débordement (pas nécessairement 0 — les jeux l'utilisent pour contrôler
  précisément la fréquence de déclenchement de l'interruption).
- **TAC** (`0xFF07`) — contrôle du timer : un bit pour activer/désactiver
  `TIMA` entièrement, plus 2 bits sélectionnant *à quelle vitesse* il
  avance.

## Pourquoi `internal_div` est en 16 bits, pas 8

```rust
pub fn read(&self, addr: u16) -> u8 {
    match addr {
        0xFF04 => (self.internal_div >> 8) as u8,
        // ...
    }
}
```

`DIV` est un registre 8 bits du point de vue du jeu, mais le vrai hardware
pilote en réalité à la fois `DIV` *et* la fréquence variable de `TIMA` à
partir d'un seul compteur 16 bits partagé, plus large, qui fonctionne en
arrière-plan. Cela compte pour l'astuce ingénieuse de la section suivante.

## Avancer cycle par cycle, avec détection de front descendant

```rust
pub fn tick(&mut self, cycles: u8) -> bool {
    for _ in 0..cycles {
        let before = self.internal_div;
        self.internal_div = self.internal_div.wrapping_add(1);

        if self.tac & 0b0000_0100 != 0 { // timer enabled?
            let clock_select = self.tac & 0b0000_0011;
            let bit = match clock_select {
                0 => 9,
                1 => 3,
                2 => 5,
                3 => 7,
                _ => unreachable!(),
            };

            let was_set = (before >> bit) & 1 == 1;
            let is_set = (self.internal_div >> bit) & 1 == 1;

            if was_set && !is_set {
                if (self.tima as u16).wrapping_add(1) > 0xFF {
                    self.tima = self.tma; // overflow: reload from TMA
                    return true;          // signal: fire the Timer interrupt
                } else {
                    self.tima = self.tima.wrapping_add(1);
                }
            }
        }
    }
    false
}
```

C'est une technique élégante (et très fidèle au vrai hardware) qui mérite
qu'on s'y attarde. Plutôt qu'un compteur séparé pour la vitesse
configurable de `TIMA`, l'implémentation surveille **un bit précis du
compteur 16 bits partagé**, et n'incrémente `TIMA` que sur le **front
descendant (falling edge)** de ce bit — le moment exact où il passe de `1`
à `0` (`was_set && !is_set`). Puisque le bit `N` d'un compteur binaire
bascule à une fréquence prévisible et fixe à mesure que le compteur entier
s'incrémente, choisir quel bit surveiller revient à choisir la fréquence
d'avancement de `TIMA` — ce qui est exactement ce que contrôle le champ 2
bits `clock_select` de `TAC`. C'est exactement l'idée de front descendant
sur un bit de compteur que le vrai hardware Game Boy utilise lui-même en
interne, pas seulement un choix d'implémentation coïncidant avec cela.

La fonction avance **un cycle entier à la fois**, dans une boucle, plutôt
que de calculer directement « combien de fois `TIMA` s'incrémenterait sur
N cycles » en une seule opération — simple à écrire correctement, au prix
d'être plus lent qu'une approche groupée (batched). Ce compromis
(simplicité maintenant, optimisation éventuelle plus tard si cela compte)
est un thème que vous verrez revenir régulièrement dans ce projet.

## La boucle principale apparaît : `emulator.rs`

Jusqu'à présent, `main.rs` était un harnais rapide, jetable. Ce commit
introduit une vraie structure, `CrabbyBoy`, avec une méthode `run` qui est
la véritable boucle fetch-decode-execute-tick qui s'exécute pour le reste
du livre :

```rust
// src/emulator.rs
loop {
    if cpu.stopped {
        println!("CPU STOPPED. Waiting interrupts");
        break;
    }
    if cpu.halt {
        bus.tick(4);
        let ie = bus.get_ie();
        let if_flag = bus.get_io().get_if();
        if (ie & if_flag) != 0 {
            cpu.halt = false;
        }
        continue;
    }

    match cpu.execute(&mut bus) {
        Some(tick) => {
            bus.tick(tick);
            cpu.handle_interrupts(&mut bus);
        }
        None => panic!("Error in getting the cycles"),
    }
}
```

Deux changements méritent d'être soulignés par rapport aux chapitres
précédents :

- `CPU::execute` retourne désormais `Option<u8>` (combien de cycles
  l'instruction a pris), pas un simple `bool`. C'est ce qui permet enfin
  à `bus.tick(tick)` de faire avancer le timer (et, plus tard, chaque
  autre périphérique matériel) exactement de la bonne quantité après
  chaque instruction — reliant pour la première fois dans ce projet
  l'exécution du CPU et le timing du hardware.
- Même en état d'attente (halt), `bus.tick(4)` continue de s'exécuter à
  chaque itération de la boucle — car le vrai hardware ne gèle pas le
  timer/PPU/APU simplement parce que le CPU lui-même est en halt ; seule
  l'exécution des instructions du CPU est en pause.

## Les tests passent en vrais tests Rust

```rust
#[cfg(test)]
macro_rules! cpu_instr_test {
    ($name: ident, $path: expr) => {
        #[test]
        fn $name() {
            let mut crabby = CrabbyBoy::new();
            assert_eq!(crabby.run($path), Ok(()));
        }
    };
}
```

Au lieu de modifier manuellement `main.rs` pour pointer vers une autre ROM
de test et d'examiner la sortie imprimée à l'œil nu (ce que nous faisions
depuis le Chapitre 4), les ROMs de test s'exécutent désormais comme de
vrais tests `cargo test` automatisés, chacun n'étant qu'une invocation de
macro d'une seule ligne nommant un fichier de ROM. `CrabbyBoy::run`
lui-même détecte le texte « Passed »/« Failed » dans la sortie série
(Chapitre 4) et le transforme en un vrai `Result`. C'est un bond
significatif en confort d'utilisation, et c'est la fondation sur laquelle
le Chapitre 11 construit un pipeline complet d'intégration continue (CI).

## Ce que nous avons maintenant

- Un vrai `Timer` avec un comportement `DIV`/`TIMA`/`TMA`/`TAC` précis au
  front descendant, y compris le déclenchement de l'interruption du timer
  au débordement.
- Une vraie boucle principale (`CrabbyBoy::run`) qui fait avancer le
  hardware après chaque instruction et gère correctement le réveil de
  `HALT` sur une interruption en attente.
- Des fonctions `#[test]` automatisées pour les ROMs de test, remplaçant
  la modification manuelle de `main.rs`.
- De nouvelles ROMs de test pour le timing des instructions et des accès
  mémoire, utilisées au prochain chapitre.

## Ce qui manque encore

- `cpu.handle_interrupts` est appelé ici mais le véritable *dispatch*
  d'interruption (sauter au bon vecteur, empiler `PC`, effacer `IME`)
  n'est correctement couvert qu'au Chapitre 14 — ce chapitre se concentre
  sur le timer lui-même et la structure de boucle qui l'entoure.
- La plupart des invocations de `cpu_instr_test!` sont encore mises en
  commentaire à ce stade (`read_timing` est la seule active) — la
  correction du timing des accès mémoire (pourquoi cela compte, et comment
  le corriger) fait l'objet du Chapitre 10.
