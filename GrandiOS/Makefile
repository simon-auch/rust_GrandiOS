default:
	./setup_env.sh
	make build

build:
	LD_PRELOAD=/home/mi/linnert/arm/lib/libpng.so.0 ./kernel_build.sh "/home/mi/linnert/arm/bin/qemu-bsprak" -piotelnet
