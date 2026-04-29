#!/bin/sh

set -eu

script_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
root_dir=$(CDPATH= cd -- "$script_dir/.." && pwd)
log_dir="$root_dir/.logs"
log_file="$log_dir/gdb-qemu.log"
pid_file="$log_dir/gdb-qemu.pid"

mkdir -p "$log_dir"

if [ -f "$pid_file" ]; then
        old_pid=$(cat "$pid_file")
        if kill -0 "$old_pid" 2>/dev/null; then
                kill "$old_pid" 2>/dev/null || true
                sleep 1
        fi
        rm -f "$pid_file"
fi

program_path="$1"
case "$program_path" in
        /*) ;;
        *) program_path="$root_dir/$program_path" ;;
esac

ARGS="-kernel \"$program_path\""
ARGS="$ARGS -device isa-debug-exit,iobase=0xf4,iosize=0x04"
ARGS="$ARGS -S -s"
ARGS="$ARGS -no-reboot -no-shutdown"
ARGS="$ARGS -monitor none"

case "$(uname -s)" in
        Darwin)
                ARGS="$ARGS -display cocoa,zoom-to-fit=on"
                ;;
        Linux)
                ARGS="$ARGS -display gtk,zoom-to-fit=on"
                ;;
esac

eval "nohup qemu-system-i386 $ARGS >\"$log_file\" 2>&1 &"

qemu_pid=$!
echo "$qemu_pid" >"$pid_file"

sleep 1

if ! kill -0 "$qemu_pid" 2>/dev/null; then
        printf '%s\n' "QEMU failed to start. See $log_file for details." >&2
        exit 1
fi
