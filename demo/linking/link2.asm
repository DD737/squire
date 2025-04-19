# contains the data

# -----SECTION_BSS------

%exp running
running: db 1, 1

# -----SECTION_DATA-----

%exp space
space: db 0," ",0
%exp backspace
backspace: db 0,'\b,0
%exp newline
%exp newline_0
newline: db 0
newline_0: db '\n,0
%exp quote
quote: db 34,0

%exp pre_shell
pre_shell: db "> ",0

%exp name_cmd_help
%exp name_cmd_exit
%exp name_cmd_hi
%exp name_cmd_quote
name_cmd_help: db "help",0
name_cmd_exit: db "exit",0
name_cmd_hi: db "hi",0
name_cmd_quote: db "quote",0

%exp msg_cmd_help_0
msg_cmd_help_0: 
    db "help      -> prints this help\n"
    db "exit      -> exits the program\n"
    db "hi        -> prints a little greeting\n"
    db "quote str -> prints the given text in quotation marks\n"
    db 0

%exp msg_cmd_exit_0
msg_cmd_exit_0: db "Exiting...",0

%exp msg_cmd_hi_0
msg_cmd_hi_0: db "hewwwoooo :3333\n",0

%exp msg_cmd_none_0
%exp msg_cmd_none_1
msg_cmd_none_0: db "ERROR: COMMAND '",0
msg_cmd_none_1: db "' DOESNT EXIST!\n",0

%exp input_section_cmd
%exp input_section_arg
%exp input_section_all
input_section_cmd: resb 101
input_section_arg: resb 401
input_section_all: resb 601
