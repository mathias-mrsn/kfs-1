global main

section .text
bits 32
main:
    ; print `OK` to screen
    mov dword [0xb8000], 0x2f4b2f4f
    hlt
