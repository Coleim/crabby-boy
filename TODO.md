[] Implement OPCodes
[] Implement Serial (for testing)

[] gerer les flags en common method

[] Implementer les I/O Registers



https://rgbds.gbdev.io/docs/v1.0.0/gbz80.7#LD_r8,r8

// HALT a faire mieux

HALT

Enter CPU low-power consumption mode until an interrupt occurs.

The exact behavior of this instruction depends on the state of the IME flag, and whether interrupts are pending (i.e. whether ‘[IE] & [IF]’ is non-zero):

If the IME flag is set:
    The CPU enters low-power mode until after an interrupt is about to be serviced. The handler is executed normally, and the CPU resumes execution after the HALT when that returns.
If the IME flag is not set, and no interrupts are pending:
    As soon as an interrupt becomes pending, the CPU resumes execution. This is like the above, except that the handler is not called.
If the IME flag is not set, and some interrupt is pending:
    The CPU continues execution after the HALT, but the byte after it is read twice in a row (PC is not incremented, due to a hardware bug).

Cycles: -

Bytes: 1

Flags: None affected.

// Idem pour IME 

EI

Enable Interrupts by setting the IME flag.

The flag is only set after the instruction following EI.

Cycles: 1

Bytes: 1

Flags: None affected.
