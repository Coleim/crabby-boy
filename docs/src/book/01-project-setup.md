# 1. Mise en place du projet et chargement d'une ROM

Tout projet d'émulateur commence de la même manière modeste : un projet
Rust capable de lire un fichier de jeu en mémoire et d'en afficher
quelque chose. Ce chapitre correspond au tout premier commit de
crabby-boy.

## Le projet Cargo

```toml
# Cargo.toml
[package]
name = "CrabbyBoy"
version = "0.1.0"
edition = "2024"

[dependencies]
```

Rien de spécial pour l'instant — une crate binaire toute simple, sans
dépendances. Les émulateurs sont une excellente excuse pour écrire
beaucoup de code avec très peu de bibliothèques externes, puisque la
majeure partie du travail consiste à « calculer fidèlement ce que le
hardware calculerait », et non à « assembler des bibliothèques
existantes ».

## Qu'est-ce qu'un fichier ROM, physiquement ?

Une cartouche Game Boy est, du point de vue du CPU, simplement de la
mémoire adressable supplémentaire — un bloc d'octets dont le CPU peut
lire des instructions et des données. Un fichier ROM `.gb` est une copie
octet par octet de la puce ROM de cette cartouche. Donc « charger une
ROM » revient simplement à lire le fichier dans un `Vec<u8>` :

```rust
// src/main.rs
mod memory;
use memory::Memory;

fn main() {
    let file_path = "./cpu_instrs.gb";
    let rom = std::fs::read(file_path).unwrap();
    let memory = Memory::new(&rom);

    let opcode = rom[0];
    println!("opcode: 0b{:08b} - 0x{:X}", opcode, opcode);

    let logo: [u8; 48] = read_nintendo_logo(&rom);
    print_logo_ascii(&logo);
}

fn read_nintendo_logo(rom: &[u8]) -> [u8; 48] {
    let logo_start = 0x0104;
    let logo_end = 0x0133;
    let mut logo: [u8; 48] = [0; 48];
    logo.copy_from_slice(&rom[logo_start..=logo_end]);
    logo
}

fn print_logo_ascii(logo: &[u8; 48]) {
    for str in logo {
        print!("{:02X} ", str);
    }
}
```

Quelques points à remarquer, car ils donnent le ton pour tout le projet :

- `rom[0]` — le tout premier octet du fichier est déjà significatif :
  c'est la première instruction CPU que la console va exécuter. Nous
  l'affichons à la fois en binaire et en hexadécimal, car vous
  traduirez constamment entre les deux en lisant la documentation du
  hardware (l'hexadécimal est compact ; le binaire rend les motifs de
  bits/flags évidents).
- `0x0104..=0x0133` est une région fixe à l'intérieur de la ROM réservée
  pour le bitmap du **logo Nintendo** — la véritable boot ROM du hardware
  refuse de démarrer un jeu dont les octets du logo ne correspondent pas
  exactement (c'est ainsi que Nintendo faisait respecter ses licences).
  Nous ferons notre propre validation de cela plus tard (Chapitre 2) —
  pour l'instant nous nous contentons de le lire et de l'afficher.

## Un premier (tout petit) modèle de mémoire

```rust
// src/memory.rs
pub struct Memory {
    data: [u8; 0x10000],
}

impl Memory {
    pub fn new(rom: &[u8]) -> Self {
        let mut memory = Memory { data: [0; 0x10000] };
        memory.data[0x0000..(0x0000 + rom.len())].copy_from_slice(rom);
        memory
    }
}
```

`0x10000` vaut 65 536 en décimal — l'intégralité de l'espace d'adressage
qu'une adresse 16 bits peut atteindre, comme expliqué au Chapitre 0.
Cette première version est volontairement naïve : elle se contente de
copier toute la ROM au début d'un unique grand tableau plat, sans encore
comprendre que « cette partie de l'espace d'adressage correspond à la
VRAM », ou que « les ROM de plus de 32 Ko ont besoin de commutation de
banques (bank switching) ». Ce n'est pas grave — ce fichier sera
entièrement réécrit plus d'une fois à mesure que le projet grandit (voir
le `Bus` du Chapitre 6 et le banking de ROM du Chapitre 8). Le but d'un
premier commit est d'avoir *quelque chose* qui compile et fait une chose
honnête et utile.

## Ce que nous avons maintenant

- Un projet Rust qui lit un fichier `.gb` en mémoire.
- Un tableau mémoire plat de 64 Ko.
- L'affichage du premier octet opcode et des octets du logo Nintendo,
  comme vérification que le fichier est correctement lu.

## Ce qui manque encore

- Pas encore de véritable CPU — rien n'exécute d'instructions.
- Pas de réelle compréhension de ce qui se trouve dans un en-tête de
  cartouche (prochain chapitre).
- Aucune séparation entre ROM, RAM, VRAM, registres d'E/S, etc. — tout
  n'est qu'un seul tableau pour l'instant.
