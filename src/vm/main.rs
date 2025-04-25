#![allow(non_snake_case)]
#![allow(unused_parens)]

use std::{fs::read, path::Path};
use colored::{Colorize, ColoredString};
use vm::VM;
use squire::instructions::Error;

pub mod vm;
pub mod fs;

fn print_err(e: impl std::fmt::Display)
{
    println!("{}", ColoredString::from(format!("{e}")).bright_red());
}

fn handle_err<T>(t: Result<T, Error>) -> Option<T>
{
    match t
    {
        Ok(t) => Some(t),
        Err(e) => 
        {
            print_err(e);
            None
        }
    }
}

fn main()
{

    let mut _enable_debug_print  = false;
    let mut _enable_section_mode = false;
    let mut _register_dump = false;

    let args = std::env::args().skip(1);
    let mut infile: Option<String> = None;

    for a in args
    {
        match a.as_str()
        {
            "-d" => _enable_debug_print  = true,
            "-s" => _enable_section_mode = true,
            "-r" => _register_dump = true,
            _ => 
            {
                
                if let Some(ref f) = infile
                {
                    print_err(format!("Multiple input files specified! Only one file can be executed. ({}, {})", f, a));
                    return;
                }

                let path = Path::new(&a);

                if(!path.exists())
                {
                    print_err(format!("'{a}' doesnt exist!"));
                    return;
                }

                if(!path.is_file())
                {
                    print_err(format!("'{a}' is not a file!"));
                    return;
                }

                infile = Some(a);

            },
        }
    }

    let infile = match infile
    {
        Some(f) => f,
        None =>
        {
            print_err("No input file specified!".to_string());
            return;
        }
    };
    
    println!("{}", "-------------------------".magenta());

    let mut vm = VM::new();

    let bytes = match read(infile)
    {
        Ok(s) => s,
        Err(e) =>
        {
            print_err(format!("Error reading file: {e}"));
            return;
        }
    };

    let _ = vm.load_executable(bytes);

    if(_enable_debug_print ) { vm.enable_debug_print (); }
    if(_enable_section_mode) { vm.enable_section_mode(); }

    if handle_err(vm.run()).is_none()
    {

        println!("Current RIP: {:#04x} ({:#04x} in bin)", vm.instruction_pointer, vm.instruction_pointer.overflowing_add(32).0);

        let dump_start = std::cmp::max(vm.instruction_pointer, 0x0005) - 5;
        let dump_end   = std::cmp::min(vm.instruction_pointer, 0xFFFA) + 5;

        let mut pre_line = String::new();

        for ptr in dump_start..dump_end
        {
            if(ptr < vm.instruction_pointer)
            {
                pre_line.push_str("     ");
            }
            print!("{}", format!("{:#04x} ", vm.memget(ptr).unwrap()).blue());
        }
        println!();

        println!("{pre_line}{}", "^^^^".bright_blue());

        return;

    }

    println!("{}", "\n\r-------------------------".magenta());
    println!("\rExecution finished!");

    if(_register_dump)
    {
        println!("Register dump:");
        for r in vm.registers
        {
            print!("{:#x} ", r);
        }
        println!();
    }
    
    println!("{}", "\r-------------------------".magenta());

}
