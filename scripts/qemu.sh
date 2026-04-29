#!/bin/sh

ARGS="-kernel \"$1\""
ARGS="$ARGS -device isa-debug-exit,iobase=0xf4,iosize=0x04"
ARGS="$ARGS -m 4G"

if [ "${CI:-}" = "true" ]; then
        ARGS="$ARGS -display none"
        ARGS="$ARGS -monitor none"
        ARGS="$ARGS -no-reboot"
else
        case "$(uname -s)" in
        Darwin)
                ARGS="$ARGS -display cocoa,zoom-to-fit=on"
                ;;
        Linux)
                ARGS="$ARGS -display gtk,zoom-to-fit=on"
                ;;
        esac

        ARGS="$ARGS -monitor stdio"
fi

eval "qemu-system-i386 $ARGS"

if [ $? -ne 33 ]; then
        exit 1
fi
