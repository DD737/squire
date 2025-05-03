#![allow(dead_code)]
#![allow(unused_parens)]
#![allow(non_upper_case_globals)]
#![allow(non_snake_case)]

use colored::{Colorize, ColoredString};
use eval::Evaluator;
use erebos::instructions::Error;

pub mod preproc;
pub mod token;
pub mod parse;
pub mod eval;

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
    {
        Some(p) => p,
        None => return,
    };

    let tree = match handle_err(eval.generate_ir())
    {
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
        {
            Some(s) => s,
            None => return,
        };

        for s in steps
        {
            println!("{:?} = {:?}", s.0, s.1);
        }

    }

    println!("{}", "--------------------------------------------------".red());

    // let l = squire::instructions::SourceLocation::default();
    // let v = Evaluator::eval_type_check(&EvalValue::Complex(eval::eval_value::EvalValueComplex::OpBinary(
    //     eval::eval_value::EvalValueComplexOpBinary::ADD, 
    //     Box::new(EvalValue::Number(12, l.clone())), 
    //     Box::new(EvalValue::Symbol(
    //             eval::eval_value::EvalSymbol::Variable(
    //                 std::rc::Rc::new(RefCell::new(EvalVariable
    //                 {
    //                     constant: false,
    //                     id: 14,
    //                     initializer: Some(EvalValue::Number(2, l.clone())),
    //                     name: "asdf".to_string(),
    //                     unique_name: "asdf".to_string(),
    //                     parent: std::rc::Rc::downgrade(&tree),
    //                     storage: eval::EvalStorage::Register(squire::instructions::IRRegister::RA),
    //                     r#type: Rc::new(RefCell::new(eval::types::EvalType::Internal(eval::types::EvalTypeInternal::Void))),
    //                     value: None,
    //                 }))
    //             )
    //         , l.clone()))
    // ), l.clone()));
    //
    // if let Some(a) = handle_err(v)
    // { println!("{a:?}"); }
    //
    // println!("{}", "--------------------------------------------------".red());

}
