%public_static_void_main_string_args MAIN
%section code

%ext putstr
%ext readline
%ext print_pre_shell
%ext shell_readline
%ext shell_split_args
%ext shell_input_buffer
%ext shell_input_vector
%ext shell_try_run_cmd
%ext shell_info_running
%ext terminate_running_program

MAIN:

    # set up interrupt handler at INT
    movir 1, ra
    __io 0xF0
    movir INT, ra
    __io 0x02

    movim 1, shell_info_running
    # activate shell

MAIN_LOOP:
    cmprm rz, shell_info_running
    jifi MAIN_END, E # shell running false => end
    cali SHELL
    jmpi MAIN_LOOP

MAIN_END:
    hlt

_INT_get_adr:
    movir 2, ra
    __io 0xF0 # iodevice = MemoryManager
    __io 0x01 # ResumeMapping()
        lea rb # get adr
    __io 0x00 # SuspendMapping()
    
    movir 1, ra
    __io 0xF0 # iodevice = InterruptHandler
    ret

INT:
    movrm r1, TMP_REG
    pshr ra
    
    movir 2, ra
    __io 0xF0 # iodevice = MemoryManager
    __io 0x00 # SuspendMapping()
    
    movir 1, ra
    __io 0xF0 # iodevice = InterruptHandler
    __io 0x00 # GetInterruptID()

    movir 2, r1

    cmprr ra, r1
    popr ra
    jifi _INT_default, AB # skip if not syscall

    cmprr ra, rz
 jifi _INT_sys_not_00, AB
        movir txt3, ra
        cali putstr
        hlt
 _INT_sys_not_00:

    cmprr ra, r1 # out
 jifi _INT_sys_not_02, AB
        __out rb
 _INT_sys_not_02:

    movir 1, r1
    cmprr ra, r1
 jifi _INT_sys_not_01, AB # exit
        cali terminate_running_program
        movir '\n, ra
        __out ra
        popr r1
        jmpi MAIN_LOOP
 _INT_sys_not_01:

    movir 3, r1
    cmprr ra, r1
 jifi _INT_sys_not_03, AB # in
        __in  rb
 _INT_sys_not_03:

    movir 4, r1
    cmprr ra, r1
 jifi _INT_sys_not_04, AB # putstr
        cali _INT_get_adr
        movrr rb, ra
        cali putstr
 _INT_sys_not_04:

    movir 5, r1
    cmprr ra, r1
 jifi _INT_sys_not_05, AB # getstr
        cali _INT_get_adr
        movrr rb, ra
        cali readline
 _INT_sys_not_05:

    movir 6, r1
    cmprr ra, r1
 jifi _INT_sys_not_06, AB # ray call
    movir 2, ra
    __io 0xF0 # iodevice = MemoryManager
    __io 0x01 # ResumeMapping()
        movrr rb, ra
        __io 0xF1
    __io 0x00 # SuspendMapping()
    
    movir 1, ra
    __io 0xF0 # iodevice = InterruptHandler
 _INT_sys_not_06:

    movmr TMP_REG, r1
    jmpi _INT_sysret

 _INT_default:
    movmr TMP_REG, r1
    movir msg1, ra
    cali putstr    
 _INT_sysret:

    movir 2, ra
    __io 0xF0 # iodevice = MemoryManager
    __io 0x01 # ResumeMapping()
    
    movir 1, ra
    __io 0xF0 # select int handler for io

    __io 0x06 # ResolveInterruptNoRSP()
    dbg # unreachable
hlt

SHELL:
    movim 0, shell_input_vector

    cali print_pre_shell
    cali shell_readline
    cali shell_split_args

    # cali SHELL_DBG

    cali shell_try_run_cmd

    cmprr ra, rz
    jifi SHELL_ERR, E

    ret

SHELL_ERR:
    movir txt2_0, ra
    cali putstr
    movir shell_input_buffer, ra
    cali putstr
    movir txt2_1, ra
    cali putstr
    ret

SHELL_DBG:
    movir txt0, ra
    cali putstr
    movir shell_input_buffer, ra
    cali putstr
    movir '\n, ra
    __out ra

    movir shell_input_vector, ra
 __loop:
    movrar ra, rb
    cmprr rb, rz
    jifi __end, E
    inc ra inc ra inc ra inc ra
    pshr ra
    movir txt1, ra
    cali putstr
    movrr rb, ra
    cali putstr
    movir '\n, ra
    __out ra
    popr ra
    jmpi __loop
 __end:
    movir '\n, ra
    __out ra
    __out ra
    ret

%section data

TMP_REG: dd 0

msg0: db "hewwwooo mrrrrrp :3\n",0
msg1: db "INTERRUPT\n",0

txt0: db "CMD: ",0
txt1: db "ARG: ",0
txt2_0: db "The command '",0
txt2_1: db "' doesnt exist!\n",0
txt3: db "Invalid 0 syscall!\n",0

