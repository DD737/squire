# shell functionality

%section code

%ext putstr

%ext info_shell_hostname
%ext info_shell_location
%exp print_pre_shell
print_pre_shell:
    movir print_pre_shell_0, ra
    cali putstr

    movir info_shell_hostname, ra
    cali putstr

    movir print_pre_shell_1, ra
    cali putstr

    movir info_shell_location, ra
    cali putstr

    movir print_pre_shell_2, ra
    cali putstr

    ret

%ext shell_input_buffer
%ext readline
%exp shell_readline
shell_readline:
    movir shell_input_buffer, ra
    cali readline
    ret

%ext char_quote
%ext char_space
%ext shell_input_vector
%exp shell_split_args
shell_split_args:
    movir 0, rb # non zero means should put entry on not space
    movir shell_input_buffer, rc
    movir shell_input_vector, rd
 _shell_split_args_loop:
    bmovrar rc, ra # read char
    cmprm ra, char_space
    jifi _shell_split_args_isspace, E # handle space char
    
    cmprr ra, rz
    jifi _shell_split_args_break, E # if char is 0 then exit

    cmprr rb, rz
    jifi _shell_split_args_continue, E # if entry flag isnt set, skip

    cmprm ra, char_quote
    jifi _shell_split_args_quote_mode, E # equal and entry mode, quote mode

    movir 0, rb # unset entry flag
    cali _shell_split_args_addentry
    jmpi _shell_split_args_continue

 _shell_split_args_quote_mode:
    bmovrra rz, rc # replace space with 0
    inc rc
    cali _shell_split_args_addentry
    movir 0, rb # unset entry flag
 _shell_split_args_quote_mode_loop:
    bmovrar rc, ra # read char

    cmprm ra, char_quote
    jifi _shell_split_args_isspace, E # basically that situation
    
    cmprr ra, rz
    jifi _shell_split_args_break, E # if char is 0 then exit

    inc rc
    jmpi _shell_split_args_quote_mode_loop

 _shell_split_args_isspace:
    bmovrra rz, rc # replace space with 0
    movir 1, rb # set entry flag
 _shell_split_args_continue:
    inc rc
    jmpi _shell_split_args_loop

 _shell_split_args_break:
    ret

 _shell_split_args_addentry: # subroutine, val is in rc, rd is ptr
    movrra rc, rd
    inc rd inc rd
    ret

%ext streq
%exp shell_test_entry
shell_test_entry: # ra contains the entry adr, put 1 in ra on match, 0 on unequ
    movrar ra, ra # deref entry
    inc ra inc ra # ignore func ptr
    movir shell_input_buffer, rb
    cali streq
    ret

%exp shell_run_entry
shell_run_entry: # ra has entry adr
    movrar ra, ra # deref
    movrar ra, ra # deref func ptr
    calr ra
    ret

%ext shell_command_index
%exp shell_try_run_cmd
shell_try_run_cmd: # 1 in ra on success, 0 if not found
    movir shell_command_index, rc 
 _shell_try_run_cmd_loop:
    movrar rc, rd #deref
    cmprr rd, rz
    jifi _shell_try_run_cmd_exit_fail, E # end of registered cmds => exit fail

    pshr rc
    movrr rc, ra
    cali shell_test_entry
    popr rc

    cmprr rz, ra
    jifi _shell_try_run_cmd_found, AB # if not zero => found fit

    inc rc inc rc
    jmpi _shell_try_run_cmd_loop

 _shell_try_run_cmd_found:
    movrr rc, ra
    cali shell_run_entry
    movir 1, ra
    ret
 _shell_try_run_cmd_exit_fail:
    movir 0, ra
    ret

%section data

print_pre_shell_0: db 0,0
print_pre_shell_1: db " ",0
print_pre_shell_2: db " $ ",0

