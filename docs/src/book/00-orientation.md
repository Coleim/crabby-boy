# 0. Ce qu'il y a vraiment dans une Game Boy

Avant d'écrire la moindre ligne de Rust, construisons une carte mentale de
la machine que nous allons simuler. Vous n'avez pas besoin de mémoriser ce
chapitre — revenez-y chaque fois qu'un chapitre ultérieur mentionne un
terme dont vous ne vous souvenez plus.

## Que signifie réellement « émuler » une Game Boy ?

Une vraie Game Boy est un petit circuit imprimé avec quelques puces
spécialisées dessus. Chaque puce a un rôle :

- Un **CPU** (le « cerveau ») qui exécute le code d'un jeu, instruction par
  instruction.
- Un **PPU** (« Picture Processing Unit ») qui transforme les données en
  mémoire en pixels que vous voyez à l'écran.
- Un **APU** (« Audio Processing Unit ») qui génère le son.
- Des puces de **mémoire** (ROM sur la cartouche, RAM à l'intérieur de la
  console et parfois aussi sur la cartouche).
- Des **boutons** (le joypad).

« Émuler » la Game Boy signifie écrire un programme qui se comporte
exactement comme ce hardware se comporterait, instruction par instruction,
cycle par cycle — afin qu'un jeu original non modifié, qui n'a aucune idée
qu'il ne tourne pas sur du vrai silicium, fonctionne quand même
correctement. Nous ne réimplémentons pas des *jeux* ; nous réimplémentons
la *machine* sur laquelle ils tournent.

Tout ce livre consiste à construire, morceau par morceau, un modèle
logiciel de chaque puce ci-dessus, et à les relier entre elles exactement
comme le vrai circuit imprimé les relie.

## Le CPU : un Sharp LR35902

Le CPU de la Game Boy est une puce personnalisée, similaire à deux CPU
plus anciens et bien connus combinés ensemble (Z80 et 8080, si ces noms
vous disent quelque chose — sinon, ne vous en souciez pas). Ce qui compte
pour nous :

- Il possède une poignée de **registres** : de petits emplacements de
  stockage nommés qui contiennent chacun un octet (ou, combinés par
  paires, deux octets). Les registres sont nommés `A`, `B`, `C`, `D`, `E`,
  `H`, `L` (un octet chacun), plus un registre spécial `F` qui contient
  les **flags** (nous y reviendrons plus en détail), un **pointeur de
  pile** (stack pointer) 16 bits (`SP`), et un **compteur de programme**
  (program counter) 16 bits (`PC`) qui pointe toujours vers la prochaine
  instruction à exécuter.
- Il tourne en boucle : **récupérer** (fetch) l'octet à `PC`, **décoder**
  (decode) quelle instruction cet octet représente, l'**exécuter**
  (execute), et recommencer indéfiniment. C'est ce qu'on appelle le cycle
  fetch-decode-execute, et c'est le battement de cœur de tout CPU, pas
  seulement celui de la Game Boy.
- Chaque instruction prend un nombre fixe et connu de cycles (pensez à un
  cycle comme « un tic de l'horloge matérielle ») pour s'exécuter. Avoir
  ce timing correct compte énormément — nous verrons pourquoi plus en
  détail dans la Partie IV.

## La mémoire : un gigantesque tableau, divisé en zones

Le CPU de la Game Boy peut adresser 65 536 emplacements mémoire différents
(adresses `0x0000` à `0xFFFF` — c'est ce que 16 bits permettent :
2^16 = 65536). Cette plage entière s'appelle la **carte mémoire** (memory
map), et différents segments de celle-ci signifient des choses totalement
différentes :

| Plage d'adresses | Ce qui s'y trouve |
|---|---|
| `0x0000`–`0x3FFF` | ROM, banque 0 (fixe, provenant de la cartouche) |
| `0x4000`–`0x7FFF` | ROM, banque commutable (provenant de la cartouche, voir Chapitre 8) |
| `0x8000`–`0x9FFF` | VRAM — mémoire vidéo, lue par le PPU |
| `0xA000`–`0xBFFF` | RAM externe (sur la cartouche, si présente) |
| `0xC000`–`0xDFFF` | RAM de travail (Work RAM) — la RAM polyvalente propre à la console |
| `0xFE00`–`0xFE9F` | OAM — mémoire des attributs de sprites, également lue par le PPU |
| `0xFF00`–`0xFF7F` | Registres d'E/S (I/O) — c'est ainsi que le CPU communique avec le PPU, l'APU, le timer, le joypad, etc. |
| `0xFF80`–`0xFFFE` | RAM haute (High RAM) — une petite zone de travail supplémentaire |
| `0xFFFF` | Un seul octet : le registre d'activation des interruptions |

Voilà beaucoup de noms que vous n'avez jamais entendus — ne vous inquiétez
pas, chacun aura son propre chapitre. L'idée importante pour l'instant :
**lire ou écrire une « adresse mémoire » peut ne pas toucher à de la RAM
du tout** — selon l'adresse, cela pourrait plutôt lire un octet de ROM de
la cartouche, écrire une commande vers la puce sonore, ou basculer un bit
qui contrôle l'écran. Dans notre code, cette logique d'aiguillage vit dans
quelque chose que nous appellerons le **Bus** (Chapitre 6).

## Les registres d'E/S : la télécommande du CPU pour tout le reste

De nombreuses adresses dans `0xFF00`–`0xFF7F` ne sont pas du tout de la
mémoire — elles ressemblent plutôt à des boutons et cadrans étiquetés que
le CPU peut lire ou sur lesquels il peut écrire, pour contrôler ou
interroger toutes les autres puces. Par exemple (vous rencontrerez tous
ces éléments en détail plus tard) :

- `0xFF00` — état des boutons du joypad.
- `0xFF04`–`0xFF07` — le timer.
- `0xFF0F` et `0xFFFF` — flags d'interruption (quels événements sont en
  attente / autorisés à interrompre le CPU).
- `0xFF10`–`0xFF26` — canaux sonores.
- `0xFF40`–`0xFF4B` — contrôle du PPU (quoi dessiner et comment).

## Le PPU : dessiner une ligne à la fois

L'écran fait 160×144 pixels. Le PPU ne dessine pas toute l'image d'un
coup — il la dessine une ligne horizontale à la fois, de manière répétée,
60 fois par seconde, en lisant les données de tuiles depuis la VRAM et les
données de sprites depuis l'OAM. Nous aurons un premier aperçu, très
minimal, de tout cela dans la Partie V, et une implémentation beaucoup
plus profonde et réelle est construite dans le guide complémentaire
[Guide du rendu de l'arrière-plan du PPU](../ppu-background.md) juste
après le dernier chapitre de ce livre.

## L'APU : quatre canaux sonores

L'APU possède 4 « canaux » générateurs de son indépendants : deux ondes
carrées, une forme d'onde personnalisable, et un générateur de bruit, tous
mixés ensemble pour produire l'audio final que vous entendez. La Partie VI
couvre chacun d'eux.

## Les interruptions : le hardware tapant sur l'épaule du CPU

Parfois une puce a besoin de dire au CPU « quelque chose s'est produit,
mets en pause ce que tu fais et va t'en occuper » — une nouvelle frame a
fini d'être dessinée, un timer est arrivé à zéro, un bouton a été
appuyé. Ce mécanisme s'appelle une **interruption**, et il est
suffisamment central pour que l'ébauche de PPU (Partie V) et le timer
(Partie IV) ne deviennent véritablement utiles qu'une fois les
interruptions fonctionnelles. Nous couvrons cela correctement au
Chapitre 14.

## Une note sur « DMG »

Vous verrez l'abréviation **DMG** dans les noms de fichiers, les
descriptions de registres, et les ROM de test tout au long de ce livre et
de la communauté plus large des développeurs Game Boy. Cela signifie
« Dot Matrix Game » — le nom de code interne de Nintendo pour le hardware
original de la Game Boy de 1989 (par opposition aux modèles couleur/
advance ultérieurs). Ce projet cible uniquement la DMG.

## Et ensuite

Avec cette carte en main, le Chapitre 1 démarre le projet proprement dit :
lire un fichier ROM en mémoire et mettre en place les premiers fichiers
Rust.
