#![allow(non_snake_case)]
#![allow(unused_parens)]

use std::path::Path;
use colored::*;
use erebos::instructions::Error;
use disasm::*;

pub mod disasm;

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

    let mut infile: Option<String> = None;
    let mut outfile = String::new();

    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next()
    {
        match a.as_str()
        {
            "-o" =>
            {

                let a = match args.next()
                {
                    Some(s) => s,
                    None =>
                    {
                        print_err("Expected output file after option '-o'!");
                        return;
                    }
                };

                let path = Path::new(&a);

                if(path.exists() && !path.is_file())
                {
                    print_err(format!("'{a}' is not a file!"));
                    return;
                }

                outfile = a;

            },
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

    let mut d = match handle_err(DASM::new(infile.into()))
    {
        Some(d) => d,
        None => return,
    };

    let out = match handle_err(d.disassemble())
    {
        Some(o) => o,
        None => return,
    };

    if(outfile.is_empty())
    {
        println!("{}", out);
    }
    else if let Err(e) = std::fs::write(outfile, out)
    {
        print_err(e);
    }
    
    println!("{}", "-------------------------".magenta());

}
