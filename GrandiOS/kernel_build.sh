#!/bin/bash

LINKER_SIMON="arm-linux-gnueabihf-ld"
LINKER_FU="/home/mi/linnert/arm/bin/arm-none-eabi-ld"
LINKER_ARCH="/usr/arm-none-eabi/bin/ld"

LINKER=$LINKER_FU
which $LINKER_SIMON >/dev/null 2>&1
if [ $? -eq 0 ]; then
  LINKER=$LINKER_SIMON
fi
which $LINKER_ARCH >/dev/null 2>&1
if [ $? -eq 0 ]; then
  LINKER=$LINKER_ARCH
fi

#make sure cargo,rustup,xargo is in the path
if [ -f $HOME/.cargo/env ]; then
  source $HOME/.cargo/env
fi
which xargo >/dev/null 2>&1
if [ $? -eq 1 ]; then
  PATH=~/.cargo/bin:$PATH
fi

#build
xargo clean
xargo build --target armv4t-none-eabi
#link + cleanup
$LINKER --gc-sections -Tkernel.lds -o kernel target/armv4t-none-eabi/debug/libGrandiOS.a

if [[ $? == 0 && "$@" != "" ]]; then
  $@ -kernel kernel
fi
