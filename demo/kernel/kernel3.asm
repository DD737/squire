# command index and internal commands
%section code

%ext putstr
%ext listlen
%ext shell_input_vector
%ext __FILE
%ext __FILE_length
%ext execute_loaded_file

_cmd_help_fun:
    movir shell_command_index, rb
 _cmd_help_fun_loop:
    movrar rb, ra
    cmprr ra, rz
    jifi _cmd_help_fun_end, E

    inc rb inc rb inc rb inc rb
    inc ra inc ra inc ra inc ra # skip fn ptr

    pshr rb # secure rb
        cali putstr
    popr rb

    movir '\n, ra
    __out ra
    jmpi _cmd_help_fun_loop

 _cmd_help_fun_end:
    ret

_cmd_greet_fun:
    movir _cmd_greet_msg0, ra
    cali putstr
    ret

_cmd_exit_fun:
    movim 0, shell_info_running
    movir _cmd_exit_msg0, ra
    cali putstr
    ret

_cmd_echo_fun:
    movir shell_input_vector, ra
    cali listlen
    movir 1, rb
    cmprr ra, rb
    jifi _cmd_echo_fun_err, B # args.len() < 1 => err

    movir shell_input_vector, ra
    movrar ra, ra
    cali putstr
    movir '\n, ra
    __out ra
    ret

 _cmd_echo_fun_err:
    movir _cmd_echo_err, ra
    cali putstr
    ret

_cmd_exec_fun:

    movir 0, ra
    __io 0xF0 # iodevice = FileSystem

    movir shell_input_vector, ra
    cali listlen
    movir 1, rb
    cmprr ra, rb
    jifi _cmd_exec_fun_err0, B # args.len() < 1 => err

    movir shell_input_vector, ra
    movrar ra, ra

    __io 0x04 # FileExists()

    cmprr rb, rz
    jifi _cmd_exec_fun_err0, E # if file doesnt exist err
    
    movir __FILE, rb
    __io 0x0E # QuickRead()

    cmprr rz, rd
    jifi _cmd_exec_fun_err1, AB # unequ 0 => err

    movrm rc, __FILE_length

    cali execute_loaded_file
 # unreachable
    dbg
    ret

 _cmd_exec_fun_err0:
    movir _cmd_exec_err0, ra
    cali putstr
    ret
 _cmd_exec_fun_err1: # result in rd
    pshr rd

    movir _cmd_exec_err1_0, ra
    cali putstr

    popr ra
    movir '0, rb
    addrrr rb, ra, ra
    __out ra

    movir _cmd_exec_err1_1, ra
    cali putstr

    ret

hlt

%section data

%exp shell_info_running
shell_info_running: db 0,0,0,0

%exp shell_command_index
shell_command_index: # contains ptrs
    # data is encoded as following: u32: ptr to func
    #                               str: name of the cmd, 0 terminated
    db _shell_command_help
    db _shell_command_exit
    db _shell_command_echo
    db _shell_command_greet
    db _shell_command_exec
    resb 100

_shell_command_help:
    db  _cmd_help_fun
    db "help",0
_shell_command_exit:
    db  _cmd_exit_fun
    db "exit",0
_shell_command_echo:
    db  _cmd_echo_fun
    db "echo",0
_shell_command_greet:
    db  _cmd_greet_fun
    db "greet",0
_shell_command_exec:
    db  _cmd_exec_fun
    db "exec",0

_cmd_greet_msg0: db "Hello! Welcome to the kernel-demo!\n",0
_cmd_exit_msg0: db "Exiting...\n",0

_cmd_echo_err: db "Echo: expects an input argument!\n",0
_cmd_exec_err0: db "Exec: expects a file!\n",0
_cmd_exec_err1_0: db "Exec: Error when reading file: ",0
_cmd_exec_err1_1: db "Exec: \n",0

