# 26. Visualiser la VRAM — votre premier regard sur les données de tuiles

Ce chapitre (`cc4a9d1`) constitue le pont naturel vers la dernière partie
du livre : c'est la première fois que ce projet **décode et affiche
réellement des données graphiques de tuiles** — exactement les calculs de
décodage de pixels dont le guide associé
[PPU Background Rendering Guide](../ppu-background.md) a besoin, construit
ici d'abord comme une vue de débogage autonome, sans encore aucune
considération de timing/scanline du PPU.

## L'approche : décoder chaque tuile de la VRAM dans une seule grande image

```rust
// src/display/vram_registers.rs
pub fn render(frame: &mut Frame, area: Rect, vram: &[u8]) {
    let dyn_img = generate_buffer(vram);
    // ... même motif de rendu ratatui-image que les chapitres 22/25
}

const COLORS: [Srgb; 4] = [
    Srgb::new(1.0, 1.0, 1.0),
    Srgb::new(0.83, 0.83, 0.83),
    Srgb::new(0.5, 0.5, 0.5),
    Srgb::new(0.0, 0.0, 0.0),
];

fn generate_buffer(vram: &[u8]) -> DynamicImage {
    const TILES_PER_ROW: usize = 32;
    const BYTES_PER_TILE: usize = 16;
    let number_of_tiles = vram.len() / BYTES_PER_TILE;
    let tile_rows = number_of_tiles.div_ceil(TILES_PER_ROW);
    let width_px = TILES_PER_ROW * 8;
    let height_px = tile_rows * 8;

    let mut raw_pixels: Vec<u8> = Vec::with_capacity(width_px * height_px * 4);
    for y in 0..height_px {
        for x in 0..width_px {
            let tile_idx = (y / 8) * TILES_PER_ROW + (x / 8);
            let offset = tile_idx * BYTES_PER_TILE;
            let row = y % 8;
            let lo = vram.get(offset + row * 2).copied().unwrap_or(0);
            let hi = vram.get(offset + row * 2 + 1).copied().unwrap_or(0);

            let pixel_idx_x = 7 - (x % 8);
            let color_idx: u8 = (((hi >> pixel_idx_x) & 1) << 1) | ((lo >> pixel_idx_x) & 1);
            let color: Srgb<u8> = COLORS[color_idx as usize].into_format();

            raw_pixels.push(color.red);
            raw_pixels.push(color.green);
            raw_pixels.push(color.blue);
            raw_pixels.push(255);
        }
    }
    DynamicImage::ImageRgba8(RgbaImage::from_raw(width_px as u32, height_px as u32, raw_pixels).unwrap())
}
```

Si cette formule de décodage à 2 bits par pixel
(`(((hi >> bit) & 1) << 1) | ((lo >> bit) & 1)`) vous semble familière,
c'est parce que c'est exactement le même format de données de tuile
expliqué de manière conceptuelle dans la carte mémoire du chapitre 0 et
(indirectement) utilisé depuis la discussion sur l'interprétation des
octets du chapitre 2 — ceci est simplement le premier chapitre qui
*implémente* réellement sa lecture. Chaque bloc de 16 octets de VRAM est
traité comme une tuile 8×8 ; `TILES_PER_ROW = 32` les dispose toutes côte
à côte, 32 par ligne, en une seule grande image — ce qui n'est pas en
soi une "image" significative qu'un jeu afficherait telle quelle (elle
ignore entièrement les tile maps, le défilement et les palettes), mais
constitue une vue *brute* extrêmement utile : elle montre chaque graphique
8×8 distinct actuellement présent en VRAM, indépendamment de la façon
dont le PPU choisirait actuellement de les agencer à l'écran.

## Une palette fixe, codée en dur — et pourquoi c'est très bien ici

```rust
const COLORS: [Srgb; 4] = [
    Srgb::new(1.0, 1.0, 1.0), // index de couleur 0 → blanc
    Srgb::new(0.83, 0.83, 0.83),
    Srgb::new(0.5, 0.5, 0.5),
    Srgb::new(0.0, 0.0, 0.0), // index de couleur 3 → noir
];
```

Le chapitre 0 mentionnait que la nuance réelle affichée à l'écran pour un
index de couleur donné à 2 bits dépend du registre de palette `BGP`
(chapitre 7), pas seulement de l'index brut. Ce visualiseur de débogage
saute intentionnellement cette étape et associe l'index à une nuance
directement, 1 pour 1 — approprié pour un outil dont le but est "montre-
moi les *formes* brutes des tuiles indépendamment de la palette actuelle
d'un jeu en particulier", plutôt que "montre-moi exactement à quoi
ressemblerait l'écran en ce moment". Savoir quand une simplification est
appropriée pour l'outil que vous construisez (une vue de débogage brute)
par rapport à quand elle serait incorrecte (un affichage émulé fidèle)
est en soi une compétence utile.

## Ce que nous avons maintenant

- Un décodage fidèle des tuiles à 2 bits par pixel, réutilisable partout
  où de vraies données graphiques de tuiles doivent être transformées en
  pixels.
- Une vue rendue dans le terminal de chaque tuile actuellement stockée en
  VRAM, utilisant la même approche `ratatui-image` établie dans les
  chapitres 22 et 25.
- Chaque pièce que ce projet a construite jusqu'ici — CPU, bus, timer,
  interruptions, APU, stub de joypad, interface terminal, et maintenant
  un vrai décodage de tuiles — coexistant avec un PPU (chapitre 12) qui
  ne suit encore que le timing des scanlines.

## Ce qui manque encore

- Aucune *tile map* n'est lue (le concept de "tile map" du chapitre 0/du
  guide associé) — ceci montre chaque tuile existant en VRAM, pas
  lesquelles un jeu a effectivement choisi de placer où sur son
  arrière-plan (background).
- Pas de défilement (`SCX`/`SCY`), pas de palette (`BGP`), pas de fenêtre
  (window), pas de sprites — tous explicitement hors du champ de cet
  outil de débogage brut, et tous repris correctement dans le regard vers
  l'avenir du chapitre 27, puis dans les chapitres 28-29 et le guide
  [PPU — Background Rendering Guide](../ppu-background.md).
