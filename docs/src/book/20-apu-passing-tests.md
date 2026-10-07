# 20. Réussir les tests `dmg_sound` de Blargg

Avec les 4 canaux construits (la partie VI jusqu'ici), ce chapitre (`bb33d06` →
`c8be255`) est le même genre de jalon que le chapitre 10 l'était pour le timing
du CPU : passer de « j'ai implémenté ce que dit la documentation » à « je l'ai
vérifié par rapport à une suite de tests fiable et indépendante » — les ROMs
`dmg_sound` de Blargg.

## La suite de tests, un sous-test à la fois

```rust
// src/crabby_boy.rs (test module)
cpu_instr_test!(sound_01, "./tests/dmg_sound/01-registers.gb");
cpu_instr_test!(sound_02, "./tests/dmg_sound/02-len ctr.gb");
cpu_instr_test!(sound_03, "./tests/dmg_sound/03-trigger.gb");
cpu_instr_test!(sound_04, "./tests/dmg_sound/04-sweep.gb");
cpu_instr_test!(sound_05, "./tests/dmg_sound/05-sweep details.gb");
cpu_instr_test!(sound_06, "./tests/dmg_sound/06-overflow on trigger.gb");
cpu_instr_test!(sound_07, "./tests/dmg_sound/07-len sweep period sync.gb");
cpu_instr_test!(sound_08, "./tests/dmg_sound/08-len ctr during power.gb");
cpu_instr_test!(sound_09, "./tests/dmg_sound/09-wave read while on.gb");
cpu_instr_test!(sound_10, "./tests/dmg_sound/10-wave trigger while on.gb");
cpu_instr_test!(sound_11, "./tests/dmg_sound/11-regs after power.gb");
cpu_instr_test!(sound_12, "./tests/dmg_sound/12-wave write while on.gb");
cpu_instr_test!(sound_all, "./tests/dmg_sound.gb");
```

Chacun de ces tests cible un domaine précis : la bonne lecture/écriture des
registres, les cas limites (edge cases) du compteur de longueur, la séquence exacte de
« déclenchement » (trigger) qui se produit quand un canal est (re)démarré, le
comportement du sweep et son cas limite de dépassement (overflow), etc. — faisant
écho à la leçon « plusieurs petites ROMs de test ciblées valent mieux qu'une seule
grosse » des chapitres 10-11 et 13.

## Une véritable classe de bug que cette suite détecte : la séquence de « trigger »

Plusieurs des bizarreries du DMG évoquées dans les chapitres 16/18/19 (la
bizarrerie de changement de mode du sweep, la bizarrerie de cadence précoce du
compteur de longueur) sont en réalité toutes des variations sur un même thème :
**ce qui se passe exactement à l'instant où un canal est déclenché** (écriture dans
son registre `NRx4` avec le bit de déclenchement activé). Déclencher n'est pas
simplement « démarrer le canal » — c'est une séquence précise : recharger le
compteur de longueur s'il était à zéro, recharger le minuteur (timer) et le volume
de l'enveloppe, recharger le registre fantôme (shadow) du sweep et vérifier
immédiatement s'il y a dépassement, réinitialiser la position de la forme d'onde,
et (pour le canal 3 spécifiquement) parfois corrompre la RAM de forme d'onde si
redéclenché à exactement le mauvais moment. Obtenir l'*ordre* correct de cette
séquence, pas seulement chaque élément isolément, est exactement ce que les tests
`03-trigger.gb`, `06-overflow on trigger.gb` et `10-wave trigger while on.gb` sont
conçus pour détecter — et c'est exactement le genre de chose qu'il est quasiment
impossible de bien faire par simple intuition, d'où l'intérêt de s'appuyer sur la
suite de tests plutôt que de deviner.

## `12-wave write while on.gb` : le timing compte même pour une écriture « simple »

La RAM de forme d'onde du canal 3 (chapitre 18) peut être lue et écrite par le
CPU à presque n'importe quel moment — *sauf* pendant que le canal joue
activement, période durant laquelle le matériel réel n'autorise l'accès que
pendant une fenêtre très étroite et précise par échantillon, et renvoie/ignore
des données corrompues le reste du temps. C'est la même idée « un certain état
matériel bloque un certain accès mémoire » que les restrictions d'accès à la
VRAM/OAM du guide PPU associé — découverte ici, pour la RAM de forme d'onde, via
cette ROM de test spécifique.

## Ce que nous avons maintenant

- Les 4 canaux vérifiés par rapport à la suite `dmg_sound` de Blargg, sous-test
  par sous-test, sous forme d'entrées `cargo test` automatisées (la CI du
  chapitre 11 les exécute aussi à chaque push).
- Une séquence de déclenchement bien plus fidèle et un timing d'accès à la RAM
  de forme d'onde corrigé, trouvés et réparés spécifiquement parce que ces tests
  existaient.

## Ce qu'il manque encore

- Ceci clôt la partie VI. Le son fonctionne maintenant et est vérifié — mais
  tout ce qui suit à partir d'ici (parties VII-VIII) a été construit en parallèle
  du côté graphique du projet, et non après ; la partie IX est l'endroit où ce
  livre revient terminer ce côté-là.
