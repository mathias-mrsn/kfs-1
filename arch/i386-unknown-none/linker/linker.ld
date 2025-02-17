/*
 * https://medium.com/%40connorstack/how-does-a-higher-half-kernel-work-107194e46a64
 */

ENTRY(_start)

SECTIONS
{
  . = 0x00100000;

	_kernel_start = .;
  .boot.text : {
    KEEP(*(.multiboot))
    *(.boot.text)
  }

  .boot.data ALIGN (4K): {
    *(.boot.pdt)
  }

	. += 0xC0000000;

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
	}
	_kernel_end = .;
}
