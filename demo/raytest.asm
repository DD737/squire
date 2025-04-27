%entry 0
%section code

pshi 800
pshi 600
pshi title
bpshi 1
movir 0xEF, ra # OpenWindow
__io 0xF1


pshi 60
movir 0xED, ra # SetTargetFPS
__io 0xF1

loop:

movir 0x01, ra # BeginDrawing
__io 0xF1

movir 0xD0, ra # IsWindowResized
__io 0xF1
bpopr ra
cmprr ra, rz
jifi __not_resized, E

movir 0xD1, ra # GetWindowWidth
__io 0xF1
popm WIDTH
movir 0xD2, ra # GetWindowHeight
__io 0xF1
popm HEIGHT

__not_resized:

pshm RED
movir 0x05, ra # ClearBackground
__io 0xF1

pshi 0
pshi 0

pshm WIDTH
pshi 2
divs

pshm HEIGHT
pshi 2
divs

pshm BLUE
movir 0x03, ra # DrawRectangle
__io 0xF1

pshi 10
pshi 10
movir 0x04, ra # DrawFPS
__io 0xF1

movir 0x02, ra # EndDrawing
__io 0xF1

movir 0x00, ra # WindowShouldClose
__io 0xF1
bpopr ra
cmprr ra, rz
jifi loop, E

movir 0xEE, ra # CloseWindow
__io 0xF1

hlt

%section data

title: db "test window",0
RED:  db 0xFF, 0x00, 0x00, 0xFF
BLUE: db 0x00, 0x00, 0xFF, 0xFF

WIDTH:  dd 800
HEIGHT: dd 600
