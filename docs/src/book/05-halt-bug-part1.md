# 5. Le bug HALT, partie 1

Blotti à la toute fin de l'effort sur les opcodes (`dc9a258`, avec une
ROM de test dédiée ajoutée juste après dans `5631832`) se trouve l'une
des bizarreries les plus célèbres du CPU de la Game Boy : le **bug
HALT**. C'est un excellent premier exemple de quelque chose auquel ce
livre reviendra plus d'une fois — le hardware réel se comporte parfois
de manières qui ressemblent à des bugs évidents, sauf qu'il ne s'agit pas
de bugs, c'est exactement ce que fait la puce, et votre émulateur doit
reproduire ce « bug » intentionnellement.

## Ce que `HALT` est censé faire

`HALT` (opcode `0x76`) dit au CPU « arrête d'exécuter des instructions et
ne fais rien jusqu'à ce qu'une interruption (interrupt) survienne ». Cela
économise de l'énergie sur le hardware réel, et en termes logiciels,
c'est simplement : arrêter d'avancer, continuer à faire avancer
(ticking) les autres composants (timer, PPU, APU), et reprendre dès
qu'une interruption devient en attente.

```rust
pub halt: bool,
```

Un simple flag booléen. Quand il est vrai, la boucle principale saute
entièrement l'exécution du CPU (nous verrons cette boucle au Chapitre 11)
jusqu'à ce que quelque chose le repositionne à faux.

## La bizarrerie : `HALT` avec des interruptions en attente mais désactivées

Voici l'étrangeté. Il existe deux concepts indépendants :

- **IME** (« Interrupt Master Enable ») — un interrupteur interne au
  CPU : les interruptions sont-elles autorisées à réellement interrompre
  le CPU en ce moment ?
- **IE** (`0xFFFF`) et **IF** (`0xFF0F`) — quels *types* d'interruption
  sont activés, et lesquels sont actuellement en attente, indépendamment
  d'IME.

Si un jeu exécute `HALT` au moment précis où `IME` est désactivé, mais
qu'une interruption est à la fois activée (`IE`) et déjà en attente
(`IF`), le hardware réel ne se met **pas du tout** en pause (halt). Il
continue immédiatement l'exécution — mais avec un bug : **l'octet
d'opcode de l'instruction suivante est récupéré (fetched) deux fois**,
c'est-à-dire que `PC` échoue à avancer pour une récupération, ré-exécutant
silencieusement l'effet sur un seul octet que l'opcode suivant avait
produit. C'est exactement ce à quoi renvoie le nom « bug HALT » — non pas
un bug dans un émulateur, mais une bizarrerie documentée de la puce
réelle que tout émulateur fidèle doit reproduire.

## Première tentative d'implémentation

```rust
pub halt_bug: bool,
```

```rust
0x76 => {
    let ie = bus.get_ie();
    let if_flag = bus.get_io().get_if();

    if !self.ime && (ie & if_flag) != 0 {
        // HALT BUG — don't halt, just corrupt next fetch
        self.halt_bug = true;
    } else {
        self.halt = true;
    }
}
```

Et ensuite, tout en haut de l'étape de récupération (fetch) :

```rust
let mut next_pc: u16 = if self.halt_bug {
    self.halt_bug = false;
    self.pc // don't advance — re-fetch the same address next time
} else {
    self.pc.wrapping_add(1)
};
```

La logique en langage clair : quand `HALT` s'exécute, vérifier si `IME`
est désactivé *et* qu'une interruption activée et en attente existe déjà.
Si oui, positionner `halt_bug = true` au lieu de `halt = true`. La
prochaine fois qu'un opcode est récupéré, si `halt_bug` était positionné,
ne pas avancer `PC` au-delà de l'opcode que nous sommes sur le point
d'exécuter — ce qui signifie que la récupération *suivante* relira ce
même emplacement d'octet. C'est l'effet de « récupération dupliquée »,
reproduit.

## Pourquoi cela mérite sa propre ROM de test

Un comportement de CPU aussi marginal (edge case) que celui-ci est
exactement le genre de chose qu'il est facile d'implémenter de façon
subtilement incorrecte (décalage d'un cran dans exactement *quel* octet
est re-récupéré, ou une condition IME/IE/IF légèrement fausse), et des
versions subtilement incorrectes peuvent encore réussir la plupart des
jeux par chance tout en échouant sur des cas de test spécifiques et
délibérément conçus. C'est pourquoi la communauté maintient des ROMs de
test ciblées comme `tests/halt_bug.gb`, ajoutée ici spécifiquement pour
valider ce seul comportement de manière isolée, indépendamment de la
suite `cpu_instrs.gb` plus large du Chapitre 4.

## Ce que nous avons maintenant

- Un flag `halt` qui arrête l'exécution du CPU jusqu'à une interruption.
- Une première tentative pour le bug HALT : détecter la condition « IME
  désactivé, interruption déjà en attente » et corrompre la prochaine
  récupération au lieu de réellement mettre en pause.
- Une ROM de test dédiée pour valider ce comportement.

## Ce qui manque encore

- Cette première tentative n'est **pas encore totalement correcte** —
  remarquez que la condition `!self.ime && (ie & if_flag) != 0` est
  vérifiée, mais il reste encore un écart dans la façon exacte dont la
  récupération corrompue interagit avec les instructions multi-octets,
  ce qui apparaît comme un véritable bug plus tard. Le Chapitre 13
  (« Le bug HALT, partie 2 ») revient corriger cela proprement, une fois
  qu'il existe une véritable source d'interruption (le VBlank du PPU, de
  la Partie V) contre laquelle la déclencher en pratique plutôt que
  seulement dans la ROM de test isolée.
- Aucune *gestion* d'interruption n'existe encore du tout à ce stade (pas
  de saut vers les vecteurs d'interruption, pas d'effacement d'`IE`/`IF`
  au dispatch) — juste assez du concept `IME`/`IE`/`IF` pour rendre
  testable la bizarrerie de cette seule instruction. La gestion complète
  des interruptions, c'est le Chapitre 14.
