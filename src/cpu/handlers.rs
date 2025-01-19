use super::InterruptStackFrame;

pub extern "x86-interrupt" fn divide_error_handler(stack_frame: InterruptStackFrame)
{
    panic!("EXCEPTION: DIVIDE ERROR\n{:#?}", stack_frame);
}

pub extern "x86-interrupt" fn debug_handler(stack_frame: InterruptStackFrame)
{
    panic!("EXCEPTION: DEBUG\n{:#?}", stack_frame);
}

pub extern "x86-interrupt" fn nmi_interrupt_handler(stack_frame: InterruptStackFrame)
{
    panic!("EXCEPTION: NON-MASKABLE INTERRUPT\n{:#?}", stack_frame);
}

pub extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame)
{
    panic!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}

pub extern "x86-interrupt" fn overflow_handler(stack_frame: InterruptStackFrame)
{
    panic!("EXCEPTION: OVERFLOW\n{:#?}", stack_frame);
}

pub extern "x86-interrupt" fn bound_handler(stack_frame: InterruptStackFrame)
{
    panic!("EXCEPTION: BOUND RANGE EXCEEDED\n{:#?}", stack_frame);
}

pub extern "x86-interrupt" fn invalid_opcode_handler(stack_frame: InterruptStackFrame)
{
    panic!("EXCEPTION: INVALID OPCODE\n{:#?}", stack_frame);
}

pub extern "x86-interrupt" fn device_not_available_handler(stack_frame: InterruptStackFrame)
{
    panic!("EXCEPTION: DEVICE NOT AVAILABLE\n{:#?}", stack_frame);
}

pub extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: u32,
)
{
    panic!(
        "EXCEPTION: DOUBLE FAULT\nError Code: {}\n{:#?}",
        error_code, stack_frame
    );
}

pub extern "x86-interrupt" fn coprocessor_segment_overrun_handler(stack_frame: InterruptStackFrame)
{
    panic!("EXCEPTION: COPROCESSOR SEGMENT OVERRUN\n{:#?}", stack_frame);
}

pub extern "x86-interrupt" fn invalid_tss_handler(
    stack_frame: InterruptStackFrame,
    error_code: u32,
)
{
    panic!(
        "EXCEPTION: INVALID TSS\nError Code: {}\n{:#?}",
        error_code, stack_frame
    );
}

pub extern "x86-interrupt" fn segment_not_present_handler(
    stack_frame: InterruptStackFrame,
    error_code: u32,
)
{
    panic!(
        "EXCEPTION: SEGMENT NOT PRESENT\nError Code: {}\n{:#?}",
        error_code, stack_frame
    );
}

pub extern "x86-interrupt" fn stack_segment_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: u32,
)
{
    panic!(
        "EXCEPTION: STACK SEGMENT FAULT\nError Code: {}\n{:#?}",
        error_code, stack_frame
    );
}

pub extern "x86-interrupt" fn general_protection_handler(
    stack_frame: InterruptStackFrame,
    error_code: u32,
)
{
    panic!(
        "EXCEPTION: GENERAL PROTECTION FAULT\nError Code: {}\n{:#?}",
        error_code, stack_frame
    );
}

pub extern "x86-interrupt" fn page_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: u32,
)
{
    panic!(
        "EXCEPTION: PAGE FAULT\nError Code: {}\n{:#?}",
        error_code, stack_frame
    );
}

pub extern "x86-interrupt" fn fpu_handler(stack_frame: InterruptStackFrame)
{
    panic!("EXCEPTION: X87 FLOATING POINT\n{:#?}", stack_frame);
}

pub extern "x86-interrupt" fn alignment_check_handler(
    stack_frame: InterruptStackFrame,
    error_code: u32,
)
{
    panic!(
        "EXCEPTION: ALIGNMENT CHECK\nError Code: {}\n{:#?}",
        error_code, stack_frame
    );
}

pub extern "x86-interrupt" fn machine_check_handler(stack_frame: InterruptStackFrame)
{
    panic!("EXCEPTION: MACHINE CHECK\n{:#?}", stack_frame);
}

pub extern "x86-interrupt" fn simd_handler(stack_frame: InterruptStackFrame)
{
    panic!("EXCEPTION: SIMD FLOATING POINT\n{:#?}", stack_frame);
}

pub extern "x86-interrupt" fn virtualization_handler(stack_frame: InterruptStackFrame)
{
    panic!("EXCEPTION: VIRTUALIZATION\n{:#?}", stack_frame);
}

pub extern "x86-interrupt" fn control_protection_handler(
    stack_frame: InterruptStackFrame,
    error_code: u32,
)
{
    panic!(
        "EXCEPTION: CONTROL PROTECTION\nError Code: {}\n{:#?}",
        error_code, stack_frame
    );
}
