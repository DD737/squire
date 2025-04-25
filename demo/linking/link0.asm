# manages sub components

%section code

%header stack size 500
%header stack loc  500

%public_static_void_main_string_args main
main:
    cali shell
    hlt

%ext running

%ext shell_reset
%ext shell_preprint
%ext shell_read_line
%ext shell_split


shell:
    cmpmr running, rz
    jifi _shell_end, E # if not running, exit

    cali shell_reset
    cali shell_preprint
    cali shell_read_line
    cali shell_split

    cali shell_decide_cmd # execute the command

    jmpi shell # re-run the shell

_shell_end:
    ret

%ext str_eq

%ext input_section_cmd
%ext input_section_arg

%ext name_cmd_help
%ext name_cmd_exit
%ext name_cmd_hi
%ext name_cmd_quote

%ext exec_cmd_help
%ext exec_cmd_exit
%ext exec_cmd_hi
%ext exec_cmd_quote
%ext exec_cmd_none

shell_decide_cmd:

    movir name_cmd_help, ra
    movir input_section_cmd, rb
    cali str_eq
    cmprr ra, rz
    jifi _shell_decide_cmd_not_help, E # if strings not equal, dont execute
        cali exec_cmd_help
        ret
_shell_decide_cmd_not_help:

    movir name_cmd_exit, ra
    movir input_section_cmd, rb
    cali str_eq
    cmprr ra, rz
    jifi _shell_decide_cmd_not_exit, E # if strings not equal, dont execute
        cali exec_cmd_exit
        ret
_shell_decide_cmd_not_exit:

    movir name_cmd_hi, ra
    movir input_section_cmd, rb
    cali str_eq
    cmprr ra, rz
    jifi _shell_decide_cmd_not_hi, E # if strings not equal, dont execute
        cali exec_cmd_hi
        ret
_shell_decide_cmd_not_hi:

    movir name_cmd_quote, ra
    movir input_section_cmd, rb
    cali str_eq
    cmprr ra, rz
    jifi _shell_decide_cmd_not_quote, E # if strings not equal, dont execute
        cali exec_cmd_quote
        ret
_shell_decide_cmd_not_quote:

    # no command found => execute error command
    cali exec_cmd_none
    ret
