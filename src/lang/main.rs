#![allow(dead_code)]
#![allow(unused_parens)]
#![allow(non_upper_case_globals)]
#![allow(non_snake_case)]

use colored::{Colorize, ColoredString};
use squire::instructions::Error;

pub mod preproc;
pub mod token;
pub mod parse;

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

    let mut parser = match handle_err(parse::Parser::file("code.lure".into()))
    {
        Some(p) => p,
        None => return,
    };

    println!("{}", "--------------------------------------------------".red());
    
    while(!parser.tokenizer.end_of_tokens())
    {
        //println!("{:?}", parser.tokenizer.get_next_token(Some(true)));
        let s = match handle_err(parser.parse_statement())
        {
            Some(s) => s,
            None => return,
        };
        println!("{s:?}");
        //println!("{:?}", parser.parse_statement());
    }

    println!("{}", "--------------------------------------------------".red());

}
