use colored::{ColoredString, Colorize};
use crate::preproc::Preprocessor;

use std::sync::Arc;
use erebos::instructions::Error;
pub type Location = erebos::instructions::SourceLocation;
use erebos::error_in;

pub static operators_binary: &[&[&'static str]] = 
&[
    &["/", "*", "%",],
    &["-", "+",     ],
    &["^", "|", "&",],
    &["<<", ">>",   ],
    &["==", "!=", "<", ">", "<=", ">="],
    &["&&", "||",   ],
    &["=", "+=", "-=", "*=", "/=", "%=", "&=", "|=", "^=", "<<=", ">>="],
];

pub static operators_unary: &[&'static str] = 
&[
    "~", "-", "+", "!",
];


#[derive(Clone, PartialEq)]
pub enum TokenType
{
    Error,

    KeywordVarDef,
    KeywordFuncDef,
    KeywordStructDef,
    KeywordReturn,
    KeywordIf,
    KeywordElse,
    KeywordWhile,
    KeywordFor,

    TypeIndicator,
    LineTermination,
    ListSeperator,
    ObjectAccess,

    Identifier,
    Operator,

    BlockOpen,
    BlockClose,
    GroupOpen,
    GroupClose,
}

impl std::fmt::Debug for TokenType
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result 
    {
        match *self
        {
            TokenType::Error            => write!(f, "{}", " Error            ".magenta()),
            TokenType::KeywordVarDef    => write!(f, "{}", " KeywordVarDef    ".magenta()),
            TokenType::KeywordFuncDef   => write!(f, "{}", " KeywordFuncDef   ".magenta()),
            TokenType::KeywordStructDef => write!(f, "{}", " KeywordStructDef ".magenta()),
            TokenType::KeywordReturn    => write!(f, "{}", " KeywordReturn    ".magenta()),
            TokenType::KeywordIf        => write!(f, "{}", " KeywordIf        ".magenta()),
            TokenType::KeywordElse      => write!(f, "{}", " KeywordElse      ".magenta()),
            TokenType::KeywordWhile     => write!(f, "{}", " KeywordWhile     ".magenta()),
            TokenType::KeywordFor       => write!(f, "{}", " KeywordFor       ".magenta()),
            TokenType::TypeIndicator    => write!(f, "{}", " TypeIndicator    ".magenta()),
            TokenType::LineTermination  => write!(f, "{}", " LineTermination  ".magenta()),
            TokenType::ListSeperator    => write!(f, "{}", " ListSeperator    ".magenta()),
            TokenType::ObjectAccess     => write!(f, "{}", " ObjectAccess     ".magenta()),
            TokenType::Identifier       => write!(f, "{}", " Identifier       ".magenta()),
            TokenType::Operator         => write!(f, "{}", " Operator         ".magenta()),
            TokenType::BlockOpen        => write!(f, "{}", " BlockOpen        ".magenta()),
            TokenType::BlockClose       => write!(f, "{}", " BlockClose       ".magenta()),
            TokenType::GroupOpen        => write!(f, "{}", " GroupOpen        ".magenta()),
            TokenType::GroupClose       => write!(f, "{}", " GroupClose       ".magenta()),
        }
    }
}

#[derive(Clone)]
pub struct Token
{
    pub t_type: TokenType,
    pub raw: String,
    pub loc: Location,
}

impl std::fmt::Debug for Token
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result 
    {
        let raw = "\"".to_owned() + &self.raw + "\"";
        write!(f, "{} {{ {}: {:?}, {}: {:10} }}", "Token".green(), "t_type".cyan(), self.t_type, "raw".cyan(), (ColoredString::from(raw).bright_green()))
    }
}

fn keyword(kw: &str) -> Option<TokenType>
{
    match kw 
    {
        "struct" => Some(TokenType::KeywordStructDef),
        "var"    => Some(TokenType::KeywordVarDef),
        "func"   => Some(TokenType::KeywordFuncDef),
        "return" => Some(TokenType::KeywordReturn),
        "if"     => Some(TokenType::KeywordIf),
        "else"   => Some(TokenType::KeywordElse),
        "for"    => Some(TokenType::KeywordFor),
        "while"  => Some(TokenType::KeywordWhile),
        _        => None,
    }
}

#[derive(Debug)]
pub struct Tokenizer
{
    pub preproc: Preprocessor,
}
impl Tokenizer
{

    pub fn file(file: Arc<str>) -> Result<Self, Error>
    {
        Ok(Self
        {
            preproc: Preprocessor::file(file)?,
        })
    }

    pub fn peek(&mut self) -> Option<(char, Location)>
    { 
        let t = self.preproc.peek()?;
        Some(( *t.0, t.1 ))
    }
    pub fn next(&mut self) -> Option<(char, Location)> { self.preproc.next() }
    pub fn end_of_tokens(&mut self) -> bool { self.preproc.peek().is_none() }

    pub fn get_next_token(&mut self, inexistence_fine:Option<bool>) -> Result<Option<Token>, Error>
    {

        if(self.end_of_tokens())
        {
            if(!inexistence_fine.unwrap_or(false))
            {
                return Err(error_in!((self.preproc.get_loc()), "Requesteing too many tokens!"))
            }
            return Ok(None);
        }

        let loc = self.preproc.get_loc();

        let mut val = self.peek().unwrap_or(('\0', loc.clone()));
        while(val.0.is_whitespace()) { self.next(); val = self.peek().unwrap_or(('\0', loc.clone())); }

        if(self.end_of_tokens())
        {
            return Ok(None);
        }

        let token = match val.0
        {
            ';' => Some(Token { raw: val.0.to_string(), loc: val.1.clone(), t_type: TokenType::LineTermination }),
            '{' => Some(Token { raw: val.0.to_string(), loc: val.1.clone(), t_type: TokenType::BlockOpen }),
            '}' => Some(Token { raw: val.0.to_string(), loc: val.1.clone(), t_type: TokenType::BlockClose }),
            '(' => Some(Token { raw: val.0.to_string(), loc: val.1.clone(), t_type: TokenType::GroupOpen }),
            ')' => Some(Token { raw: val.0.to_string(), loc: val.1.clone(), t_type: TokenType::GroupClose }),
            ',' => Some(Token { raw: val.0.to_string(), loc: val.1.clone(), t_type: TokenType::ListSeperator }),
            ':' => Some(Token { raw: val.0.to_string(), loc: val.1.clone(), t_type: TokenType::TypeIndicator }),
            '.' => Some(Token { raw: val.0.to_string(), loc: val.1.clone(), t_type: TokenType::ObjectAccess }),
             _  => 
             'a: {
                
                let mut elligable_ops: Vec<&'static str> = Vec::new();

                for op in operators_unary 
                {
                    if(op.chars().nth(0).unwrap() == val.0)
                    {
                        elligable_ops.push(op);
                    }
                }
                for list in operators_binary 
                {
                    for op in *list 
                    {
                        if(op.chars().nth(0).unwrap() == val.0)
                        {
                            elligable_ops.push(op);
                        }
                    }
                }

                if(elligable_ops.len() < 1) { break 'a None; }

                self.next();

                let mut OP: &str = "";

                for op in elligable_ops.iter()
                {
                    if(op.len() > 1 && op.chars().nth(1).unwrap() == self.peek().unwrap().0)
                    {
                        OP = op;
                        self.next();
                        break;
                    }
                }

                if(OP.is_empty())
                {
                    for op in elligable_ops.iter()
                    {
                        if(op.len() == 1)
                        {
                            OP = op;
                            break;
                        }
                    }
                }

                if(OP.is_empty())
                {
                    panic!("No elligable operator found! {}", val.0);
                }

                Some(Token {
                    t_type: TokenType::Operator,
                    raw: OP.to_string(),
                    loc: val.1.clone(),
                })

             }
        };
        if let Some(token) = token
        {
            self.next();
            return Ok(Some(token));
        }

        let mut ident = String::new();

        while let Some(c) = match self.peek() 
        {
            Some(('_',_)) => Some('_'),
            Some((c,_)) if c.is_ascii_alphanumeric() => Some(c),
            Some((_,_)) => None,
            None    => None,
        }
        {
            self.next();
            ident.push(c);
        }

        if(ident.is_empty())
        {
            return Err(error_in!((val.1), "Magically empty identifier!! '{}'", self.peek().unwrap().0));
        }

        Ok(Some(Token
        {
            t_type: keyword(&ident).unwrap_or(TokenType::Identifier),
            raw: ident,
            loc: val.1,
        }))

    }

}
