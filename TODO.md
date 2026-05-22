
[X] Find a way to run any ROM (create an emulator class)
[X] Map the serial to somehting and rea the bus for the tests  

[X] Create tests to run roms
[] Implementer et tourner les tests suivants


exit prev pc 
afficher l'eram en debug
voir ce qu'il se passe apres pour fix les bugs

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

