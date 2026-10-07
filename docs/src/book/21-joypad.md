# 21. Le Joypad

Ce commit (`e128e88`, intitulé "Joypad all button released") est un bon
exemple d'une pratique très courante en développement d'émulateur :
corriger un problème de compatibilité avec la valeur *correcte la plus
simple possible*, plutôt que de construire la fonctionnalité complète tout
de suite.

## Le problème résolu

```rust
pub fn read(&self) -> u8 {
    0xF // était : self.p1
}
```

Rappelez-vous du chapitre 0/7 que le registre du joypad (`P1`, `0xFF00`)
utilise une logique **active-low** : un bit à `0` signifie "ce bouton est
appuyé", et `1` signifie "non appuyé". Avant ce changement, `read()`
renvoyait simplement ce qui avait été *écrit* en dernier dans le registre
(le stub d'origine du chapitre 7) — ce qui est incorrect d'une manière qui
compte vraiment : de nombreux jeux, pendant le boot ou les boucles de
sondage (polling) des entrées, écrivent une valeur de sélecteur "quel
groupe de boutons voulez-vous" puis relisent immédiatement le résultat,
s'attendant à voir "aucun bouton appuyé" (tous les bits concernés à `1`)
si rien n'est maintenu. Renvoyer l'écriture telle quelle pouvait faire
croire au jeu que des boutons étaient maintenus alors que ce n'était pas
le cas, perturbant les séquences de boot ou la logique d'entrée. Forcer
`0xF` (les 4 bits concernés à `1`) affirme, sans condition, "rien n'est
appuyé, jamais" — ce n'est pas encore un vrai joypad, mais c'est
suffisant pour arrêter cette catégorie de bug.

## Pourquoi "tous les boutons relâchés" est un excellent tremplin

C'est la même idée de bootstrap-avec-un-placeholder que dans les
chapitres 6 et 7 (valeurs LCDC/STAT/LY codées en dur avant qu'un vrai PPU
n'existe) : un placeholder *conservateur*, toujours sûr, permet au code
dépendant (ici, la logique de sondage des entrées de n'importe quel jeu)
de progresser correctement dans le cas commun ("est-ce que quelque chose
est appuyé en ce moment ? Non.") sans investir tout de suite dans la
fonctionnalité complète (véritable association clavier-bouton, vrai
protocole de sélection de groupe de boutons du chapitre 0). À ce stade du
projet, c'est exactement là où en sont les choses — et c'est toujours
vrai dans la base de code actuelle : aucune touche de clavier n'est
encore associée à un bouton de Game Boy. La fonction `handle_events` de
l'interface terminal (chapitre 23) ne reconnaît qu'une seule touche, `q`,
pour quitter :

```rust
// src/display/ratatui_display.rs
fn handle_events(&mut self) {
    if let Ok(Event::Key(key_event)) = event::read() {
        match key_event.code {
            KeyCode::Char('q') => self.running = false,
            _ => {}
        }
    }
}
```

## Une petite correction sans rapport qui passe en même temps : la plage d'adresses de la wave RAM

```diff
-            0xFF10..=0xFF26 => self.audio.read(addr),
+            0xFF10..=0xFF3F => self.audio.read(addr),
```

La wave RAM du canal 3 (chapitre 18) se trouve en réalité à
`0xFF30`-`0xFF3F`, juste après la fin des registres "normaux" de contrôle
du son (`0xFF26`). Cet élargissement d'une ligne de la plage filtrée est
ce qui a rendu tout ce bloc d'adresses enfin accessible — un autre
exemple (comme le zéro manquant du chapitre 13) montrant qu'une petite
limite de plage facile à manquer peut avoir un impact important en
pratique.

## Ce que nous avons maintenant

- Un registre du joypad qui signale de manière fiable "rien n'est
  appuyé" — suffisant pour les jeux dont la logique de boot/entrée n'a
  besoin que de cette garantie pour progresser correctement.
- La plage d'adresses complète de la wave RAM enfin accessible.

## Ce qui manque encore

- Aucune véritable association bouton-touche n'existe encore, nulle part
  dans le projet — c'est explicitement un travail futur ouvert, pas
  quelque chose que l'historique des commits de ce livre a encore
  résolu.
- Aucune interruption (interrupt) Joypad (`IF` bit 4, mentionnée depuis
  le chapitre 14) n'est jamais déclenchée, car rien ne détecte
  actuellement un véritable appui sur un bouton pour la déclencher.
