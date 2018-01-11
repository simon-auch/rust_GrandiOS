#!/bin/bash
ARGS=""
CMD=""
TARGET="debug"
CLEAN=false
ARGS_COUNT=$((${#@}))
CMD=""
if [[ "${@: -1}" != '--'* ]]; then
  #the last argument is the path to qemu
  CMD=${@: -1}
  ARGS_COUNT=$((${ARGS_COUNT}-1))
fi
TEMP_ARGS=${@:1:$ARGS_COUNT}
if [[ $TEMP_ARGS = *'--release'* ]]; then
  TARGET="release"
  ARGS=$ARGS"--release "
fi
if [[ $TEMP_ARGS = *'--clean'* ]]; then
  CLEAN=true
fi

LINKER_SIMON="arm-none-eabi-ld"
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



OBJCOPY_SIMON="arm-none-eabi-objcopy"
OBJCOPY_FU="/home/mi/linnert/arm/bin/arm-none-eabi-objcopy"
OBJCOPY_ARCH="/usr/arm-none-eabi/bin/objcopy"

OBJCOPY=$OBJCOPY_FU
which $OBJCOPY_SIMON >/dev/null 2>&1
if [ $? -eq 0 ]; then
  OBJCOPY=$OBJCOPY_SIMON
fi
which $OBJCOPY_ARCH >/dev/null 2>&1
if [ $? -eq 0 ]; then
  OBJCOPY=$OBJCOPY_ARCH
fi

#make sure cargo,rustup,xargo is in the path
if [ -f $HOME/.cargo/env ]; then
  source $HOME/.cargo/env
fi
which xargo >/dev/null 2>&1
if [ $? -eq 1 ]; then
  PATH=~/.cargo/bin:$PATH
fi
RUST_TARGET_PATH=$PWD
export RUST_TARGET_PATH

#build
#First we build the shell
cd shell
if [[ "$CLEAN" = true ]]; then #ja das ist bescheuert = true zu machen, aber es ist bash und sonst kann man das kaput machen
  xargo clean
fi
xargo build --target armv4t-none-eabi $ARGS
if [ $? -ne 0 ]; then exit; fi
#add prefixes for the symbols.
$OBJCOPY --prefix-symbols=_shell target/armv4t-none-eabi/$TARGET/libshell.a shell.a
cd ..

#Now we build the kernel. the binarys of the programs will be statically linked into the kernel
if [[ "$CLEAN" = true ]]; then #ja das ist bescheuert = true zu machen, aber es ist bash und sonst kann man das kaput machen
  xargo clean
fi
xargo build --target armv4t-none-eabi $ARGS
if [ $? -ne 0 ]; then exit; fi
$OBJCOPY target/armv4t-none-eabi/$TARGET/libGrandiOS.a kernel.a
#link + cleanup
$LINKER --gc-sections -Tlinker.lds -o kernel kernel.a shell/shell.a

if [[ "$CMD" != "" ]]; then
  $CMD -kernel kernel
fi
