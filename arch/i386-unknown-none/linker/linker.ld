/*
 * https://medium.com/%40connorstack/how-does-a-higher-half-kernel-work-107194e46a64
 */

ENTRY(_start)

SECTIONS
{
  . = 0x00100000;

  .boot.text : {
    KEEP(*(.multiboot))
    *(.boot.text)
  }

  kernel_start = .;
	.text : ALIGN (4K)
	{
		*(.text .text.*)
	}

	.rodata : ALIGN (4K)
	{
		*(.rodata .rodata.*)
	}

	.data : ALIGN (4K)
	{
		*(.data .data.*)
	}

	.bss : ALIGN (4K)
	{
		*(COMMON)
		*(.bss .bss.*)
	}
	kernel_end = .;
}
