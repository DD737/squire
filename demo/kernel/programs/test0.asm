%entry 0
%section code

movir 0x04, ra
movir msg, rb
int 0x02 # putstr

movir 0x05, ra
movir buffer, rb
int 0x02 # getstr

movir 0x04, ra
movir buffer, rb
int 0x02 # putstr

movir 0x01, ra
int 0x02 # exit

%section data

msg: db "Please enter a message: ",0
buffer: resb 100
