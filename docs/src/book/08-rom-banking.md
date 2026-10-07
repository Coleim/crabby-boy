# 8. Banking de ROM

Le Chapitre 2 a analysé l'octet « type de cartouche » mais n'a jamais agi
en fonction de celui-ci. C'est ici (`136bdd1`) que cela devient enfin
important : la prise en charge des jeux plus volumineux que ce qui tient
dans la plage directement adressable par le CPU.

## Le problème : 32 Ko ne suffisent pas

Le CPU peut adresser `0x0000`–`0x7FFF` pour la ROM — soit 32 Ko. Mais de
nombreux jeux réels (et `cpu_instrs.gb`, celui que nous utilisons comme
test depuis le début) sont plus volumineux que cela. L'astuce employée
par les fabricants de cartouches : placer une petite puce supplémentaire —
un **Memory Bank Controller (MBC)** — sur la cartouche elle-même, entre la
puce de ROM et la console. Cette puce intercepte les écritures vers
certaines adresses et les utilise non pas comme « stocker cet octet en
ROM » (impossible d'écrire dans une ROM, c'est un hardware en lecture
seule !) mais comme des *commandes* : « à partir de maintenant, quand le
CPU lit depuis `0x4000`–`0x7FFF`, donne-lui plutôt des données provenant
d'un autre bloc de 16 Ko de la ROM bien plus grande. »

Ainsi, `0x4000`–`0x7FFF` est une **fenêtre commutable (switchable
window)** : son contenu change selon le « bank » (bloc de 16 Ko)
sélectionné en dernier, tandis que `0x0000`–`0x3FFF` affiche toujours le
premier bank fixe.

## Lire à travers le bank actif

```rust
// src/bus/bus.rs
0x4000..=0x7FFF => {
    let offset: u16 = self.bank_number as u16 * 0x4000;
    self.rom[(offset + (addr - 0x4000)) as usize]
}
```

`0x4000` correspond à la taille d'un bank (16 Ko, cohérent avec la taille
de la fenêtre mentionnée ci-dessus). Donc : on prend le `bank_number`
actuellement sélectionné, on le multiplie par la taille d'un bank pour
trouver où ce bank commence dans le `Vec<u8>` représentant la ROM
complète, puis on ajoute le décalage *à l'intérieur* de la fenêtre que le
CPU est en train de lire. Si `bank_number` vaut 2 et que le CPU lit
`0x4100`, cela calcule `2 * 0x4000 + (0x4100 - 0x4000) = 0x8100` dans les
données réelles sous-jacentes de la ROM.

## Sélectionner un bank : les écritures en ROM ne sont pas vraiment des écritures

```rust
0x2000..=0x3FFF => {
    // Enable bank
    self.bank_number = val & 0b0011111; // keep only the low 5 bits
    if self.bank_number == 0 {
        self.bank_number = 1;
    }
}
```

C'est le détail qui déroute la plupart des gens la première fois : le CPU
« écrit en ROM », ce qui paraît contradictoire, mais la puce MBC ne stocke
jamais réellement cet octet quelque part comme donnée — elle *remarque*
simplement que l'écriture a eu lieu et *réagit* en changeant un état
interne (ici, `bank_number`). Du point de vue du code du jeu, cela
ressemble exactement à une écriture en mémoire ; en réalité, chaque
« écriture » de ce type est interceptée et réinterprétée comme une
commande. Ce bloc `0x2000..=0x3FFF` dans `Bus::write` est l'endroit où
cette interception se produit.

Le `val & 0b0011111` masque l'écriture à 5 bits (cette première
implémentation de type MBC1 prend ainsi en charge jusqu'à 32 banks), et la
règle « si c'est 0, mettre 1 » reflète une particularité (quirk) réelle du
MBC : le bank 0 est déjà toujours visible à `0x0000`–`0x3FFF`, donc
sélectionner le « bank 0 » pour la fenêtre commutable serait redondant —
le hardware traite simplement une demande de bank 0 comme une demande de
bank 1.

## Ce qui reste un bouchon (stub) ici

```rust
0x0000..=0x1FFF => {
    println!("RAM Enable (Write Only)");
}
// ...
0x6000..=0x7FFF => {
    println!("Banking Mode Select (Write Only): {:02x}", val);
}
```

Les vraies cartouches MBC1 ont plus de fonctionnalités que la simple
sélection de bank de ROM : l'activation/désactivation de la RAM externe
(`0x0000`–`0x1FFF`) et un commutateur de « mode » de banking affectant la
manière dont les bits d'adresse supérieurs sont interprétés
(`0x6000`–`0x7FFF`). À ce stade, elles sont reconnues (de sorte que les
écritures du jeu ne disparaissent pas silencieusement dans le filet
générique) mais sans effet — elles sont journalisées (log) et laissées
pour un affinement ultérieur, la même technique de bootstrap que celle du
Chapitre 6 (bouchon d'abord, comportement réel une fois réellement
nécessaire).

## Une petite mais importante correction de `STOP`, en passant

```rust
0x10 => {
    self.stopped = true; // STOP n8 2  4
    next_pc = next_pc.wrapping_add(1);
}
```

`STOP` est une instruction de 2 octets (opcode + un octet d'opérande,
conventionnellement toujours `0x00`), pas 1 octet — ce commit corrige
`next_pc` pour qu'il saute effectivement cet octet d'opérande. Un bon
rappel : même bien avancé dans les chapitres suivants, il vous arrivera de
revenir en arrière corriger de petites erreurs provenant de chapitres bien
antérieurs, souvent en travaillant sur quelque chose de totalement
différent (le banking de ROM, ici). C'est tout à fait normal.

## Ce que nous avons maintenant

- Une commutation de bank de ROM fonctionnelle, permettant à
  `cpu_instrs.gb` (et à toute autre ROM utilisant ce style de banking) de
  se charger et de s'exécuter correctement au-delà de 32 Ko.
- Des écritures d'activation de RAM et de mode de banking reconnues (même
  si pas encore implémentées).
- Une longueur d'instruction `STOP` corrigée.

## Ce qui manque encore

- Pas encore de véritable banking de RAM externe (`eram` existe comme un
  tableau plat dans `Bus`, sans sa propre commutation de bank).
- Une seule « famille » de comportement MBC est modélisée — les vraies
  cartouches peuvent utiliser plusieurs puces MBC différentes (MBC1, MBC3,
  MBC5, ...) avec des particularités différentes ; ce projet prend en
  charge le cas simple courant nécessaire pour ses ROMs de test, pas
  toutes les variantes.
- Toujours aucune interruption (interrupt) déclenchée depuis où que ce
  soit, pas de véritable interruption pilotée par le timer, pas de
  comportement du PPU — la Partie IV s'en occupe ensuite.
