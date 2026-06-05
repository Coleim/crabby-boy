

[] Faire l'APU minimal 


## Les logs audio


[AUDIO REG] READ NOT IMPLEMENTED FOR ADDR: FF14
[AUDIO REG] READ NOT IMPLEMENTED FOR ADDR: FF19
[AUDIO REG] READ NOT IMPLEMENTED FOR ADDR: FF1E
[AUDIO REG] READ NOT IMPLEMENTED FOR ADDR: FF23
[AUDIO REG] READ NOT IMPLEMENTED FOR ADDR: FF14
[AUDIO REG] READ NOT IMPLEMENTED FOR ADDR: FF19
[AUDIO REG] READ NOT IMPLEMENTED FOR ADDR: FF1E
[AUDIO REG] READ NOT IMPLEMENTED FOR ADDR: FF23
[AUDIO REG] READ NOT IMPLEMENTED FOR ADDR: FF14
[AUDIO REG] READ NOT IMPLEMENTED FOR ADDR: FF19
[AUDIO REG] READ NOT IMPLEMENTED FOR ADDR: FF1E
[AUDIO REG] READ NOT IMPLEMENTED FOR ADDR: FF23




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

