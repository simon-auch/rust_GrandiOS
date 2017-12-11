//This represantates the memory layout that gets pushed onto the stack when the interrupt starts.
macro_rules! build_register_stack {
    ($($name:ident),*) => (
        #[derive(Debug)]
        #[repr(C)]
        pub struct RegisterStack {
            $(pub $name: u32),*
        }
        impl RegisterStack {
            pub fn new() -> Self{
                RegisterStack{
                    $($name : 0),*
                }
            }
            pub fn copy(&mut self, source: & Self){
                $(self.$name = source.$name);*;
            }
        }
    );
}
build_register_stack!(sp, lr_usr, cpsr, r0, r1, r2, r3, r4, r5, r6, r7, r8, r9, r10, r11, r12, lr_irq);

#[macro_export]
macro_rules! trampolin {
    ($lr_offset:expr, $handler:ident) => (
        let reg_sp: u32;
        asm!(concat!(
           "sub r14, ", $lr_offset, "\n",
           "push   {r14}
            push   {r0-r12}  //save everything except the Stack pointer (useless since we are in the wrong mode) and r0 as we want to write our result to there
            mrs    r0, SPSR  //move the program status from the interrupted program into r0
            push   {r0}
            mrs    r3, CPSR  //switch to ARM_MODE_SYS to save the stack pointer and the lr register (the only shadowed ones)
            mov    r2, r3
            orr    r2, #0x1F
            msr    CPSR, r2
            mov    r0, sp    //move the stack pointer from thread into r0
            mov    r1, lr
            msr    CPSR, r3  //get back to interrupt mode
            push   {r0, r1}
            mov    r0, sp    //move the stackpointer to r0 to know where r0-r12,r14 is stored
            sub    sp, 0x40" //make a bit of space on the stack for rust, since rust creates code like: "str r0, [pc, #4]" it expects the sp to be decremented before once. The 0x40 is a random guess and provides space for a few var$
        ):"={r0}"(reg_sp)
        :::"volatile");
        {
            $handler(reg_sp);
        }
        asm!("
            add    sp, 0x40
            pop    {r0, r1}
            mrs    r3, CPSR  //switch to ARM_MODE_SYS to save the stack pointer
            mov    r2, r3
            orr    r2, #0x1F
            msr    CPSR, r2
            mov    sp, r0
            mov    lr, r1
            msr    CPSR, r3
            pop    {r0}
            msr    SPSR, r0
            pop    {r0-r12}
            pop    {r14}
            movs   pc, r14"
            ::::"volatile"
        );
    );
}
