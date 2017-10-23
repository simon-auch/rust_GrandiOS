#!/bin/bash

DUMP_SIMON="arm-linux-gnueabihf-objdump"
DUMP_FU="/home/mi/linnert/arm/bin/arm-none-eabi-objdump"

$DUMP_FU -fhd kernel > kernel.dump
