use std::sync::Arc;
use std::fs::read_to_string;
use errors::Error;
use helpers::parse_escape;
use squire::instructions::*;
use squire::{error, error_in};

#[derive(Debug, Clone)]
pub enum Token
{
    Comma(SourceLocation),
    Colon(SourceLocation),
    String(String, SourceLocation),
    Number(i32, SourceLocation),
    Identifier(String, SourceLocation),
    NewLine(SourceLocation),
    Directive(SourceLocation),
}
impl PartialEq for Token
{
    fn ne(&self, other: &Self) -> bool { !(self == other) }
    fn eq(&self, other: &Self) -> bool 
    {
        match self
        {
            Token::NewLine    (_   ) => match other { Token::NewLine    (_   ) => true, _ => false },
            Token::Directive  (_   ) => match other { Token::Directive  (_   ) => true, _ => false },
            Token::Comma      (_   ) => match other { Token::Comma      (_   ) => true, _ => false },
            Token::Colon      (_   ) => match other { Token::Colon      (_   ) => true, _ => false },
            Token::String     (_, _) => match other { Token::String     (_, _) => true, _ => false },
            Token::Number     (_, _) => match other { Token::Number     (_, _) => true, _ => false },
            Token::Identifier (_, _) => match other { Token::Identifier (_, _) => true, _ => false },
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
    line: i32,
    column: i32,
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
            return self.peek();
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
            while let Some(c) = self.next()
            {
                if(c.0 == '\n') { break; }
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

                    let mut str = String::new();

                    while let Some(c) = self.peek()
                    {
                        if(!c.0.is_numeric()) { break; }
                        else { str.push(self.next()?.0); }
                    }

                    let num = match str.parse::<i32>()
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
    use __asm::Label;

    #[derive(Debug)]
    pub struct DirDefinition
    {
        pub loc: SourceLocation,
        pub name: String,
        pub value: Option<Token>,
    }

    #[derive(Debug)]
    pub struct AsmDirector
    {
        pub tokenizer: AsmTokenizer,
        token: Option<Token>,
        definitions: Vec<DirDefinition>,
        allow_block_close: u8,
        pub exposed_labels: Vec<Label>,
        pub external_labels: Vec<Label>,
        pub entry_point: Option<Label>,
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
                exposed_labels: Vec::new(),
                external_labels: Vec::new(),
                entry_point: None,
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
                exposed_labels: Vec::new(),
                external_labels: Vec::new(),
                entry_point: None,
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
                        "exp" | "ext" =>
                        {
                            
                            let name = match match self.expect(Token::Identifier(String::new(), SourceLocation::new()))
                            {
                                Ok(t) => t,
                                Err(e) => return Some(Err(e)),
                            }.unwrap()
                            {
                                Token::Identifier(n, _) => n,
                                _ => unreachable!(),
                            };

                            let label = Label 
                            {
                                name,
                                fileloc: l,
                                pos: 0,
                                imported: (n == "ext"),
                            };

                            if(n == "exp")
                            {
                                self.exposed_labels.push(label);
                            }
                            else
                            {
                                self.external_labels.push(label);
                            }

                            self.get_token()

                        },
                        "entry" | "public_static_void_main_string_args" =>
                        {

                            if let Some(entry) = &self.entry_point
                            {
                                return Some(Err(error_in!(l, "Cannot re-define entry point! (Already defined here: {})", entry.fileloc)));
                            }

                            let name = match match self.expect(Token::Identifier(String::new(), SourceLocation::new()))
                            {
                                Ok(t) => t,
                                Err(e) => return Some(Err(e)),
                            }.unwrap()
                            {
                                Token::Identifier(n, _) => n,
                                _ => unreachable!(),
                            };

                            self.entry_point = Some(Label {
                                name,
                                fileloc: l.clone(),
                                pos: 0,
                                imported: false,
                            });

                            self.get_token()

                        },
                        "def" => 
                        {
                            
                            let name = match match self.expect(Token::Identifier(String::new(), SourceLocation::new()))
                            {
                                Ok(t) => t,
                                Err(e) => return Some(Err(e)),
                            }.unwrap()
                            {
                                Token::Identifier(n, _) => n,
                                _ => unreachable!(),
                            };

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
                            
                            let name = match match self.expect(Token::Identifier(String::new(), SourceLocation::new()))
                            {
                                Ok(t) => t,
                                Err(e) => return Some(Err(e)),
                            }.unwrap()
                            {
                                Token::Identifier(n, _) => n,
                                _ => unreachable!(),
                            };
                            
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

                            let t = self.get_token();
                            
                            t

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
                                None => return Some(Err(error_in!(l, "The definition '{}' doesnt hold a value that could be put here!", n))),
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

            match name.as_str()
            {
                "if"
                | "else"
                | "ifdef"
                | "ifndef"
                 => true,
                _ => false,
            }

        }
        fn is_block_close(tok: &Token) -> bool
        {
            
            let name = match tok
            {
                Token::Identifier(n, _) => n,
                _ => return false,
            };

            match name.as_str()
            {
                "endif"
                 => true,
                _ => false,
            }

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
    use helpers::u16_2_u8;

    use super::*;

    #[derive(Debug, Clone)]
    pub struct Label
    {
        pub name: String,
        pub fileloc: SourceLocation,
        pub pos: usize,
        pub imported: bool,
    }
    impl PartialEq for Label
    {
        fn eq(&self, other: &Self) -> bool { self.name == other.name }
    }

    #[derive(Debug)]
    pub enum Literal
    {
        String(String, SourceLocation),
        Number(i32, SourceLocation),
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
    struct LabelRequest
    {
        name: String,
        loc: SourceLocation,
        pos: usize,
    }

    #[derive(Debug)]
    pub enum Statement
    {
        Label(Label),
        Expression(Expression),
        //Literal(Literal),
    }

    pub struct ParseResult
    {
        pub bytes: Vec<u8>,
        pub exposed_labels: Vec<Label>,
        pub entry_point: Option<Label>,
    }

    #[derive(Debug)]
    pub struct ASM
    {
        director: AsmDirector,
        token: Option<Token>,
        label_requests: Vec<LabelRequest>,
        labels: Vec<Label>,
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
                label_requests: Vec::new(),
                labels: Vec::new(),
            }
        }
        pub fn file(file: Arc<str>) -> Result<Self, Error>
        {
            Ok(Self
            {
                director: AsmDirector::file(file)?,
                token: None,
                label_requests: Vec::new(),
                labels: Vec::new(),
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

        pub fn parse_statement(&mut self) -> Result<Option<Statement>, Error>
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
            
            if let Some(next) = self.peek()?
            {
                match next
                {
                    Token::Colon(_) => 
                    { 
                        self.next()?; 
                        return Ok(Some(Statement::Label(Label { name: name.0, fileloc: name.1, pos: 0, imported: false })));
                    },
                    _ => {},
                }
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

            Ok(Some(Statement::Expression(expr)))

        }

        fn register_label_request(&mut self, name: String, loc: SourceLocation, pos: usize)
        {
            self.label_requests.push(LabelRequest {
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
        fn expression_to_instruction(&mut self, exp: Expression, curr_byte_off: usize) -> Result<IRInstruction, Error>
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
                    Literal::Number(n, _) => Ok(*n as u16), 
                    Literal::Identifier(s, l) => { self.register_label_request(s.clone(), l.clone(), curr_byte_off + off + 1); Ok(0) },
                    Literal::String(s, l) => 
                    {
                        if(s.len() == 1)
                        {
                            Ok(s.chars().nth(0).unwrap() as u16)
                        }
                        else
                        {
                            Err(error_in!(l, "Only string of exactly one character are allowed here!"))
                        }
                    }
                }
            };

            let mut name = exp.name.to_ascii_lowercase();

            match name.as_str()
            {
                "nop" => return Ok(IRInstruction::NOP),
                "hlt" => return Ok(IRInstruction::HLT),
                "clf" => return Ok(IRInstruction::CLF),
                // RAY
                "dbg" => return Ok(IRInstruction::DBG),
                "ret" => return Ok(IRInstruction::RET),

                "__out" =>
                {
                    if(exp.args.len() < 1)
                    {
                        return Err(err_expect_args("__out", 1));
                    }
                    let reg = get_reg(0, "__out")?;
                    return Ok(IRInstruction::SER_OUT(reg));
                }
                "__in" =>
                {
                    if(exp.args.len() < 1)
                    {
                        return Err(err_expect_args("__in", 1));
                    }
                    let reg = get_reg(0, "__in")?;
                    return Ok(IRInstruction::SER_IN(reg));
                }

                _ => {}
            }

            Ok(
                if(name.starts_with("mov") || name.starts_with("bmov"))
                {

                    let ins_width = if(!name.starts_with("b")) { IRInstructionWidth::B16 } else { IRInstructionWidth::B8 };
                    if(name.starts_with("b")) { name = name[1..name.len()].to_string(); }
        
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
                            offset = 1;
                            let m = IRInstructionModifier::MemoryAddress(get_imm(0, _inner_offset, "movma*")?);
                            m
                        }
                        else if(args.starts_with("r"))
                        {
                            IRInstructionModifier::Register(get_reg(0, "movr*")?)
                        }
                        else if(args.starts_with("m"))
                        {
                            let _inner_offset = if(contains_regs) { 1 } else { 0 };
                            let m = IRInstructionModifier::Memory(get_imm(0, _inner_offset, "movm*")?);
                            m
                        }
                        else if(args.starts_with("i"))
                        {
                            let _inner_offset = if(contains_regs) { 1 } else { 0 };
                            no_left_adr_mode = true;
                            let i = IRInstructionModifier::Immediate(get_imm(0, _inner_offset, "movi*")?);
                            i
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
                            let _inner_offset = if(contains_regs) { 1 } else { 2 };
                            if(no_left_adr_mode) { return Err(error_in!((exp.loc), "Address parameter isnt allowed as second argument here!")); }
                            IRInstructionModifier::MemoryAddress(get_imm(1, _inner_offset, "movma*")?)
                        }
                        else if(args.starts_with("r"))
                        {
                            IRInstructionModifier::Register(get_reg(1, "movr*")?)
                        }
                        else if(args.starts_with("m"))
                        {
                            let _inner_offset = if(contains_regs) { 1 } else { 2 };
                            IRInstructionModifier::Memory(get_imm(1, _inner_offset, "movm*")?)
                        }
                        else if(args.starts_with("i"))
                        {
                            return Err(error_in!((exp.loc), "Mov instructions dont allow and immediate as the destination! Did you mean 'm' for memory?"))
                        }
                        else { return Err(err_unknown()); };

                    IRInstruction::MOV(ins_width, ( mod0, mod1 ))

                }
                else if(name.starts_with("psh") || name.starts_with("bpsh"))
                {

                    let ins_width = if(!name.starts_with("b")) { IRInstructionWidth::B16 } else { IRInstructionWidth::B8 };
                    if(name.starts_with("b")) { name = name[1..name.len()].to_string(); }
        
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

                    IRInstruction::PSH(ins_width, mod0)

                }
                else if(name.starts_with("pop") || name.starts_with("bpop"))
                {

                    let ins_width = if(!name.starts_with("b")) { IRInstructionWidth::B16 } else { IRInstructionWidth::B8 };
                    if(name.starts_with("b")) { name = name[1..name.len()].to_string(); }
        
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

                    IRInstruction::POP(ins_width, mod0)

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

                    IRInstruction::JMP(mod0)

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
                        "r" => { _inner_offset += 1; IRInstructionModifier::Register (get_reg(0,    "jifr")?) },
                        "m" => { _inner_offset += 2; IRInstructionModifier::Memory   (get_imm(0, 0, "jifm")?) },
                        "i" => { _inner_offset += 2; IRInstructionModifier::Immediate(get_imm(0, 0, "jifi")?) },
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

                    IRInstruction::JIF(mod0, f)

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

                    IRInstruction::CAL(mod0)

                }
                else if(name.starts_with("not") || name.starts_with("cmp"))
                {

                    let __base = if(name.starts_with("not")) { "not" } else { "cmp" };

                    let ins = |m|format!("{}{}",__base,m);

                    if(exp.args.len() != 2)
                    {
                        return Err(err_expect_args(&__base, 2));
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
                            mem_byte_off += 2;
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
            
                    IRInstruction::ALU(IRALUInstruction::Simple(
                        if(name.starts_with("not")) 
                        { 
                            _IRALUInstruction2::NOT(( mod0, mod1 ))
                        } 
                        else 
                        { 
                            _IRALUInstruction2::CMP(( mod0, mod1 ))
                        }
                    ))

                }
                else 
                {

                    let mut offset = 3;

                    if(name.starts_with("nand")) { offset = 4; }
                    if(name.starts_with(  "or")) { offset = 2; }
                    
                    let __base = name[0..offset].to_string();

                    let ins = |m|format!("{}{}",__base,m);

                    let args = name[offset..name.len()].to_string();

                    let mut mem_byte_off = if(args.contains('r')) { 1 } else { 0 };

                    let mut _modifiers: IRALUInstructionModifier3 = None;

                    if(args == "s")
                    {
                        if(exp.args.len() != 0)
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
                                mem_byte_off += 2;
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
                                mem_byte_off += 2;
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

                    IRInstruction::ALU(IRALUInstruction::Complex(
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
                    ))

                }
            )

        }

        fn parse_expression(&mut self, off: usize, exp: Expression, mut push: impl FnMut(u8) -> Result<(), Error>) -> Result<(), Error>
        {
            
            let err_expect_args  = |ins:&str,num:usize| error_in!((&exp.loc), "Instruction {} expected {} arguments, {} found!", ins, num, exp.args.len());

            match exp.name.as_str()
            {
                "db" =>
                {

                    if(exp.args.len() < 1)
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
                                self.register_label_request(n, l, curr_byte_off + off);
                                push(0)?; push(0)?;
                                curr_byte_off += 2;
                            },
                            Literal::Number(n, _) =>
                            {
                                push(n as u8)?;
                                curr_byte_off += 1;
                            },
                            Literal::String(s, _) =>
                            {
                                for c in s.chars()
                                {
                                    push(c as u8)?;
                                    curr_byte_off += 1;
                                }
                            }
                        }
                    }
                    
                    Ok(())

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

                    Ok(())

                },
                _ => 
                {
                    let ins = self.expression_to_instruction(exp, off)?;
                    ins_to_bytes(ins, push)
                }
            }

        }

        pub fn parse(&mut self) -> Result<ParseResult, Error>
        {

            let mut bytes: Vec<u8> = Vec::new();

            while let Some(s) = self.parse_statement()?
            {

                match s
                {
                    Statement::Label(mut label) =>
                    {
                        label.pos = bytes.len();
                        for l in &self.labels
                        {
                            if(l.name == label.name)
                            {
                                return Err(error_in!((label.fileloc), "Label '{}' already exists! (Defined here: {})", label.name, l.fileloc));
                            }
                        }
                        self.labels.push(label);
                    },
                    Statement::Expression(exp) =>
                    {
                        let loc = exp.loc.clone();
                        self.parse_expression(bytes.len(), exp, |v| 
                            {
                                if(bytes.len() >= 0x10000)
                                {
                                    return Err(error_in!((&loc), "FATAL: EXCEEDING MAXIMUM BYTES OF {}", 0x10000));
                                }
                                bytes.push(v);
                                Ok(())
                            }
                        )?;
                    },
                }

            }

            for exp in &mut self.director.exposed_labels
            {
                for l in &self.labels
                {
                    if(l.name == exp.name)
                    {
                        exp.pos = l.pos;
                        exp.fileloc = l.fileloc.clone();
                    }
                }
            }

            if let Some(entry) = &mut self.director.entry_point
            {
                let mut found_label = false;
                for lab in &self.labels
                {
                    if(lab.name == entry.name)
                    {
                        found_label = true;
                        entry.fileloc = lab.fileloc.clone();
                        entry.pos = lab.pos;
                    }
                }
                if(!found_label)
                {
                    return Err(error_in!((&entry.fileloc), "Label '{}' cannot be used as an entry point as is does not exist!", entry.name));
                }
            }

            Ok(
                ParseResult
                {
                    bytes,
                    exposed_labels: std::mem::take(&mut self.director.exposed_labels),
                    entry_point:    std::mem::take((&mut self.director.entry_point)),
                }
            )

        }

        pub fn apply_replacers<'a>(&mut self, bytes: &mut Vec<u8>, offset: usize, get_label: impl Fn(String) -> Option<&'a Label>) -> Result<(), Error>
        {

            for l in &mut self.labels
            {
                l.pos += offset;
            }

            for r in &self.label_requests
            {

                let mut label: Option<&Label> = None;

                for l in &self.labels { if(r.name == l.name) { label = Some(l); break; } }

                for l in &self.director.external_labels
                {
                    if(l.name == r.name)
                    {
                        if let Some(l) =  get_label(r.name.clone())
                        {
                            label = Some(l);
                            break;
                        }
                    }
                }

                if(label.is_none())
                {

                    return Err(error_in!((&r.loc), "Interpreted as label: Label '{}' doesnt exist!", r.name));

                }

                let pos: u16 = label.unwrap().pos as u16;

                let (l0, l1) = u16_2_u8(pos);

                bytes[r.pos + 0] = l0;
                bytes[r.pos + 1] = l1;

            }

            Ok(())

        }

    }

}

pub type ASM = __asm::ASM;

pub mod __linc
{
    use colored::Colorize;
    use squire::instructions::SourceLocation;

    use crate::asm::__asm::Label;

    use super::ASM;
    use super::__asm::ParseResult;

    use squire::instructions::Error;
    use squire::{error, error_in};


    pub struct Linker
    {
        assemblers: Vec<(ASM, Option<ParseResult>)>,
    }
    impl Linker
    {
        
        pub fn code(src: String) -> Self
        {
            Self
            {
                assemblers: vec![ (ASM::code(src), None) ],
            }
        }
        pub fn file(files: Vec<std::sync::Arc<str>>) -> Result<Self, Error>
        {
            Ok(Self
            {
                assemblers: files.into_iter().map(ASM::file).collect::<Result<Vec<ASM>, Error>>()?.into_iter().map(|a| (a, None)).collect::<Vec<(ASM, Option<ParseResult>)>>()
            })
        }

        fn parse_all(&mut self) -> Result<usize, Error>
        {

            let mut entry_file_index: Option<usize> = None;
            
            for i in 0..self.assemblers.len()
            {
                let p = &mut self.assemblers[i];
                let ps = p.0.parse()?;
                if let Some(e) = &ps.entry_point
                {
                    if let Some(_i) = entry_file_index
                    {
                        let other = self.assemblers[_i].1.as_ref().unwrap().entry_point.as_ref().unwrap();
                        return Err(error!("Multiple entry points defined! First here: {}, another here: {}", e.fileloc, other.fileloc));
                    }
                    entry_file_index = Some(i);
                }
                p.1 = Some(ps);
            }

            let entry_file_index = match entry_file_index
            {
                Some(e) => e,
                None => 
                {
                    println!("{} No entry point specified, asuming start of first file as entry!", "Notice: ".cyan());
                    let label = Label {
                        name: "[ASSUMED ENTRY]".to_string(),
                        fileloc: SourceLocation::from(1, 1, self.assemblers[0].0.get_file()),
                        pos: 0,
                        imported: false,
                    };
                    self.assemblers[0].1.as_mut().unwrap().entry_point = Some(label);
                    0
                }
            };

            Ok(entry_file_index)

        }

        pub fn link(&mut self) -> Result<Vec<u8>, Error>
        {

            let entry_index = self.parse_all()?;

            let mut intermediate_collection: Vec<(ParseResult, usize, ASM)> = Vec::new();

            let entry_part = self.assemblers.remove(entry_index);

            let entry_part = ( entry_part.0, entry_part.1.unwrap() );

            let mut offset = entry_part.1.bytes.len();

            intermediate_collection.push(( entry_part.1, 0, entry_part.0 ));

            for part in self.assemblers.drain(..)
            {
                let part = ( part.0, part.1.unwrap() );
                let len = part.1.bytes.len();
                intermediate_collection.push(( part.1, offset, part.0 ));
                offset += len;
            }

            let mut label_index: Vec<Label> = Vec::new();

            for part in &intermediate_collection
            {
                for l in &part.0.exposed_labels
                {
                    
                    let mut label = l.clone();

                    for _l in &label_index
                    {
                        if(_l.name == l.name)
                        {
                            return Err(error_in!((&l.fileloc), "Another label with the same name has already been exposed here: {}", _l.fileloc));
                        }
                    }
                    
                    label.pos += part.1;
                    label_index.push(label);

                }
            }

            let mut code: Vec<u8> = Vec::new();

            for part in &mut intermediate_collection
            {
                part.2.apply_replacers(&mut part.0.bytes, part.1, 
                    |name|
                    {

                        for l in &label_index
                        {
                            if(l.name == name) { return Some(l); }
                        }
                        
                        None

                    }
                )?;
                code.extend(&part.0.bytes);
            }

            Ok(code)

        }

    }

}

pub type Linker = __linc::Linker;
