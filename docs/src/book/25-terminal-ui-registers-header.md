# 25. Afficher les registres CPU et les infos de la cartouche

Avec une architecture d'affichage solide en place (chapitre 23), ce
chapitre (`6846f97`) commence à vraiment l'utiliser pour quelque chose de
réellement utile : une vue de débogage en direct, lisible par un humain,
directement issue de l'état interne du CPU — inestimable pour comprendre
ce que fait votre propre émulateur pendant qu'il tourne, et pas
seulement s'il réussit les tests.

## Chaque widget n'est qu'une fonction : `(Frame, Rect, data) -> ()`

```rust
// src/display/registers.rs
pub fn render(frame: &mut Frame, area: Rect, cpu: &CPU) {
    let block = Block::bordered()
        .title(Line::from("CPU"))
        .style(Style::new().light_magenta());

    let rows = vec![
        Row::new(vec![
            Cell::from(Line::from(vec!["AF".bold().gray()])),
            Cell::from(Line::from(vec![format!("{:04X}", cpu.get_af()).bold().white()])),
            Cell::from(Line::from(vec!["BC".bold().gray()])),
            Cell::from(Line::from(vec![format!("{:04X}", cpu.get_bc()).bold().white()])),
            Cell::from(Line::from(vec!["PC".bold().gray()])),
            Cell::from(Line::from(vec![format!("{:04X}", cpu.pc).bold().white()])),
        ]),
        // ... ligne DE / HL / SP
    ];

    let table = Table::new(rows, /* largeurs de colonnes */).block(block);
    frame.render_widget(table, area);
}
```

C'est la forme la plus simple possible que peut prendre une fonction de
widget `ratatui` : étant donné un `Frame` dans lequel dessiner, une
région d'écran (`Rect`), et un élément d'état de l'émulateur à lire,
construire et afficher une interface. Pas de structure, pas
d'implémentation de trait nécessaire pour cette partie —
`ratatui_display.rs` (du chapitre 23) appelle simplement
`registers::render(frame, area, &crabby.cpu)` directement à l'intérieur
de son propre `draw`. Cela compte comme rappel : `get_af()`/`get_bc()`/
`get_hl()` (les accesseurs de paires de registres du chapitre 4) sont
désormais rentabilisés dans un endroit complètement différent du
décodage d'opcode — un petit bloc de construction bien nommé et bien
testé a tendance à être réutilisé dans des endroits que vous n'aviez pas
prévus à l'origine.

## Réutiliser l'en-tête de la cartouche une deuxième fois

```rust
// src/display/cartridge_header.rs (ébauche, même motif que registers.rs)
pub fn render(frame: &mut Frame, area: Rect, header: &CartdrigeHeader) {
    // titre, type de cartouche, taille ROM/RAM, etc., chacun en ligne étiquetée
}
```

`CartdrigeHeader` (chapitre 2) avait déjà une méthode `print()` pour le
débogage console brut depuis sa toute première version — maintenant ses
champs analysés ont droit à une deuxième présentation, plus agréable :
un panneau bordé dans l'interface terminal, au lieu de (ou en plus de) la
simple sortie `println!`. Les données sous-jacentes et la logique
d'analyse ne changent absolument pas ; seule leur façon d'être affichées
change — un bon exemple de séparation entre "ce qu'est la donnée" et
"comment elle est montrée", afin que la même source de vérité puisse
servir plusieurs présentations.

## Un premier écran de jeu (encore un placeholder)

```rust
// src/display/game.rs
const WIDTH: u32 = 160;
const HEIGHT: u32 = 144;

pub fn render(frame: &mut Frame, area: Rect) {
    // TODO: Remplacer par le vrai tampon (buffer) du PPU
    let dyn_img = generate_buffer(); // toujours un motif de test en dégradé
    // ... même approche de rendu ratatui-image que le détour du chapitre 22
}
```

C'est la continuation directe de l'idée abandonnée de `DisplayInterface`
du chapitre 22, maintenant correctement intégrée dans la vraie
architecture : un widget image 160×144, dimensionné exactement pour
correspondre à la vraie résolution de l'écran Game Boy (chapitre 0/12),
actuellement alimenté par un dégradé placeholder au lieu de vrais pixels
— explicitement marqué par un `TODO` comme étant exactement cela. C'est
l'emplacement dans lequel le guide associé
[PPU Background Rendering Guide](../ppu-background.md) viendra finalement
brancher de vraies images rendues.

## Ce que nous avons maintenant

- Une vue en direct des registres CPU et une vue de l'en-tête de la
  cartouche dans l'interface terminal.
- Un motif réutilisable de petite fonction-widget pour ajouter d'autres
  vues de débogage plus tard (c'est exactement le même motif que suit
  ensuite le visualiseur VRAM du chapitre 26).
- Un écran de jeu placeholder correctement dimensionné, prêt pour de
  vrais pixels.

## Ce qui manque encore

- L'écran de jeu affiche encore un dégradé, pas la vraie sortie Game Boy.
- Aucune visualisation VRAM pour l'instant — c'est la suite, et c'est le
  dernier tremplin avant le chapitre de clôture de ce livre.
