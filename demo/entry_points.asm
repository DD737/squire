
# this demo shows how to switch entry points
# set this tp a different main and watch is print a different message!
%entry main0

%section code

main0: 
    movir msg0, ra
    cali putstr
    hlt
    resb 8
main1: 
    movir msg1, ra
    cali putstr
    hlt
    resb 8
main2: 
    movir msg2, ra
    cali putstr
    hlt
    resb 8

hlt

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

msg0: db "this is message 0",0
msg1: db "this is message 1",0
msg2: db "this is message 2",0
