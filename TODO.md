

## Joypad
https://docs.rs/gilrs/latest/gilrs/

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

