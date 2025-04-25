syscalls: [int 0x02]
syscall id in ra

00 -> invalid AKA error
    invalid syscall code
01 -> exit
    exits the program
02 -> calls __out on rb
03 -> calls __in  on rb