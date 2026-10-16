# 28. La VRAM a deux étages : les tiles, et la tile map

Ce chapitre, comme les chapitres 32 et 33 plus loin, n'introduit aucun
nouveau commit — c'est une pause conceptuelle, nécessaire avant de
comprendre le code du chapitre 29. Le visualiseur du chapitre 26 décode
déjà fidèlement des tuiles individuelles ; mais pour comprendre *ce que
le jeu affiche réellement à l'écran*, il manque encore une pièce : la
**tile map**.

## Deux types de données, dans deux zones différentes de la VRAM

La VRAM (`$8000`–`$9FFF`, chapitre 0) contient en réalité deux choses
bien distinctes, qu'il ne faut jamais confondre :

1. **Les données de tuile (tile data)**, en `$8000`–`$97FF` — c'est ce
   que le chapitre 26 décode déjà. Chaque bloc de 16 octets décrit une
   image 8×8 pixels, 2 bits par pixel (4 nuances possibles). Pensez-y
   comme à une *réserve de dessins* : chaque tuile est un petit motif
   réutilisable (un bloc d'herbe, un coin de mur, une lettre d'un menu de
   texte...), sans aucune idée de *où* il sera affiché à l'écran.
2. **La tile map**, en `$9800`–`$9BFF` ou `$9C00`–`$9FFF` — une grille
   de **32×32 nombres**, où chaque nombre est un *tile index* : littéralement
   "à cette case de la grille, dessine la tuile numéro X de la réserve
   ci-dessus". Chaque tile map fait 32×32 = 1024 octets.

Décrit autrement : la tile data est l'**alphabet** (chaque lettre/dessin
disponible), la tile map est le **texte** (quelle lettre va à quelle
position). Le visualiseur du chapitre 26 montre l'alphabet entier, pêle-mêle ;
ce qui manque, c'est de lire le texte.

## La tile map, c'est en gros l'écran du jeu — en plus grand

Puisque chaque tuile fait 8×8 pixels, une tile map complète de 32×32
tuiles représente une image de **256×256 pixels**. Or l'écran de la
Game Boy ne fait que **160×144 pixels** (chapitre 0).

C'est la clé de tout : **la tile map n'est pas l'écran, elle est plus
grande que l'écran**. L'écran affiché n'est qu'une **fenêtre/sous-écran**
de 160×144 pixels, découpée quelque part à l'intérieur de cette image
256×256 — un peu comme une fenêtre qu'on déplace sur une grande affiche,
sans jamais voir l'affiche entière d'un coup.

Deux registres pilotent cette fenêtre :

- **`SCY`** (`$FF42`) et **`SCX`** (`$FF43`) : à quel pixel de la grande
  image 256×256 correspond le coin supérieur gauche de l'écran, à cet
  instant.
- Le jeu change ces deux valeurs pour faire défiler (scroller) la scène
  — Mario qui avance ne redessine pas tout le niveau à chaque pas, il se
  contente de déplacer la petite fenêtre de lecture sur une tile map qui,
  elle, ne bouge pas.

Si le défilement pousse la fenêtre au-delà du bord de l'image 256×256,
elle **boucle** vers l'autre côté (modulo 256) — comme un motif de papier
peint qui se répète.

## Deux emplacements possibles, et pourquoi

La Game Boy a en réalité **deux** tile maps disponibles en VRAM :
`$9800`–`$9BFF` et `$9C00`–`$9FFF`. À tout instant, le registre `LCDC`
(`$FF40`) décide laquelle des deux est utilisée pour le Background (bit 3)
— et, séparément, laquelle est utilisée pour la Window (bit 6, voir
chapitre 33). Un jeu peut dessiner son niveau dans l'une et son écran de
pause dans l'autre, et basculer instantanément entre les deux en
changeant un seul bit, sans avoir à réécrire quoi que ce soit en VRAM.

## Ce que ça change pour le code du chapitre suivant

Le visualiseur du chapitre 26 faisait : *"pour chaque position de la
grille d'affichage, prends directement la tuile à cette position dans
VRAM"* — un balayage brut, sans indirection. Le chapitre 29 ajoute
l'étage manquant : *"pour chaque position de la grille, lis d'abord dans
la tile map quel identifiant de tuile s'y trouve, puis va chercher les
pixels de cette tuile précise"*. Une seule ligne de différence dans le
code, mais c'est exactement la différence entre "montrer toute la
réserve de dessins" et "montrer ce que le jeu a choisi d'afficher".

## Ce que nous avons maintenant

- Un vocabulaire clair pour ne plus confondre tile data (l'alphabet) et
  tile map (le texte).
- La compréhension que l'écran visible n'est qu'une fenêtre 160×144 sur
  une tile map bien plus grande (256×256), déplacée via `SCX`/`SCY`.
- Le rôle du bit 3 (et du bit 6, pour la Window) de `LCDC` pour choisir
  entre les deux tile maps disponibles.

## Ce qui manque encore

- Rien n'est encore codé ici — c'est le chapitre 29 qui transforme cette
  compréhension en un vrai visualiseur de tile map dans le terminal.
- Le défilement (`SCX`/`SCY`) lui-même n'est pas encore appliqué nulle
  part dans le code de rendu — le guide
  [PPU — Background Rendering Guide](../ppu-background.md), section 3.4,
  s'en charge en détail pour le vrai rendu PPU.
