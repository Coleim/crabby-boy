# 10. Correction du timing mémoire

Réussir `cpu_instrs.gb` (Chapitre 4) prouve que vos instructions
produisent les *bons résultats*. Cela ne prouve **pas** qu'elles prennent
le *bon temps*, ni que la mémoire est touchée aux *bons moments* au cours
de chaque instruction. Ce chapitre (`ca64263` → `6e90bef`) porte sur ce
second type de correction, bien plus subtil — et sur une nouvelle
catégorie de ROM de test qui cible spécifiquement cela.

## Pourquoi « avancer une fois par instruction » n'est pas assez précis

La boucle principale du Chapitre 9 appelait `bus.tick(cycles)` une seule
fois par instruction, après que `cpu.execute` ait terminé. Sur le vrai
hardware, cependant, une seule instruction n'est pas un événement
atomique du point de vue du reste du système — elle est composée de
plusieurs accès mémoire distincts (lire l'opcode, éventuellement lire un
octet d'opérande, éventuellement lire/écrire une adresse mémoire), chacun
prenant sa propre tranche de temps, le timer/PPU/APU continuant tous
d'avancer (tick) *entre* ces accès individuels, pas seulement une fois à
la toute fin. Une instruction qui lit la mémoire deux fois et écrit une
fois devrait laisser le reste du hardware avancer en 3 étapes séparées, pas
en une seule somme globale à la fin.

## La correction : avancer à chaque accès au bus

```rust
// src/bus/bus.rs
pub fn internal_tick(&mut self) {
    self.io.tick();
}

pub fn read(&mut self, addr: u16) -> u8 {
    let val = self._read(addr);
    self.internal_tick();
    val
}

pub fn write(&mut self, addr: u16, val: u8) {
    self._write(addr, val);
    self.internal_tick();
}

fn _read(&self, addr: u16) -> u8 { /* the actual match on addr, as before */ }
fn _write(&mut self, addr: u16, val: u8) { /* ditto */ }
```

La vraie logique de lecture/écriture se déplace dans des fonctions
d'assistance (helpers) privées `_read`/`_write`, et les fonctions
publiques `read`/`write` les enveloppent (wrap) d'un appel automatique à
`internal_tick()` après chaque accès. C'est un remaniement (refactor)
satisfaisant : chaque endroit du code source qui touche à la mémoire —
chaque gestionnaire d'instruction, depuis le Chapitre 3 — fait désormais
automatiquement avancer le timer (et plus tard le PPU/APU) de la bonne
quantité, sans avoir à se rappeler de le faire manuellement où que ce
soit. `Bus::tick(cycles)` (la version du Chapitre 9, appelée une fois par
instruction) disparaît entièrement, remplacé par cet appel intégré
directement dans `read`/`write`.

Remarquez l'effet d'entraînement : `read`/`write` nécessitent désormais
`&mut self` au lieu de `&self`/`&mut self` respectivement (`read` ne
mutait pas d'état auparavant — elle doit maintenant le faire, pour faire
avancer l'état interne). C'est exactement le genre de changement qui
paraît petit dans un diff mais touche chaque site d'appel dans tout le
projet, car `read` est appelée depuis des dizaines de gestionnaires
d'instructions.

## Une nouvelle famille de ROMs de test : `mem_timing`

Avec l'avancement par accès en place, une nouvelle catégorie de ROM de
test devient pertinente : les suites `mem_timing` et `mem_timing-2` de
Blargg, spécifiquement conçues pour détecter précisément ce type de bug
(une instruction qui *calcule* le bon résultat, mais touche la mémoire au
mauvais moment par rapport à l'horloge). Elles rejoignent la suite de test
aux côtés de `instr_timing.gb` (correction du nombre de cycles
d'instruction globale).

## Deux manières différentes dont les ROMs de test rapportent les résultats

Travailler sur ces nouvelles ROMs de test met en lumière quelque chose qui
mérite d'être documenté une bonne fois pour toutes et réutilisé : toutes
les ROMs de test de style Blargg ne rapportent pas leurs résultats de la
même manière. Les propres notes de ce projet (`TEST_ROM_SPECS.MD`)
exposent les deux conventions rencontrées en pratique :

| Convention | Utilisée par | Comment les résultats sont rapportés |
|---|---|---|
| Ancienne (`shell.inc`) | `cpu_instrs`, `instr_timing`, `mem_timing` | Écrit chaque caractère de résultat sur le **port série** (Chapitre 4), se termine par une boucle infinie sur elle-même |
| Nouvelle | `mem_timing-2`, `dmg_sound`, `oam_bug`, `halt_bug` | Écrit un octet de statut + une signature fixe (`DE B0 61`) + le texte de résultat directement dans la **RAM externe** à `0xA000`, se termine par une boucle infinie sur elle-même |

```text
$A000     = status (0x80 = running, 0x00 = passed, anything else = error code)
$A001-03  = signature DE B0 61
$A004+    = result text (null-terminated)
```

Les deux conventions se terminent de la même manière : une boucle infinie
sur elle-même (`JP $`/`JR $`, c'est-à-dire une instruction de saut dont la
cible est elle-même). C'est en réalité le signal le plus fiable qu'« une
ROM de test est terminée » — si `PC` cesse de changer entre les itérations
de la boucle principale, rien ne va se produire que vous n'ayez déjà
observé, donc il est sûr de s'arrêter et de vérifier les résultats. C'est
exactement la technique sur laquelle s'appuie déjà le harnais de test de
`CrabbyBoy` (Chapitre 9, et développée au Chapitre 11).

## Ce que nous avons maintenant

- Un avancement précis du hardware par accès mémoire, au lieu d'une somme
  globale en fin d'instruction.
- Les deux conventions de résultat de ROM de test, basées sur le port
  série et sur la RAM, comprises et documentées.
- Un nombre croissant de ROMs de test sensibles au timing qui réussissent :
  `instr_timing`, `mem_timing`, `mem_timing-2` (et ses sous-tests : timing
  de lecture/écriture/modification).

## Ce qui manque encore

- Les ROMs de test `oam_bug` et `dmg_sound` sont déjà connues (d'après
  `TEST_ROM_SPECS.MD`) mais pas encore exécutables — elles nécessitent
  respectivement un comportement OAM/PPU et une vraie APU, qui n'existent
  pas encore à ce stade.
- Il s'agit encore ici de « timing du CPU et du bus », pas de « timing du
  PPU » — rien n'est encore affiché à l'écran, et le PPU (Chapitre 7) ne
  fait encore que stocker des octets de registre. Le PPU (Partie V) et
  l'APU (Partie VI) restent tous deux à venir.
