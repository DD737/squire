#![allow(unused_assignments)]
#![allow(unused_parens)]
use std::{fmt::Display, fs::write, path::Path};
use std::sync::Arc;
use asm::{AsmTokenizer, __asm::ASM, __dir::AsmDirector};
use erebos::instructions::Error;
use colored::*;
use erebos::link::Linker;
use erebos::executable::__internal::Format;

pub mod asm;

fn print_err(e: impl Display)
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

fn print_tokens(file: Arc<str>) -> Result<(), Error>
{

    let mut tk = AsmTokenizer::file(file)?;

    loop
    {
        let s = tk.get_token().transpose()?;
        match s
        {
            Some(s) => println!("{:?}", s),
            None => break,
        }
    }

    println!();

    Ok(())

}
fn print_direct(file: Arc<str>) -> Result<(), Error>
{
    
    let mut dr = AsmDirector::file(file)?;

    loop
    {
        let s = dr.get_token().transpose()?;
        match s
        {
            Some(s) => println!("{:?}", s),
            None => break,
        }
    }

    println!();

    Ok(())

}
fn print_parser(file: Arc<str>) -> Result<(), Error>
{
    
    let mut a = ASM::file(file)?;

    loop
    {
        let s = a.parse_statement()?;
        match s
        {
            Some(s) => println!("{:?}", s),
            None => break,
        }
    }

    println!();

    Ok(())

}

fn main()
{

    let mut _debug_print_tokens = false;
    let mut _debug_print_direct = false;
    let mut _debug_print_parser = false;

#[allow(unused_variables)]
    let mut outfile = String::from("out.bin");
    let mut infiles: Vec<Arc<str>> = Vec::new();

    let mut debug_cr_file: Option<String> = None;
    let mut debug_hr_file: Option<String> = None;

    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next()
    {
        match a.as_str()
        {

            "-t" => _debug_print_tokens = true,
            "-p" => _debug_print_direct = true,
            "-s" => _debug_print_parser = true,

            "-o" =>
            {
                outfile = match args.next()
                {
                    Some(s) => s,
                    None =>
                    {
                        print_err("Expected file after -o!");
                        return;
                    }
                }
            },

            "-d" =>
            {
                debug_cr_file = match args.next()
                {
                    Some(s) => Some(s),
                    None =>
                    {
                        print_err("Expected file after -d!");
                        return;
                    }
                }
            },
            "-D" =>
            {
                debug_hr_file = match args.next()
                {
                    Some(s) => Some(s),
                    None =>
                    {
                        print_err("Expected file after -d!");
                        return;
                    }
                }
            },

            _ => 
            {
                let path = Path::new(&a);
                if(!path.exists())
                {
                    print_err(format!("'{}' doesnt exist!", a));
                    return;
                }
                if(!path.is_file())
                {
                    print_err(format!("The '{}' is not a file!", a));
                    return;
                }
                infiles.push(a.into());
            }

        }
    }

    let _print_any = _debug_print_tokens | _debug_print_direct | _debug_print_parser;

    println!("{}", "-------------------------".magenta());

    if(_print_any)
    {
        for file in &infiles
        {
            println!("{}{}{}", "For file: ".green(), file.red(), ":".green());
            if(_debug_print_tokens)
            {
                println!("{}", "Tokens:".cyan());
                match handle_err(print_tokens(file.clone()))
                {
                    Some(_) => {},
                    None => return,
                }
            }
            if(_debug_print_direct)
            {
                println!("{}", "Tokens [post directive]:".cyan());
                match handle_err(print_direct(file.clone()))
                {
                    Some(_) => {},
                    None => return,
                }
            }
            if(_debug_print_parser)
            {
                println!("{}", "Statements:".cyan());
                match handle_err(print_parser(file.clone()))
                {
                    Some(_) => {},
                    None => return,
                }
            }
        }
    }

    let assemblers = match handle_err(infiles.into_iter().map(ASM::file).collect::<Result<Vec<ASM>, Error>>())
    {
        Some(a) => a,
        None => return,
    };
    let formats = match handle_err(assemblers.into_iter().map(|mut a|a.parse()).collect::<Result<Vec<Format>, Error>>())
    {
        Some(f) => f,
        None => return,
    };
    
    let mut ln = Linker::formats(formats);

    let exe = match handle_err(ln.link())
    {
        Some(e) => e,
        None => return,
    };

    let bytes = Linker::executable_to_bytes(exe.0, true);
    
    let _ = write(outfile, bytes);

    if let Some(debug_hr_file) = debug_hr_file
    {
        
        let mut output = String::new();

        for d in &exe.1
        {
            output.push_str(&d.to_line());
            output.push('\n');
        }

        let _ = write(debug_hr_file, output);

    }
    if let Some(debug_cr_file) = debug_cr_file
    {
        
        let mut output: Vec<u8> = vec![0xFF, 0xFF];

        for d in &exe.1
        {
            output.append(&mut d.to_bytes());
        }

        let _ = write(debug_cr_file, output);

    }

    println!("{}", "-------------------------".magenta());

}




//    let mut a = ASM::code(code);
//
//    let bytes = match a.parse()
//    {
//        Err(e) => { println!("{}", ColoredString::from(format!("{e}")).bright_red()); return; }
//        Ok(s) => s,
//    };
//
//    let _ = write("test.bin", bytes.bytes);
