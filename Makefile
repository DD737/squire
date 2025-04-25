ifndef args
	$(error use args="..." to pass arguments)
endif

vm:
	@cargo run --bin squire_vm -- $(args)
asm:
	@cargo run --bin squire_asm -- $(args)
dasm:
	@cargo run --bin squire_dasm -- $(args)
