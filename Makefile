KERNEL_NAME := $(shell cargo metadata --format-version 1 | jq -r '.packages[].targets[] | select( .kind | map(. == "bin") | any ) | select ( .src_path | contains(".cargo/registry") | . != true ) | .name')

LOGSDIR := ./.logs
TARGET_MODE := debug
TARGET := i386-unknown-none
CARGO_OPTIONS :=
BUILDX_PLATFORM := linux/amd64
IMG_NAME := ${KERNEL_NAME}_image
ISO_IMAGE := ${KERNEL_NAME}.iso

ifeq ($(TARGET_MODE), release)
	CARGO_OPTIONS += --release 
	TARGET_MODE = release
endif

OUTPUTDIR := ./target/${TARGET}/${TARGET_MODE}

.PHONY: all
all:	build copy remove

.PHONY: docker-build
docker-build:
			@printf "${YELLOW}${BOLD}%-10s${WHITE}%s${END}\n" "[ LOG ]" "Building ${KERNEL_NAME} docker image..."
			@mkdir -p ${LOGSDIR}
			@docker buildx build \
				--platform ${BUILDX_PLATFORM} \
				-t ${IMG_NAME} \
				--load \
				-f ./docker/Dockerfile \
				--build-arg TARGET_MODE=${TARGET_MODE} \
				. \
				&> ${LOGSDIR}/build.log || \
				(printf "${RED}${BOLD}%-10s${WHITE}%s${END}\n" "[ FATAL ]" "Error while building ${KERNEL_NAME} docker image. Check \".logs/build.log\" for more details." && \
				exit 1)
			@printf "${GREEN}${BOLD}%-10s${WHITE}%s${END}\n" "[ OK ]" "Docker image created"

.PHONY: docker-copy
docker-copy:
			@printf "${YELLOW}${BOLD}%-10s${WHITE}%s${END}\n" "[ LOG ]" "Creating ${KERNEL_NAME} ${BUILDX_PLATFORM} container..."
			@mkdir -p ${LOGSDIR}
			@docker create \
				--name ${KERNEL_NAME} \
				--platform ${BUILDX_PLATFORM} \
				${IMG_NAME} \
				&> ${LOGSDIR}/create.log || \
				(printf "${RED}${BOLD}%-10s${WHITE}%s${END}\n" "[ FATAL ]" "Error while crating ${KERNEL_NAME} docker container. Check \".logs/create.log\" for more details." && \
				exit 1)
			@printf "${GREEN}${BOLD}%-10s${WHITE}%s${END}\n" "[ OK ]" "${KERNEL_NAME} container created"
			@printf "${YELLOW}${BOLD}%-10s${WHITE}%s${END}\n" "[ LOG ]" "Copying ${KERNEL_NAME} output directory to host..."
			@docker cp \
				${KERNEL_NAME}:/kfs/target \
				target \
				&> ${LOGSDIR}/copy.log || \
				(printf "${RED}${BOLD}%-10s${WHITE}%s${END}\n" "[ FATAL ]" "Error while copying ${KERNEL_NAME} output directory. Check \".logs/copy.log\" for more details." && \
				exit 1)
			@printf "${GREEN}${BOLD}%-10s${WHITE}%s${END}\n" "[ OK ]" "${KERNEL_NAME} output directory succesfully copied"

.PHONY: remove
docker-remove:
			@printf "${YELLOW}${BOLD}%-10s${WHITE}%s${END}\n" "[ LOG ]" "Removing ${KERNEL_NAME} container..."
			@docker rm \
				-f \
				${KERNEL_NAME} \
				&> ${LOGSDIR}/remove.log || \
				(printf "${RED}${BOLD}%-10s${WHITE}%s${END}\n" "[ FATAL ]" "Error while removing ${KERNEL_NAME} container. Check \".logs/remove.log\" for more details." && \
				exit 1)
			@printf "${GREEN}${BOLD}%-10s${WHITE}%s${END}\n" "[ OK ]" "${KERNEL_NAME} container succesfully removed"

# TODO: Delete this rule
.PHONY: docker-run
docker-run:
			@printf "${YELLOW}${BOLD}%-10s${WHITE}%s${END}\n" "[ LOG ]" "Running ${KERNEL_NAME} container..."
			@docker run \
				--name ${KERNEL_NAME} \
				--platform ${BUILDX_PLATFORM} \
				-it ${IMG_NAME}

.PHONY: build
build:
			@printf "${YELLOW}${BOLD}%-10s${WHITE}%s${END}\n" "[ LOG ]" "Building ${KERNEL_NAME} binary..."
			@cargo build ${CARGO_OPTIONS}
			@printf "${GREEN}${BOLD}%-10s${WHITE}%s${END}\n" "[ OK ]" "${KERNEL_NAME} binary succesfully built"
	
.PHONY: iso
iso: build
			@printf "${YELLOW}${BOLD}%-10s${WHITE}%s${END}\n" "[ LOG ]" "Creating ${KERNEL_NAME} image..."
			@mkdir -p iso/boot/grub
			@cp ${OUTPUTDIR}/${KERNEL_NAME} iso/boot/${KERNEL_NAME}.bin
			@cp arch/${TARGET}/grub/grub.cfg iso/boot/grub/grub.cfg
			@sed -i "s/_KERNEL_NAME_/${KERNEL_NAME}/g" iso/boot/grub/grub.cfg
			@grub-mkrescue -o ${OUTPUTDIR}/${KERNEL_NAME}.iso iso
			@printf "${GREEN}${BOLD}%-10s${WHITE}%s${END}\n" "[ OK ]" "${KERNEL_NAME} image succesfully built"

.PHONY: clear
clear:
			@printf "${YELLOW}${BOLD}%-10s${WHITE}%s${END}\n" "[ LOG ]" "Deleting ${KERNEL_NAME} binary and image..."
			@rm -rf ${KERNEL_NAME}.iso
			@cargo clean
			@printf "${GREEN}${BOLD}%-10s${WHITE}%s${END}\n" "[ OK ]" "${KERNEL_NAME} binary and image succesfully removed"
		

.PHONY: run-bin
run-bin:
			@printf "${YELLOW}${BOLD}%-10s${WHITE}%s${END}\n" "[ LOG ]" "Running ${KERNEL_NAME} binary in QEMU..."
			@qemu-system-i386 -kernel ${OUTPUTDIR}/${KERNEL_NAME}

.PHONY: run-iso
run-iso:
			@printf "${YELLOW}${BOLD}%-10s${WHITE}%s${END}\n" "[ LOG ]" "Running ${KERNEL_NAME} image in QEMU..."
			@qemu-system-i386 -cdrom ${OUTPUTDIR}/${KERNEL_NAME}.iso


GREY=	$'\033[30m
RED=	$'\033[31m
GREEN=	$'\033[32m
YELLOW=$'\033[33m
BLUE=	$'\033[34m
PURPLE=$'\033[35m
CYAN=	$'\033[36m
WHITE=	$'\033[37m
END= $'\033[37m
