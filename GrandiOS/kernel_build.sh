#!/bin/bash

#make sure cargo,rustup,xargo is in the path
source $HOME/.cargo/env

#build
xargo clean
xargo build --target armv4t-none-eabi
#link + cleanup
arm-linux-gnueabihf-ld --gc-sections -Tkernel.lds -o kernel target/armv4t-none-eabi/debug/libGrandiOS.a
