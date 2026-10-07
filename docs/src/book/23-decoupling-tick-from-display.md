# 23. Découpler la vitesse d'émulation du rafraîchissement de l'affichage

Ce commit (`a3ec152`, "refactor emulator to be able to be ticked from
external process... useful for having a UI that framerate differs from
CPU") est la correction directe du problème de couplage identifié à la
fin du chapitre 22. Il introduit l'architecture que le projet utilise
encore aujourd'hui.

## Un trait `Display` : l'interface utilisateur devient une interface enfichable

```rust
// src/display/display.rs
pub trait Display {
    fn draw(&mut self, emulator: &CrabbyBoy);
    fn handle_events(&mut self);
    fn is_running(&self) -> bool;
}
```

Au lieu d'une seule structure concrète possédant à la fois "faire tourner
l'émulateur" et "dessiner dans le terminal" (le `DisplayInterface` du
chapitre 22), il existe désormais un petit trait que n'importe quelle
implémentation d'affichage peut satisfaire. `CrabbyBoy` (l'émulateur
lui-même) et ce qui implémente `Display` deviennent deux éléments
indépendants, reliés uniquement par le fait que `draw` reçoit une
référence en lecture seule à l'état actuel de l'émulateur.

## `CrabbyBoy::tick_for_duration` : exécuter "environ ce nombre de cycles", pas "une seule étape"

```rust
// src/crabby_boy.rs
const RUNTIME_STEPS_PER_SEC: f64 = 400_000.0;

pub fn tick_for_duration(&mut self, frame_delta: Duration) {
    let dt = frame_delta.as_secs_f64().clamp(0.001, 0.5);
    let number_of_steps = (RUNTIME_STEPS_PER_SEC * dt) as usize;
    for _ in 0..number_of_steps {
        self.tick();
    }
}
```

C'est la nouvelle idée clé. Au lieu que la boucle principale fasse
avancer le CPU exactement une fois par itération (comme c'est le cas
depuis le chapitre 9), l'appelant dit maintenant "quel que soit le temps
réel qui vient de s'écouler (`frame_delta`), exécute *approximativement*
l'équivalent de ce temps émulé en étapes". La boucle d'affichage peut se
redessiner au rythme qui convient pour un terminal (quelques fois par
seconde) pendant que l'émulateur, à l'intérieur, avance toujours à la
bonne vitesse globale, par rafale, entre deux rafraîchissements.

## La boucle principale, désormais pilotée depuis `main.rs`, et non `emulator.rs`

```rust
// src/main.rs
let mut crabby = CrabbyBoy::new(file_path)?;
let mut display = RatatuiDisplay::new();
let target_frame = Duration::from_micros(16_667); // ~60 FPS
let mut last_tick_instant = Instant::now();

while display.is_running() {
    let frame_start = Instant::now();
    let dt = frame_start.duration_since(last_tick_instant);
    last_tick_instant = frame_start;

    display.handle_events();
    crabby.tick_for_duration(dt);
    display.draw(&crabby);

    let frame_elapsed = frame_start.elapsed();
    if frame_elapsed < target_frame {
        std::thread::sleep(target_frame - frame_elapsed);
    }
}
```

Comparez cela à la boucle d'origine du chapitre 9, qui vivait entièrement
à l'intérieur de `CrabbyBoy::run` et ne retournait jamais avant que toute
la ROM ne soit terminée (ou qu'une condition de test ne soit remplie).
Désormais, `main.rs` possède la boucle externe, mesure le temps réel
écoulé en horloge murale (`dt`) à chaque itération, et se cale
explicitement sur environ 60 images (frame) par seconde — en mettant en
veille le temps restant si une itération se termine en avance. C'est la
forme standard d'une boucle de simulation en temps réel : mesurer le
temps écoulé, faire avancer la simulation d'autant, afficher, recommencer.

## `RatatuiDisplay` : la première implémentation concrète de `Display`

```rust
// src/display/ratatui_display.rs (ébauche)
pub struct RatatuiDisplay {
    terminal: DefaultTerminal,
    running: bool,
    fps: FpsCounter,
}
```

Un nouveau `FpsCounter` (`src/display/fps_counter.rs`) suit le taux de
rafraîchissement (refresh rate) réel mesuré pour le débogage à l'écran —
pratique dès lors qu'on cale délibérément une boucle de cette manière,
car il est facile d'introduire des bugs subtils qui la font tourner plus
vite ou plus lentement que prévu.

## Une correction de suivi rapide et honnête

Le commit suivant immédiat, `8b32399` ("Fixing test after refactor. It's
cleaner now... Love it!"), existe parce que ce refactoring — comme la
plupart des refactorings touchant une boucle centrale — a cassé la suite
de tests automatisés existante (la macro `cpu_instr_test!` des
chapitres 9/11), qui appelait directement l'ancienne forme
`CrabbyBoy::run`/`tick` unique. Mettre à jour les tests pour
correspondre à une nouvelle architecture, juste après l'avoir introduite,
est tout à fait normal et mérite d'être assumé plutôt que caché.

## Ce que nous avons maintenant

- Un trait `Display` propre, découplant "comment l'émulateur tourne" de
  "comment il est affiché".
- Un avancement (tick) cadencé en temps réel (`tick_for_duration`),
  permettant à la vitesse d'émulation et au taux de rafraîchissement de
  l'affichage de différer.
- Le premier `RatatuiDisplay` fonctionnel, et un `FpsCounter` pour
  vérifier qu'il reste honnête.

## Ce qui manque encore

- À ce stade, `RatatuiDisplay` n'affiche encore presque rien de
  significatif — les véritables vues de registres CPU/en-tête
  (chapitre 25) et le visualiseur de tuiles VRAM (chapitre 26) sont
  construits sur cette base dans les chapitres suivants.
