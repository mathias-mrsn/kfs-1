KERNEL_NAME := kfs1
KERNELDIR := /kfs
OUTPUTDIR := ./output
DOCKER_PATH := ./docker
BUILDX_PLATFORM := linux/amd64
IMG_NAME := ${KERNEL_NAME}_image
LOGSDIR := ./.logs

.PHONY: all
all:	build copy remove

.PHONY: build
build:
			@printf "${YELLOW}${BOLD}%-10s${WHITE}%s${END}\n" "[ LOG ]" "Building ${KERNEL_NAME} docker image..."
			@mkdir -p ${LOGSDIR}
			@docker buildx build \
				--platform ${BUILDX_PLATFORM} \
				--target build \
				-t ${IMG_NAME} \
				--load \
				-f ${DOCKER_PATH}/Dockerfile \
				--build-arg KERNEL_NAME=${KERNEL_NAME} \
				--build-arg KERNELDIR=${KERNELDIR} \
				. \
				&> ${LOGSDIR}/build.log || \
				(printf "${RED}${BOLD}%-10s${WHITE}%s${END}\n" "[ FATAL ]" "Error while building ${KERNEL_NAME} docker image. Check \".logs/build.log\" for more details." && \
				exit 1)
			@printf "${GREEN}${BOLD}%-10s${WHITE}%s${END}\n" "[ OK ]" "Docker image created"

.PHONY: copy
copy:
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
				${KERNEL_NAME}:${KERNELDIR}/output/. \
				${OUTPUTDIR} \
				&> ${LOGSDIR}/copy.log || \
				(printf "${RED}${BOLD}%-10s${WHITE}%s${END}\n" "[ FATAL ]" "Error while copying ${KERNEL_NAME} output directory. Check \".logs/copy.log\" for more details." && \
				exit 1)
			@printf "${GREEN}${BOLD}%-10s${WHITE}%s${END}\n" "[ OK ]" "${KERNEL_NAME} output directory succesfully copied"

.PHONY: remove
remove:
			@printf "${YELLOW}${BOLD}%-10s${WHITE}%s${END}\n" "[ LOG ]" "Removing ${KERNEL_NAME} container..."
			@docker rm \
				-f \
				${KERNEL_NAME} \
				&> ${LOGSDIR}/remove.log || \
				(printf "${RED}${BOLD}%-10s${WHITE}%s${END}\n" "[ FATAL ]" "Error while removing ${KERNEL_NAME} container. Check \".logs/remove.log\" for more details." && \
				exit 1)
			@printf "${GREEN}${BOLD}%-10s${WHITE}%s${END}\n" "[ OK ]" "${KERNEL_NAME} container succesfully removed"

.PHONY: run
run:
			@printf "${YELLOW}${BOLD}%-10s${WHITE}%s${END}\n" "[ LOG ]" "Running ${KERNEL_NAME} container..."
			@docker run \
				--name ${KERNEL_NAME} \
				--platform ${BUILDX_PLATFORM} \
				-it ${IMG_NAME}

.PHONY: run-bin
run-bin:
			@printf "${YELLOW}${BOLD}%-10s${WHITE}%s${END}\n" "[ LOG ]" "Running ${KERNEL_NAME}.bin in QEMU..."
			@qemu-system-i386 -kernel ${OUTPUTDIR}/${KERNEL_NAME}.bin

.PHONY: run-iso
run-iso:
			@printf "${YELLOW}${BOLD}%-10s${WHITE}%s${END}\n" "[ LOG ]" "Running ${KERNEL_NAME}.iso in QEMU..."
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

