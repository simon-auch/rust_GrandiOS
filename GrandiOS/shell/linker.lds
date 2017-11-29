ENTRY(_start_shell) 
SECTIONS
{ 
. = 0x0;
.init : { *(.init) }
.text : { *(.text) }
}
