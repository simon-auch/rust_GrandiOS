default:
	./setup_env.sh
	make build

build:
	./kernel_build.sh

run:
	LD_PRELOAD=/home/mi/linnert/arm/lib/libpng.so.0 ./kernel_build.sh "/home/mi/linnert/arm/bin/qemu-bsprak" -piotelnet
