# 2. Analyse de l'en-tête de la cartouche

Chaque cartouche Game Boy réserve une petite région fixe de la ROM —
`0x0100`–`0x014F` — pour un **en-tête** : des métadonnées structurées sur
le jeu lui-même (titre, fonctionnalités hardware requises, taille de sa
ROM/RAM, etc.). La boot ROM du hardware original lit cet en-tête avant de
céder le contrôle au jeu. Nous faisons de même.

Référence : <https://gbdev.io/pandocs/The_Cartridge_Header.html>

## La structure de l'en-tête

```rust
// src/header.rs
pub struct CartdrigeHeader {
    entry_point: u8,
    nintendo_logo: [u8; 48],
    title: String,
    manufacturer_code: String,
    cgb_flag: String,
    licensee: String,
    sgb_flag: String,
    cartridge_type: String,
    rom_size: String,
    ram_size: String,
    destination_code: String,
    version_number: u8,
    header_checksum: u8,
    global_checksum: u8,
}
```

Chaque champ correspond à une plage d'octets précise à l'intérieur de
l'en-tête. Passons en revue les plus intéressants.

## Le logo Nintendo, et pourquoi nous le validons

```rust
pub fn is_valid(&self) -> Result<(), String> {
    if self.logo_hex().eq("CE ED 66 66 CC 0D 00 0B 03 73 00 83 00 0C 00 0D \
       00 08 11 1F 88 89 00 0E DC CC 6E E6 DD DD D9 99 BB BB 67 63 6E 0E EC \
       CC DD DC 99 9F BB B9 33 3E") {
        println!("Nintendo Logo is valid");
        Ok(())
    } else {
        Err("Nintendo Logo is invalid".to_string())
    }
}
```

Pourquoi se donner la peine de vérifier cela ? Sur le vrai hardware, la
boot ROM **affiche** réellement ce bitmap de logo à l'écran pendant le
démarrage, et refuse de continuer si les octets ne correspondent pas
exactement à cette séquence précise. C'était la façon dont Nintendo
faisait respecter ses licences : cloner une cartouche impliquait aussi de
copier illégalement le bitmap du logo protégé par le droit d'auteur de
Nintendo. Nous n'émulons pas (encore) l'animation de la boot ROM, mais
nous réutilisons la même vérification comme un test de cohérence rapide
pour nous assurer que nous lisons un véritable fichier ROM intact.

## Titre et code fabricant : simplement des octets qui se trouvent être du texte

```rust
fn parse_title(rom: &[u8]) -> String {
    let title_start = 0x0134;
    let title_end = 0x0143;
    let title_bytes = &rom[title_start..=title_end];
    String::from_utf8_lossy(title_bytes).to_string()
}

fn parse_manufacturercode(rom: &[u8]) -> String {
    let start = 0x013F;
    let end = 0x0142;
    let code = &rom[start..=end];
    String::from_utf8_lossy(code).to_string()
}
```

C'est un bon moment pour intégrer quelque chose d'important : **un
« octet » n'a pas de signification inhérente** — c'est juste un nombre de
0 à 255. Qu'un octet donné signifie « une partie d'une instruction »,
« une partie d'une image », ou « un caractère de texte » dépend
entièrement de l'endroit où il se trouve et de la manière dont le code qui
le lit choisit de l'interpréter. Ici, les octets `0x0134`–`0x0143` sont
définis (par convention, par Nintendo) pour signifier « du texte ASCII, le
titre du jeu » — nous les lisons donc et les convertissons en `String`.
Quelques octets plus tôt et le même type d'octet brut signifierait
quelque chose de totalement différent (une partie du bitmap du logo).
Gardez cela à l'esprit pour chaque chapitre à venir.

## Code du licencié : ancien format vs. nouveau format

```rust
fn parse_licensee(rom: &[u8]) -> String {
    let old_code = &rom[0x014B];
    if *old_code == 0x33 {
        let new_code = &rom[0x0144..=0x0145];
        let code_cow = String::from_utf8_lossy(new_code);
        return format!(
            "[NEW] {}",
            NEW_LICENSEE_MAP.get(code_cow.as_ref()).unwrap_or(&"Unknown")
        );
    }
    format!("[OLD] {}", OLD_LICENSEE_MAP.get(old_code).unwrap_or(&"Unknown"))
}
```

Celui-ci illustre un thème récurrent de la Game Boy : **des bizarreries de
rétrocompatibilité intégrées directement dans le format de données.** Les
cartouches plus anciennes stockent l'éditeur (le « licencié ») sous forme
d'un seul octet à `0x014B`, recherché dans une ancienne table. Plus tard,
Nintendo a manqué de codes, donc les cartouches plus récentes définissent
`0x014B = 0x33` comme une sentinelle signifiant « ignore-moi, le vrai code
se trouve sous forme de deux caractères ASCII à `0x0144`-`0x0145`,
recherche-le plutôt dans la *nouvelle* table ». Les deux tables de
correspondance (lookup tables) (`OLD_LICENSEE_MAP`, `NEW_LICENSEE_MAP`)
sont de grandes tables codées en dur dans `src/mappings/licensee_map.rs`
— ce n'est pas quelque chose que l'on calcule, juste des données qu'on
retranscrit depuis la spécification.

## Type de cartouche, taille de ROM, taille de RAM : encore des recherches en table

```rust
fn parse_cartidge(rom: &[u8]) -> String {
    CARTRIDGE_TYPE_MAP.get(&rom[0x0147]).unwrap_or(&"None").to_string()
}

fn parse_rom_size(rom: &[u8]) -> String {
    ROM_SIZE_MAP.get(&rom[0x0148]).unwrap_or(&"Unknown").to_string()
}

fn parse_ram_size(rom: &[u8]) -> String {
    RAM_SIZE_MAP.get(&rom[0x149]).unwrap_or(&"Unknown").to_string()
}
```

L'octet du **type de cartouche** (`0x0147`) est l'un des champs les plus
importants de tout l'en-tête : il nous indique si cette cartouche est une
« ROM simple » (tout tient directement dans la plage adressable de 32 Ko)
ou utilise un **Memory Bank Controller** (une petite puce supplémentaire
sur la cartouche qui permet aux jeux plus gros que 32 Ko de faire entrer
et sortir des morceaux de ROM dans l'espace d'adressage du CPU à la
demande). Nous n'agissons pas encore sur cette information — c'est au
Chapitre 8 que le banking de ROM est réellement implémenté — mais nous
l'analysons et l'affichons déjà ici.

## Ce que nous avons maintenant

- Un `CartdrigeHeader` qui lit et étiquette chaque champ de la région
  d'en-tête, en utilisant des tables de correspondance pour les champs
  qui sont des « codes » plutôt que des nombres bruts ou du texte (type
  de cartouche, licencié, taille ROM/RAM).
- La validation du logo, comme première vérification « est-ce un vrai
  fichier ROM non endommagé ».
- Une méthode `print()` pour afficher l'intégralité de l'en-tête analysé
  à des fins de débogage — précieuse à ces débuts, puisqu'il n'y a pas
  encore de CPU pour réellement faire tourner le jeu et vous montrer si
  l'analyse était correcte.

## Ce qui manque encore

- Les champs `entry_point`, `version_number`, `header_checksum`, et
  `global_checksum` existent dans la structure mais ne sont pas encore
  réellement analysés à ce stade de l'historique (ils restent à leur
  valeur par défaut `0`) — ils ne seront remplis correctement que plus
  tard.
- Rien n'*agit* encore sur le type de cartouche (pas d'implémentation du
  banking de ROM — voir Chapitre 8).
- Toujours pas de CPU. C'est la suite, dans la Partie II.
