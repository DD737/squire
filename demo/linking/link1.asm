# all the little functions

%section code

# exports
%exp putstr
%exp str_eq
%exp exec_cmd_help
%exp exec_cmd_exit
%exp exec_cmd_hi
%exp exec_cmd_quote
%exp exec_cmd_none

%exp shell_reset
%exp shell_preprint
%exp shell_read_line
%exp shell_split

# imports
%ext running
%ext space
%ext backspace
%ext newline
%ext newline_0
%ext quote
%ext pre_shell

%ext input_section_cmd
%ext input_section_arg
%ext input_section_all

%ext msg_cmd_help_0
%ext msg_cmd_exit_0
%ext msg_cmd_hi_0
%ext msg_cmd_none_0
%ext msg_cmd_none_1

# -----SECTION_FUNC-----


exec_cmd_help:
    movir msg_cmd_help_0, ra
    cali putstr
    ret
exec_cmd_exit:
    movir msg_cmd_exit_0, ra
    cali putstr
    movim 0, running
    ret
exec_cmd_hi:
    movir msg_cmd_hi_0, ra
    cali putstr
    ret
exec_cmd_quote:
    movir quote, ra
    cali putstr
    movir input_section_arg, ra
    cali putstr
    movir quote, ra
    cali putstr
    movir newline_0, ra
    cali putstr
    ret
exec_cmd_none:
    movir msg_cmd_none_0, ra
    cali putstr
    movir input_section_cmd, ra
    cali putstr
    movir msg_cmd_none_1, ra
    cali putstr
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

str_eq:
    bmovrar ra, rc
    bmovrar rb, rd # fetch chars for str a and str b
    cmprr rc, rd
    jifi _str_eq_fine, E # if theyre equ, jump to fine

    movir 0, ra # if not return 0
    ret

_str_eq_fine: # both equ
    cmprr rc, rz
    jifi _str_eq_exit, E # rc == rd == 0 => both strings equal

    # != 0 => more chars to check
    movir 1, rc
    addrrr rc, ra, ra
    addrrr rc, rb, rb # inc both pointers
    jmpi str_eq

_str_eq_exit:
    movir 1, ra
    ret

clear_str:
    movir 1, rc
_clear_str_loop:
    bmovrar ra, rb
    cmprr rb, rz
    jifi _clear_str_end, E
    bmovrra rz, ra
    addrrr rc, ra, ra
    jmpi _clear_str_loop
_clear_str_end:
    ret



shell_reset:
    movir input_section_cmd, ra
    cali clear_str
    movir input_section_arg, ra
    cali clear_str
    movir input_section_all, ra
    cali clear_str
    ret
    


shell_preprint:
    movir pre_shell, ra
    cali putstr
    ret



shell_read_line:
    movir input_section_all, rb # ptr to input section
    movir 1, rc

_shell_read_line_loop:
    __in ra
    cmprm ra, backspace
    jifi _shell_read_line_backspace, E # if read character == '\b', go handle backspace
    __out ra # get next char
    cmprm ra, newline
    jifi _shell_read_line_end, E # if read character == '\n', stop reading
    
    bmovrra ra, rb # write read char
    addrrr rc, rb, rb # inc ptr
    jmpi _shell_read_line_loop

_shell_read_line_backspace:

    movir input_section_all, rd
    cmprr rb, rd
    jifi _shell_read_line_backspace_skip_dec, EB # if rb is <= its initial, dont dec
    
    __out ra
    movir " ", rd
    __out rd # print " " to overwrite prev char
    __out ra # print \b to set caret at correct position
    subrrr rb, rc, rb # dec ptr

_shell_read_line_backspace_skip_dec:
    movir 0, rd
    bmovrra rd, rb # unwrite read char
    jmpi _shell_read_line_loop

_shell_read_line_end:
    ret



shell_split:
    movir input_section_all, rb
    movir 1, rc
    movir input_section_cmd, rd
_shell_split_cmd_loop:
    bmovrar rb, ra

    cmprr ra, rz
    jifi _shell_split_end, E # stop on string end
    cmprm ra, space
    jifi _shell_split_arg_mode, E # switch to arg mode on space

    bmovrra ra, rd # write char 
    addrrr rb, rc, rb # inc src ptr
    addrrr rd, rc, rd # inc dst ptr
    jmpi _shell_split_cmd_loop

_shell_split_arg_mode:
    addrrr rb, rc, rb # inc src ptr
    bmovrar rb, ra

    cmprm ra, space
    jifi _shell_split_arg_mode, E # skip if its another space
    
    movir input_section_arg, rd # switch dst ptr to the arg location
_shell_split_arg_loop:
    bmovrar rb, ra

    cmprr ra, rz
    jifi _shell_split_end, E # stop on string end

    bmovrra ra, rd # write char
    addrrr rd, rc, rd # inc dst ptr
    addrrr rb, rc, rb # inc src ptr
    jmpi _shell_split_arg_loop

_shell_split_end:
    ret
