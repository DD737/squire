%entry 0
%section code

movir 0x02, ra
movir 'a, rb
int 0x02 # putc

movir 0x01, ra
int 0x02 # exit
