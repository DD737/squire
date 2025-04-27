use std::sync::Arc;
use std::fs::read_to_string;
use errors::Error;
use helpers::parse_escape;
use squire::instructions::*;
use squire::{error, error_in};
use squire::debug::DebugSymbol;

#[derive(Debug, Clone)]
pub enum Token
{
    Comma(SourceLocation),
    Colon(SourceLocation),
    String(String, SourceLocation),
    Number(i64, SourceLocation),
    Identifier(String, SourceLocation),
    NewLine(SourceLocation),
    Directive(SourceLocation),
}
impl PartialEq for Token
{
    fn eq(&self, other: &Self) -> bool 
    {
        match self
        {
            Token::NewLine    (_   ) => matches!(other, Token::NewLine    (_   )),
            Token::Directive  (_   ) => matches!(other, Token::Directive  (_   )),
            Token::Comma      (_   ) => matches!(other, Token::Comma      (_   )),
            Token::Colon      (_   ) => matches!(other, Token::Colon      (_   )),
            Token::String     (_, _) => matches!(other, Token::String     (_, _)),
            Token::Number     (_, _) => matches!(other, Token::Number     (_, _)),
            Token::Identifier (_, _) => matches!(other, Token::Identifier (_, _)),
        }
    }
}

#[allow(dead_code)]
fn token_location(tok: &Token) -> SourceLocation
{
    match tok
    {
        Token::Comma      (   l) => l.clone(),
        Token::Colon      (   l) => l.clone(),
        Token::String     (_, l) => l.clone(),
        Token::Number     (_, l) => l.clone(),
        Token::Identifier (_, l) => l.clone(),
        Token::NewLine    (   l) => l.clone(),
        Token::Directive  (   l) => l.clone(),
    }
}

#[derive(Debug)]
pub struct AsmTokenizer
{
    source: String,
    position: usize,
    last_char: Option<char>,
    line: i64,
    column: i64,
    file: Arc<str>,
}
impl AsmTokenizer
{

    pub fn code(src: String) -> Self
    {
        Self
        {
            source: src,
            position: 0,
            last_char: None,
            line: 1,
            column: 1,
            file: Arc::from("[unspecified]"),
        }
    }

    pub fn get_pos(&self) -> SourceLocation { SourceLocation::from(self.line, self.column, self.file.clone()) }

    pub fn file(file: Arc<str>) -> Result<Self, Error>
    {

        Ok(Self
        {
            source: match read_to_string(&*file)
            {
                Ok(s) => s,
                Err(e) => return Err(Error::IO(e)),
            },
            position: 0,
            last_char: None,
            line: 1,
            column: 1,
            file,
        })
    }

    fn _peek(&mut self) -> Option<&char>
    {
        if(self.last_char.is_none())
        {
            let c = self.source[self.position..].chars().next()?;
            self.position += c.len_utf8();
            self.last_char = Some(c);
        }
        self.last_char.as_ref()
    }
    fn _next(&mut self) -> Option<char>
    {
        self._peek()?;
        self.last_char.take()
    }

    fn peek(&mut self) -> Option<(&char, SourceLocation)>
    { 
        if(self._peek()? == &'\r')
        {
            self.next();
            self.peek()
        }
        else
        {
            let loc = self.get_pos();
            Some(( self._peek()?, loc )) 
        }
    }
    fn next(&mut self) -> Option<(char, SourceLocation)>
    {
        let c = self._next()?;
        if(c == '\n')
        {
            self.line += 1;
            self.column = 1;
        }
        else { self.column += 1; }
        Some(( c, SourceLocation::from(self.line, self.column, self.file.clone()) )) 
    }

    pub fn get_token(&mut self) -> Option<Result<Token, Error>>
    {

        {
            let c = self.peek()?;
            let c = (*c.0, c.1);
            if(c.0 == '\n')
            {
                self.next()?;
                return Some(Ok(Token::NewLine(c.1)));
            }
        }

        while let Some(c) = self.peek()
        {
            if(c.0.is_whitespace()) { self.next(); }
            else { break; }
        }

        let c = self.peek()?;
        let c = ( *c.0, c.1 );

        if(c.0 == '#')
        {
            while let Some(c) = self.peek()
            {
                if(*c.0 == '\n') { break; }
                else { self.next()?; }
            }
            return self.get_token();
        }

        if(c.0 == '%')
        {
            self.next()?;
            return Some(Ok(Token::Directive(c.1)));
        }

        let loc = c.1;

        Some(Ok(match c.0
        {

            ',' => { self.next(); Token::Comma(loc) },
            ':' => { self.next(); Token::Colon(loc) },

            _ => 
            {

                if(c.0 == '"' || c.0 == '\'')
                {

                    self.next()?;

                    let mut str = String::new();
                    if(c.0 == '"')
                    {
                        while let Some(c) = self.next()
                        {
                            if(c.0 == '"') { break; }
                            str.push(c.0);
                        }
                    }
                    else
                    {
                        while let Some(c) = self.peek()
                        {
                            if(!c.0.is_ascii_alphanumeric() && c.0 != &'\\' && c.0 != &'_') { break; }
                            str.push(self.next()?.0);
                        }
                    }

                    let str = match parse_escape(str, &loc)
                    {
                        Ok(s) => s,
                        Err(e) => return Some(Err(e)),
                    };

                    if(str.is_empty())
                    {
                        return Some(Err(error!("{}: Empty string are not allowed!", loc)))
                    }

                    Token::String(str, loc)

                }
                else if(c.0.is_numeric())
                {

                    let mut base = 10;

                    let mut str = String::new();
                    
                    if(c.0 == '0')
                    {
                        self.next()?;
                        let n = self.peek()?;
                        match n.0.to_ascii_lowercase()
                        {
                            'x' => { base = 16; self.next()?; },
                            'd' => { base = 10; self.next()?; },
                            'o' => { base =  8; self.next()?; },
                            'b' => { base =  2; self.next()?; },
                            _ =>
                            {
                                str.push(c.0);
                                // if(!n.0.is_numeric())
                                // {
                                //     return Some(Err(error_in!((n.1), "Invalid token '{}' in number!", n.0)));
                                // }
                            }
                        }
                    }


                    while let Some(c) = self.peek()
                    {
                        if(!c.0.is_numeric())
                        {
                            if(base != 16)
                            {
                                break;
                            }
                            match c.0.to_ascii_lowercase()
                            {
                                'a'|'b'|'c'|'d'|'e'|'f' => {},
                                _ => break,
                            }
                        }
                        str.push(self.next()?.0);
                    }

                    let num = match i64::from_str_radix(&str, base)
                    {
                        Ok(n) => n,
                        Err(_) => return Some(Err(error!("{}: Could not parse number! [if this was not supposed to be a number, prefix it with a non-numeric character]", loc))),
                    };

                    Token::Number(num, loc)

                }
                else
                {

                    let mut ident = String::new();

                    while let Some(c) = self.peek()
                    {
                        if(!c.0.is_alphanumeric() && *c.0 != '_' && *c.0 != '-') { break; }
                        else { ident.push(self.next()?.0); }
                    }
                    
                    Token::Identifier(ident, loc)

                }

            },

        }))

    }

}

pub mod __dir
{
    
    use super::*;
    use squire::executable::__internal::{Label, Section};
    use squire::instructions::helpers::HeaderConstructor;

    #[derive(Debug)]
    pub struct DirDefinition
    {
        pub loc: SourceLocation,
        pub name: String,
        pub value: Option<Token>,
    }

    pub struct AsmDirector
    {

        pub tokenizer: AsmTokenizer,

        token: Option<Token>,
        definitions: Vec<DirDefinition>,
        allow_block_close: u8,
        
        pub exported_labels: Vec<Label>,
        pub external_labels: Vec<Label>,

        pub section: Section,
        pub switched_section: bool,

        pub constructor: HeaderConstructor,

    }
    impl AsmDirector
    {

        pub fn code(src: String) -> Self
        {
            Self
            {

                tokenizer: AsmTokenizer::code(src),

                token: None,
                definitions: Vec::new(),
                allow_block_close: 0,

                exported_labels: Vec::new(),
                external_labels: Vec::new(),

                section: Section::None,
                switched_section: false,

                constructor: HeaderConstructor::new(),

            }
        }
        pub fn file(file: Arc<str>) -> Result<Self, Error>
        {
            Ok(Self
            {

                tokenizer: AsmTokenizer::file(file)?,

                token: None,
                definitions: Vec::new(),
                allow_block_close: 0,

                exported_labels: Vec::new(),
                external_labels: Vec::new(),

                section: Section::None,
                switched_section: false,

                constructor: HeaderConstructor::new(),

            })
        }

        pub fn get_pos(&self) -> SourceLocation { self.tokenizer.get_pos() }

        fn peek(&mut self) -> Result<Option<&Token>, Error>
        {
            if(self.token.is_none())
            {
                self.token = match self.tokenizer.get_token()
                {
                    Some(s) => match s 
                    {
                        Ok(s) => Some(s),
                        Err(e) => return Err(e),
                    },
                    None => None,
                };
            }
            Ok(self.token.as_ref())
        }
        fn next(&mut self) -> Result<Option<Token>, Error>
        {
            let _ = self.peek()?;
            Ok(self.token.take())
        }
        fn expect(&mut self, tok: Token) -> Result<Option<Token>, Error>
        {
            
            let token = match self.next()?
            {
                Some(t) => t,
                None => return Err(error_in!((self.tokenizer.get_pos()), "Expected {:?}, none given!", tok)),
            };

            if(token != tok)
            {
                return Err(error_in!((token_location(&token)), "Expected {:?}, {:?} given!", tok, token))
            }

            Ok(Some(token))

        }

        fn get_identifier(&mut self) -> Result<(String, SourceLocation), Error>
        {
            Ok(match self.expect(Token::Identifier(String::new(), SourceLocation::new()))?.unwrap()
            {
                Token::Identifier(n, l) => (n, l),
                _ => unreachable!(),
            })
        }
        fn get_string(&mut self) -> Result<(String, SourceLocation), Error>
        {
            Ok(match self.expect(Token::String(String::new(), SourceLocation::new()))?.unwrap()
            {
                Token::String(n, l) => (n, l),
                _ => unreachable!(),
            })
        }
        fn get_number(&mut self) -> Result<(i64, SourceLocation), Error>
        {
            Ok(match self.expect(Token::Number(0, SourceLocation::new()))?.unwrap()
            {
                Token::Number(n, l) => (n, l),
                _ => unreachable!(),
            })
        }


        fn header_parse(&mut self, name: &str, l: SourceLocation) -> Option<Result<(), Error>>
        {
            
            match name
            {

                "header" =>
                {

                    let (name,l) = match self.get_identifier() { Ok(s) => s, Err(e) => return Some(Err(e)) };

                    match name.as_str()
                    {
                        "stack" =>
                        {
                            
                            let (name,l) = match self.get_identifier() { Ok(s) => s, Err(e) => return Some(Err(e)) };

                            match name.as_str()
                            {
                                "loc" =>
                                {

                                    let (num,l) = match self.get_number() { Ok(s) => s, Err(e) => return Some(Err(e)) };

                                    if let Err(e) = self.constructor.set_stack_pos(num as u32, l)
                                    {
                                        return Some(Err(e))
                                    }
        
                                },
                                "size" =>
                                {

                                    let (num,l) = match self.get_number() { Ok(s) => s, Err(e) => return Some(Err(e)) };

                                    if let Err(e) = self.constructor.set_stack_size(num as u32, l)
                                    {
                                        return Some(Err(e));
                                    }
        
                                },
                                _ => return Some(Err(error_in!(l, "Expected valid option after %header stack! ('{}' given)", name)))
                            }

                        },
                        "flags" =>
                        {

                            let (num,l) = match self.get_number() { Ok(s) => s, Err(e) => return Some(Err(e)) };

                            if let Err(e) = self.constructor.set_flags(num as u8, l)
                            {
                                return Some(Err(e));
                            }

                        },
                        "files" =>
                        {

                            let (path,l) = match self.get_string() { Ok(s) => s, Err(e) => return Some(Err(e)) };

                            if let Err(e) = self.constructor.set_file_loc(path, l)
                            {
                                return Some(Err(e));
                            }

                        },
                        "version" =>
                        {
                            
                            if(self.constructor.constructing)
                            {
                                return Some(Err(error_in!(l, "Cannot change version after partially constructing the header!")))
                            }

                            let (num,_) = match self.get_number() { Ok(s) => s, Err(e) => return Some(Err(e)) };

                            self.constructor.version = num as u16;

                        },
                        _ => return Some(Err(error_in!(l, "Expected valid option after %header! ('{}' given)", name)))
                    }

                },

                "entry" | "public_static_void_main_string_args" =>
                {

                    if let Some(entry) = &self.constructor.entry
                    {
                        return Some(Err(error_in!(l, "Cannot re-define entry point! (Already defined here: {})", entry.fileloc)));
                    }

                    let token = match match self.next()
                    {
                        Ok(t) => t,
                        Err(e) => return Some(Err(e)),
                    }
                    {
                        Some(t) => t,
                        None => return Some(Err(error_in!(l, "Expected specifier for entry point!"))),
                    };

                    match token
                    {
                        Token::Identifier(name, l) =>
                        {
                            match self.constructor.set_entry(Label {
                                name,
                                fileloc: l.clone(),
                                pos: 0,
                            }, l)
                            {
                                Ok(_) => {},
                                Err(e) => return Some(Err(e)),
                            };
                        },
                        Token::Number(num, l) =>
                        {
                            match self.constructor.set_straight_entry(num as u32, l)
                            {
                                Ok(_) => {},
                                Err(e) => return Some(Err(e)),
                            };
                        },
                        _ => return Some(Err(error_in!(l, "Expected specifier for entry point!"))),
                    }

                },

                _ => return None,

            }

            Some(Ok(()))

        }

        fn parse_directive(&mut self) -> Option<Result<Token, Error>>
        {
            
            let token = match self.expect(Token::Identifier(String::new(), SourceLocation::new()))
            {
                Ok(t) => t,
                Err(e) => return Some(Err(e)),
            }.unwrap();

            match token
            {
                Token::Identifier(n, l) => 
                {
                    match n.as_str()
                    {
                        "section" =>
                        {
                            
                            let (name,_) = match self.get_identifier() { Ok(s) => s, Err(e) => return Some(Err(e)) };

                            let prev_section = self.section;

                            match name.to_ascii_lowercase().as_str()
                            {
                                "code" => self.section = Section::Code,
                                "data" => self.section = Section::Data,
                                _ => return Some(Err(error_in!(l, "Unrecognised section name '{}'!", name))),
                            };
                            self.switched_section = self.section != prev_section;

                            self.get_token()

                        },
                        "exp" | "ext" =>
                        {
                            
                            let (name,_) = match self.get_identifier() { Ok(s) => s, Err(e) => return Some(Err(e)) };

                            let label = Label 
                            {
                                name,
                                fileloc: l,
                                pos: 0,
                            };

                            if(n == "exp")
                            {
                                self.exported_labels.push(label);
                            }
                            else
                            {
                                self.external_labels.push(label);
                            }

                            self.get_token()

                        },
                        "def" => 
                        {
                            
                            let (name,_) = match self.get_identifier() { Ok(s) => s, Err(e) => return Some(Err(e)) };

                            let value = match self.get_token().transpose()
                            {
                                Ok(t) => t,
                                Err(e) => return Some(Err(e)),
                            };

                            self.definitions.push(DirDefinition
                            {
                                loc: l,
                                name,
                                value,
                            });

                            self.get_token()

                        },
                        "ifdef" | "ifndef" =>
                        {
                            
                            let (name,_) = match self.get_identifier() { Ok(s) => s, Err(e) => return Some(Err(e)) };
                            
                            let mut def: bool = false;

                            for d in &self.definitions
                            {
                                if(d.name == name)
                                {
                                    def = true;
                                    break;
                                }
                            }

                            if(n == "ifndef") { def = !def; }

                            if(def)
                            {
                                self.allow_block_close += 1;
                            }
                            else
                            {
                                match self.skip_block(l)
                                {
                                    Ok(_) => {},
                                    Err(e) => return Some(Err(e)),
                                };
                            }

                            self.get_token()

                        },
                        "endif" => 
                        {
                            if(self.allow_block_close < 1)
                            {
                                return Some(Err(error_in!(l, "Stray %endif found! No block to be closed")));
                            }
                            self.allow_block_close -= 1;
                            self.get_token()
                        }
                        "line"     => Some(Ok(Token::Number(l.line            , l))),
                        "line_str" => Some(Ok(Token::String(l.line.to_string(), l))),
                        _ => 
                        {

                            match self.header_parse(n.as_str(), l.clone())
                            {
                                Some(s) => 
                                { 
                                    match s
                                    {
                                        Ok(_) => {},
                                        Err(e) => return Some(Err(e)),
                                    };
                                    self.get_token() 
                                },
                                None =>
                                {

                                    let mut def: Option<&DirDefinition> = None;

                                    for d in &self.definitions
                                    {
                                        if(d.name == n)
                                        {
                                            def = Some(d);
                                            break;
                                        }
                                    }
                                    
                                    let def = match def
                                    {
                                        Some(d) => d,
                                        None => return Some(Err(error_in!(l, "There is no directive or definition name '%{}'!", n))),
                                    };

                                    match &def.value
                                    {
                                        Some(v) => Some(Ok(v.clone())),
                                        None => Some(Err(error_in!(l, "The definition '{}' doesnt hold a value that could be put here!", n))),
                                    }

                                }

                            }

                        }
                    }                            
                },
                _ => unreachable!(),
            }

        }

        pub fn get_token(&mut self) -> Option<Result<Token, Error>>
        {
            
            let token = match self.next().transpose()?
            {
                Ok(t) => t,
                Err(e) => return Some(Err(e)),
            };

            match token
            {
                Token::Directive(_) => self.parse_directive(),
                _ => Some(Ok(token)),
            }

        }

        fn is_block_open(tok: &Token) -> bool
        {
            
            let name = match tok
            {
                Token::Identifier(n, _) => n,
                _ => return false,
            };

            matches!(name.as_str(), 
                | "if"
                | "else"
                | "ifdef"
                | "ifndef"
            )

        }
        fn is_block_close(tok: &Token) -> bool
        {
            
            let name = match tok
            {
                Token::Identifier(n, _) => n,
                _ => return false,
            };

            matches!(name.as_str(), 
                | "endif"
            )

        }

        fn skip_block(&mut self, start_loc: SourceLocation) -> Result<(), Error>
        {

            loop
            {

                let tok = match self.next()?
                {
                    Some(t) => t,
                    None => return Err(error_in!(start_loc, "Expected to close the block!")),
                };

                match tok
                {
                    Token::Directive(_) => {},
                    _ => continue
                }

                let name = match self.next()?
                {
                    Some(t) => t,
                    None => continue,
                };

                if(AsmDirector::is_block_open  (&name)) { self.skip_block(token_location(&name))?; }
                if(AsmDirector::is_block_close (&name)) { break;                                   }

            }

            Ok(())

        }

    }

}

pub type AsmDirector = __dir::AsmDirector;

pub mod __asm
{

    use __dir::AsmDirector;
    use _instruction_conversion::ins_to_bytes;
    use squire::{executable::__internal::{Format, Label, LabelRequest, Section, SectionData, SectionFormat}, instructions::helpers::*};

    use super::*;

    #[derive(Debug)]
    pub enum Literal
    {
        String(String, SourceLocation),
        Number(i64, SourceLocation),
        Identifier(String, SourceLocation),
    }
    #[derive(Debug)]
    pub struct Expression
    {
        name: String,
        loc: SourceLocation,
        args: Vec<Literal>,
    }

    #[derive(Debug)]
    pub enum Statement
    {
        Label(Label),
        Expression(Expression),
    }

    pub struct ASM
    {

        director: AsmDirector,
        token: Option<Token>,
        requested_labels: Vec<LabelRequest>,

    }
    impl ASM
    {

        pub fn get_file(&self) -> Arc<str>
        { self.director.tokenizer.file.clone() }

        pub fn code(src: String) -> Self
        {
            Self
            {

                director: AsmDirector::code(src),
                token: None,
                requested_labels: Vec::new(),

            }
        }
        pub fn file(file: Arc<str>) -> Result<Self, Error>
        {
            Ok(Self
            {

                director: AsmDirector::file(file)?,
                token: None,
                requested_labels: Vec::new(),

            })
        }

        fn peek(&mut self) -> Result<Option<&Token>, Error>
        {
            if(self.token.is_none())
            {
                self.token = match self.director.get_token()
                {
                    Some(s) => match s 
                    {
                        Ok(s) => Some(s),
                        Err(e) => return Err(e),
                    },
                    None => None,
                };
            }
            Ok(self.token.as_ref())
        }
        fn next(&mut self) -> Result<Option<Token>, Error>
        {
            let _ = self.peek()?;
            Ok(self.token.take())
        }
        fn expect(&mut self, tok: Token) -> Result<Option<Token>, Error>
        {
            
            let token = match self.next()?
            {
                Some(t) => t,
                None => return Err(error_in!((self.director.get_pos()), "Expected {:?}, none given!", tok)),
            };

            if(token != tok)
            {
                return Err(error_in!((token_location(&token)), "Expected {:?}, {:?} given!", tok, token))
            }

            Ok(Some(token))

        }

        pub fn parse_statement(&mut self) -> Result<Option<(Statement, DebugSymbol)>, Error>
        {

            while let Some(t) = self.peek()?
            {
                match t
                {
                    Token::NewLine(_) => { self.next()?; },
                    _ => break,
                }
            }

            match self.peek()?
            {
                Some(_) => {},
                None => return Ok(None),
            };

            let name = match match self.expect(Token::Identifier(String::new(), SourceLocation::new()))?
            {
                Some(s) => s,
                None => return Ok(None),
            }
            {
                Token::Identifier(id, loc) => (id, loc),
                _ => unreachable!(),
            };

            let debug_symbol = DebugSymbol::new(name.1.clone(), 0);
            
            if let Some(Token::Colon(_)) = self.peek()?
            {
                self.next()?; 
                return Ok(Some((Statement::Label(Label { name: name.0, fileloc: name.1, pos: 0 }), debug_symbol)));
            }

            let mut expr = Expression
            {
                name: name.0,
                loc: name.1,
                args: Vec::new(),
            };

            while let Some(arg) = self.next()?
            {

                let lit = match arg
                {
                    Token::Identifier (n, l) => Literal::Identifier (n, l),
                    Token::String     (s, l) => Literal::String     (s, l),
                    Token::Number     (n, l) => Literal::Number     (n, l),
                    Token::Colon   (l) => return Err(error_in!(l, "Unexpected colon!")),
                    Token::Comma   (l) => return Err(error_in!(l, "Unexpected comma!")),
                    Token::NewLine (_) => break,
                    Token::Directive(_) => unreachable!(),
                };

                expr.args.push(lit);

                if let Some(n) = self.peek()?
                {
                    match n
                    {
                        Token::Comma(_) => { self.next()?; },
                        _ => break,
                    }
                }

            }

            Ok(Some((Statement::Expression(expr), debug_symbol)))

        }

        fn register_label_request(&mut self, name: String, loc: SourceLocation, pos: u32)
        {
            self.requested_labels.push(LabelRequest {
                name, loc, pos
            });
        }

        fn reg_from_str(name: String, loc: SourceLocation) -> Result<IRRegister, Error>
        {
            
            Ok(match name.to_ascii_uppercase().as_str()
            {
                "RA"  => IRRegister::RA,
                "RB"  => IRRegister::RB,
                "RC"  => IRRegister::RC,
                "RD"  => IRRegister::RD,
                "R1"  => IRRegister::R1,
                "R2"  => IRRegister::R2,
                "R3"  => IRRegister::R3,
                "R4"  => IRRegister::R4,
                "R5"  => IRRegister::R5,
                "R6"  => IRRegister::R6,
                "R7"  => IRRegister::R7,
                "R8"  => IRRegister::R8,
                "R9"  => IRRegister::R9,
                "RZ"  => IRRegister::RZ,
                "RIP" => IRRegister::RIP,
                "RSP" => IRRegister::RSP,
                _ => return Err(error_in!(loc, "Expected register! '{}' is not the name of a register", name)),
            })

        }
        fn expression_to_instruction(&mut self, exp: Expression, curr_byte_off: usize) -> Result<(IRInstruction, DebugSymbol), Error>
        {

            let err_expect_args  = |ins:&str,num:usize| error_in!((&exp.loc), "Instruction {} expected {} arguments, {} found!", ins, num, exp.args.len());
            let err_expect_ident = |ins:&str,arg:usize| error_in!((&exp.loc), "Instruction {} expected indentifier as {} argument!", ins, arg);

            let err_unknown = || error_in!((&exp.loc), "Unrecognised instruction '{}'!", exp.name);

            let get_reg = |index: usize, ins: &str| 
            {
                let (r, l) = match &exp.args[index] 
                { 
                    Literal::Identifier(r, l) => Ok((r.to_string(), l)), 
                    _ => Err(err_expect_ident(ins, index)) 
                }?;
                ASM::reg_from_str(r, l.clone())
            };
            let mut get_imm = |index: usize, off:usize, _ins: &str| 
            {
                match &exp.args[index] 
                { 
                    Literal::Number(n, _) => Ok(*n as u32), 
                    Literal::Identifier(s, l) => { self.register_label_request(s.clone(), l.clone(), (curr_byte_off + off + 1) as u32); Ok(0) },
                    Literal::String(s, l) => 
                    {
                        if(s.len() == 1)
                        {
                            Ok(s.chars().nth(0).unwrap() as u32)
                        }
                        else
                        {
                            Err(error_in!(l, "Only string of exactly one character are allowed here!"))
                        }
                    }
                }
            };

            let mut name = exp.name.to_ascii_lowercase();

            let debug = DebugSymbol::new(exp.loc.clone(), curr_byte_off as u32);

            match name.as_str()
            {
                "nop"    => return Ok((IRInstruction::NOP   , debug)),
                "hlt"    => return Ok((IRInstruction::HLT   , debug)),
                "clf"    => return Ok((IRInstruction::CLF   , debug)),
                "pshflg" => return Ok((IRInstruction::PSHFLG, debug)),
                "popflg" => return Ok((IRInstruction::POPFLG, debug)),
                "dbg"    => return Ok((IRInstruction::DBG   , debug)),
                "ret"    => return Ok((IRInstruction::RET   , debug)),

                "lea" =>
                {
                    if(exp.args.is_empty())
                    {
                        return Err(err_expect_args("lea", 1));
                    }
                    let reg = get_reg(0, "lea")?;
                    return Ok((IRInstruction::LEA(reg), debug));
                },

                "inc" =>
                {
                    if(exp.args.is_empty())
                    {
                        return Err(err_expect_args("inc", 1));
                    }
                    let reg = get_reg(0, "inc")?;
                    return Ok((IRInstruction::INC(reg), debug));
                },
                "dec" =>
                {
                    if(exp.args.is_empty())
                    {
                        return Err(err_expect_args("dec", 1));
                    }
                    let reg = get_reg(0, "dec")?;
                    return Ok((IRInstruction::DEC(reg), debug));
                },

                "__out" =>
                {
                    if(exp.args.is_empty())
                    {
                        return Err(err_expect_args("__out", 1));
                    }
                    let reg = get_reg(0, "__out")?;
                    return Ok((IRInstruction::SER_OUT(reg), debug));
                },
                "__in" =>
                {
                    if(exp.args.is_empty())
                    {
                        return Err(err_expect_args("__in", 1));
                    }
                    let reg = get_reg(0, "__in")?;
                    return Ok((IRInstruction::SER_IN(reg), debug));
                },
                "__io" =>
                {
                    if(exp.args.is_empty())
                    {
                        return Err(err_expect_args("__io", 1));
                    }
                    let imm = get_imm(0, 0, "__io")?;
                    return Ok((IRInstruction::SER_IO(imm), debug));
                },
                "int" =>
                {
                    if(exp.args.is_empty())
                    {
                        return Err(err_expect_args("int", 1));
                    }
                    let imm = get_imm(0, 0, "int")?;
                    return Ok((IRInstruction::INT(imm), debug));
                },


                _ => {}
            }

            Ok(
                if(name.starts_with("mov") || name.starts_with("wmov") || name.starts_with("dmov") || name.starts_with("bmov"))
                {

                    let ins_width =
                             if(name.starts_with("b")) { IRInstructionWidth::B8  }
                        else if(name.starts_with("w")) { IRInstructionWidth::B16 }
                        else                           { IRInstructionWidth::B32 };
                    if(name.starts_with("b") || name.starts_with("w") || name.starts_with("d")) { name = name[1..name.len()].to_string(); }
        
                    if(exp.args.len() != 2)
                    {
                        return Err(err_expect_args("mov", 2));
                    }

                    let args = name[3..name.len()].to_string();
                    let mut offset = 1;

                    let mut no_left_adr_mode = false;

                    let contains_regs = args.contains('r');

                    let mod0: IRInstructionModifier = 
                        if(args.starts_with("ra"))
                        {
                            no_left_adr_mode = true;
                            offset = 2;
                            IRInstructionModifier::RegisterAddress(get_reg(0, "movra*")?)
                        }
                        else if(args.starts_with("ma"))
                        {
                            let _inner_offset = if(contains_regs) { 1 } else { 0 };
                            no_left_adr_mode = true;
                            offset = 2;
                            IRInstructionModifier::MemoryAddress(get_imm(0, _inner_offset, "movma*")?)
                        }
                        else if(args.starts_with("r"))
                        {
                            IRInstructionModifier::Register(get_reg(0, "movr*")?)
                        }
                        else if(args.starts_with("m"))
                        {
                            let _inner_offset = if(contains_regs) { 1 } else { 0 };
                            IRInstructionModifier::Memory(get_imm(0, _inner_offset, "movm*")?)
                        }
                        else if(args.starts_with("i"))
                        {
                            let _inner_offset = if(contains_regs) { 1 } else { 0 };
                            no_left_adr_mode = true;
                            IRInstructionModifier::Immediate(get_imm(0, _inner_offset, "movi*")?)
                        }
                        else { return Err(err_unknown()); };
                        
                        let args = args[offset..args.len()].to_string();
                        
                        let mod1: IRInstructionModifier = 
                        if(args.starts_with("ra"))
                        {
                            if(no_left_adr_mode) { return Err(error_in!((exp.loc), "Address parameter isnt allowed as second argument here!")); }
                            IRInstructionModifier::RegisterAddress(get_reg(1, "movra*")?)
                        }
                        else if(args.starts_with("ma"))
                        {
                            let _inner_offset = if(contains_regs) { 1 } else { 4 };
                            if(no_left_adr_mode) { return Err(error_in!((exp.loc), "Address parameter isnt allowed as second argument here!")); }
                            IRInstructionModifier::MemoryAddress(get_imm(1, _inner_offset, "movma*")?)
                        }
                        else if(args.starts_with("r"))
                        {
                            IRInstructionModifier::Register(get_reg(1, "movr*")?)
                        }
                        else if(args.starts_with("m"))
                        {
                            let _inner_offset = if(contains_regs) { 1 } else { 4 };
                            IRInstructionModifier::Memory(get_imm(1, _inner_offset, "movm*")?)
                        }
                        else if(args.starts_with("i"))
                        {
                            return Err(error_in!((exp.loc), "Mov instructions dont allow and immediate as the destination! Did you mean 'm' for memory?"))
                        }
                        else { return Err(err_unknown()); };

                    (IRInstruction::MOV(ins_width, ( mod0, mod1 )), debug)

                }
                else if(name.starts_with("psh") || name.starts_with("dpsh") || name.starts_with("wpsh") || name.starts_with("bpsh"))
                {

                    let ins_width =
                             if(name.starts_with("b")) { IRInstructionWidth::B8  }
                        else if(name.starts_with("w")) { IRInstructionWidth::B16 }
                        else                           { IRInstructionWidth::B32 };
                    if(name.starts_with("b") || name.starts_with("w") || name.starts_with("d")) { name = name[1..name.len()].to_string(); }
        
                    if(exp.args.len() != 1)
                    {
                        return Err(err_expect_args("psh", 1));
                    }
                    
                    let mod0: IRInstructionModifier = match &name[3..name.len()]
                    {
                        "r" => IRInstructionModifier::Register (get_reg(0,    "pshr")?),
                        "m" => IRInstructionModifier::Memory   (get_imm(0, 0, "pshm")?),
                        "i" => IRInstructionModifier::Immediate(get_imm(0, 0, "pshi")?),
                        _  => return Err(err_unknown()),
                    };

                    (IRInstruction::PSH(ins_width, mod0), debug)

                }
                else if(name.starts_with("pop") || name.starts_with("dpop") || name.starts_with("wpop") || name.starts_with("bpop"))
                {

                    let ins_width =
                             if(name.starts_with("b")) { IRInstructionWidth::B8  }
                        else if(name.starts_with("w")) { IRInstructionWidth::B16 }
                        else                           { IRInstructionWidth::B32 };
                    if(name.starts_with("b") || name.starts_with("w") || name.starts_with("d")) { name = name[1..name.len()].to_string(); }
        
                    if(exp.args.len() != 1)
                    {
                        return Err(err_expect_args("pop", 1));
                    }
                    
                    let mod0: IRInstructionModifier = match &name[3..name.len()]
                    {
                        "r" => IRInstructionModifier::Register (get_reg(0,    "popr")?),
                        "m" => IRInstructionModifier::Memory   (get_imm(0, 0, "popm")?),
                         _  => return Err(err_unknown()),
                    };

                    (IRInstruction::POP(ins_width, mod0), debug)

                }
                else if(name.starts_with("jmp"))
                {

                    if(exp.args.len() != 1)
                    {
                        return Err(err_expect_args("jmp", 1));
                    }
                    
                    let mod0: IRInstructionModifier = match &name[3..name.len()]
                    {
                        "r" => IRInstructionModifier::Register (get_reg(0,    "jmpr")?),
                        "m" => IRInstructionModifier::Memory   (get_imm(0, 0, "jmpm")?),
                        "i" => IRInstructionModifier::Immediate(get_imm(0, 0, "jmpi")?),
                         _  => return Err(err_unknown()),
                    };

                    (IRInstruction::JMP(mod0), debug)

                }
                else if(name.starts_with("jif"))
                {

                    if(exp.args.len() != 2)
                    {
                        return Err(err_expect_args("jif", 2));
                    }
                    
                    let mut _inner_offset = 0;

                    let mod0: IRInstructionModifier = match &name[3..name.len()]
                    {
                        "r" => { IRInstructionModifier::Register (get_reg(0,    "jifr")?) },
                        "m" => { IRInstructionModifier::Memory   (get_imm(0, 0, "jifm")?) },
                        "i" => { IRInstructionModifier::Immediate(get_imm(0, 0, "jifi")?) },
                         _  => return Err(err_unknown()),
                    };

                    let f  = match &exp.args[1]
                    {
                        Literal::Identifier(n, _) =>
                        {

                            let mut flags: u8 = 0;

                            let n = n.to_ascii_lowercase();

                            if(n.contains('e')) { flags |= FLAG_E; }
                            if(n.contains('a')) { flags |= FLAG_A; }
                            if(n.contains('b')) { flags |= FLAG_B; }
                            if(n.contains('z')) { flags |= FLAG_Z; }
                            if(n.contains('c')) { flags |= FLAG_C; }

                            flags

                        },
                        Literal::Number(n, _) => *n as u8,
                        _ => return Err(err_expect_ident("jif", 1)),
                    };

                    //let f = get_imm(1, _inner_offset, "jif")? as u8;

                    (IRInstruction::JIF(mod0, f), debug)

                }
                else if(name.starts_with("cal"))
                {

                    if(exp.args.len() != 1)
                    {
                        return Err(err_expect_args("cal", 1));
                    }
                    
                    let mod0: IRInstructionModifier = match &name[3..name.len()]
                    {
                        "r" => IRInstructionModifier::Register (get_reg(0,    "calr")?),
                        "m" => IRInstructionModifier::Memory   (get_imm(0, 0, "calm")?),
                        "i" => IRInstructionModifier::Immediate(get_imm(0, 0, "cali")?),
                         _  => return Err(err_unknown()),
                    };

                    (IRInstruction::CAL(mod0), debug)

                }
                else if(name.starts_with("not") || name.starts_with("cmp"))
                {

                    let __base = if(name.starts_with("not")) { "not" } else { "cmp" };

                    let ins = |m|format!("{}{}",__base,m);

                    if(exp.args.len() != 2)
                    {
                        return Err(err_expect_args(__base, 2));
                    }

                    let args = name[3..name.len()].to_string();

                    let mut mem_byte_off = if(args.contains('r')) { 1 } else { 0 };

                    let mod0: IRInstructionModifier = 
                        if(args.starts_with("r"))
                        {
                            IRInstructionModifier::Register(get_reg(0, &ins("r*"))?)
                        }
                        else if(args.starts_with("m"))
                        {
                            let m = IRInstructionModifier::Memory(get_imm(0, mem_byte_off, &ins("m*"))?);
                            mem_byte_off += 4;
                            m
                        }
                        else { return Err(err_unknown()); };
                    
                    let args = args[1..args.len()].to_string();

                    let mod1: IRInstructionModifier = 
                        if(args.starts_with("r"))
                        {
                            IRInstructionModifier::Register(get_reg(1, &ins("r*"))?)
                        }
                        else if(args.starts_with("m"))
                        {
                            IRInstructionModifier::Memory(get_imm(1, mem_byte_off, &ins("m*"))?)
                        }
                        else { return Err(err_unknown()); };
            
                    (IRInstruction::ALU(IRALUInstruction::Simple(
                        if(name.starts_with("not")) 
                        { 
                            _IRALUInstruction2::NOT(( mod0, mod1 ))
                        } 
                        else 
                        { 
                            _IRALUInstruction2::CMP(( mod0, mod1 ))
                        }
                    )), debug)

                }
                else 
                {

                    let mut offset = 3;

                    if(name.len() < 3)
                    {
                        return Err(error_in!((&exp.loc), "Unrecognised instruction '{}'!", name));
                    }

                    if(name.starts_with("nand")) { offset = 4; }
                    if(name.starts_with(  "or")) { offset = 2; }
                    
                    let __base = name[0..offset].to_string();

                    let ins = |m|format!("{}{}",__base,m);

                    let args = name[offset..name.len()].to_string();

                    let mut mem_byte_off = if(args.contains('r')) { 1 } else { 0 };

                    let mut _modifiers: IRALUInstructionModifier3 = None;

                    if(args == "s")
                    {
                        if(!exp.args.is_empty())
                        {
                            return Err(err_expect_args(&ins("s"), 0));
                        }
                    }
                    else
                    {

                        if(exp.args.len() != 3)
                        {
                            return Err(err_expect_args(&__base, 3));
                        }

                        let mod0: IRInstructionModifier = 
                            if(args.starts_with("r"))
                            {
                                IRInstructionModifier::Register(get_reg(0, &ins("r*"))?)
                            }
                            else if(args.starts_with("m"))
                            {
                                let m = IRInstructionModifier::Memory(get_imm(0, mem_byte_off, &ins("m*"))?);
                                mem_byte_off += 4;
                                m
                            }
                            else { return Err(err_unknown()); };
                        
                        let args = args[1..args.len()].to_string();

                        let mod1: IRInstructionModifier = 
                            if(args.starts_with("r"))
                            {
                                IRInstructionModifier::Register(get_reg(1, &ins("r*"))?)
                            }
                            else if(args.starts_with("m"))
                            {
                                let m = IRInstructionModifier::Memory(get_imm(1, mem_byte_off, &ins("m*"))?);
                                mem_byte_off += 4;
                                m
                            }
                            else { return Err(err_unknown()); };
                    
                        let args = args[1..args.len()].to_string();

                        let mod2: IRInstructionModifier = 
                            if(args.starts_with("r"))
                            {
                                IRInstructionModifier::Register(get_reg(2, &ins("r*"))?)
                            }
                            else if(args.starts_with("m"))
                            {
                                IRInstructionModifier::Memory(get_imm(2, mem_byte_off, &ins("m*"))?)
                            }
                            else { return Err(err_unknown()); };

                        _modifiers = Some(( mod0, mod1, mod2 ));

                    }

                    (IRInstruction::ALU(IRALUInstruction::Complex(
                             if(name.starts_with( "add")) { _IRALUInstruction3:: ADD(_modifiers) }
                        else if(name.starts_with( "sub")) { _IRALUInstruction3:: SUB(_modifiers) }
                        else if(name.starts_with( "mul")) { _IRALUInstruction3:: MUL(_modifiers) }
                        else if(name.starts_with( "div")) { _IRALUInstruction3:: DIV(_modifiers) }
                        else if(name.starts_with( "mod")) { _IRALUInstruction3:: MOD(_modifiers) }
                        else if(name.starts_with( "and")) { _IRALUInstruction3:: AND(_modifiers) }
                        else if(name.starts_with(  "or")) { _IRALUInstruction3::  OR(_modifiers) }
                        else if(name.starts_with( "xor")) { _IRALUInstruction3:: XOR(_modifiers) }
                        else if(name.starts_with( "shl")) { _IRALUInstruction3:: SHL(_modifiers) }
                        else if(name.starts_with( "shr")) { _IRALUInstruction3:: SHR(_modifiers) }
                        else if(name.starts_with("nand")) { _IRALUInstruction3::NAND(_modifiers) }
                        else if(name.starts_with( "nor")) { _IRALUInstruction3:: NOR(_modifiers) }
                        else { return Err(err_unknown()); }
                    )), debug)

                }
            )

        }

        fn parse_expression(&mut self, off: usize, exp: Expression, mut push: impl FnMut(u8) -> Result<(), Error>) -> Result<DebugSymbol, Error>
        {
            
            let err_expect_args  = |ins:&str,num:usize| error_in!((&exp.loc), "Instruction {} expected {} arguments, {} found!", ins, num, exp.args.len());

            match exp.name.as_str()
            {
                "db" | "dw" | "dd" =>
                {

                    if(exp.args.is_empty())
                    {
                        return Err(error_in!((&exp.loc), "Instruction db expects at least one argument!"));
                    }

                    let mut curr_byte_off: usize = 0;

                    for a in exp.args
                    {
                        match a
                        {
                            Literal::Identifier(n, l) =>
                            {
                                self.register_label_request(n, l, (curr_byte_off + off) as u32);
                                push(0)?; push(0)?; push(0)?; push(0)?;
                                curr_byte_off += 2;
                            },
                            Literal::Number(n, _) =>
                            {
                                match exp.name.as_str()
                                {
                                    "db" => 
                                    {
                                        push(n as u8)?;
                                        curr_byte_off += 1;
                                    },
                                    "dw" => 
                                    {
                                        let v = u16_2_u8(n as u16);
                                        push(v.0)?;
                                        push(v.1)?;
                                        curr_byte_off += 2;
                                    },
                                    "dd" => 
                                    {
                                        let v = u32_2_u8(n as u32);
                                        push(v.0)?;
                                        push(v.1)?;
                                        push(v.2)?;
                                        push(v.3)?;
                                        curr_byte_off += 4;
                                    },
                                    _ => unreachable!(),
                                }
                            },
                            Literal::String(s, _) =>
                            {
                                for n in s.chars()
                                {
                                    match exp.name.as_str()
                                    {
                                        "db" => 
                                        {
                                            push(n as u8)?;
                                            curr_byte_off += 1;
                                        },
                                        "dw" => 
                                        {
                                            let v = u16_2_u8(n as u16);
                                            push(v.0)?;
                                            push(v.1)?;
                                            curr_byte_off += 2;
                                        },
                                        "dd" => 
                                        {
                                            let v = u32_2_u8(n as u32);
                                            push(v.0)?;
                                            push(v.1)?;
                                            push(v.2)?;
                                            push(v.3)?;
                                            curr_byte_off += 4;
                                        },
                                        _ => unreachable!(),
                                    }
                                }
                            }
                        }
                    }
                    
                    Ok(DebugSymbol::new(exp.loc.clone(), off as u32))

                },
                "resb" | "resw" => 
                {

                    let put_words = exp.name == "resw";

                    if(exp.args.len() != 1) { return Err(err_expect_args(&exp.name, 1)); }

                    let arg = &exp.args[0];

                    let n = match arg
                    {
                        Literal::Number(n, _) => n,
                        Literal::Identifier(_, l) |
                        Literal::String(_, l) =>
                        {
                            return Err(error_in!(l, "{} expected a number as an amount of bytes!", exp.name));
                        }
                    };

                    if(put_words)
                    {
                        for _ in 0..*n
                        {
                            push(0)?;
                            push(0)?;
                        }
                    }
                    else
                    {
                        for _ in 0..*n
                        {
                            push(0)?;
                        }
                    }

                    Ok(DebugSymbol::new(exp.loc.clone(), off as u32))

                },
                _ => 
                {
                    let ins = self.expression_to_instruction(exp, off)?;
                    ins_to_bytes(ins.0, push)?;
                    Ok(ins.1)
                }
            }

        }

        fn parse_section(&mut self, statement: Option<(Statement, DebugSymbol)>, all_labels: &mut Vec<Label>) -> Result<(SectionFormat, Option<(Statement, DebugSymbol)>), Error>
        {

            self.requested_labels = Vec::new();

            let mut section = SectionFormat
            {
                section: SectionData
                {
                    data: Vec::new(),
                    section: self.director.section,
                },
                labels: Vec::new(),
                exposed_labels: Vec::new(),
                requested_labels: Vec::new(),
                symbols: Vec::new(),
            };

            let mut statement: Option<(Statement, DebugSymbol)> = statement;

            while let Some(mut s) = statement
            {

                s.1.pos = section.section.len() as u32;

                match s.0
                {
                    Statement::Label(mut label) =>
                    {
                        label.pos = section.section.len() as i64;
                        for l in (&mut *all_labels)
                        {
                            if(l.name == label.name)
                            {
                                return Err(error_in!((label.fileloc), "Label '{}' already exists! (Defined here: {})", label.name, l.fileloc));
                            }
                        }
                        all_labels.push(label.clone());
                        section.labels.push(label);
                    },
                    Statement::Expression(exp) =>
                    {
                        let len = section.section.len();
                        self.parse_expression(len, exp, |v| { section.section.data.push(v); Ok(()) })?;
                    },
                };

                section.symbols.push(s.1);

                statement = self.parse_statement()?;

                if(self.director.switched_section) { break; }

            }

            self.director.switched_section = false;

            section.requested_labels = std::mem::take(&mut self.requested_labels);
            
            Ok((section, statement))

        }

        pub fn parse(&mut self) -> Result<Format, Error>
        {

            let mut head_format: Option<SectionFormat> = None;
            let mut code_format: Option<SectionFormat> = None;
            let mut data_format: Option<SectionFormat> = None;
            
            let mut all_labels: Vec<Label> = Vec::new();

            let mut statement: Option<(Statement, DebugSymbol)>;

            loop
            {

                statement = self.parse_statement()?;

                if(self.director.switched_section) { break; }

                let s = match statement
                {
                    Some(s) => s,
                    None => break,
                };

                match s.0
                {
                    Statement::Label(label) =>
                    {
                        return Err(error_in!((label.fileloc), "Cannot define a label outside of sections!"));
                    },
                    Statement::Expression(exp) =>
                    {
                        let loc = exp.loc.clone();
                        self.parse_expression(0, exp, |_| { Err(error_in!((&loc), "Cannot write bytes outside of sections!")) })?;
                    },
                };

            }

            self.director.switched_section = false;

            while statement.is_some()
            {

                let (section, st) = self.parse_section(statement, &mut all_labels)?;
                statement = st;

                match section.section.section
                {
                    Section::None => unreachable!(),
                    Section::Code =>
                    {
                        if(code_format.is_some())
                        {
                            return Err(error!("Code section defined multiple times!"));
                        }
                        code_format = Some(section);
                    },
                    Section::Data =>
                    {
                        if(data_format.is_some())
                        {
                            return Err(error!("Data section defined multiple times!"));
                        }
                        data_format = Some(section);
                    },
                }

            }

            for exp in &mut self.director.exported_labels
            {

                let mut found_any = false;

                if let Some(format) = &mut head_format
                {
                    for l in &format.labels
                    {
                        if(l.name == exp.name)
                        {
                            exp.fileloc = l.fileloc.clone();
                            exp.pos = l.pos;
                            format.exposed_labels.push(exp.clone());
                            found_any = true;
                            break;
                        }
                    }
                    if(found_any) { continue; }
                }

                if let Some(format) = &mut code_format
                {
                    for l in &format.labels
                    {
                        if(l.name == exp.name)
                        {
                            exp.fileloc = l.fileloc.clone();
                            exp.pos = l.pos;
                            format.exposed_labels.push(exp.clone());
                            found_any = true;
                            break;
                        }
                    }
                    if(found_any) { continue; }
                }

                if let Some(format) = &mut data_format
                {
                    for l in &format.labels
                    {
                        if(l.name == exp.name)
                        {
                            exp.fileloc = l.fileloc.clone();
                            exp.pos = l.pos;
                            format.exposed_labels.push(exp.clone());
                            found_any = true;
                            break;
                        }
                    }
                    if(found_any) { continue; }
                }
                
                if(!found_any)
                {
                    return Err(error_in!((&exp.fileloc), "Exposed label '{}' is not defined!", exp.name));
                }

            }

            let mut sections: Vec<SectionFormat> = Vec::new();
            if let Some(format) = head_format { sections.push(format); }
            if let Some(format) = code_format { sections.push(format); }
            if let Some(format) = data_format { sections.push(format); }

            let header = if(self.director.constructor.constructing) { Some(self.director.constructor.clone()) } else { None };

            Ok(
                Format
                {
                    sections,
                    external_labels: std::mem::take(&mut self.director.external_labels),
                    header,
                }
            )

        }

    }

}

pub type ASM = __asm::ASM;
