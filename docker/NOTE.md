docker buildx build --platform linux/amd64 --target build -t rust --load -f docker/Dockerfile .


nasm -felf32 boot/i686/assembly/boot.nasm -o boot.o

cargo build --target i686-unknown-linux-gnu

gcc -T boot/i686/linker/linker.ld -m32 -o os.bin -nostdlib boot.o target/i686-unknown-linux-gnu/debug/deps/crate_name-9e49ae5c2961af79.3ddzjgbe0qh0iywh.rcgu.o -lgcc

