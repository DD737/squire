# Welcome to the squire project I guess
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
### Programming language that compiles to squire_asm
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