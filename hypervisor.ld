OUTPUT_ARCH(riscv)
ENTRY(main)

SECTIONS
{
  . = 0x80200000; 

  .text : {
    *(.main)
    *(.text)
  }
  
  .rodata : {
    *(.strings)
    *(.rodata)
    *(.rodata.*)
  }

  .data : {
    *(.data)
  }

  .bss : {
    *(.bss)
    *(COMMON)
  }

  . = ALIGN(4);
  . += 128 * 1024; /* 128KB */
  __stack_top = .;

}
