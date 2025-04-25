
%public_static_void_main_string_args 0
%section code

%def ptr r9
%def pos r3

%def last r5
%def curr r6

cali read_input

movmr space, %last
movir list, %ptr
movir input, %pos

cali split_words

movir list, %ptr
loop:
    movrar %ptr, ra
    cmprr ra, rz
    jifi end, E
    cali putstr
    addmrr two, %ptr, %ptr
    movir '\n, ra
    __out ra
    jmpi loop
end:
hlt

read_input:
    movir input, rc
    movir '\n, rb
_read_input_loop:
    __in ra
    __out ra
    cmprr ra, rb
    jifi _read_input_end, E
    bmovrra ra, rc
    addrmr rc, one, rc
    jmpi _read_input_loop
_read_input_end:
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

split_words:

    bmovrar %pos, %curr

    cmprr %curr, rz
    jifi _split_words_skip, E

    cmprm %curr, space
    jifi _split_words_space_mode, E

    cmprm %last, space
    jifi _split_words_next_mode, AB

    movrra %pos, %ptr
    addmrr two, %ptr, %ptr
    
_split_words_next_mode:

    addmrr one, %pos, %pos
    movrr %curr, %last
    cali split_words
_split_words_skip:
    ret
    
_split_words_space_mode:
    bmovrra rz, %pos

    addmrr one, %pos, %pos
    movmr space, %last
        cali split_words
    ret

hlt

%section data

one: db 0, 1
two: db 0, 2
space: db 0, " "

input: resb 400
db 0, 0
list: resw 200
db 0,0
