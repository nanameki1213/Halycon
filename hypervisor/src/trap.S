.macro VM_EXIT
  csrw sscratch, sp
  la sp, _intr_stack_end
  addi sp, sp, -8*31
  sd x0, 0*8(sp)
  sd x1, 1*8(sp)
  sd x2, 2*8(sp)
  sd x3, 3*8(sp)
  sd x4, 4*8(sp)
  sd x5, 5*8(sp)
  sd x6, 6*8(sp)
  sd x7, 7*8(sp)
  sd x8, 8*8(sp)
  sd x9, 9*8(sp)
  sd x10, 10*8(sp)
  sd x11, 11*8(sp)
  sd x12, 12*8(sp)
  sd x13, 13*8(sp)
  sd x14, 14*8(sp)
  sd x15, 15*8(sp)
  sd x16, 16*8(sp)
  sd x17, 17*8(sp)
  sd x18, 18*8(sp)
  sd x19, 19*8(sp)
  sd x20, 20*8(sp)
  sd x21, 21*8(sp)
  sd x22, 22*8(sp)
  sd x23, 23*8(sp)
  sd x24, 24*8(sp)
  sd x25, 25*8(sp)
  sd x26, 26*8(sp)
  sd x27, 27*8(sp)
  sd x28, 28*8(sp)
  sd x29, 29*8(sp)
  sd x30, 30*8(sp)
  sd x31, 31*8(sp)
.endm

.macro VM_ENTRY
  mv x31, a0
  ld x0, 0*8(x31)
  ld x1, 1*8(x31)
  ld x2, 2*8(x31)
  ld x3, 3*8(x31)
  ld x4, 4*8(x31)
  ld x5, 5*8(x31)
  ld x6, 6*8(x31)
  ld x7, 7*8(x31)
  ld x8, 8*8(x31)
  ld x9, 9*8(x31)
  ld x10, 10*8(x31)
  ld x11, 11*8(x31)
  ld x12, 12*8(x31)
  ld x13, 13*8(x31)
  ld x14, 14*8(x31)
  ld x15, 15*8(x31)
  ld x16, 16*8(x31)
  ld x17, 17*8(x31)
  ld x18, 18*8(x31)
  ld x19, 19*8(x31)
  ld x20, 20*8(x31)
  ld x21, 21*8(x31)
  ld x22, 22*8(x31)
  ld x23, 23*8(x31)
  ld x24, 24*8(x31)
  ld x25, 25*8(x31)
  ld x26, 26*8(x31)
  ld x27, 27*8(x31)
  ld x28, 28*8(x31)
  ld x29, 29*8(x31)
  ld x30, 30*8(x31)
  ld x31, 31*8(x31)
  csrr sp, sscratch
  sret
.endm

.section .text
.global vm_exit
.balign 256
vm_exit:
  VM_EXIT

.section .text
.global vm_entry
.balign 256
vm_entry:
  VM_ENTRY

.section .text
.global machine_vector_table
.balign 256
machine_vector_table:
    j machine_exception_handler

.section .text
.global supervisor_vector_table
.balign 256
supervisor_vector_table:
    j supervisor_exception_handler
    
.text
.global machine_exception_handler 
.balign 256
machine_exception_handler:
    csrr a0, mstatus
    li t0, 0x8000000000
    and t1, a0, t0
    beq t1, x0, NOT_STACK_M
    csrw mscratch, sp
    la sp, _intr_stack_end
NOT_STACK_M:
    addi sp, sp, -8*31
    sd x0, 0*8(sp)
    sd x1, 1*8(sp)
    sd x2, 2*8(sp)
    sd x3, 3*8(sp)
    sd x4, 4*8(sp)
    sd x5, 5*8(sp)
    sd x6, 6*8(sp)
    sd x7, 7*8(sp)
    sd x8, 8*8(sp)
    sd x9, 9*8(sp)
    sd x10, 10*8(sp)
    sd x11, 11*8(sp)
    sd x12, 12*8(sp)
    sd x13, 13*8(sp)
    sd x14, 14*8(sp)
    sd x15, 15*8(sp)
    sd x16, 16*8(sp)
    sd x17, 17*8(sp)
    sd x18, 18*8(sp)
    sd x19, 19*8(sp)
    sd x20, 20*8(sp)
    sd x21, 21*8(sp)
    sd x22, 22*8(sp)
    sd x23, 23*8(sp)
    sd x24, 24*8(sp)
    sd x25, 25*8(sp)
    sd x26, 26*8(sp)
    sd x27, 27*8(sp)
    sd x28, 28*8(sp)
    sd x29, 29*8(sp)
    sd x30, 30*8(sp)
    sd x31, 31*8(sp)
    call machine_handler
    csrr a0, mstatus
    li t0, 0x8000000000
    and t1, a0, t0
    beq t1, x0, END_INTR_M 
    csrr sp, mscratch
END_INTR_M:
    ld x0, 0*8(sp)
    ld x1, 1*8(sp)
    ld x2, 2*8(sp)
    ld x3, 3*8(sp)
    ld x4, 4*8(sp)
    ld x5, 5*8(sp)
    ld x6, 6*8(sp)
    ld x7, 7*8(sp)
    ld x8, 8*8(sp)
    ld x9, 9*8(sp)
    ld x10, 10*8(sp)
    ld x11, 11*8(sp)
    ld x12, 12*8(sp)
    ld x13, 13*8(sp)
    ld x14, 14*8(sp)
    ld x15, 15*8(sp)
    ld x16, 16*8(sp)
    ld x17, 17*8(sp)
    ld x18, 18*8(sp)
    ld x19, 19*8(sp)
    ld x20, 20*8(sp)
    ld x21, 21*8(sp)
    ld x22, 22*8(sp)
    ld x23, 23*8(sp)
    ld x24, 24*8(sp)
    ld x25, 25*8(sp)
    ld x26, 26*8(sp)
    ld x27, 27*8(sp)
    ld x28, 28*8(sp)
    ld x29, 29*8(sp)
    ld x30, 30*8(sp)
    ld x31, 31*8(sp)
    addi sp, sp, 8*31
    mret

.text
.global supervisor_exception_handler 
.balign 256
supervisor_exception_handler:
  VM_EXIT
  mv a0, sp
  call exception_handler
  mv a0, sp
  VM_ENTRY
