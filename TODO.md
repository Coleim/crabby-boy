
- [] Find a way to run any ROM (create an emulator class)
- [] Map the serial to somehting and read the bus for the tests  
- [] Create a UI to Run the Emulator ? 
- [] Play sound



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

