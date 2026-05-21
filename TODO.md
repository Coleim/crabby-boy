
[X] Find a way to run any ROM (create an emulator class)
[X] Map the serial to somehting and rea the bus for the tests  

[X] Create tests to run roms
[] Implementer et tourner les tests suivants
    ## `mem_timing` puis `mem_timing-2` — **Timing des accès mémoire**
    no need for registering the cycles anymore
    every bus write/read ticks 4 -> rename it to read_and_tick
    some opcode got internal ticks 

    https://gekkio.fi/
    ## `halt_bug.gb` — **Bug HALT**
    ## `interrupt_time` — **Timing des interruptions**

[] Faire le PPU minimal (compter les cycles)
[] Faire l'APU minimal ? 

<!-- ## 6. `dmg_sound` — **Audio (DMG)** -->
<!-- ## 8. `oam_bug` — **Bug OAM** -->


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
Audio → cpal
Input → winit

