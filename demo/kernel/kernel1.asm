# utility place and stuff

%section code

%exp listlen
listlen: # ra=ptr
    pshr ra
 _listlen_loop:
    movrar ra, rb
    cmprr rb, rz
    jifi _listlen_end, E
    inc ra inc ra inc ra inc ra
    jmpi _listlen_loop
 _listlen_end:
    popr rb
    subrrr ra, rb, ra
    ret

%exp putstr
putstr: # ra=ptr, 0 terminated string, prints chars till 0 is encountered
    bmovrar ra, rc
    cmprr rc, rz
    jifi __putstr_end, E
    __out rc
    inc ra
    jmpi putstr
 __putstr_end:
    ret

%exp streq
streq: # ra=str1, rb=str2, returns in ra 1 if both 0 terminated strings are equal, 0 if not
    bmovrar ra, rc
    bmovrar rb, rd
    cmprr rc, rd
    jifi _streq_exit_fail, AB # uneq if unequal

    cmprr rc, rz
    jifi _streq_exit_succ, E # equ when ending reached

    inc ra
    inc rb
    jmpi streq

 _streq_exit_succ:
    movir 1, ra
    ret
 _streq_exit_fail:
    movir 0, ra
    ret

%exp readline
readline: # ra=ptr, reads chars till \n, puts them into buffer, also adds 0 at the end
          # puts read length into rb [including 0 char]
    movrr ra, rb # copy ptr into counter
 _readline_loop:
    __in rc
    cmprm rc, char_backspace
    jifi _readline_back, E # handle backspace
    __out rc
    cmprm rc, char_newline
    jifi _readline_end, E # end on newline
    cali _readline_putc
    jmpi _readline_loop

 _readline_back:
    __out rc
    movmr char_space, rd
    __out rd
    __out rc # go back, print space, go back -> replaces last char with space and goes back

    cmprr ra, rb
    jifi _readline_loop, EA # if curr <= ptr dont reduce

    dec rb
    bmovrra rz, rb # reduce ptr, replace last char with 0
    jmpi _readline_loop

 _readline_end:
    movir 0, rc
    cali _readline_putc
    subrrr rb, ra, rb # subtracting original ptr changes it to amount of chars written
    ret

 _readline_putc: # sub routine; has char in rc
    bmovrra rc, rb
    inc rb
    ret

%ext __FILE_entry
%ext __FILE_length
%exp execute_loaded_file
execute_loaded_file:
    movir 2, ra
    __io 0xF0 # switch io to MemoryManager

    movir 0, ra
    movmr __FILE_length, rb # put length in rb
    movir __FILE_entry, rc # map adr
    __io 0x02 # SetMap()
    movrm rd, info_execution_mem_map_id # save ID

    movir 1, ra
    __io 0xF0 # switch io to InterruptHandler

    __io 0x1 # SetUserMode()

    movir __FILE_entry, rip # jump to execution

 # unreachable
    dbg
    ret

%exp terminate_running_program
terminate_running_program:

    movir 2, ra
    __io 0xF0 # switch io to MemoryManager
    
    movmr info_execution_mem_map_id, ra
    __io 3 # RmvMap()
    
    movir 1, ra
    __io 0xF0 # switch io to InterruptHandler
    
    __io 0x4 # RemoveInterrupt()
    
    ret

hlt

%section data

%exp info_execution_mem_map_id
info_execution_mem_map_id: db 0,0,0,0

%exp char_newline
%exp char_newline0
char_newline: db 0,0,0
char_newline0: db '\n,0

%exp char_backspace
%exp char_backspace0
char_backspace: db 0,0,0
char_backspace0: db '\b,0

%exp char_space
%exp char_space0
char_space: db 0,0,0
char_space0: db " ",0

%exp char_quote
%exp char_quote0
char_quote: db 0,0,0
char_quote0: db 34,0

%exp info_shell_hostname
info_shell_hostname: db "kernel-demo",0

%exp info_shell_location
info_shell_location: db "/" resb 200

%exp shell_input_buffer
shell_input_buffer: resb 500
%exp shell_input_vector
shell_input_vector: resb 100

