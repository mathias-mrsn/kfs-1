FROM rust:latest AS build

ARG KERNEL_NAME="kfs"
ARG ISODIR="/iso"
ARG KERNELDIR="/kfs"

COPY . ${KERNELDIR}

RUN apt-get update -y && \
    apt-get install -y \
    nasm \
    binutils \
    clang \
    gcc-multilib \
    vim \
    grub \
    xorriso \
    grub-pc-bin

RUN rustup default nightly && \
    rustup component add rust-src --toolchain nightly-x86_64-unknown-linux-gnu

# Rename kernel name
RUN sed -i "s/{KERNEL_NAME}/${KERNEL_NAME}/g" \
      ${KERNELDIR}/arch/i386/grub/grub.cfg \
      ${KERNELDIR}/Cargo.toml

RUN mkdir -p ${KERNELDIR}/output

RUN nasm -felf32 \
    -o ${KERNELDIR}/output/multiboot.o \
    ${KERNELDIR}/arch/i386/boot/multiboot.nasm

# Compile rust code.
RUN cargo build \
    -Z build-std \
    --release \
    --manifest-path=${KERNELDIR}/Cargo.toml \
    --target="${KERNELDIR}/arch/i386/target/i386-unknown-none.json"
RUN cp -R \
    ${KERNELDIR}/target/i386-unknown-none/release/lib${KERNEL_NAME}.a \
    ${KERNELDIR}/output/lib${KERNEL_NAME}.a

# Link kernel.
RUN ld \
    -m elf_i386 \
    -T "${KERNELDIR}/arch/i386/linker/linker.ld" \
    -o "${KERNELDIR}/output/${KERNEL_NAME}.bin" \
    ${KERNELDIR}/output/multiboot.o \
    ${KERNELDIR}/output/lib${KERNEL_NAME}.a

# Remove object file and static library.
RUN find ${KERNELDIR}/output -type f \( -name "*.o" -o -name "*.a" \) -delete

# Create iso file.
RUN mkdir -p ${ISODIR}/boot/grub
RUN cp ${KERNELDIR}/output/${KERNEL_NAME}.bin ${ISODIR}/boot/${KERNEL_NAME}.bin
RUN cp ${KERNELDIR}/arch/i386/grub/grub.cfg ${ISODIR}/boot/grub/grub.cfg
RUN grub-mkrescue -o ${KERNELDIR}/output/${KERNEL_NAME}.iso ${ISODIR}

WORKDIR ${KERNELDIR}

