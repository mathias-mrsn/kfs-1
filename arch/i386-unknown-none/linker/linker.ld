/*
 * https://medium.com/%40connorstack/how-does-a-higher-half-kernel-work-107194e46a64
 */

/* The bootloader will look at this image and start execution at the symbol
   designated as the entry point. */
ENTRY(_start)

/* Tell where the various sections of the object files will be put in the final
   kernel image. */
SECTIONS
{
  . = 0x00100000;

	_kernel_start = .;
  .multiboot_header ALIGN(4K): {
    KEEP(*(.multiboot))
  }

  .boot.data ALIGN (4K): {
    *(.boot.pdt)
  }

  .boot.text : {
    *(.boot.text)
  }

	. += 0xC0000000;
	/* Add a symbol that indicates the start address of the kernel. */
	.text ALIGN (4K) : AT (ADDR (.text) - 0xC0000000)
	{
		*(.text .text.*)
	}

	.rodata ALIGN (4K) : AT (ADDR (.rodata) - 0xC0000000)
	{
		*(.rodata .rodata.*)
	}

	.data ALIGN (4K) : AT (ADDR (.data) - 0xC0000000)
	{
		*(.data .data.*)
	}

	.bss ALIGN (4K) : AT (ADDR (.bss) - 0xC0000000)
	{
		*(COMMON)
		*(.bss .bss.*)
		*(.bootstrap_stack)
	}
	/* Add a symbol that indicates the end address of the kernel. */
	_kernel_end = .;
}
