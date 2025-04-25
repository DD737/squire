
both: asm run
	$:

asm:
	cargo run --bin squire_asm -- -o kernel.bin kernel0.asm kernel1.asm kernel2.asm kernel3.asm kernel_last.asm

run: 
	cargo run --bin squire_vm kernel.bin

program:
	cargo run --bin squire_asm -- -o test.bin programs/test0.asm
