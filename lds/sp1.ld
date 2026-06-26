/*
 * Linker script for EL guests (reth/zesu) linked against SP1's libzkevm.a,
 * derived from external/sp1/zkevm/zkvm.ld. The sole deviation is `_heap_end`.
 * EL guests take its address via PC-relative addressing (R_RISCV_PCREL_HI20,
 * +/-2 GiB reach) from code at 0x78000000, so SP1's stock 0x1C00000000 would
 * overflow the relocation and it is capped to 0xF0000000, a ~1.875 GiB heap.
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
     * libzkevm's embedded TLSF (its modexp/pairing precompiles allocate) is set
     * up over [_end, ...) and the EL bump allocator also starts at _end, so
     * without a split both hand out the same memory and corrupt each other.
     * Reserve _libzkevm_tlsf_size for the TLSF and start the EL heap above it.
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
