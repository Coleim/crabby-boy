# PPU — Guide du rendu du fond (background)

Ce guide explique, étape par étape, comment faire en sorte que votre PPU
dessine la couche de fond (background) sur un écran de 160x144 pixels.
On avance lentement. Rien n'est présupposé. À la fin, vous comprendrez
exactement quelles données se trouvent où, et comment elles se
transforment en pixels.

> **Portée de ce guide : uniquement le fond (background).** Pas de
> window, pas de sprites. Ces éléments viendront plus tard, par-dessus
> ce que vous construisez ici.

Les documents de référence officiels (Pan Docs) sont liés tout au long
du texte. En cas de doute, allez lire la page liée — ce guide simplifie
les choses pour les rendre plus faciles à apprendre, les docs officielles
restent la vérité de référence.

## 1. La vue d'ensemble

Chaque 1/60e de seconde, la Game Boy dessine une image complète (une
« frame ») à l'écran. L'écran fait 160 pixels de large et 144 pixels de
haut.

L'image n'est PAS dessinée d'un seul coup. Elle est dessinée **une ligne
horizontale à la fois**, du haut (ligne 0) vers le bas (ligne 143), de
gauche à droite sur chaque ligne. Cela correspond à la façon dont
fonctionnaient physiquement les anciens écrans à tube cathodique (CRT) :
un faisceau balaie horizontalement, puis descend, et recommence.

Une puce appelée le **PPU** (Pixel Processing Unit) est responsable de
cela. Elle fonctionne en continu, en parfaite synchronisation avec le
CPU, en comptant le temps en unités appelées **dots**. Une ligne complète
prend exactement **456 dots**. Il y a **154 lignes** au total par frame
(pas 144 — on y revient dans un instant).

Référence : <https://gbdev.io/pandocs/Rendering.html>

### Pourquoi 154 lignes, et pas 144 ?

L'écran n'a que 144 lignes visibles. Mais le PPU continue de passer du
temps à « faire semblant » de dessiner 10 lignes invisibles
supplémentaires (lignes 144 à 153) une fois l'image réelle terminée. Ce
temps supplémentaire s'appelle **VBlank** (« vertical blank »), et il
existe pour que le jeu dispose d'une fenêtre sûre pour mettre à jour des
choses avant que la frame suivante ne commence à se dessiner. Rien n'est
dessiné à l'écran pendant ces 10 lignes — pensez-y comme une pause.

Donc : 144 lignes réelles + 10 lignes de « pause » = 154 lignes par
frame.

## 2. Les 4 modes, et ce qui se passe pendant chaque ligne

Pour chacune des 144 premières lignes, le PPU traverse 3 phases, toujours
dans le même ordre. Puis, une fois par frame, après la ligne 143, il
passe 10 lignes entières dans une 4e phase (VBlank). Voici la timeline
pour UNE ligne visible (disons, la ligne 0) :

```text
dot:     0 ────────────── 80 ────────────── 252 ──────────────── 456
mode:    |   Mode 2        |     Mode 3       |      Mode 0        |
         |   OAM Scan      |     Drawing      |      HBlank        |
         |   (80 dots)     |   (172 dots)     |   (204 dots)       |
```

- **Mode 2 — OAM Scan** (les 80 premiers dots de la ligne) : le PPU
  vérifierait normalement quels sprites apparaissent sur cette ligne.
  **Nous ne traitons pas les sprites dans ce guide**, donc pour nous
  cette phase ne fait actuellement rien — c'est juste un espace réservé.
  Sachez simplement qu'elle existe et qu'elle prend 80 dots.

- **Mode 3 — Drawing** (les 172 dots suivants, dans notre version
  simplifiée) : c'est la phase importante. C'est là que le PPU détermine
  réellement la couleur de chacun des 160 pixels de cette ligne et les
  écrit à l'écran. **C'est le cœur de ce que ce guide vous apprend à
  construire.**

  > Remarque : sur le hardware réel, la durée du Mode 3 varie (172 à 289
  > dots) selon les sprites, la couche window et la position de scroll.
  > Nous utilisons délibérément une durée **fixe de 172 dots** pour
  > simplifier. C'est une simplification connue — voir la
  > [section 7](#7-ce-que-ce-guide-ne-couvre-délibérément-pas) pour ce
  > que le hardware réel fait différemment.

- **Mode 0 — HBlank** (les dots restants jusqu'à 456) : rien ne se passe,
  c'est juste du temps mort pour que la ligne entière totalise exactement
  456 dots.

Après les 144 lignes visibles, on obtient :

- **Mode 1 — VBlank** (10 lignes x 456 dots = 4560 dots) : le PPU est
  inactif, la frame est considérée comme terminée et prête à être
  affichée. Une interruption se déclenche ici pour que le code du jeu
  sache qu'une nouvelle frame vient de se terminer.

Référence (tableau des durées de modes) :
<https://gbdev.io/pandocs/Rendering.html#ppu-modes>

## 3. D'où viennent réellement les données de l'image ?

C'est la partie qui déroute souvent les gens : **la VRAM ne contient pas
de pixels.** Elle contient deux types de choses différents, et vous avez
besoin des deux pour produire un pixel :

1. **Les données de tuile (tile data)** : les formes réelles (images de
   8x8 pixels), stockées dans un format compressé de 2 bits par pixel.
2. **La tilemap (tile map)** : une grille de 32x32 nombres, où chaque
   nombre dit « placer la tuile (tile) n°N ici ». C'est comme une
   feuille d'instructions de mosaïque : « la tuile 5 va en haut à
   gauche, la tuile 12 va juste à côté », etc.

Donc, pour trouver la couleur d'un pixel, vous devez :

1. Déterminer quelle tuile (depuis la TILEMAP) couvre ce pixel.
2. Rechercher les données d'image réelles de cette tuile (depuis la zone
   de données de tuile).
3. Extraire le seul pixel dont vous avez besoin de cette image de tuile
   8x8.

Passons en revue chaque élément en détail.

### 3.1 La tilemap

La VRAM contient **deux** tilemaps possibles, à des adresses fixes :

- `$9800`–`$9BFF`
- `$9C00`–`$9FFF`

Chacune fait 32x32 = 1024 octets. Chaque octet est un **index de tuile**
(0–255), indiquant quelle tuile de la zone de données de tuile dessiner à
cette position de la grille.

Puisque chaque tuile fait 8x8 pixels, une tilemap complète de 32x32
représente une image de 256x256 pixels — bien plus grande que l'écran de
160x144. Seule une fenêtre de 160x144 dans cette image plus grande est
affichée à un instant donné (plus de détails dans la
[section 3.4](#34-le-scroll-scx--scy), le scroll).

Quelle tilemap des deux est utilisée pour le fond est contrôlé par le
**bit 3** du registre LCDC (`$FF40`) :

- bit 3 = 0 → utiliser `$9800`
- bit 3 = 1 → utiliser `$9C00`

Référence : <https://gbdev.io/pandocs/Tile_Maps.html>
Référence (bits de LCDC) : <https://gbdev.io/pandocs/LCDC.html>

### 3.2 Les données de tuile (les formes de pixels réelles)

Chaque tuile fait 8x8 pixels, mais un pixel Game Boy n'est pas une
couleur complète — c'est juste un **nombre de 2 bits (0, 1, 2 ou 3)**,
appelé un « index de couleur ». La couleur que ce nombre représente
réellement à l'écran est décidée plus tard par une palette
([section 3.5](#35-la-palette-bgp)).

Chaque tuile est stockée sur **16 octets** : 2 octets par ligne, 8
lignes. Pourquoi 2 octets par ligne de 8 pixels ? Parce que chaque pixel
nécessite 2 bits, et 8 pixels x 2 bits = 16 bits = exactement 2 octets.
Les deux octets fonctionnent ensemble : pour un pixel donné à la colonne
`c` (0 = le plus à gauche, 7 = le plus à droite) :

```text
bit_position = 7 - c          // bit 7 is the leftmost pixel, bit 0 is rightmost
low_bit  = (first_byte  >> bit_position) & 1
high_bit = (second_byte >> bit_position) & 1
color_index = (high_bit << 1) | low_bit   // combine into a 0-3 value
```

C'est exactement la logique de décodage déjà écrite dans
`src/display/vram_registers.rs` (`generate_buffer`), que vous avez déjà
fait fonctionner pour le visualiseur de tuiles VRAM. Bonne nouvelle :
vous n'avez pas besoin de réinventer cette partie, réutilisez simplement
la même formule.

Voici cette logique encapsulée dans une fonction réutilisable — étant
donné l'**adresse de données résolue** d'une tuile (la section 3.3
ci-dessous calcule cela) et une **position à l'écran** où la dessiner,
elle décode les 8 lignes de cette tuile et écrit les pixels colorés
directement dans le buffer de frame (frame buffer) :

```rust
/// Decodes one 8x8 tile's pixel data and writes it into the frame buffer
/// at the given screen position, applying the BGP palette.
fn draw_tile(
    vram: &[u8],
    tile_data_addr: usize,
    bgp: u8,
    frame_buffer: &mut [u8],
    screen_x: usize,
    screen_y: usize,
) {
    for pixel_row in 0..8 {
        let lo = vram[(tile_data_addr + pixel_row * 2) - 0x8000];
        let hi = vram[(tile_data_addr + pixel_row * 2 + 1) - 0x8000];

        for pixel_col in 0..8 {
            let bit = 7 - pixel_col;
            let color_index = ((hi >> bit) & 1) << 1 | ((lo >> bit) & 1);
            let shade = (bgp >> (color_index * 2)) & 0b11; // section 3.5

            let x = screen_x + pixel_col;
            let y = screen_y + pixel_row;
            frame_buffer[y * 160 + x] = shade;
        }
    }
}
```

Remarquez que cette fonction ne sait pas, et ne se soucie pas, de savoir
*pourquoi* elle est appelée — elle se contente de dessiner une tuile à
une position d'écran. C'est délibéré : la section 4 l'appelle depuis une
boucle naïve plein écran, et elle sera tout aussi utilisable plus tard
depuis l'intérieur du véritable fetcher par dot (section 5).

Référence : <https://gbdev.io/pandocs/Tile_Data.html>

### 3.3 Trouver l'adresse d'une tuile à partir de son index

Voici une bizarrerie : il existe **deux façons différentes**
d'interpréter un nombre d'index de tuile en une adresse mémoire, et le
jeu choisit laquelle via le **bit 4 de LCDC** :

- **Si le bit 4 de LCDC = 1** (mode « non signé ») : l'index de tuile est
  simplement utilisé tel quel, de 0 à 255, à partir de l'adresse
  `$8000`.

  ```text
  address = 0x8000 + tile_index * 16
  ```

- **Si le bit 4 de LCDC = 0** (mode « signé ») : l'index de tuile est
  traité comme un nombre signé (-128 à 127 !), et l'adresse de base est
  `$9000` à la place.

  ```text
  address = 0x9000 + (tile_index_as_signed_byte) * 16
  ```

  Donc ici, un index de 0 signifie toujours `$9000`, mais un index de
  255 (qui, en tant qu'octet signé, vaut -1) signifie
  `$9000 - 16 = $8FF0`.

Pourquoi cela existe-t-il ? Des raisons historiques liées au hardware —
les deux modes se chevauchent dans la plage mémoire `$8800`–`$97FF`, qui
est partagée. Vous n'avez pas besoin de savoir pourquoi, seulement que
vous devez vérifier le bit 4 de LCDC avant de calculer l'adresse.

Sous forme de fonction réutilisable :

```rust
/// Resolves a tile index to its tile data address, per LCDC bit 4.
fn resolve_tile_data_addr(tile_index: u8, lcdc: u8) -> usize {
    if lcdc & 0b0001_0000 != 0 {
        // Unsigned addressing mode
        0x8000 + (tile_index as usize) * 16
    } else {
        // Signed addressing mode
        (0x9000_i32 + (tile_index as i8 as i32) * 16) as usize
    }
}
```

Référence : <https://gbdev.io/pandocs/Tile_Data.html> — voir
« Addressing modes ».

### 3.4 Le scroll (SCX / SCY)

Rappelez-vous que la tilemap représente une image de 256x256, mais que
l'écran n'en montre que 160x144. Les registres **SCX** (`$FF43`) et
**SCY** (`$FF42`) indiquent « quel pixel de cette grande image 256x256
apparaît dans le coin supérieur gauche de l'écran ? »

Donc pour le pixel d'écran `(x, y)`, le pixel correspondant dans la
grande image de fond 256x256 est :

```text
bg_x = (x + SCX) % 256
bg_y = (y + SCY) % 256
```

Le `% 256` (modulo) est important : si le scroll vous pousse au-delà du
bord de l'image 256x256, il **boucle** vers l'autre côté, comme un motif
de papier peint répétitif, plutôt que de simplement afficher un espace
vide.

Référence : <https://gbdev.io/pandocs/Scrolling.html>

### 3.5 La palette (BGP)

Une fois que vous avez un index de couleur (0–3) pour un pixel, ce n'est
toujours pas une couleur finale — c'est un index dans une table de
correspondance à 4 entrées appelée **BGP** (`$FF47`, « BG Palette »). BGP
est un octet qui regroupe 4 valeurs distinctes de 2 bits, une par index
de couleur possible :

```text
BGP byte:   bit7 bit6 | bit5 bit4 | bit3 bit2 | bit1 bit0
             shade for    shade for   shade for   shade for
             index 3      index 2     index 1     index 0
```

Pour obtenir la teinte finale (également 0–3, mais signifiant maintenant
une véritable nuance de gris : 0=blanc, 3=noir, approximativement) pour
un `color_index` donné :

```text
shade = (BGP >> (color_index * 2)) & 0b11
```

Référence : <https://gbdev.io/pandocs/Palettes.html>

## 4. Assembler le tout : la méthode simple (mais incorrecte)

Juste pour bâtir l'intuition, voici la façon « évidente mais pas la
façon dont le hardware fonctionne réellement » de dessiner une frame
entière : parcourir chaque **tuile** qui tient à l'écran (pas chaque
pixel), rechercher son index dans la tilemap, la résoudre et la
dessiner, en utilisant les deux fonctions des sections 3.2 et 3.3 :

```rust
let tile_map_base: usize = 0x9800; // section 3.1 — assumes LCDC bit 3 = 0

for tile_row in 0..18 {       // 144 / 8 = 18 tile rows fit on screen
    for tile_col in 0..20 {   // 160 / 8 = 20 tile columns fit on screen
        let offset = tile_row * 32 + tile_col;
        let tile_index = vram[(tile_map_base + offset) - 0x8000];
        let tile_data_addr = resolve_tile_data_addr(tile_index, lcdc); // section 3.3

        draw_tile(vram, tile_data_addr, bgp, &mut frame_buffer, tile_col * 8, tile_row * 8); // section 3.2
    }
}
```

Quelques points à noter sur cet exemple :

- `18` et `20`, pas `144`/`160` — ceci parcourt les **tuiles qui tiennent
  à l'écran**, pas les pixels individuels. Chaque itération traite un
  bloc entier de 8x8 d'un coup, via `draw_tile`.
- Ceci ignore totalement le scroll `SCX`/`SCY` (suppose que les deux
  valent 0, et lit toujours la tilemap 1) — c'est très bien comme point
  de départ réellement « naïf pour une première scène », mais vous
  devriez revenir dessus (en appliquant les formules de la section 3.4
  avant de calculer `tile_row`/`tile_col`) une fois que le scroll
  compte.
- Pour une seule frame statique, cela produit exactement les mêmes
  pixels qu'une boucle pixel par pixel qui recalculerait
  `tile_row = y / 8` / `tile_col = x / 8` pour chacun des 160x144 pixels
  individuellement — c'est simplement moins redondant, puisque la
  recherche/le décodage de chaque tuile se fait une fois ici au lieu de
  64 fois.

Cela produit une image correcte ! Mais il y a un problème plus profond :
cela calcule la **frame entière** d'un seul coup, ce qui **n'est pas ce
que fait le véritable hardware du PPU**. Le hardware réel produit les
pixels un par un, progressivement, ligne par ligne, pendant le Mode 3
(172 dots) de chaque ligne. Nous voulons construire le mécanisme réel,
parce que :

- Ce n'est vraiment pas beaucoup plus difficile.
- C'est l'architecture réelle dont vous aurez besoin plus tard pour la
  window/les sprites.
- Cela correspond aux Pan Docs, donc vous pouvez comparer votre code
  directement à la spécification.

Alors construisons plutôt la version réelle.

## 5. Le mécanisme réel : le pipeline (FIFO) de pixels et le fetcher

Le hardware réel utilise deux éléments qui coopèrent, fonctionnant
uniquement pendant le Mode 3 :

- **Le fetcher** : une petite machine séquentielle qui lit la tilemap et
  les données de tuile ([sections 3.1–3.3](#31-la-tilemap)) et prépare
  une ligne de 8 pixels à la fois.
- **Le FIFO** (file « First In First Out ») : un buffer qui peut
  contenir jusqu'à 16 pixels en attente. Le fetcher y pousse des lots de
  8 pixels. À chaque dot, si le FIFO a des pixels en attente, l'un
  d'eux est retiré et dessiné à l'écran.

Pensez-y comme un tapis roulant : le fetcher est un ouvrier qui pose des
pixels sur le tapis par lots de 8, et l'écran retire un pixel du tapis
par dot. Tant que l'ouvrier suit le rythme (ne laisse pas le tapis se
vider), les pixels s'écoulent de façon régulière.

Référence : <https://gbdev.io/pandocs/pixel_fifo.html>

### 5.1 Les 5 étapes du fetcher

Le fetcher répète ce cycle de 5 étapes, indéfiniment, pendant le Mode 3.
Voici l'état dont il a besoin, et une esquisse de chaque étape sous
forme de code, en réutilisant `resolve_tile_data_addr` de la section
3.3 :

```rust
enum FetcherStep { GetTile, GetTileDataLow, GetTileDataHigh, Sleep, Push }

struct Ppu {
    bg_fifo: VecDeque<u8>,       // pending color indices (0-3)
    fetcher_step: FetcherStep,
    fetcher_tile_col: u8,        // which tile map column we're fetching (0-31)
    tile_index: u8,              // scratch: tile index just read
    tile_data_lo: u8,            // scratch: low byte of the tile row
    tile_data_hi: u8,            // scratch: high byte of the tile row
    lcd_x: u8,                   // next screen column to output (0-159)
    lcd_y_coord: u8,             // current scanline (LY)
    scx: u8,
    scy: u8,
    lcdc: u8,
    bgp: u8,
    // ... dots, mode, etc. from the minimal PPU you already have
}

fn tick_fetcher(ppu: &mut Ppu, vram: &[u8]) {
    match ppu.fetcher_step {
        FetcherStep::GetTile => {
            // section 3.1: which tile map, which row/col
            let tile_map_base: usize = if ppu.lcdc & 0b0000_1000 != 0 { 0x9C00 } else { 0x9800 };
            let tile_row = ((ppu.lcd_y_coord as u16 + ppu.scy as u16) % 256) / 8;
            let offset = tile_row as usize * 32 + ppu.fetcher_tile_col as usize;
            ppu.tile_index = vram[(tile_map_base + offset) - 0x8000];
            ppu.fetcher_step = FetcherStep::GetTileDataLow;
        }
        FetcherStep::GetTileDataLow => {
            let addr = resolve_tile_data_addr(ppu.tile_index, ppu.lcdc); // section 3.3
            let row = (ppu.lcd_y_coord as u16 + ppu.scy as u16) % 8;
            ppu.tile_data_lo = vram[(addr + row as usize * 2) - 0x8000];
            ppu.fetcher_step = FetcherStep::GetTileDataHigh;
        }
        FetcherStep::GetTileDataHigh => {
            let addr = resolve_tile_data_addr(ppu.tile_index, ppu.lcdc);
            let row = (ppu.lcd_y_coord as u16 + ppu.scy as u16) % 8;
            ppu.tile_data_hi = vram[(addr + row as usize * 2 + 1) - 0x8000];
            ppu.fetcher_step = FetcherStep::Sleep;
        }
        FetcherStep::Sleep => {
            ppu.fetcher_step = FetcherStep::Push;
        }
        FetcherStep::Push => {
            if ppu.bg_fifo.is_empty() {
                // same 2bpp decode formula as draw_tile (section 3.2),
                // just pushed into the FIFO instead of written directly
                for pixel_col in 0..8 {
                    let bit = 7 - pixel_col;
                    let color_index = ((ppu.tile_data_hi >> bit) & 1) << 1 | ((ppu.tile_data_lo >> bit) & 1);
                    ppu.bg_fifo.push_back(color_index);
                }
                ppu.fetcher_tile_col = (ppu.fetcher_tile_col + 1) & 0x1F;
                ppu.fetcher_step = FetcherStep::GetTile;
            }
            // if the FIFO wasn't empty, stay on this step and retry next dot
        }
    }
}
```

Chacune des étapes `GetTile`, `GetTileDataLow`, `GetTileDataHigh` et
`Sleep` prend 2 dots (donc en pratique vous ne feriez avancer la machine
à états que tous les deux dots — simplifié ici par souci de clarté).
`Push` est tentée à chaque dot jusqu'à ce qu'elle réussisse. Donc :
2+2+2+2 = 8 dots minimum par tuile, plus éventuellement des dots
supplémentaires passés à réessayer `Push` si le FIFO ne s'est pas encore
vidé.

### 5.2 Chaque dot, indépendamment : essayer de produire un pixel

En même temps que le fetcher fait son travail, **à chaque dot**, pendant
le Mode 3, ceci se produit aussi :

```rust
fn output_pixel(ppu: &mut Ppu, frame_buffer: &mut [u8]) {
    if let Some(color_index) = ppu.bg_fifo.pop_front() {
        let shade = (ppu.bgp >> (color_index * 2)) & 0b11; // section 3.5
        let y = ppu.lcd_y_coord as usize;
        let x = ppu.lcd_x as usize;
        frame_buffer[y * 160 + x] = shade;
        ppu.lcd_x += 1;
    }
    // if the FIFO was empty, do nothing this dot — the screen waits for
    // the fetcher to catch up
}
```

Une fois que `lcd_x` atteint 160, le dessin de cette ligne est terminé —
basculez immédiatement en Mode 0 (HBlank), quelle que soit l'étape en
cours du fetcher.

### 5.3 Pourquoi s'embêter avec cela plutôt qu'avec la méthode simple ?

Parce que c'est véritablement ainsi que fonctionne le hardware, et que
la « méthode simple » de la
[section 4](#4-assembler-le-tout-la-méthode-simple-mais-incorrecte) est
un raccourci qui ne produit la même image finale **que parce que nous ne
faisons pas encore quoi que ce soit qui change en cours de ligne** (comme
basculer vers la couche window au milieu d'une rangée, ou mélanger des
sprites). Une fois que vous voudrez prendre en charge ces
fonctionnalités, vous aurez besoin de la véritable machinerie
fetcher+FIFO de toute façon — autant la construire maintenant et éviter
une réécriture plus tard.

## 6. Ce que vous devez ajouter à votre code (checklist)

Sur votre structure `PPU`, ajoutez :

- `frame_buffer: [u8; 160 * 144]` — une teinte (0–3) par pixel, stockant
  le pixel `(x, y)` à l'index `y * 160 + x`.
- `bg_fifo: VecDeque<u8>` — contient les index de couleur en attente
  (0–3), taille maximale 16 en théorie (nous n'approcherons jamais
  vraiment cette limite dans cette version simple).
- `fetcher_step` — une petite énumération : `GetTile`, `GetTileDataLow`,
  `GetTileDataHigh`, `Sleep`, `Push`.
- `fetcher_tile_col: u8` — sur quelle colonne (0–31) de la tilemap le
  fetcher travaille actuellement pour cette ligne.
- `lcd_x: u8` — quelle colonne d'écran (0–159) est sur le point d'être
  écrite ensuite.
- Des champs de travail pour mémoriser les données entre les étapes du
  fetcher : `tile_index: u8`, `tile_data_lo: u8`, `tile_data_hi: u8`.
- L'énumération `mode: PpuMode` : `OamScan`, `Drawing`, `HBlank`,
  `VBlank` — pour toujours savoir dans laquelle des 4 phases
  ([section 2](#2-les-4-modes-et-ce-qui-se-passe-pendant-chaque-ligne))
  vous vous trouvez.

Comportement à ajouter, piloté par votre compteur `dots` existant :

1. En entrant en Mode 3 (dots == 80) pour une ligne : réinitialiser
   `lcd_x = 0`, `fetcher_tile_col = 0`, `fetcher_step = GetTile`, vider
   `bg_fifo`.
2. Pendant le Mode 3 : faire avancer la machine à états du fetcher
   ([section 5.1](#51-les-5-étapes-du-fetcher)) et tenter de produire un
   pixel
   ([section 5.2](#52-chaque-dot-indépendamment-essayer-de-produire-un-pixel)),
   à chaque dot.
3. Quand `lcd_x` atteint 160 : basculer en Mode 0 pour le reste de la
   ligne.
4. Si le bit 0 de LCDC vaut 0 (fond désactivé) : ignorer tout ce qui
   précède, écrire simplement la teinte 0 (blanc) pour toute la ligne.

> **Remarque d'implémentation :** votre `PPU::tick()` existant fait
> avancer `dots` de 4 à la fois (l'équivalent d'une étape d'instruction
> CPU), pas dot par dot. La correction la plus simple : à l'intérieur de
> `tick()`, exécutez la « logique d'un dot du fetcher + sortie de pixel »
> dans une petite boucle, 4 fois, plutôt que de réécrire toute votre
> boucle de timing pour qu'elle soit appelée une fois par dot.

## 7. Ce que ce guide ne couvre délibérément pas

Pour rester accessible, les comportements réels suivants sont omis. Ils
ne cassent rien de ce que vous construisez ici — ce sont des ajouts pour
plus tard :

- **La couche window** : pas du tout traitée pour l'instant. Bonne
  nouvelle : elle réutilise exactement ce même fetcher et ce même FIFO,
  simplement avec une condition de déclenchement qui change quelle
  tilemap/ligne est lue. Ce n'est pas une réécriture, c'est un ajout.
- **Les sprites (OBJs)** : pas du tout traités pour l'instant. Celui-ci
  EST un sous-système séparé (son propre FIFO, sa propre logique de scan
  OAM, et des règles de mélange de pixels) — plus de travail, ajouté
  plus tard par-dessus tout ceci.
- **Le Mode 3 de longueur variable** : le Mode 3 du hardware réel peut
  prendre de 172 à 289 dots selon le scroll/les sprites/la window. Nous
  utilisons une durée fixe de 172. Voir
  <https://gbdev.io/pandocs/Rendering.html#mode-3-length> pour les
  règles réelles, quand vous serez prêt pour elles.
- **Les interruptions STAT** et les **restrictions de timing du DMA
  OAM** : non couvertes ici.

## 8. Résumé-aide-mémoire

| Concept | Où | Référence |
|---|---|---|
| Frame = 154 lignes x 456 dots | — | Rendering.html |
| Ligne = Mode 2 (80) + Mode 3 (172, simplifié) + Mode 0 (le reste) | — | Rendering.html#ppu-modes |
| Tilemap (quelle tuile va où) | `$9800`/`$9C00`, choisie par le bit 3 de LCDC | Tile_Maps.html |
| Données de tuile (formes de pixels réelles) | base `$8000`/`$9000`, choisie par le bit 4 de LCDC | Tile_Data.html |
| Formule de décodage de pixel 2bpp | réutilisée depuis `vram_registers.rs` | Tile_Data.html |
| Scroll | SCX (`$FF43`), SCY (`$FF42`), boucle modulo 256 | Scrolling.html |
| Palette | BGP (`$FF47`), 2 bits par index de couleur | Palettes.html |
| Mécanisme réel par pixel | Fetcher (5 étapes) + FIFO | pixel_fifo.html |
