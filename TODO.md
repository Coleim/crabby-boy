

[] Faire l'APU minimal 


## Les logs audio

Les manquants importants pour entendre quelque chose :

| Adresse | Registre | Priorité |
|---|---|---|
| `FF26` | NR52 | **CRITIQUE** — master on/off, sans ça tout est silencieux |
| `FF25` | NR51 | **CRITIQUE** — panning, quel channel va à gauche/droite |
| `FF24` | NR50 | **CRITIQUE** — master volume |

Implémente ces 5 là en priorité, surtout `FF26` et `FF19` (trigger CH2).


<!-- ## 6. `dmg_sound` — **Audio (DMG)** -->
<!-- ## 8. `oam_bug` — **Bug OAM** -->

[] PPU

[] Play sound

[] Create a UI to Run the Emulator ? 



main loop:

while running:
    cycles = cpu.step()

    ppu.step(cycles)
    apu.step(cycles)

    if frame_ready:
        render()

    if audio_buffer_ready:
        send_to_cpal()



Affichage → pixels ou wgpu ou minifb
ou macroquad
Audio → cpal
Input → winit

