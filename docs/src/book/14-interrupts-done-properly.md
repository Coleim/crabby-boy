# 14. Les interruptions faites correctement

Avec le Timer (Chapitre 9) et le VBlank du PPU (Chapitre 12) désormais
tous deux capables de positionner des bits dans `IF`, il est enfin temps
d'implémenter ce qui se passe réellement lorsqu'une interruption se
déclenche : le **dispatch** — mettre en pause ce que le CPU était en
train de faire et sauter vers une adresse de gestionnaire fixe. Ce
chapitre couvre `5af7b4b` et le nettoyage du cadre de test dans
`7e246c4`.

## La logique de dispatch

```rust
pub fn handle_interrupts(&mut self, bus: &mut Bus) {
    if self.ime == false {
        return; // interrupts globally disabled — do nothing
    }
    let ie = bus.get_ie();
    let iflag = bus.get_io().get_if();
    let triggered = ie & iflag; // enabled AND pending

    if triggered == 0 {
        return;
    }

    let bit = triggered.trailing_zeros() as u8;
    let mask = 1 << bit;
    bus.clear_if(mask); // mark this one as handled
    self.ime = false;   // disable further interrupts until RETI

    // 2 wait cycles are executed
    bus.internal_tick();
    bus.internal_tick();

    // Push current PC onto the stack, so RETI can come back here
    self.sp = self.sp.wrapping_sub(2);
    self.write16bytes(bus, self.sp, self.pc);

    bus.internal_tick();

    match bit {
        0 => self.pc = 0x0040, // VBlank
        1 => self.pc = 0x0048, // LCD STAT
        2 => self.pc = 0x0050, // Timer
        3 => self.pc = 0x0058, // Serial
        4 => self.pc = 0x0060, // Joypad
        _ => unreachable!(),
    }
}
```

Étape par étape, en reprenant le résumé conceptuel du Chapitre 0 :

1. **Le verrou `IME`** — si l'interrupteur maître interne au CPU est
   désactivé, rien ne se passe, peu importe ce qui est en attente. C'est
   aussi exactement la condition dont se préoccupe le bug HALT (Chapitres
   5 et 13).
2. **`IE & IF`** — seules les interruptions qui sont *à la fois*
   individuellement activées (`IE`, `0xFFFF`) *et* actuellement en
   attente (`IF`, `0xFF0F`) comptent.
3. **Priorité via `trailing_zeros()`** — si plusieurs bits d'interruption
   sont positionnés simultanément, la Game Boy traite toujours en
   premier le **bit numéroté le plus bas** (VBlank passe avant LCD STAT,
   qui passe avant Timer, qui passe avant Serial, qui passe avant
   Joypad). `u8::trailing_zeros()` est une astuce élégante en une ligne
   pour obtenir "l'index du bit positionné le plus bas" — exactement
   l'ordre de priorité requis, gratuitement.
4. **Effacer ce bit d'`IF`**, et **effacer `IME`** — pendant la gestion
   de cette interruption, aucune autre interruption (même de priorité
   supérieure) ne peut se déclencher par-dessus, jusqu'à ce que le
   gestionnaire réactive explicitement les interruptions (typiquement en
   se terminant par l'instruction `RETI` du Chapitre 4, qui restaure
   `IME`).
5. **Empiler `PC`** — exactement comme le fait l'instruction `CALL`, afin
   qu'une fois le gestionnaire terminé, l'exécution normale puisse
   reprendre exactement là où elle s'était arrêtée.
6. **Sauter vers une adresse de vecteur fixe** — chaque type
   d'interruption possède un point d'entrée fixe et codé en dur (`0x0040`
   pour VBlank, et ainsi de suite) ; il n'y a aucun décodage impliqué,
   ces adresses sont une constante matérielle, de la même manière que
   `0x0100` est toujours le point d'entrée de la cartouche (Chapitre 3).

## Pourquoi les appels supplémentaires à `bus.internal_tick()` comptent

Remarquez les deux tics internes avant d'empiler `PC`, et un de plus
après. Le dispatch d'une interruption n'est pas instantané sur le vrai
hardware — il prend un nombre fixe de cycles (5 cycles M au total, dans
la comptabilité de cette implémentation : 2 cycles "d'attente", 2 pour
empiler les 2 octets de `PC` sur la pile — correspondant au propre tick
interne de `write16bytes` issu du modèle de tick par accès du Chapitre
10 — plus 1 de plus). Omettre ceux-ci rendrait le dispatch d'interruption
"gratuit" en termes de timing, ce qui fausserait toute ROM de test (comme
`interrupt_time.gb`, ajoutée dans ce même commit) vérifiant précisément
combien de cycles s'écoulent autour du déclenchement d'une interruption.

## Ce que nous avons maintenant

- Un dispatch d'interruption complet : ordonnancement par priorité,
  verrouillage `IME`/`IE`/`IF`, empilement correct, saut de vecteur
  correct, coût en cycles correct.
- `interrupt_time.gb` ajoutée comme ROM de test spécifiquement pour
  valider ce timing.
- Les deux sources d'interruption du projet jusqu'ici (Timer, VBlank)
  *font désormais réellement quelque chose* lorsqu'elles se déclenchent,
  au lieu de simplement positionner un bit auquel personne ne réagit.

## Ce qui manque encore

- Seuls 2 des 5 types d'interruption (VBlank, Timer) ont actuellement une
  véritable source matérielle derrière eux — les interruptions Serial et
  Joypad existent comme adresses de vecteur dans ce `match`, mais rien
  ne positionne encore leurs bits `IF`. Celle de Joypad est câblée
  correctement au Chapitre 21.
- Les interruptions LCD STAT dépendent spécifiquement du suivi du "mode"
  du PPU, qui (selon le Chapitre 12) n'existe pas encore.
- Ceci clôture la Partie IV. À partir d'ici, l'histoire du projet se
  ramifie en sous-systèmes largement indépendants construits côte à côte :
  le son (Partie VI, suivant), les entrées (Partie VII), et une interface
  terminal (Partie VIII) — avant que ce livre ne revienne une dernière
  fois sur le PPU dans la Partie IX.
