#!/bin/bash

LINKER_SIMON="arm-linux-gnueabihf-ld"
LINKER_FU="/home/mi/linnert/arm/bin/arm-none-eabi-ld"

LINKER=$LINKER_FU

#make sure cargo,rustup,xargo is in the path
source $HOME/.cargo/env

#build
xargo clean
xargo build --target armv4t-none-eabi
#link + cleanup
$LINKER --gc-sections -Tkernel.lds -o kernel target/armv4t-none-eabi/debug/libGrandiOS.a
