
%header version 0

%entry main

%section code

main:
    movir shell_pre, ra
    cali putstr

    cali readline

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

    cali readfile

    movir '\n, ra
    __out ra
    __out ra

    jmpi main

invalid_err:
    movir error_invalid0, ra
    cali putstr
    movir buffer, ra
    cali putstr
    movir error_invalid1, ra
    cali putstr
    jmpi main

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

putstr_len: # ra=ptr, rb=len
    addrrr ra, rb, rb # ptr + len = upper bound
    movir 1, rd
 _putrstr_loop:
    cmprrr ra, rb
    jifi _putrstr_end, AE # is curr >= upper bound then end

    bmovrar ra, rc
    __out rc # output the curr char

    addrrr rd, ra, ra # inc curr
    jmpi _putrstr_loop
 _putrstr_end:
    ret

strlen: # ra=ptr
    movrr ra, rb
    movir 1, rc
 _strlen_loop:
    bmovrar ra, rd
    cmprr rz, rd
    jifi _strlen_end, E

    addrrr rc, ra, ra
    jmpi _strlen_loop

 _strlen_end:
    subrrr ra, rb, ra
    ret

readfile:
    movir buffer, ra
    movir data, rb
    __io 0x0E # QuickRead
    # rc holds length
    movir data, ra    # ra = first ptr
    addrrr ra, rc, rc # rc = last  ptr + 1
 _readfile_loop:
    cmprr ra, rc
    jifi _readfile_end, EA # counter >= length, break

    movrar ra, rb
    __out rb # get char and print
    inc ra
    jmpi _readfile_loop

 _readfile_end:
    ret

hlt

%section data

shell_pre: db "Please insert a file to read\n> ", 0
msg_preline: db "- ",0
msg_reading0: db "Reading file '",0
msg_reading1: db "':\n",0

error_invalid0: db "The path '",0
error_invalid1: db "' is not a file!\n\n",0

backspace: db 0,'\b,0
newline: db 0,'\n,0
buffer: resb 200
data: resb 500
