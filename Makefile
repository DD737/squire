ifndef args
	$(error use args="..." to pass arguments)
endif

vm:
	@cargo run --bin erebos_vm -- $(args)
asm:
	@cargo run --bin erebos_asm -- $(args)
dasm:
	@cargo run --bin erebos_dasm -- $(args)
