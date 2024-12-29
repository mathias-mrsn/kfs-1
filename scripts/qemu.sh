#!/bin/sh

qemu-system-i386 -kernel $1 -device isa-debug-exit,iobase=0xf4,iosize=0x04 # -machine type=pc-i440fx-3.1

if [ $? -ne 33 ]; then
  exit 1
fi
