#![allow(unused_parens)]
use std::{fmt::Display, fs::write, path::Path};
use std::sync::Arc;
use asm::{AsmTokenizer, __asm::ASM, __dir::AsmDirector, __linc::Linker};
use squire::instructions::Error;
use colored::*;

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

    println!("");

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

    println!("");

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

    println!("");

    Ok(())

}

fn main()
{

    let mut _debug_print_tokens = false;
    let mut _debug_print_direct = false;
    let mut _debug_print_parser = false;

    let mut outfile = String::from("out.bin");
    let mut infiles: Vec<Arc<str>> = Vec::new();

    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next()
    {
        match a.as_str()
        {

            "-t" => _debug_print_tokens = true,
            "-d" => _debug_print_direct = true,
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

    println!("{}", "-------------------------".red());

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

    let mut link = match handle_err(Linker::file(infiles))
    {
        Some(l) => l,
        None => return,
    };

    let bytes = match handle_err(link.link())
    {
        Some(l) => l,
        None => return,
    };

    let _ = write(outfile, bytes);

    println!("{}", "-------------------------".red());

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
