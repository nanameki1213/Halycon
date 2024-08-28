OUTPUT_ARCH(riscv)
ENTRY(main)

SECTIONS
{
  . = 0x80200000; 
  
  .text : {
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
  
}
