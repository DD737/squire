
%public_static_void_main_string_args 0
%section code

movir greeting, ra
cali putstr

cali query_name

movir answer0, ra
cali putstr

movir name, ra
cali putstr

movir answer1, ra
cali putstr

hlt

query_name:
    movir name, rc
    movir 1, rb
    movir '\n, rd
_query_name_loop:
    cali _query_char
    cmprr ra, rd
    jifi _query_name_end, e
    cali _query_push
    jmpi _query_name_loop
_query_name_end:
    ret

_query_char:
    __in ra
    __out ra
    ret

_query_push:
    bmovrra ra, rc
    addrrr rc, rb, rc
    ret

putstr:
    movrr ra, r1
    movir 1, r2
__putstr_loop:
    bmovrar r1, r3
    cmprr r3, rz
    jifi __putstr_end, e
    __out r3
    addrrr r1, r2, r1
    jmpi __putstr_loop
__putstr_end:
    ret

hlt

%section data

greeting: db "Please enter your name: ", 0
answer0: db "Hello ", 0
answer1: db ", how are you today?\n", 0
name:
