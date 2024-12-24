FROM rust:latest

ARG TARGET_MODE

WORKDIR /kfs

COPY . /kfs

RUN apt-get update -y && \
    apt-get install -y \
    jq \
    gcc-multilib \
    grub \
    xorriso \
    grub-pc-bin

RUN make iso TARGET_MODE=${TARGET_MODE}
