
%header version 0

%entry _MAIN

%section code

_MAIN:

    movir 1, ra
    __io 0xF0 # switch io to InterruptHandler

    movir INT, ra
    __io 0x2 # SetInterruptHandlerLocation()

    movir 0, ra
    __io 0xF0 # switch io to FileSystem

MAIN:
    movir shell_pre, ra
    cali putstr

    cali readline

    bmovmr buffer, rb
    cmprr rb, rz
jifi _MAIN_skip, AB # unequ => non empty
    hlt # input empty
_MAIN_skip:

    movir buffer, ra # ptr to path
    __io 0x04 # FileExists()
    
    cmprr rz, rb
    jifi invalid_err, E # print invalid err if res != 0
    
    movir msg_reading0, ra
    cali putstr
    movir buffer, ra
    cali putstr
    movir msg_reading1, ra
    cali putstr

    movir buffer, ra
    movir data, rb
    __io 0x0E # QuickRead()

    bmovrm rz, buffer

    movir 2, ra
    __io 0xF0 # switch io to MemoryManager

    movir 0, ra
    movrr rc, rb # put length in rb
    movir data_entry, rc # map adr
    __io 0x02 # SetMap()
    pshr rd

    movir 1, ra
    __io 0xF0 # switch io to InterruptHandler

    __io 0x5 # SetSubMode()

    movir 0, ra
    __io 0xF0 # switch io to FileSystem
    
    movir data_entry, rip # actual jump to data

    hlt # unreachable

INT:

    movir 2, ra
    __io 0xF0 # switch io to MemoryManager

    popr ra
    __io 3 # RmvMap()

    movir 1, ra
    __io 0xF0 # switch io to InterruptHandler

    __io 0x4 # RemoveInterrupt()

    movir 0, ra
    __io 0xF0 # switch io to FileSystem

    movir '\n, ra
    __out ra
    
    jmpi MAIN

invalid_err:
    movir error_invalid0, ra
    cali putstr
    movir buffer, ra
    cali putstr
    movir error_invalid1, ra
    cali putstr
    jmpi MAIN

readline:
    movir buffer, rb # ptr to input section
    movir 1, rc

 _readline_loop:
    __in ra
    cmprm ra, backspace
    jifi _readline_backspace, E # if read character == '\b', go handle backspace
    __out ra # get next char
    cmprm ra, newline
    jifi _readline_end, E # if read character == '\n', stop reading
    
    bmovrra ra, rb # write read char
    addrrr rc, rb, rb # inc ptr
    jmpi _readline_loop

 _readline_backspace:

    movir buffer, rd
    cmprr rb, rd
    jifi _readline_backspace_skip_dec, EB # if rb is <= its initial, dont dec
    
    __out ra
    movir " ", rd
    __out rd # print " " to overwrite prev char
    __out ra # print \b to set caret at correct position
    subrrr rb, rc, rb # dec ptr

 _readline_backspace_skip_dec:
    movir 0, rd
    bmovrra rd, rb # unwrite read char
    jmpi _readline_loop

 _readline_end:
    ret

putstr:
    movir 1, rb
 __putstr_loop:
    bmovrar ra, rc
    cmprr rc, rz
    jifi __putstr_end, E
    __out rc
    addrrr ra, rb, ra
    jmpi __putstr_loop
 __putstr_end:
    ret

hlt

%section data

shell_pre: db "Please insert a file to run\n> ", 0
msg_preline: db "- ",0
msg_reading0: db "Executing file '",0
msg_reading1: db "':\n\n",0

error_invalid0: db "'",0
error_invalid1: db "' is not a file!\n\n",0

backspace: db 0,'\b,0
newline: db 0,'\n,0
buffer: resb 200
data: resb 32
data_entry: resb 0x300
