tasks:

- Read input
  - Add keyboard polling path for raw scancodes
  - Add IRQ1 path for keyboard input
  - Decode PS/2 scancodes into key events
  - Separate keyboard driver from terminal input handling

- Build terminal system
  - Keep a simple VGA text console design suitable for an educational i386 kernel
  - Add basic input editing support (`\n`, `\r`, `\b`, `\t`) where useful
  - Add a simple shell layer on top of the console
  - Keep keyboard handling separate from shell command execution

- Create tests
  - Keep backend VGA tests focused on memory layout, scrolling, cursor, and CRTC state
  - Add tests for end-of-window and end-of-VRAM behavior
  - Add keyboard decoding tests once the keyboard layer exists

- Make VGA access IRQ-safe
- Add block writes and region operations to the VGA backend
- Add save/restore screen support
- Add VGA font loading support
- Add palette and blank/unblank control
