
%header version 0

%entry main

%section code

main:
    movir shell_pre, ra
    cali putstr

    cali readline

    movir namebuffer, ra # ptr to path
    __io 15 # SetRoot()
    
    cmprr rz, rb
    jifi invalid_err, AB # print invalid err if res != 0
    
    movir msg_reading0, ra
    cali putstr
    movir namebuffer, ra
    cali putstr
    movir msg_reading1, ra
    cali putstr

    cali readfiles

    movir '\n, ra
    __out ra

    jmpi main

invalid_err:
    movir error_invalid0, ra
    cali putstr
    movir namebuffer, ra
    cali putstr
    movir error_invalid1, ra
    cali putstr
    jmpi main

readline:
    movir namebuffer, rb # ptr to input section
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

    movir namebuffer, rd
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

readfiles:
    __io 0 # Reindex()
    __io 1 # GetFiles()

    cmprr rz, ra
    jifi _readfiles_end, E # skip if no files
    
    movrr ra, r2
    movir 0, r1 # 0 because loop inc at the start
    movir 1, r3

    movir '\n, r4
 _readfiles_loop:
    addrrr r3, r1, r1 # inc ptr

    movrr r1, ra # put the index
    movir namebuffer, rb # put the buffer location
    __io 16 # GetFileName()
    # rc now has the length
    pshr rc

    movir msg_preline, ra
    cali putstr

    popr rc
    movir namebuffer, ra
    movrr rc, rb
    cali putstr_len

    __out r4 # puts newline

    cmprr r1, r2
    jifi _readfiles_end, E
    jmpi _readfiles_loop

 _readfiles_end:
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

%section data

shell_pre: db "Please insert a directory to scan\n> ", 0
msg_preline: db "- ",0
msg_reading0: db "Reading directory '",0
msg_reading1: db "'...\n\n",0

error_invalid0: db "The path '",0
error_invalid1: db "' is not a valid directory!\n\n",0

backspace: db 0,'\b,0
newline: db 0,'\n,0
namebuffer: resb 200
