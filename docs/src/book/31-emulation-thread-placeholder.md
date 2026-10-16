# 31. Un thread dédié à l'émulation (à venir)

Comme le chapitre précédent, celui-ci est un espace réservé : au moment
de l'écrire, le code n'existe pas encore. Le plan détaillé, étape par
étape, vit déjà dans `THREAD_PPU.MD` à la racine du projet — ce chapitre
en résume la motivation et sera complété avec le vrai code une fois
implémenté et testé.

## Le symptôme restant après le chapitre 29

Même avec la détection du terminal mise en cache (chapitre 29), le son
peut encore se saccader et la touche `q` rester lente à réagir — en
particulier sous WSL. La cause de fond : l'émulation (CPU/APU, qui
remplit le buffer audio) et le rendu terminal tournent encore dans **une
seule boucle, un seul thread** (`main.rs`). L'écriture terminale
(tableaux, images Sixel/Kitty) reste une opération d'E/S potentiellement
lente — surtout sous WSL, à cause d'une couche d'interop supplémentaire
vers la console Windows. Tant qu'elle n'est pas terminée, rien d'autre
n'avance.

## L'idée : simulation et présentation sur deux threads séparés

C'est un pattern classique de moteur de jeu : la simulation (ici, le
CPU/l'APU) avance selon sa propre horloge réelle, mesurée en temps
réel (`Instant::now()`), indépendamment de la vitesse à laquelle
l'affichage parvient à se redessiner. La présentation (ici, le
terminal) suit du mieux qu'elle peut, sans jamais pouvoir ralentir la
simulation.

Concrètement :

- Un thread d'émulation dédié fait tourner `CrabbyBoy::tick_for_duration`
  en continu, sans jamais toucher au terminal.
- Un petit `RenderSnapshot` (quelques champs utiles à l'affichage — FPS,
  registres CPU, VRAM tiles/tilemap — copiés depuis l'état de
  l'émulateur) est partagé via un `Arc<Mutex<RenderSnapshot>>`.
- Le thread principal (affichage) verrouille ce `Mutex` juste le temps
  d'un `.clone()`, le relâche immédiatement, puis dessine dans le
  terminal **sans jamais tenir le verrou pendant cette E/S lente** — le
  point central de toute la correction.
- Un `Arc<AtomicBool>` partagé permet à la touche `q` de signaler l'arrêt
  instantanément aux deux threads.

## Un effet de bord assumé : l'affichage peut sauter des états, jamais le son

Si le rendu terminal est lent à un instant donné (typiquement sous WSL),
le thread d'émulation, lui, continue d'avancer et de mettre à jour le
snapshot sans interruption. Quand le thread d'affichage reprend la main,
il lit simplement **le snapshot le plus récent disponible** — il peut
donc "sauter" plusieurs états intermédiaires du jeu (registres, VRAM)
sans jamais perdre le fil du son, qui reste calé sur l'horloge réelle du
thread d'émulation. C'est exactement ce qui se passe dans n'importe quel
moteur de jeu qui "drop" des frames vidéo sous charge sans jamais
désynchroniser l'audio — un compromis standard, et très préférable à
l'inverse (un son qui bégaie pour ne jamais perdre une image d'un
panneau de débogage texte).

## Ce que nous avons maintenant

- Rien de nouveau en code pour l'instant — ce chapitre documente
  l'intention et le plan (`THREAD_PPU.MD`).
- Une bonne compréhension du *pourquoi* : l'audio doit rester la seule
  référence temporelle fiable, l'affichage peut toujours se permettre
  d'être "un peu en retard".

## Ce qui manque encore

- L'implémentation réelle : `RenderSnapshot`, le thread d'émulation, la
  boucle d'affichage révisée dans `main.rs`, et les tests sur WSL pour
  confirmer la disparition des coupures de plusieurs secondes.
- `game::render` affiche toujours un dégradé placeholder, pas le vrai
  framebuffer du PPU (chapitre 25) — une fois que ce sera le cas, le même
  raisonnement "l'audio est la référence, l'image peut sauter une frame"
  s'appliquera à l'affichage du jeu lui-même, pas seulement aux panneaux
  de débogage.
