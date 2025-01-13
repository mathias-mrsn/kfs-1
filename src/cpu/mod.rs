use core::arch::asm;

pub mod gdt;

#[unsafe(no_mangle)]
pub fn protected_mode()
{
    // TODO: Enable A20
    gdt::setup();

    unsafe {
        asm!(
            "jmp ${code_segment_offset}, $2f; 2:",
            code_segment_offset = const 0x08,
            options(att_syntax)
        );

        asm!(
            r#"
            mov eax, cr0
            or al, 1
            mov cr0, eax
        "#
        );

        asm!(
            r#"
            mov {tmp}, 0x10
            mov es, {tmp}
            mov fs, {tmp}
            mov gs, {tmp}
            mov ds, {tmp}
            mov ss, {tmp}
        "#,
            tmp = out(reg) _,
            options(nostack, nomem)
        );
    }
}
