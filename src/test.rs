#[cfg(test)]
pub fn test_runner(tests: &[&dyn Fn()])
{
    use crate::qemu;

    for test in tests {
        test();
    }
    qemu::exit(qemu::QemuExitCode::Success);
}
