/*
 * zkvm-sp1.ld -- Linker script for EL guests (reth/zesu) linked against SP1's
 * libzkevm.a. Derived from external/sp1/zkevm/zkvm.ld.
 *
 * The only deviation from the stock SP1 script is `_heap_end`. SP1's own
 * embedded allocator bounds the heap with a compile-time constant, so the
 * stock script can set `_heap_end = __input_start` (0x1C00000000, ~118 GiB up).
 * The EL guests instead take the address of the `_heap_end` linker symbol via
 * PC-relative addressing (R_RISCV_PCREL_HI20, +/-2 GiB reach), so a symbol that
 * far from the code at 0x78000000 overflows the relocation. `_heap_end` is
 * therefore capped to a PC-relative-reachable address, yielding a ~1.875 GiB
 * heap -- ample for stateless block validation -- while the reserved input
 * region at __input_start stays untouched.
 */

OUTPUT_FORMAT("elf64-littleriscv")
OUTPUT_ARCH(riscv)
ENTRY(_start)

/* SP1 memory parameters. Override with --defsym at link time if needed. */
PROVIDE(__sp1_stack_top    = 0x78000000);
PROVIDE(__sp1_max_memory   = 0x2000000000); /* 1 << 37 = 128 GiB */
PROVIDE(__sp1_input_region = 0x0400000000); /* 1 << 34 = 16  GiB */

MEMORY
{
    RAM (rwx) : ORIGIN = 0x78000000, LENGTH = 0x1B88000000  /* MAX_MEMORY - INPUT_REGION - STACK_TOP */
}

SECTIONS
{
    . = ORIGIN(RAM);

    .text : ALIGN(16)
    {
        KEEP(*(.text._start))
        *(.text .text.*)
    } > RAM

    .rodata : ALIGN(16)
    {
        *(.rodata .rodata.*)
        *(.srodata .srodata.*)
    } > RAM

    .data : ALIGN(16)
    {
        __data_start = .;
        *(.data .data.*)
        *(.sdata .sdata.*)
        __data_end = .;
    } > RAM
    __data_load = LOADADDR(.data);

    .bss (NOLOAD) : ALIGN(16)
    {
        __bss_start = .;
        *(.bss .bss.*)
        *(.sbss .sbss.*)
        *(COMMON)
        . = ALIGN(8);
        __bss_end = .;
    } > RAM

    . = ALIGN(16);
    _end = .;

    __stack_top = __sp1_stack_top;

    PROVIDE(__global_pointer$ = __data_start + 0x800);

    __input_start = __sp1_max_memory - __sp1_input_region;
    __input_end   = __sp1_max_memory;

    /*
     * Heap partition between libzkevm's TLSF and the EL guest's bump allocator.
     *
     * SP1's libzkevm keeps its own #[global_allocator] (embedded TLSF) because
     * its precompiles (modexp, BN254/BLS pairing) allocate. __start initializes
     * that TLSF over [_end, EMBEDDED_RESERVED_INPUT_START). The EL guest (reth)
     * has its own bump allocator. Left unpartitioned, both hand out memory from
     * _end and corrupt each other once a precompile allocates.
     *
     * `_libzkevm_tlsf_size` reserves the low region for libzkevm's TLSF; the EL
     * bump heap starts above it. The reserved size dwarfs precompile scratch
     * (pairings/modexp allocate kilobytes), so the TLSF never grows into the EL
     * region. `_heap_end` stays capped at 0xF0000000 for PC-relative reach.
     */
    _libzkevm_tlsf_size = 0x10000000;  /* 256 MiB reserved for libzkevm TLSF */
    _heap_start = _end + _libzkevm_tlsf_size;
    _heap_end   = 0xF0000000;

    /DISCARD/ :
    {
        *(.comment)
        *(.note .note.*)
        *(.eh_frame .eh_frame.*)
        *(.riscv.attributes)
    }
}
