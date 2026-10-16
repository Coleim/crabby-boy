# 29. Afficher la tile map, et corriger une requête terminal répétée

Ce chapitre couvre deux commits qui se suivent de près : `0c36554`
("Display vram tilemap"), qui complète le visualiseur du chapitre 26 en
y ajoutant la véritable disposition en tile map décrite au chapitre 28,
et `15a8e3e` ("cache le Picker terminal au lieu de le recréer chaque
frame"), qui corrige un bug de performance introduit sans qu'on s'en
rende compte en ajoutant tout cet affichage terminal.

## De "chaque tuile existante" à "ce que le jeu a choisi d'afficher"

Le chapitre 26 décodait chaque bloc de 16 octets de VRAM comme une tuile
indépendante, sans tenir compte de la tile map. `src/display/vram_tilemap_registers.rs`
ajoute exactement l'étage manquant identifié au chapitre 28 :

```rust
fn generate_buffer(tilemap: &[u8], tiles: &[u8]) -> DynamicImage {
    const TILES_PER_ROW: usize = 32;
    let width_px = TILES_PER_ROW * 8;
    let height_px = TILES_PER_ROW * 8;

    for y in 0..height_px {
        for x in 0..width_px {
            let tile_idx = (y / 8) * TILES_PER_ROW + (x / 8);
            let tile_id = tilemap[tile_idx];        // <- la nouveauté : une indirection par la tile map
            let start_tile = tile_id as usize * 16;  // puis on va chercher les pixels de CETTE tuile
            // ... même décodage 2 bits par pixel que le chapitre 26
        }
    }
}
```

La différence avec le chapitre 26 tient en une ligne :
`let tile_id = tilemap[tile_idx];`. Avant, l'index de tuile *était*
directement la position dans VRAM (balayage brut). Maintenant, on lit
d'abord dans la tile map *quel* identifiant de tuile se trouve à cette
position, puis on va chercher les pixels de cette tuile précise — c'est
exactement ce qu'un vrai PPU fait pour savoir quoi dessiner à chaque
case de l'arrière-plan.

### Une simplification encore assumée : l'adressage non signé uniquement

Ce code suppose toujours que `tile_id` indexe directement depuis
`$8000` (le mode d'adressage "non signé", bit 4 de `LCDC` = 1). La
vraie Game Boy peut aussi adresser les tuiles depuis `$8800` en mode
signé selon ce même bit (voir le guide associé
[PPU Background Rendering Guide](../ppu-background.md)). Ce visualiseur
de débogage, comme celui du chapitre 26, choisit la simplification la
plus commune en pratique, documentée explicitement ici plutôt que
cachée.

## Le bug : interroger le terminal à chaque frame

En ajoutant cet affichage (et plus tôt celui du chapitre 26), chaque
fonction de rendu ré-exécutait sa propre détection de terminal :

```rust
// AVANT, dans game.rs / vram_registers.rs / vram_tilemap_registers.rs,
// à CHAQUE frame (jusqu'à 60 fois par seconde, x3 widgets) :
let picker = Picker::from_query_stdio().expect("impossible de détecter le terminal");
```

`Picker::from_query_stdio()` envoie des séquences d'échappement au
terminal et **bloque** en attendant sa réponse, pour savoir quel
protocole d'image il supporte (Kitty, iTerm2, Sixel...). C'est une
opération ponctuelle par nature — le terminal ne change pas de capacités
en cours de route — mais le code la refaisait à chaque image, sur le
même thread que celui qui fait tourner le CPU/APU et remplit le buffer
audio.

Tant que ce thread attend la réponse du terminal, plus rien d'autre
n'avance : le buffer audio se vide (son saccadé), et les touches ne sont
relues qu'à la frame suivante (`q` lent à être pris en compte). Sur
macOS/Ghostty l'aller-retour est rapide, le symptôme est discret ; sous
WSL, il traverse la couche d'interop vers la console Windows et peut
prendre plusieurs secondes — d'où des coupures audio de 5 à 10 secondes.

## Le correctif : détecter une fois, réutiliser partout

```rust
// src/display/ratatui_display.rs
pub struct RatatuiDisplay {
    running: bool,
    terminal: DefaultTerminal,
    picker: Picker, // détecté une seule fois
}

impl RatatuiDisplay {
    pub fn new() -> Self {
        RatatuiDisplay {
            running: true,
            terminal: ratatui::init(),
            picker: Picker::from_query_stdio().expect("impossible de détecter le terminal"),
        }
    }
}
```

Chaque fonction de rendu reçoit maintenant `picker: &Picker` en
paramètre au lieu de le recréer :

```rust
pub fn render(frame: &mut Frame, picker: &Picker, area: Rect, vram: &[u8]) {
    // ...
    let image = picker.new_protocol(dyn_img, size, Resize::Scale(None)).unwrap();
    // ...
}
```

`picker.new_protocol(...)` reste appelé à chaque frame — c'est normal et
nécessaire, puisque l'image elle-même change à chaque frame. Ce qui ne
doit **pas** être refait, c'est la détection du terminal lui-même.

### Un détail de borrow checker instructif

```rust
fn draw(&mut self, emulator: &CrabbyBoy) {
    self.terminal
        .draw(|frame| Self::draw_ui(&self.picker, frame, emulator))
        .ok();
}
```

`self.terminal.draw(...)` emprunte `self.terminal` mutablement ; la
closure, elle, n'emprunte que `self.picker`. Depuis l'édition 2021 de
Rust, les closures capturent les *champs précis* qu'elles utilisent
plutôt que `self` en entier — ces deux emprunts portent donc sur des
champs disjoints du même struct, et le compilateur les autorise sans
conflit.

## Ce que nous avons maintenant

- Un vrai rendu de tile map (pas seulement un balayage brut de la VRAM),
  complétant le visualiseur du chapitre 26 et concrétisant le chapitre
  28.
- La détection du terminal faite une seule fois au démarrage, au lieu
  d'à chaque frame — un bug de performance invisible sur un terminal
  rapide, mais sévère sous WSL, maintenant corrigé.

## Ce qui manque encore

- Toujours pas de défilement (`SCX`/`SCY`), ni de palette (`BGP`), ni de
  fenêtre, ni de sprites — ces vues restent des outils de débogage bruts.
- Même sans requête terminal répétée, le rendu terminal reste en soi une
  opération d'E/S plus coûteuse qu'une simple avance de CPU — sous WSL
  en particulier, des coupures peuvent encore survenir si le rendu
  lui-même traîne. Le chapitre 31 s'attaque à cette dernière source de
  couplage en déplaçant l'émulation vers son propre thread. Avant cela,
  le chapitre 30 ajoute un visualiseur pour la dernière zone mémoire
  encore jamais affichée : l'OAM.
