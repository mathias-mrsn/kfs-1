use super::CPUIDFeatureEDX;

pub fn does_cpu_has_apic() -> bool
{
    let cpuid = unsafe { core::arch::x86::__cpuid(1) };
    (cpuid.edx & CPUIDFeatureEDX::APIC.bits()) != 0
}
