# THREAD_PPU — Plan pas à pas : thread d'émulation dédié

But : corriger le son saccadé et la lenteur de la touche `q` qui restent
même après le fix du `Picker` (commit `15a8e3e`), en séparant
l'émulation (CPU/APU → audio) du rendu terminal dans un thread dédié.

Rappel du diagnostic : tout tourne encore dans **une seule boucle/un
seul thread** (`main.rs`). Le rendu terminal (écriture + images
Sixel/Kitty) reste une E/S potentiellement lente, surtout sous WSL. Tant
qu'il n'est pas terminé, l'émulation n'avance plus : le buffer audio se
vide (saccades), et `q` n'est relu qu'à la frame suivante.

Objectif : l'émulation tourne sur un thread jamais bloqué par le rendu.
L'affichage lit le dernier état disponible, sans jamais forcer
l'émulation à l'attendre.

---

## Étape 1 — Un `RenderSnapshot` : ce dont l'affichage a besoin, rien de plus

Nouveau fichier `src/display/render_snapshot.rs` :

```rust
pub struct RenderSnapshot {
    pub fps: f32,
    pub header_summary: String, // ou clone de CartdrigeHeader si vous préférez
    pub cpu: CPU,                // CPU doit dériver Clone, Copy
    pub vram_tiles: Vec<u8>,     // clone de bus.vram_tiles (6 Ko)
    pub vram_tilemap: Vec<u8>,   // clone de bus.vram_tilemap (2 Ko)
}
```

Pourquoi un snapshot et pas tout `CrabbyBoy` derrière un `Mutex` : si le
thread d'affichage verrouillait tout `CrabbyBoy` pendant tout son
`draw()` (lent), on retomberait dans le bug actuel — l'émulation serait
de nouveau bloquée pendant le rendu. Le snapshot garantit que le verrou
n'est tenu que le temps d'une copie rapide de quelques Ko, jamais
pendant l'E/S terminale.

`CPU` (`src/cpu/cpu.rs`) n'a que des champs `u8`/`u16`/`bool` : ajoutez
`#[derive(Clone, Copy)]` sur `pub struct CPU` pour pouvoir le cloner
directement.

`RenderSnapshot` lui-même doit dériver `Clone` (utile à l'étape 5).

## Étape 2 — `CrabbyBoy::snapshot(&self) -> RenderSnapshot`

Dans `src/crabby_boy.rs` :

```rust
pub fn snapshot(&self) -> RenderSnapshot {
    RenderSnapshot {
        fps: self.fps.fps(),
        cpu: self.cpu,
        vram_tiles: self.bus.vram_tiles.to_vec(),
        vram_tilemap: self.bus.vram_tilemap.to_vec(),
        header_summary: /* ce qui vous est utile depuis self.header */,
    }
}
```

## Étape 3 — Adapter le trait `Display` et ses implémentations

`src/display/display.rs` :

```rust
pub trait Display {
    fn draw(&mut self, snapshot: &RenderSnapshot);
    fn handle_events(&mut self);
    fn is_running(&self) -> bool;
}
```

Puis dans `ratatui_display.rs`, `game.rs`, `cpu_registers.rs`,
`cartridge_header.rs`, `vram_registers.rs`, `vram_tilemap_registers.rs` :
remplacer `&emulator.cpu` par `&snapshot.cpu`, `&emulator.bus.vram_tiles`
par `&snapshot.vram_tiles`, etc. Refactor mécanique : laissez le
compilateur vous guider erreur par erreur (`cargo build`).

## Étape 4 — État partagé entre les deux threads

Dans `src/main.rs` :

```rust
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

let mut crabby = CrabbyBoy::new(file_path)?;
let _audio_output = /* comme aujourd'hui, AVANT de déplacer crabby */;

let running = Arc::new(AtomicBool::new(true));
let snapshot = Arc::new(Mutex::new(crabby.snapshot()));

let emu_running = running.clone();
let emu_snapshot = snapshot.clone();
let emulation_thread = std::thread::spawn(move || {
    let mut crabby = crabby; // déplacé dans le thread
    let mut last_instant = std::time::Instant::now();
    while emu_running.load(Ordering::Relaxed) {
        let now = std::time::Instant::now();
        let dt = now.duration_since(last_instant);
        last_instant = now;

        crabby.tick_for_duration(dt);

        if let Ok(mut guard) = emu_snapshot.lock() {
            *guard = crabby.snapshot();
        }

        std::thread::sleep(std::time::Duration::from_millis(1));
    }
});
```

Points importants :
- `Ordering::Relaxed` suffit : on signale juste un arrêt, pas de
  synchronisation de données complexe.
- Le `sleep(1ms)` évite de saturer un cœur de CPU à 100% ;
  `tick_for_duration` reste correct quel que soit le `dt` réel mesuré.
- `set_audio_sample_rate` et la création de l'`AudioOutput` doivent se
  faire **avant** de déplacer `crabby` dans la closure (comme
  aujourd'hui dans `main.rs`, ne changez pas cet ordre).

## Étape 5 — La boucle d'affichage (thread principal)

```rust
let mut display = RatatuiDisplay::new();
let target_frame = Duration::from_micros(16_667); // ou plus lent, voir étape 7

while running.load(Ordering::Relaxed) && display.is_running() {
    let frame_start = Instant::now();

    display.handle_events();
    if !display.is_running() {
        running.store(false, Ordering::Relaxed);
        break;
    }

    let snap = snapshot.lock().unwrap().clone(); // verrou bref, puis on dessine sans lui
    display.draw(&snap);

    let elapsed = frame_start.elapsed();
    if elapsed < target_frame {
        std::thread::sleep(target_frame - elapsed);
    }
}

emulation_thread.join().ok();
```

Le point central de toute la correction : **ne jamais tenir le
`Mutex<RenderSnapshot>` pendant `display.draw(...)`**. On verrouille,
on clone, on déverrouille, puis seulement après on dessine.

## Étape 6 — `q` instantané

`handle_events()` n'a pas besoin de changer de logique interne ; ce qui
change, c'est que la boucle qui l'appelle n'est plus jamais ralentie par
un verrou tenu longtemps ni par l'émulation. Optionnel, pour plus de
réactivité perçue : vider toutes les touches en attente d'un coup avec
une petite boucle `while event::poll(Duration::ZERO) { ... }` plutôt
qu'une touche par itération.

## Étape 7 — (optionnel) ralentir le rendu terminal lui-même

Indépendamment du threading, vous pouvez passer `target_frame` à
`Duration::from_millis(33)` (30 FPS) ou `50` (20 FPS) pour réduire la
charge d'E/S terminale (images Sixel/Kitty), sans rien changer à la
vitesse de l'émulation/son puisqu'ils sont maintenant découplés.

## Étape 8 — Tester

- macOS/Ghostty : le son doit rester fluide, `q` instantané.
- WSL : le vrai test. Les pauses de 5-10 secondes doivent disparaître
  complètement, même si l'affichage lui-même reste visuellement lent
  sous WSL (attendu : l'affichage peut "sauter" des états, jamais le
  son).

## Risques / pièges à surveiller en codant

- Ne pas oublier `set_audio_sample_rate` **avant** de déplacer `crabby`
  dans le thread.
- `RenderSnapshot` (et chaque champ qu'il contient) doit être `Clone`
  pour l'étape 5.
- `CPU` doit dériver `Clone, Copy` (pas de `String`/`Vec` dedans
  actuellement, donc c'est sûr).
- Le `Mutex<RenderSnapshot>` ne doit **jamais** être gardé verrouillé
  pendant `display.draw(...)`.
- À la fermeture (`q`), bien faire `emulation_thread.join()` avant que
  `main` ne se termine, sinon le thread d'émulation pourrait être tué en
  plein milieu d'un tick.

---

## Et après ?

Une fois ce plan codé et testé (surtout sous WSL), on pourra rédiger les
nouveaux chapitres du livre (déjà ébauchés dans
`/Users/coliva/.opencode/plan/thread-emulation-and-book-updates.md`,
partie B) à partir du vrai code écrit.
