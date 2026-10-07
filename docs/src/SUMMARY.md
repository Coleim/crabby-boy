# Sommaire

[Introduction](./README.md)

# Guides

- [PPU — Guide du rendu de l'arrière-plan](./ppu-background.md)

# Construire un émulateur Game Boy en Rust

- [0. Ce qu'il y a vraiment dans une Game Boy](./book/00-orientation.md)

- [Partie I — Premiers octets]()
  - [1. Mise en place du projet et chargement d'une ROM](./book/01-project-setup.md)
  - [2. Analyse de l'en-tête de la cartouche](./book/02-cartridge-header.md)

- [Partie II — Le cerveau : construire le CPU]()
  - [3. Registres, flags et votre premier opcode](./book/03-cpu-registers-first-opcode.md)
  - [4. Développer le jeu d'instructions](./book/04-instruction-set.md)
  - [5. Le bug du halt, partie 1](./book/05-halt-bug-part1.md)

- [Partie III — Grandir : donner à l'émulateur une véritable architecture]()
  - [6. Séparer en CPU / Bus / Hardware](./book/06-cpu-bus-hardware-split.md)
  - [7. Compléter les registres manquants](./book/07-missing-registers-stubs.md)
  - [8. Le banking de ROM](./book/08-rom-banking.md)

- [Partie IV — Garder le temps]()
  - [9. Les timers](./book/09-timers.md)
  - [10. La justesse du timing mémoire](./book/10-memory-timing.md)
  - [11. Infrastructure de tests et CI](./book/11-testing-and-ci.md)

- [Partie V — Une première ébauche de PPU]()
  - [12. Les scanlines 101 et un PPU minimal](./book/12-minimal-ppu.md)
  - [13. Le bug du halt, partie 2](./book/13-halt-bug-part2.md)
  - [14. Les interruptions faites correctement](./book/14-interrupts-done-properly.md)

- [Partie VI — Le son : l'APU]()
  - [15. Architecture de l'APU et le module audio/](./book/15-apu-architecture.md)
  - [16. Canal 1 — Onde carrée avec sweep](./book/16-apu-channel1.md)
  - [17. Canal 2 — L'onde carrée la plus simple](./book/17-apu-channel2.md)
  - [18. Canal 3 — Onde personnalisée](./book/18-apu-channel3.md)
  - [19. Canal 4 — Bruit](./book/19-apu-channel4.md)
  - [20. Réussir les tests dmg_sound de Blargg](./book/20-apu-passing-tests.md)

- [Partie VII — Les entrées]()
  - [21. Le joypad](./book/21-joypad.md)

- [Partie VIII — Lui donner un visage : construire une interface terminal]()
  - [22. Un détour qui n'a pas tenu](./book/22-detour-image-test.md)
  - [23. Découpler la vitesse d'émulation du rafraîchissement de l'affichage](./book/23-decoupling-tick-from-display.md)
  - [24. Un filtre passe-haut pour un son plus propre](./book/24-audio-high-pass-filter.md)
  - [25. Afficher les registres CPU et les infos de la cartouche](./book/25-terminal-ui-registers-header.md)
  - [26. Visualiser la VRAM](./book/26-vram-tile-viewer.md)

- [Partie IX — Retour au PPU]()
  - [27. Où nous avons laissé le PPU, et ce qui vient ensuite](./book/27-back-to-the-ppu.md)
