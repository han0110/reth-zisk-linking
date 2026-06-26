/* Copied and modified from https://github.com/0xPolygonHermez/rust/blob/zisk/compiler/rustc_target/src/spec/targets/riscv64ima_zisk_zkvm_elf_linker_script.ld */
/* Modification includes only additional _heap_start and _heap_end */

OUTPUT_FORMAT("elf64-littleriscv")
OUTPUT_ARCH("riscv")
ENTRY(_start)

MEMORY {
  rom   (xa) : ORIGIN = 0x80000000, LENGTH = 0x10000000
  ram   (wxa) : ORIGIN = 0xa0020000, LENGTH = 0x1FFE0000
}

PHDRS {
  text PT_LOAD FLAGS(5);
  rodata PT_LOAD FLAGS(4);
  data PT_LOAD FLAGS(6);
  bss PT_LOAD FLAGS(6);
  output_data PT_LOAD FLAGS(6);
}

_stack_size           = 0x400000;  /* 4 MB reserved */
_output_data_size     = 0x10000;   /* 64 KB reserved */
_float_ram_data_size  = 0x10000;   /* 64 KB reserved */

/*
 * RAM layout reserves .output_data (64 KB, NOLOAD), then .data, .bss, a 4 MB
 * stack, and the heap, with the top _float_ram_data_size (64 KB) held back to
 * bound the heap.
 */

SECTIONS
{
  .text : { *(.text.init) *(.text .text.*)} >rom AT>rom :text

  . = ALIGN(8);
  PROVIDE(_global_pointer = .);
  .rodata : { *(.rodata .rodata.*)} >rom AT>rom :rodata

  .output_data (NOLOAD) : {
    PROVIDE(_output_data_start = .);
    . = . + _output_data_size; 
    PROVIDE(_output_data_end = .);
  } >ram AT>ram :output_data

  .data : { *(.data .data.* .sdata .sdata.*) } >ram AT>ram :data

  .bss : {
    PROVIDE(_bss_start = .);
    *(.bss .bss.*);
    PROVIDE(_bss_end = .);
  } >ram AT>ram :bss

  . = ALIGN(8);
  PROVIDE(_init_stack_top = . + _stack_size);

  PROVIDE(_kernel_heap_bottom = _init_stack_top);
  PROVIDE(_kernel_heap_top = ORIGIN(ram) + LENGTH(ram) - _float_ram_data_size);
  PROVIDE(_kernel_heap_size = _kernel_heap_top - _kernel_heap_bottom);

  PROVIDE(_heap_start = _kernel_heap_bottom);
  PROVIDE(_heap_end = _kernel_heap_top);

  _end = .;
}
