# 3. Registres, flags, et votre premier opcode

Il est temps de construire la partie à laquelle tout le monde pense en
premier en entendant « émulateur » : le CPU.

## Le fichier de registres

```rust
// src/cpu.rs
pub struct CPU {
    pub a: u8,
    pub f: u8, // F = Z,N,H,C
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,
    pub h: u8,
    pub l: u8,
    pub pc: u16,
    pub sp: u16,
}
```

Sept registres 8 bits à usage général (`A B C D E H L`), un registre de
flags 8 bits (`F`), et deux registres spéciaux 16 bits : `PC` (program
counter — toujours l'adresse de la prochaine instruction à récupérer) et
`SP` (stack pointer — adresse du sommet de la pile d'appels, utilisée par
`CALL`/`RET`/`PUSH`/`POP`, que nous rencontrerons correctement au
Chapitre 4).

De nombreuses instructions traitent deux registres 8 bits comme une seule
paire 16 bits (`BC`, `DE`, `HL`, et `AF`) — nous ne le voyons pas encore
dans ce premier commit, mais cela arrive presque immédiatement au
Chapitre 4.

## Pourquoi ces valeurs de départ spécifiques ?

```rust
pub fn new() -> Self {
    CPU {
        a: 0x01,
        f: 0xB0,
        b: 0x00,
        c: 0x13,
        d: 0x00,
        e: 0xD8,
        h: 0x01,
        l: 0x4D,
        pc: 0x0100,
        sp: 0xFFFE,
    }
}
```

Ces valeurs ne sont pas arbitraires. Sur le hardware réel, une petite
**ROM de démarrage (boot ROM)** interne s'exécute en premier (faisant
défiler le logo Nintendo, jouant le son de démarrage, validant l'en-tête),
et au moment où elle cède le contrôle à la cartouche à l'adresse
`0x0100`, elle a laissé les registres exactement dans cet état. Comme
nous n'émulons pas encore la boot ROM elle-même (pas pour l'instant —
certains émulateurs le font, pas nous ici), nous démarrons simplement le
CPU comme si la boot ROM avait déjà tourné : `PC = 0x0100` (le point
d'entrée réel de la cartouche, correspondant au champ d'en-tête
`entry_point` du Chapitre 2) et ces valeurs de registres spécifiques, qui
sont des constantes documentées et connues.

## Le registre de flags : 4 bits qui comptent, compactés dans un octet

`F` n'est pas un registre à usage général — chacun de ses 4 bits de poids
fort est un flag booléen indépendant, positionné ou effacé en tant
qu'*effet secondaire* de l'exécution de certaines instructions, puis
relu par des sauts conditionnels (« sauter seulement si le dernier
résultat était zéro », etc.) :

| Bit | Nom | Signification |
|---|---|---|
| 7 | Z | Zéro — le dernier résultat était 0 |
| 6 | N | Soustraction — la dernière opération était une soustraction |
| 5 | H | Demi-retenue (half-carry) — une retenue est survenue hors du bit 3 |
| 4 | C | Retenue (carry) — une retenue (ou un emprunt) est survenue hors du bit 7 |

Les 4 bits de poids faible de `F` sont toujours à 0 sur le hardware réel.
Nous allons voir exactement cela dans la première vraie instruction
ci-dessous.

## La boucle fetch-decode-execute, pour de vrai cette fois

```rust
fn read16bytes(&mut self, mem: &[u8], pc: u16) -> u16 {
    let low = mem[(pc) as usize] as u16;
    let high = mem[(pc + 1) as usize] as u16;
    (high << 8) | low
}

pub fn execute(&mut self, mem: &mut [u8]) -> bool {
    let opcode = mem[self.pc as usize];
    println!("Parsing OP CODE: {:#X}", opcode);
    let mut next_pc: u16 = self.pc + 1;
    match opcode {
        0x00 => {
            println!("NOOP")
        }
        // ... more opcodes below
        _ => {
            println!("Something else");
            return false;
        }
    }
    self.pc = next_pc;
    return true;
}
```

Voici la forme que suivra désormais chaque gestionnaire d'instruction :

1. Lire l'octet à `PC` — c'est l'**opcode** (operation code), un nombre
   qui identifie quelle instruction exécuter.
2. Faire un `match` dessus pour trouver le bon gestionnaire.
3. Le gestionnaire lit autant d'octets supplémentaires que nécessaire (0,
   1, ou 2, juste après l'octet d'opcode) et fait son travail.
4. Avancer `PC` au-delà de l'opcode et de ses octets supplémentaires (à
   moins que l'instruction elle-même n'ait changé `PC`, comme un saut).

Un détail spécifique à la Game Boy mérite d'être souligné : `read16bytes`
lit deux octets et les combine comme `(high << 8) | low` — le *premier*
octet en mémoire est l'octet de **poids faible (low)**, le second est
l'octet de **poids fort (high)**. On appelle cela l'ordre des octets
**little-endian**, et la Game Boy l'utilise partout où des valeurs
multi-octets apparaissent en mémoire. Si une valeur 16 bits vous paraît
un jour exactement inversée (par exemple lire `0x1234` comme `0x3412`),
c'est presque toujours la raison.

## Vos premières vraies instructions

```rust
0xC3 => {
    // JP nn — jump to a fixed 16-bit address
    let addr = self.read16bytes(mem, next_pc);
    next_pc = addr;
}
0x31 => {
    // LD SP, n16 — load a 16-bit immediate value into SP
    self.sp = self.read16bytes(mem, next_pc);
    next_pc += 2;
}
0x3E => {
    // LD A, n8 — load an 8-bit immediate value into A
    let val: u8 = mem[next_pc as usize];
    self.a = val;
    next_pc += 1;
}
```

Le commentaire de chaque instruction dans le code source d'origine (par
exemple `"LD SP, n16 3 12"`) est un raccourci issu directement des tables
de référence d'opcodes que toute la communauté de développement Game Boy
utilise, par exemple <https://gbdev.io/gb-opcodes/optables/> : le
mnémonique de l'instruction, la longueur en octets, et le nombre de
cycles. Familiarisez-vous avec cette table — le Chapitre 4 s'appuie
dessus en permanence.

## `SUB A, n8` : vos premiers flags

```rust
0xD6 => {
    // SUB A, n8 — A = A - n8, 2 bytes, 8 cycles, affects Z N H C
    let val: u8 = mem[next_pc as usize];
    let a = self.a;
    self.a = a.wrapping_sub(val);
    self.f = 0;
    if self.a == 0 {
        self.f |= 0x80; // Z
    }
    self.f |= 0x40; // N always set after a subtraction

    // H: did a borrow happen out of bit 4? (i.e. the low nibble
    // couldn't cover the subtraction on its own)
    if (a & 0xF) < (val & 0xF) {
        self.f |= 0x20;
    }

    // C: did a borrow happen out of bit 8? (i.e. the whole byte
    // couldn't cover the subtraction, result wrapped around)
    if a < val {
        self.f |= 0x10;
    }
    next_pc += 1;
}
```

Deux détails Rust méritent une pause pour un débutant :

- `wrapping_sub` : un simple `a - val` en Rust **panique** (en mode debug)
  si la soustraction donnerait un résultat inférieur à 0 pour un `u8`.
  Mais l'arithmétique Game Boy est censée boucler (wrap around)
  (`0x00 - 0x01 = 0xFF`), exactement comme le ferait le vrai hardware
  8 bits. `wrapping_sub`/`wrapping_add` sont la façon de dire à Rust
  « oui, je sais, boucle au lieu de paniquer ». Vous les utiliserez
  constamment.
- `self.f |= 0x80` (faire un OU bit à bit pour ajouter un bit) et
  `self.a & 0xF` (faire un ET pour isoler le nibble de poids faible) sont
  les deux opérations bit à bit que vous utiliserez le plus dans tout ce
  projet. Si les opérations ET/OU/décalage bit à bit sont floues pour
  vous, un petit détour avant le Chapitre 4 en vaut la peine — presque
  chaque instruction les utilise.

## Ce que nous avons maintenant

- Une structure `CPU` avec la disposition réelle des registres et les
  vraies valeurs au moment du démarrage.
- Une boucle fetch-decode-execute fonctionnelle.
- Une poignée d'instructions : `NOP`, `JP nn`, `LD SP,n16`, `LD [a16],A`,
  `DI` (pas encore implémentée, juste reconnue), `LD A,n8`, `SUB A,n8`.

## Ce qui manque encore

- Seule une poignée des 256 opcodes possibles existe — tout le reste
  tombe dans le cas générique `_ => { return false }`. Le Chapitre 4
  remplit (presque) le reste.
- Pas encore de paires de registres 16 bits (`BC`/`DE`/`HL`/`AF`), alors
  que de nombreuses instructions en ont besoin.
- Pas encore de connexion à l'en-tête de la cartouche ou au bus mémoire —
  `execute` prend directement une tranche brute `&mut [u8]`.
- Pas d'interruptions, pas encore de valeur de retour de synchronisation
  précise au cycle près (remarquez que `execute` renvoie un `bool`, pas
  un décompte de cycles — cela viendra plus tard).
