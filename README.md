<<<<<<< HEAD
# This project has been renamed to [Erebos](https://github.com/DD737/erebos), this branch will no longer receive updates!

# Welcome to the squire project I guess
=======
# Welcome to the Erebos project I guess
>>>>>>> 3814d3a (version 0.9.2.1, rename to erebos)
## "What the fart is this?"
This is me building a computer "from scratch" where I essentially create my own instruction set, virtual machine, assembler, linker, programmin language and maybe more.

## "How do I contribute?"
You dont?

## "Why am I reading this then?"
Good question, go do something productive.

# There are examples / demos in the [demo](./demo) folder
(The linking examples requires all three files to be assembled together)

# Near Future Roadmap
### More fundamental features
- binary header
- sectioned executable
### Wishful thinking
- abstracted linker step (linker as seperate executable) (enables linking of e.g. different languages)
- support for raylib instructions for graphics

# Hopefully Near-Ish Future Roadmap
### Programming language that compiles to erebos_asm
- likely has c-style or rust-like syntax (?) (subject to change)
- supports libraries and generally should be less of a pain to work with than the asm
### Maybe Maybe Maybe Maybe compilation from `actual` programming languages
- perhaps lua could be abused to serve this ?
- there could potentionally be a cursed 16b c compiler that waits to be sacrificed somewhere on the internet
- this will likely be harder than making my own language but would make actually using this system a lot nicer

# Changelog
- ## Version 0.8
    - initial commit
    - working VM:
        - missing binary header support
        - missing section support
        - does support command line file input
        - command line option `-d` -> prints every executed instruction
    - working assembler
        - missing binary header support
        - missing section support
        - fully supports label and linking
        - command line file specifications
        - command line args `-t`, `-d` and `-s` for debugging different assembler stages [printed for each file]
        - missing command line arg for debugging linking
- ## Version 0.9
    - finally added code sections (after many errors)
    - added binary header version 0x0000
        - `%header stack size` to set stack size, assembler enforces minimum of 1024, vm enforces 512
        - `%header stack loc` to set the stacks position in ram, assembler only allows minimum of 0x1000, vm allows minimum of 0x100
        - `%flags` to set flags (currently does nothing)
        - `%entry` and label or imm as entry point of the program, directly sets the instruction pointer of the vm [ `%entry` also has the alias `%public_static_void_main_string_args` if youre into that ]
    - missing: support of sectioned data in vm
    - missing: standalone linker, although it has been abstracted into external library
- ## Version 0.9.1
    - [almost] standalone linker
    - IO for files, interrupts and memory mapping
    - kernel, subkernel and user modes
    - [kernel-demo!](demo/kernel/) [run the make file and use `exec programs/test0.bin` to test running a program on the kernel!]
    - more demo programs that have also been used for debugging
    - expanded instruction set
    - removed ray instruction (will be moved to io device)
    - added [bad] little disassembler for debugging (please dont try to actually use it its not great)
- ## Version 0.9.2
    - added raylib support! [see [instructionset](instructionset.txt) for more details]
    - added syscall to kernel-demo to support raylib calls [check out [the new test program](demo/kernel/programs/test1.asm)!]
    - added debug symbol support for assembler and vm (use `-d filename` on the asm for a debug symbol table that can be used with `-f filename` on the vm for better errors, `-D filename` can be used on the asm to get a human readable version of the table)
    - new instruction `lea` to get the result of memory mapping [stolen from actually good assembly of course]
    - added minor new instructions to io devices (just check the [instructionset](instructionset) for more info)
    - lots of bugs have been resolved [i couldnt (be bothered to) keep track of them]
    - very close to a state that can be called 1.0 release!
- ## Version 0.9.2.1
    - rename to Erebos!!
    - significant strides to a compiled language! more on this later
    - compiled language rename to nox
