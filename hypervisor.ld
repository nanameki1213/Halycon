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

  . = ALIGN(4096);

  .intr_stack (NOLOAD) : {
    _intr_stack_start = .;
    . = . + 0x3000;
    _intr_stack_end = .;
  }

  _free_area = . ;

}
