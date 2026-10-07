# Guides du développeur Crabby Boy

Ceci est une petite collection de guides à visée pédagogique écrits en
construisant [crabby-boy](https://github.com/), un émulateur Game Boy, dans
un but d'apprentissage.

Chaque guide est écrit façon tutoriel : lentement, explicitement, avec des
références à la documentation de référence [Pan Docs](https://gbdev.io/pandocs/)
chaque fois qu'un détail est simplifié ou omis.

## Guides disponibles

- [PPU — Guide du rendu de l'arrière-plan](./ppu-background.md) : comment
  le PPU transforme les données de tuiles de la VRAM en la couche
  d'arrière-plan d'une frame (image), construit autour du véritable
  mécanisme de pixel-FIFO/fetcher.

## Le livre : Construire un émulateur Game Boy en Rust

Un tutoriel complet façon « pour les nuls », suivant le véritable
historique des commits du projet depuis le tout premier `Cargo.toml`
jusqu'à l'état actuel : un CPU fonctionnel, un bus, des timers, des
interruptions, un APU, un joypad, une interface terminal, et un premier
PPU (minimal). Il suppose une connaissance générale de la programmation
mais aucune connaissance préalable de la Game Boy. Commencez au
[Chapitre 0](./book/00-orientation.md) et progressez dans l'ordre.
