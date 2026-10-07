# 27. Là où nous avons laissé le PPU, et la suite

Ce chapitre de clôture n'introduit pas de nouveau commit — c'est une
pause délibérée pour faire le point, exactement là où le sous-titre de
ce livre promettait de s'arrêter : au PPU, le seul sous-système encore
au tout début de son parcours comparé à tout ce qui a été construit
jusqu'ici.

## Un rapide récapitulatif de tout le parcours

En regardant en arrière les parties de ce livre :

- **Parties I-II** nous ont donné un CPU capable de récupérer (fetch),
  décoder et exécuter (presque) chaque instruction du jeu d'instructions
  de la Game Boy, validé par rapport au `cpu_instrs.gb` de Blargg.
- **Partie III** a donné à ce CPU une véritable architecture `Bus`/
  `IOBridge` et le bank switching (banking) de ROM, afin que des jeux
  plus volumineux puissent même se charger.
- **Parties IV-V** ont ajouté un Timer, la distribution complète des
  interruptions, et un premier stub de PPU fidèle au timing (mais sans
  pixels) — suffisant pour déclencher correctement VBlank une fois par
  image (frame), au bon moment.
- **Partie VI** a construit une APU complète à 4 canaux, vérifiée par
  rapport à la suite `dmg_sound` de Blargg.
- **Parties VII-VIII** ont ajouté un stub de joypad, et une véritable
  architecture d'interface terminal (découplée de la vitesse
  d'émulation), aboutissant à une vue de débogage CPU/en-tête en direct
  et à un décodeur brut de tuiles VRAM.

## Ce qui existe pour le PPU spécifiquement, à l'heure actuelle

Exactement deux éléments, issus de deux chapitres différents :

1. **Le PPU minimal du chapitre 12** : comptage fidèle des dots/scanlines,
   suivi de `LY`, et une interruption VBlank correctement synchronisée.
   Pas de pixels, pas de modes STAT, pas d'accès VRAM depuis le PPU
   lui-même.
2. **Le visualiseur de tuiles VRAM du chapitre 26** : décodage fidèle des
   tuiles à 2 bits par pixel, affiché sous forme de grille de débogage
   brute — mais sans tile map, sans défilement, sans application de
   palette, et entièrement déconnecté du propre timing de scanline du
   PPU.

Entre ces deux éléments, presque tous les *ingrédients bruts* pour un
vrai rendu d'arrière-plan (background) existent déjà quelque part dans
cette base de code : un timing fidèle (chapitre 12) et un décodage de
tuiles fidèle (chapitre 26). Ce qui manque, c'est l'élément qui les
combine réellement en une vraie image (frame).

## Ce qui vient ensuite : un vrai rendu d'arrière-plan

Cette combinaison — lire la *tile map* (pas seulement les données de
tuile brutes), appliquer le défilement `SCX`/`SCY`, appliquer la vraie
palette `BGP`, et faire tout cela en synchronisation avec le timing réel
par dot du PPU en utilisant le véritable mécanisme de **pixel FIFO et de
fetcher** — est précisément le sujet du guide associé :

➡️ **[PPU — Background Rendering Guide](../ppu-background.md)**

Ce guide reprend exactement là où ce livre s'arrête, écrit dans le même
style "ne rien supposer, tout expliquer", et approfondit :

- Les 4 vrais modes du PPU (OAM Scan, Drawing, HBlank, VBlank) et leur
  timing précis par dot.
- Comment les tile maps, les données de tuile, le défilement et les
  palettes se combinent pour produire un pixel.
- La véritable machine à états du pixel FIFO/fetcher utilisée par le
  vrai matériel (hardware), construite délibérément (pas comme un
  raccourci), afin que la prise en charge de la fenêtre (window) et des
  sprites puisse être ajoutée plus tard sans réécriture.

## Au-delà de cela

Une fois qu'un vrai rendu d'arrière-plan existe, les prochaines étapes
naturelles (pas encore rédigées nulle part au moment de ce chapitre, mais
claires au vu de tout ce qui a été couvert jusqu'ici) sont : la couche de
fenêtre (window) (un ajout peu coûteux une fois que le fetcher
d'arrière-plan existe, puisqu'il réutilise le même mécanisme), et les
sprites (un sous-système véritablement séparé : un vrai OAM scan, un
second FIFO, et des règles de mélange par priorité de pixel) — plus la
finalisation du joypad (le chapitre 21 a laissé la véritable association
bouton-touche explicitement non construite) et les interruptions basées
sur STAT (le chapitre 14 a laissé le LCD STAT comme une adresse de
vecteur sans déclencheur réel pour l'instant).

C'est, authentiquement, là où en est le projet aujourd'hui. Le reste
s'écrira au fur et à mesure qu'il sera construit.
