# 30. Visualiser l'OAM (à venir)

Ce chapitre est un espace réservé : au moment de l'écrire, le code
correspondant n'existe pas encore dans le projet. Il documente
l'intention et la forme attendue, exactement comme le chapitre 25 l'a
fait en son temps pour l'écran de jeu avant que le vrai framebuffer du
PPU n'existe (toujours un dégradé placeholder aujourd'hui).

## Pourquoi l'OAM mérite son propre visualiseur

Les chapitres 26 et 29 ont donné une vue de la tile data et de la tile
map — les deux ingrédients du Background/Window. Mais la troisième
couche (chapitre 33), les **objects (sprites)**, ne passe par aucune des
deux : elle lit directement l'**OAM** (`$FE00`–`$FE9F`, chapitre 0), une
petite zone mémoire de 160 octets organisée en 40 entrées de 4 octets
chacune — une par sprite potentiellement affichable.

Sans visualiseur dédié, il est impossible de savoir, en regardant
tourner un jeu, si l'OAM contient ce qu'on attend (le bon nombre de
sprites, aux bonnes positions, avec les bons index de tuile) — exactement
le même argument de débogage qui justifiait les chapitres 26 et 29 pour
la VRAM.

## La forme attendue : un tableau, pas une image

Contrairement aux visualiseurs précédents (qui décodent des pixels),
l'OAM se prête mieux à un tableau texte façon `cpu_registers.rs`
(chapitre 25) : une ligne par sprite actif, avec ses 4 octets déjà
décodés en colonnes lisibles.

```rust
// src/display/oam_registers.rs (placeholder — à écrire)
use ratatui::{
    Frame,
    layout::Rect,
    text::Line,
    widgets::{Block, Cell, Row, Table},
};

pub fn render(frame: &mut Frame, area: Rect, oam: &[u8]) {
    let block = Block::bordered().title(Line::from("OAM"));

    // TODO: découper `oam` (160 octets) en 40 entrées de 4 octets :
    // [y, x, tile_index, attributes], et construire une Row par sprite
    // dont `y` n'est pas à sa valeur "hors écran" par défaut (0 ou 0xFF
    // selon la convention choisie), pour ne montrer que les sprites
    // pertinents plutôt que 40 lignes majoritairement vides.
    let rows: Vec<Row> = Vec::new();

    let table = Table::new(rows, []).block(block);
    frame.render_widget(table, area);
}
```

Rappel de la structure d'une entrée OAM (chapitre 33 pour le détail
complet) : position Y, position X, index de tuile (toujours en mode non
signé, direct depuis `$8000`), puis un octet d'attributs (flip
horizontal/vertical, palette, priorité objet-derrière-background).

## Ce que nous avons maintenant

- Rien de nouveau en code — ce chapitre documente une intention, pas une
  implémentation.
- Une idée claire de la forme à donner à ce visualiseur (un tableau, pas
  une image) et de pourquoi il complète les chapitres 26/29.

## Ce qui manque encore

- Tout le code réel : découpage des 160 octets d'OAM en 40 entrées,
  décodage de l'octet d'attributs, filtrage des sprites "hors écran", et
  l'intégration dans `ratatui_display.rs` aux côtés des autres panneaux.
- Une fois ce visualiseur écrit, il complétera la trilogie de débogage
  mémoire du PPU (tile data, tile map, OAM) commencée au chapitre 26.
