use core::ptr;

const GDT_MAX_DESCRIPTORS: u8 = 0x20;

#[derive(Debug, Clone, Copy)]
pub struct GDTSegmentDescriptor(u64);

impl GDTSegmentDescriptor
{
    pub fn new(
        base: u32,
        limit: u32,
        access: u8,
        flags: u8,
    ) -> Self
    {
        let mut descriptor: u64 = 0;

        // High 32 bits of the descriptor
        descriptor |= (limit as u64 & 0x000F0000) as u64;
        descriptor |= ((access as u64) << 8) & 0x0000FF00;
        descriptor |= ((flags as u64) << 20) & 0x00F00000;
        descriptor |= ((base as u64 >> 16) & 0x000000FF) as u64;
        descriptor |= ((base as u64) & 0xFF000000) as u64;

        descriptor <<= 32;

        // Low 32 bits of the descriptor
        descriptor |= ((base as u64) & 0x0000FFFF) << 16;
        descriptor |= (limit as u64) & 0x0000FFFF;

        Self(descriptor)
    }

    // pub fn get_base() -> u32 {}
    // pub fn get_limit() -> u32 {}
    // pub fn get_flags() -> u32 {}
    // pub fn get_access() -> u32 {}
}

#[derive(Debug)]
pub struct GlobalDescriptorTable<const M: usize = 1>
{
    table: [GDTSegmentDescriptor; M],
    len:   usize,
}

impl<const M: usize> GlobalDescriptorTable<M>
{
    // assert!(M > 0, "GDT need at least 1 entry");
    // assert!(
    //     M <= 8192,
    //     "GDT can be up to 65536 bytes in length (8192 entries)"
    // );

    pub fn mem_clear(&mut self)
    {
        unsafe {
            ptr::write_bytes(
                self.table.as_mut_ptr() as *mut u8,
                0x00,
                M * core::mem::size_of::<GDTSegmentDescriptor>(),
            );
        }
    }

    // pub fn fill(slice: &[GDTSegmentDescriptor]) {}
    //
    // pub fn add(seg: GDTSegmentDescriptor) {}
    // pub fn remove(index: usize) {}
}
