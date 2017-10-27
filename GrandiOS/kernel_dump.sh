#!/bin/bash

DUMP_SIMON="arm-linux-gnueabihf-objdump"
DUMP_FU="/home/mi/linnert/arm/bin/arm-none-eabi-objdump"

DUMP=$DUMP_FU

$DUMP -fhd kernel > kernel.dump
