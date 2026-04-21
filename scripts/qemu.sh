#!/bin/sh

if [ "${CI:-}" = "true" ]; then
  qemu-system-i386 \
    -kernel "$1" \
    -device isa-debug-exit,iobase=0xf4,iosize=0x04 \
    -display none \
    -monitor none \
    -no-reboot \
    -m 4G
else
  qemu-system-i386 \
    -kernel "$1" \
    -device isa-debug-exit,iobase=0xf4,iosize=0x04 \
    -monitor stdio \
    -m 4G
fi

if [ $? -ne 33 ]; then
  exit 1
fi
