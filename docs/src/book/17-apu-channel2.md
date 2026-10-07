# 17. Canal 2 — L'onde carrée la plus simple

Le canal 2 (`a10eb95`) n'apporte, par conception, presque rien de nouveau à construire :
c'est « le canal 1 sans la fonctionnalité de sweep (sweep) ». Ce chapitre est court volontairement —
la vraie leçon se trouve dans la décision de conception, pas dans du nouveau code.

## Réutiliser exactement la même structure `Channel`

```rust
// src/audio/apu.rs
pub struct APU {
    // ...
    channel1: Channel,
    channel2: Channel,
    // ...
}
```

Les deux champs ont le même type. Le canal 2 n'appelle simplement jamais
`write_sweep`, rien ne cadence jamais ses champs de sweep (inutilisés), et
ses propres méthodes d'écriture de registres (`write_nr1`/`write_nr2`/etc., partagées
avec le canal 1 via le même bloc `impl Channel` du chapitre 16) sont
tout ce dont il a besoin.

## Où se trouvent les vraies différences : le routage d'adresses d'`IOBridge`

La différence réelle entre les deux canaux ne se trouve pas du tout dans la
logique de canal — elle réside purement dans quelles adresses d'E/S sont routées
vers quelle instance de canal :

```rust
// channel 1 registers: NR10-NR14, conventionally 0xFF10-0xFF14
// channel 2 registers: NR21-NR24, conventionally 0xFF16-0xFF19 (no NR20 sweep register — skipped on purpose)
```

Cela vaut la peine d'être noté comme un modèle général : parfois, la façon la plus
fidèle de représenter « la fonctionnalité X n'existe pas sur cette variante » n'est pas
un drapeau (flag) conditionnel quelque part dans la logique partagée — c'est simplement
*ne jamais câbler l'adresse de registre* qui la contrôlerait. Le matériel
du canal 2 n'a tout simplement aucun registre de sweep ; notre code reflète cela
en n'ayant aucun chemin de code qui appellerait jamais `write_sweep` sur l'instance
`Channel` du canal 2, plutôt que, disons, un drapeau booléen comme
`has_sweep: bool` que chaque méthode devrait vérifier.

## Ce que nous avons maintenant

- Les deux canaux d'onde carrée (1 et 2) produisant une forme d'onde en duty cycle
  avec support de l'enveloppe et du compteur de longueur, partageant une seule implémentation.
- Une illustration nette de la mesure dans laquelle la complexité apparente de la Game Boy
  (4 canaux « différents ») repose en réalité sur un petit nombre de blocs de
  construction partagés (générateur de duty cycle, enveloppe, compteur de longueur, sweep)
  recombinés de façons légèrement différentes selon le canal.

## Ce qu'il manque encore

- Le canal 3 (forme d'onde arbitraire, pas basée sur un duty cycle) et le canal 4
  (bruit, pas périodique du tout) ont vraiment besoin de leurs propres
  implémentations distinctes — couvert dans la suite.
