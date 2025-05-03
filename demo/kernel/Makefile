
all: asm dasm run
	$:

asm:
	cargo run --bin erebos_asm -- -o kernel.bin kernel0.asm kernel1.asm kernel2.asm kernel3.asm kernel_last.asm -d kernel.s -D kernel.S

dasm: 
	cargo run --bin erebos_dasm kernel.bin -o kernel.o

run: 
	cargo run --bin erebos_vm kernel.bin -f kernel.s

test0:
	cargo run --bin erebos_asm -- -o programs/test0.bin programs/test0.asm -d programs/test0.s -D programs/test0.S
test1:
	cargo run --bin erebos_asm -- -o programs/test1.bin programs/test1.asm -d programs/test1.s -D programs/test1.S
