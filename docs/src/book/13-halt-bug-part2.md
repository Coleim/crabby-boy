# 13. Le bug HALT, partie 2

Le Chapitre 5 a implémenté une première tentative pour le bug HALT, avec
sa propre ROM de test dédiée, mais l'avait signalée comme "pas encore
totalement correcte". Ce chapitre (`9e46890`) est celui où `halt_bug.gb`
est réellement ajouté à la suite de tests automatisée et commence à
passer — et la véritable correction est un rappel merveilleusement
humiliant de la profondeur à laquelle de minuscules bugs peuvent se
cacher.

## Le véritable bug : un zéro manquant

```diff
-            0xFF0F => self.interrupt_flag | 0b1110_000,
+            0xFF0F => self.interrupt_flag | 0b1110_0000,
```

C'est tout. `0b1110_000` est un littéral de **7 bits** (`0b1110000` =
`0x70`) ; la valeur voulue `0b1110_0000` est la bonne valeur de **8
bits** (`0xE0`). Un seul chiffre manquant. Le Chapitre 12 a expliqué que
les 3 bits de poids fort d'`IF` se lisent toujours comme `1` sur le vrai
hardware — cette ligne est exactement ce masquage — et un unique `0`
manquant faisait que le bit 7 d'`IF` était silencieusement lu comme `0`
au lieu de `1` chaque fois que les bits de flag réels en dessous se
trouvaient être non définis. `halt_bug.gb` vérifie précisément des
valeurs de registre comme celle-ci, octet par octet, ce qui explique
exactement pourquoi elle a détecté quelque chose que l'exécution de
`cpu_instrs.gb` et `mem_timing.gb` n'avait jamais révélé.

C'est une leçon véritablement utile, peut-être la plus utile de tout ce
livre : **l'échec du test du "bug HALT" ne concernait en réalité pas du
tout une logique HALT erronée** — l'*implémentation* du bug HALT du
Chapitre 5 était correcte. Le bug était une faute de frappe sur la
largeur d'un bit dans un registre complètement différent, plusieurs
chapitres plus tôt, que seule une ROM de test très spécifique et
étroitement ciblée a fini par révéler. C'est exactement pour cela que des
ROMs de test dédiées et ciblées (par opposition à seulement de grandes
ROMs générales) méritent leur place — et exactement pourquoi, quand un
test échoue, le bug ne se trouve souvent pas là où le nom du test
suggère de regarder en premier.

## Deux corrections de justesse plus mineures au passage

```rust
0xFEA0..=0xFEFF => {
    println!("Not Usable ... Addr: {:02x}", addr);
}
```

Les écritures dans la plage `0xFEA0`–`0xFEFF` (mémoire explicitement
inutilisable, selon la carte du Chapitre 0) étaient déjà gérées côté
*lecture* mais pas côté *écriture* — désormais les deux affichent le
même avertissement au lieu de tomber dans un cas générique fourre-tout.

```rust
cpu_instr_test!(halt_bug, "./tests/halt_bug.gb");
```

Et `halt_bug.gb` rejoint enfin la liste croissante des tests automatisés
du Chapitre 9/11, maintenant qu'elle passe réellement.

## Ce que nous avons maintenant

- Un masque de lecture correct du registre `IF`, et par extension, un
  test `halt_bug.gb` qui passe.
- `halt_bug.gb` fonctionnant dans le cadre de la suite CI automatisée du
  Chapitre 11.
- Un gestionnaire côté écriture complété pour la plage mémoire
  inutilisable adjacente à l'OAM.

## Ce qui manque encore

- Rien de conceptuellement nouveau ne reste à propos de HALT lui-même —
  mais ce chapitre est un bon moment pour se rappeler : les ROMs de test
  qui passent construisent la *confiance*, pas la *preuve*. Gardez un œil
  dans votre propre projet sur "le nom de cet échec de test ne
  correspond pas à l'endroit où le vrai bug s'est avéré se trouver".
- Le *dispatch* des interruptions — sauter effectivement vers un vecteur
  d'interruption quand une se déclenche — n'est toujours pas implémenté.
  C'est le sujet suivant, au Chapitre 14.
