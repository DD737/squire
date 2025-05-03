#![allow(dead_code)]
#![allow(unused_parens)]
#![allow(non_upper_case_globals)]
#![allow(non_snake_case)]

use colored::{Colorize, ColoredString};
<<<<<<< HEAD
use squire::instructions::Error;
=======
use eval::Evaluator;
use erebos::instructions::Error;
>>>>>>> 3814d3a (version 0.9.2.1, rename to erebos)

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

<<<<<<< HEAD
    let mut parser = match handle_err(parse::Parser::file("code.lure".into()))
=======
    //println!("{}", "--------------------------------------------------".red());
    //
    //let mut parser = match handle_err(parse::Parser::file("code.nox".into()))
    //{
    //    Some(p) => p,
    //    None => return,
    //};
    //
    //while(!parser.tokenizer.end_of_tokens())
    //{
    //    //println!("{:?}", parser.tokenizer.get_next_token(Some(true)));
    //    let s = match handle_err(parser.parse_statement())
    //    {
    //        Some(s) => s,
    //        None => return,
    //    };
    //    println!("{s:?}");
    //    //println!("{:?}", parser.parse_statement());
    //}

    println!("{}", "--------------------------------------------------".red());

    let mut eval = match handle_err(eval::Evaluator::file("code.nox".into()))
>>>>>>> 3814d3a (version 0.9.2.1, rename to erebos)
    {
        Some(p) => p,
        None => return,
    };

    println!("{}", "--------------------------------------------------".red());
    
    while(!parser.tokenizer.end_of_tokens())
    {
<<<<<<< HEAD
        //println!("{:?}", parser.tokenizer.get_next_token(Some(true)));
        let s = match handle_err(parser.parse_statement())
=======
        Some(t) => t,
        None => return,
    };

    let t = tree.borrow();

    println!("{}", "--------------------\nVariables: ".yellow());
    for c in &t.variables
    {
        println!("{:?}", c.borrow());
    }

    println!("{}", "--------------------\nFunctions: ".yellow());
    for c in &t.functions
    {

        let steps = match handle_err(eval.ir_steps_sequ_scope(c.borrow().scope.clone()))
>>>>>>> 3814d3a (version 0.9.2.1, rename to erebos)
        {
            Some(s) => s,
            None => return,
        };
        println!("{s:?}");
        //println!("{:?}", parser.parse_statement());
    }

    println!("{}", "--------------------------------------------------".red());

}
