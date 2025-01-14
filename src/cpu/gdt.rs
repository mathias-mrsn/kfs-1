use crate::instructions::tables::lgdt;
use bitflags::bitflags;
use core::arch::asm;
use core::mem;

/// A structure representing a pointer to a descriptor table (GDT/IDT)
///
/// This structure matches the format required by the LGDT and LIDT instructions
/// in x86 architecture. It contains both the size limit and base address of the
/// descriptor table.
///
/// # Fields
///
/// * `limit` - The size of the descriptor table minus 1 (maximum size is 65535
///   bytes)
/// * `base` - A pointer to the start of the descriptor table in memory
///
/// # Memory Layout
///
/// The structure is marked with `#[repr(C, packed)]` to ensure:
/// * No padding bytes are added between fields
/// * The memory layout matches the CPU's expected format
///
/// # Safety
///
/// This structure is primarily used in unsafe contexts when loading descriptor
/// tables via CPU instructions. The caller must ensure:
/// * The base pointer points to a valid descriptor table
/// * The limit accurately represents the table size minus 1
/// * The structure is properly aligned when used
#[derive(Debug)]
#[repr(C, packed)]
pub struct DescriptorTablePointer
{
    pub limit: u16,
    pub base:  *const (),
}

/// A structure representing a descriptor table (GDT/IDT) with a configurable
/// size
///
/// This structure manages a table of segment or interrupt descriptors in
/// memory. Each descriptor is stored as a 64-bit value, and the table can hold
/// up to `M` entries.
///
/// # Type Parameters
///
/// * `M` - The maximum number of entries in the table (default: 1) Must be > 0
///   and <= 8192 (constrained by AssertTableLimit)
///
/// # Fields
///
/// * `table` - An array of 64-bit descriptors (segment or gate descriptors)
/// * `len` - The current number of entries in the table
///
/// # Examples
///
/// ```rust
/// // Create a GDT with space for 7 entries
/// let mut gdt: DescriptorTable<7> = DescriptorTable::new();
/// gdt.clear(); // Initialize with null descriptor
/// gdt.push(descriptor); // Add a new descriptor
/// gdt.load(); // Load the GDT into the CPU
/// ```
///
/// # Safety
///
/// This structure is used to define CPU protection mechanisms:
/// * The first entry (index 0) must be a null descriptor
/// * Descriptors must be properly formatted according to CPU requirements
/// * The table must be properly aligned when loaded into the CPU
#[derive(Debug)]
pub struct DescriptorTable<const M: usize = 1>
{
    table: [u64; M],
    len:   usize,
}

/// Compile-time validator for descriptor table size
///
/// Used as a zero-sized type to enforce size constraints on descriptor tables:
/// * Minimum size: 1 entry
/// * Maximum size: 8192 entries (65536 bytes)
struct AssertTableLimit<const M: usize>;

impl<const M: usize> AssertTableLimit<M>
{
    const OK: usize = {
        assert!(M > 0, "GDT need at least 1 entry");
        assert!(
            M <= 8192,
            "GDT can be up to 65536 bytes in length (8192 entries)"
        );
        0
    };
}

impl<const M: usize> DescriptorTable<M>
where
    [(); AssertTableLimit::<M>::OK]:,
{
    /// Initializes the descriptor table with a null descriptor.
    ///
    /// This method:
    /// * Fills the entire table with zeros
    /// * Sets the length to 1 (accounting for the required null descriptor at
    ///   index 0)
    ///
    /// # Examples
    ///
    /// ```rust
    /// let mut gdt: DescriptorTable<3> = DescriptorTable::new();
    /// gdt.clear(); // Initialize with null descriptor
    /// ```
    pub fn clear(&mut self)
    {
        self.table.fill(0);
        self.len = 1;
    }

    /// Adds a new descriptor to the table.
    ///
    /// # Arguments
    ///
    /// * `descriptor` - A 64-bit value representing the segment descriptor
    ///
    /// # Panics
    ///
    /// Panics if the table is already full (len >= M)
    ///
    /// # Examples
    ///
    /// ```rust
    /// gdt.push(DescriptorBits::KERNEL_CODE.bits());
    /// ```
    pub fn push(
        &mut self,
        descriptor: u64,
    )
    {
        assert!(self.len < M, "GDT is already full");
        self.table[self.len] = descriptor;
        self.len += 1;
    }

    /// Adds multiple descriptors to the table.
    ///
    /// # Arguments
    ///
    /// * `descriptors` - A slice of 64-bit descriptor values to add
    ///
    /// # Panics
    ///
    /// Panics if adding all descriptors would exceed the table's capacity
    ///
    /// # Examples
    ///
    /// ```rust
    /// gdt.fill(&[
    ///     DescriptorBits::KERNEL_CODE.bits(),
    ///     DescriptorBits::KERNEL_DATA.bits(),
    /// ]);
    /// ```
    pub fn fill(
        &mut self,
        descriptors: &[u64],
    )
    {
        for &descriptor in descriptors {
            self.push(descriptor);
        }
    }

    /// Loads the descriptor table into the CPU using the LGDT instruction.
    ///
    /// # Safety
    ///
    /// This method is safe to call, but it performs an unsafe operation
    /// internally as it directly interfaces with CPU instructions. The
    /// caller must ensure:
    /// * The table contains valid descriptors
    /// * The first entry is a null descriptor
    /// * The table is properly aligned
    ///
    /// # Examples
    ///
    /// ```rust
    /// gdt.clear();
    /// gdt.fill(&[/* valid descriptors */]);
    /// gdt.load(); // Load into CPU
    /// ```
    pub fn load(&self)
    {
        let ptr = self.as_ptr();
        unsafe { lgdt(&ptr) };
    }

    /// Creates a pointer to the descriptor table suitable for CPU instructions.
    ///
    /// Returns a `DescriptorTablePointer` containing:
    /// * The size of the table minus 1 (limit)
    /// * A pointer to the table's base address
    ///
    /// # Examples
    ///
    /// ```rust
    /// let ptr = gdt.as_ptr();
    /// unsafe { lgdt(&ptr) };
    /// ```
    pub fn as_ptr(&self) -> DescriptorTablePointer
    {
        DescriptorTablePointer {
            limit: (self.len * mem::size_of::<u64>() - 1) as u16,
            base:  self.table.as_ptr() as _,
        }
    }
}

bitflags! {
    /// Represents the various flags and bits used in GDT (Global Descriptor Table) entries.
    /// These bits control segment characteristics, privileges, and other attributes.
    #[derive(Debug, Clone, Copy)]
    pub struct DescriptorBits: u64 {
        /// Accessed bit. CPU sets this bit when the segment is accessed.
        /// Used for debugging and virtual memory management.
        const A = 1 << 40;

        /// Read/Write bit. For data segments: writable bit.
        /// For code segments: readable bit (execution-only if not set).
        const RW = 1 << 41;

        /// Direction/Conforming bit.
        /// For data segments: segment grows down if set.
        /// For code segments: code can be executed from lower privilege levels if set.
        const DC = 1 << 42;

        /// Executable bit. If set, code in this segment can be executed.
        /// Indicates a code segment when set, data segment when clear.
        const E = 1 << 43;

        /// Descriptor type bit. Set for code or data segments,
        /// clear for system segments.
        const S = 1 << 44;

        /// Descriptor Privilege Level 3 (User mode).
        /// Highest ring level, least privileged.
        const DPL_RING_3 = 3 << 45;

        /// Descriptor Privilege Level 2.
        /// Intermediate ring level.
        const DPL_RING_2 = 2 << 45;

        /// Descriptor Privilege Level 1.
        /// Intermediate ring level.
        const DPL_RING_1 = 1 << 45;

        /// Present bit. Must be set for valid segments.
        /// Clear for unused segment entries.
        const P = 1 << 47;

        /// Long-mode code flag. Set for 64-bit code segments.
        /// Only valid for code segments.
        const L = 1 << 53;

        /// Default operation size bit.
        /// For code segments: 1 = 32-bit, 0 = 16-bit.
        /// For data segments: 1 = 32-bit stack operations.
        const DB = 1 << 54;

        /// Granularity bit. 0 = limit scaled by 1,
        /// 1 = limit scaled by 4K (page granularity).
        const G = 1 << 55;

        /// Maximum segment limit value.
        /// Represents the maximum addressable size when combined with granularity.
        const LIMIT_MAX = 0x000F_0000_0000_FFFF;
    }
}

impl DescriptorBits
{
    /// Base configuration for data segments.
    /// Includes:
    /// * Segment bit (S)
    /// * Present bit (P)
    /// * Read/Write bit (RW)
    /// * Accessed bit (A)
    /// * Maximum segment limit
    const _DATA: Self = Self::from_bits_truncate(
        Self::S.bits() | Self::P.bits() | Self::RW.bits() | Self::A.bits() | Self::LIMIT_MAX.bits(),
    );

    /// Base configuration for code segments.
    /// Extends _DATA configuration with:
    /// * Executable bit (E)
    const _CODE: Self = Self::from_bits_truncate(Self::_DATA.bits() | Self::E.bits());

    /// Kernel code segment descriptor.
    /// Extends _CODE configuration with:
    /// * Granularity bit (G) - 4K page granularity
    /// * Default operation size bit (DB) - 32-bit operations
    const KERNEL_CODE: Self =
        Self::from_bits_truncate(Self::_CODE.bits() | Self::G.bits() | Self::DB.bits());

    /// Kernel data segment descriptor.
    /// Extends _DATA configuration with:
    /// * Granularity bit (G) - 4K page granularity
    /// * Default operation size bit (DB) - 32-bit operations
    const KERNEL_DATA: Self =
        Self::from_bits_truncate(Self::_DATA.bits() | Self::G.bits() | Self::DB.bits());

    /// Kernel stack segment descriptor.
    /// Identical to KERNEL_DATA configuration
    const KERNEL_STACK: Self = Self::KERNEL_DATA;

    /// User code segment descriptor.
    /// Extends KERNEL_CODE configuration with:
    /// * DPL Ring 3 - User mode privilege level
    const USER_CODE: Self =
        Self::from_bits_truncate(Self::KERNEL_CODE.bits() | Self::DPL_RING_3.bits());

    /// User data segment descriptor.
    /// Extends KERNEL_DATA configuration with:
    /// * DPL Ring 3 - User mode privilege level
    const USER_DATA: Self =
        Self::from_bits_truncate(Self::KERNEL_DATA.bits() | Self::DPL_RING_3.bits());

    /// User stack segment descriptor.
    /// Identical to USER_DATA configuration
    const USER_STACK: Self = Self::USER_DATA;
}

/// Sets up the Global Descriptor Table (GDT) and initializes segment registers
///
/// This function performs the following operations:
/// 1. Creates a GDT with 7 entries at physical address 0x800
/// 2. Initializes the GDT with kernel and user segment descriptors
/// 3. Loads the GDT into the CPU
/// 4. Updates all segment registers with appropriate selectors
///
/// # GDT Layout
/// - Entry 0: Null descriptor (required by CPU)
/// - Entry 1: Kernel code segment
/// - Entry 2: Kernel data segment
/// - Entry 3: Kernel stack segment
/// - Entry 4: User code segment
/// - Entry 5: User data segment
/// - Entry 6: User stack segment
///
/// # Safety
///
/// This function is unsafe because it:
/// - Writes to a raw pointer at a fixed memory address (0x800)
/// - Directly manipulates CPU state through the GDT
/// - Must be called only once during system initialization
pub unsafe fn setup()
{
    pub static mut GDT: *mut DescriptorTable<7> = 0x800 as *mut DescriptorTable<7>;

    (*GDT).clear();
    (*GDT).fill(&[
        DescriptorBits::KERNEL_CODE.bits(),
        DescriptorBits::KERNEL_DATA.bits(),
        DescriptorBits::KERNEL_DATA.bits(),
        DescriptorBits::USER_CODE.bits(),
        DescriptorBits::USER_DATA.bits(),
        DescriptorBits::USER_DATA.bits(),
    ]);
    (*GDT).load();

    asm!(
        r#"
            jmp ${kexec_offset}, $2f;
            2:
            mov {0:x}, {kcode_offset}
            mov %ds, {0:x}
            mov %es, {0:x}
            mov %fs, {0:x}
            mov %gs, {0:x}
            mov %ss, {0:x}
        "#,
        out(reg) _,
        kexec_offset = const 0x8,
        kcode_offset = const 0x10,
        options(nostack, nomem, att_syntax)
    );
}
