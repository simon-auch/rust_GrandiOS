#!/bin/bash

DUMP_SIMON="arm-linux-gnueabihf-objdump"
DUMP_FU="/home/mi/linnert/arm/bin/arm-none-eabi-objdump"
DUMP_ARCH="/usr/arm-none-eabi/bin/objdump"

DUM=$DUMP_FU
which $DUMP_SIMON >/dev/null 2>&1
if [ $? -eq 0 ]; then
  DUMP=$DUMP_SIMON
fi
which $DUMP_ARCH >/dev/null 2>&1
if [ $? -eq 0 ]; then
  DUMP=$DUMP_ARCH
fi


$DUMP -fhd kernel > kernel.dump
