use colored::{ColoredString, Colorize};

use std::borrow::Borrow;
use std::sync::Arc;
use erebos::instructions::{Error, SourceLocation};
pub type Location = erebos::instructions::SourceLocation;
use erebos::error_in;

use crate::token::{Token, TokenType, Tokenizer, operators_binary, operators_unary};
#[derive(Debug, Clone)]
pub enum StatementType
{
    Error,
    DefinitionVar,
    DefinitionFunc,
    DefinitionStruct,
    Expression,
    Return,
    Conditional,
}
#[derive(Debug, Clone)]
pub struct StatementReturn
{
    pub loc: SourceLocation,
    pub value: Option<StatementExpression>
}
#[derive(Debug, Clone, PartialEq)]
pub enum StatementDefinitionVarType
{
    Var, Con, Let,
}
#[derive(Clone)]
pub struct StatementDefinitionVar
{
    pub loc: SourceLocation,
    pub name:    (String, SourceLocation),
    pub r#type: Option<StatementExpression>,
    pub storage: (String, SourceLocation),
    pub vtype: StatementDefinitionVarType,
    pub value: Option<StatementExpression>,
}

impl std::fmt::Debug for StatementDefinitionVar
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result 
    {
        let name = ColoredString::from("\"".to_owned() + &self.name.0 + "\"").bright_green();
        let r#type = ColoredString::from("\"".to_owned() + &format!("{:?}", self.r#type) + "\"").bright_green();
        let value = match &self.value { Some(e) => format!("{:?}", e), None => "None".to_string() };
        let storage = ColoredString::from("\"".to_owned() + &self.storage.0 + "\"").bright_green();
        write!(f, "{} {{ {}: {}, {}: {}, {}: {}, {}: {} }}", "DefinitionVar".green(), "name".cyan(), name, "type".cyan(), r#type, "value".cyan(), value, "storage".cyan(), storage)
    }
}

#[derive(Debug, Clone)]
pub struct StatementDefinitionFunc
{
    pub loc: SourceLocation,
    pub name:  (String, SourceLocation),
    pub rtype: Option<StatementExpression>,
    pub params: Vec<StatementDefinitionVar>,
    pub content: Vec<Statement>,
}
#[derive(Clone)]
pub struct StatementDefinitionStruct
{
    pub loc: SourceLocation,
    pub name: (String, SourceLocation),
    pub members: Vec<Statement>,
}

impl std::fmt::Debug for StatementDefinitionStruct
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result 
    {
        let name = ColoredString::from("\"".to_owned() + &self.name.0 + "\"").bright_green();
        let mut _members: Vec<String> = Vec::new();
        self.members.iter().for_each(|m| _members.push(format!("{:?}", m)));
        let members = format!("{} {} {}", "[".bright_green(), _members.join(", "), "]".bright_green());
        write!(f, "{} {{ {}: {}, {}: {} }}", "DefinitionStruct".green(), "name".cyan(), name, "members".cyan(), members)
    }
}

#[derive(Clone)]
pub struct StatementExpressionLiteral
{
    pub loc: SourceLocation,
    pub value: String,
}

impl std::fmt::Debug for StatementExpressionLiteral
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result 
    {
        let value = ColoredString::from(format!("\"{}\"", self.value)).bright_green();
        write!(f, "{} {{ {}: {} }}", "ExpressionLiteral".green(), "value".cyan(), value)
    }
}

#[derive(Debug, Clone)]
pub struct StatementExpressionObjectAccess
{
    pub loc: SourceLocation,
    pub object: Box<StatementExpression>,
    pub member: Box<StatementExpression>,
}
#[derive(Debug, Clone)]
pub struct StatementExpressionFunctionCall
{
    pub loc: SourceLocation,
    pub name: Box<StatementExpression>,
    pub args: Vec<StatementExpression>,
}
#[derive(Debug, Clone)]
pub struct StatementExpressionUnary
{
    pub loc: SourceLocation,
    pub operator: String,
    pub expr: Box<StatementExpression>,
}
#[derive(Debug, Clone)]
pub struct StatementExpressionBinary
{
    pub loc: SourceLocation,
    pub operator: String,
    pub expr1: Box<StatementExpression>,
    pub expr2: Box<StatementExpression>,
}
#[derive(Clone)]
pub enum StatementExpression
{
    Literal      (StatementExpressionLiteral),
    ObjectAccess (StatementExpressionObjectAccess),
    FunctionCall (StatementExpressionFunctionCall),
    Unary        (StatementExpressionUnary),
    Binary       (StatementExpressionBinary),
}
impl StatementExpression
{
    pub fn loc(&self) -> SourceLocation
    {
        match self
        {
            StatementExpression::Literal      (s) => s.loc.clone(),
            StatementExpression::ObjectAccess (s) => s.loc.clone(),
            StatementExpression::FunctionCall (s) => s.loc.clone(),
            StatementExpression::Unary        (s) => s.loc.clone(),
            StatementExpression::Binary       (s) => s.loc.clone(),
        }
    }
}
impl std::fmt::Debug for StatementExpression
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result 
    {
        match self
        {
            StatementExpression::Literal      (e) => write!(f, "{:?}", e),
            StatementExpression::ObjectAccess (e) => write!(f, "{:?}", e),
            StatementExpression::FunctionCall (e) => write!(f, "{:?}", e),
            StatementExpression::Unary        (e) => write!(f, "{:?}", e),
            StatementExpression::Binary       (e) => write!(f, "{:?}", e),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Modifier
{
    pub loc: Location,
    pub name: String,
    pub args: Vec<String>,
}
#[derive(Debug, Clone)]
pub enum StatementConditionalType
{
    If,
    While,
    Else,
}
#[derive(Debug, Clone)]
pub struct StatementConditional
{
    pub loc: SourceLocation,
    pub r#type: StatementConditionalType,
    pub condition: StatementExpression,
    pub content: Vec<Statement>,
    pub r#else: Vec<Statement>,
}
#[derive(Clone)]
pub enum Statement
{
    Return           (Vec<Modifier>, StatementReturn),
    DefinitionVar    (Vec<Modifier>, StatementDefinitionVar),
    DefinitionFunc   (Vec<Modifier>, StatementDefinitionFunc),
    DefinitionStruct (Vec<Modifier>, StatementDefinitionStruct),
    Expression       (Vec<Modifier>, StatementExpression),
    Conditional      (Vec<Modifier>, StatementConditional),
}
impl Statement
{
    pub fn loc(&self) -> SourceLocation
    {
        match self
        {
            Statement::Return           (_, s) => s.loc.clone(),
            Statement::DefinitionVar    (_, s) => s.loc.clone(),
            Statement::DefinitionFunc   (_, s) => s.loc.clone(),
            Statement::DefinitionStruct (_, s) => s.loc.clone(),
            Statement::Conditional      (_, s) => s.loc.clone(),
            Statement::Expression       (_, s) => s.loc(),
        }
    }
    pub fn modifier(&self) -> Vec<Modifier>
    {
        match self
        {
            Statement::Return           (m, _) => m.clone(),
            Statement::DefinitionVar    (m, _) => m.clone(),
            Statement::DefinitionFunc   (m, _) => m.clone(),
            Statement::DefinitionStruct (m, _) => m.clone(),
            Statement::Conditional      (m, _) => m.clone(),
            Statement::Expression       (m, _) => m.clone(),
        }
    }
}
impl std::fmt::Debug for Statement
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result 
    {
        match self
        {
            Statement::Return           (m, s) => write!(f, "{:?} @ {:?}", m, s),
            Statement::DefinitionVar    (m, s) => write!(f, "{:?} @ {:?}", m, s),
            Statement::DefinitionFunc   (m, s) => write!(f, "{:?} @ {:?}", m, s),
            Statement::DefinitionStruct (m, s) => write!(f, "{:?} @ {:?}", m, s),
            Statement::Expression       (m, s) => write!(f, "{:?} @ {:?}", m, s),
            Statement::Conditional      (m, s) => write!(f, "{:?} @ {:?}", m, s),
        }
    }
}

macro_rules! check_eof
{
    ($s:expr, $e:expr) =>
    {
        match $e?
        {
            Some(e) => Ok(e),
            None => Err(error_in!(($s.tokenizer.preproc.get_loc()), "Unexpected EOF!"))
        }
    };
}

#[derive(Debug)]
pub struct Parser
{
    pub tokenizer: Tokenizer,
    token: Option<Token>,
}

impl Parser
{

    pub fn file(file: Arc<str>) -> Result<Self, Error>
    {
        Ok(Self
        {
            tokenizer: Tokenizer::file(file)?,
            token: None,
        })
    }

    pub fn peek(&mut self, inexistence_fine: Option<bool>) -> Result<Option<&Token>, Error>
    {
        if(self.token.is_none()) { self.token = self.tokenizer.get_next_token(inexistence_fine)?; }
        Ok(self.token.as_ref())
    }
    pub fn next(&mut self, inexistence_fine: Option<bool>) -> Result<Option<Token>, Error>
    {
        self.peek(inexistence_fine)?;
        Ok(self.token.take())
    }
    pub fn expect(&mut self, t_type: TokenType) -> Result<Option<Token>, Error>
    {
        let token = self.next(None)?;
        let token = match token
        {
            Some(t) => t,
            None => return Err(error_in!((self.tokenizer.preproc.get_loc()), "Expected {:?}, none given!", t_type)),
        };
        if(token.t_type != t_type)
        {
            return Err(error_in!((token.loc), "Expected {:?}, {:?} given", t_type, token.t_type));
        }
        Ok(Some(token))
    }

    fn parse_variable_definition(&mut self, skip_kw:bool, vtype: StatementDefinitionVarType) -> Result<StatementDefinitionVar, Error>
    {

        if(skip_kw)
        {
            self.next(None)?; // skip kw       
        }

        let name = self.expect(TokenType::Identifier)?.unwrap();
        let r#type = if(self.peek(None)?.unwrap().t_type == TokenType::TypeIndicator)
        {
            self.expect(TokenType::TypeIndicator)?;
            Some(self.parse_expression()?)
        }
        else { None };
        let mut value: Option<StatementExpression> = None;
        let mut storage: Option<Token> = None;

        if(self.peek(None)?.unwrap().t_type == TokenType::ModifierIndicator)
        {
            
            self.next(None)?;
            storage = Some(self.expect(TokenType::Identifier)?.unwrap());

            let next = self.peek(Some(true))?;

            if let Some(next) = next
            {
                if(next.t_type == TokenType::GroupOpen)
                {
                    self.next(None)?;
                    let val = self.expect(TokenType::Identifier)?.unwrap().raw;
                    self.expect(TokenType::GroupClose)?;
                    storage.as_mut().unwrap().raw.push_str(format!("({val})").as_str());
                }
            }
            
        }

        if(self.peek(None)?.unwrap().t_type == TokenType::Operator && self.peek(None)?.unwrap().raw == "=")
        {
            self.next(None)?;
            value = Some(self.parse_expression()?);
        }

        let name    = ( name   .raw, name   .loc );
        let storage = if let Some(storage) = storage { ( storage.raw, storage.loc ) } else { (String::new(), SourceLocation::new() )};

        Ok(StatementDefinitionVar 
        {
            loc: name.1.clone(),
            name,
            r#type,
            value,  
            vtype,  
            storage,        
        })

    }

    fn parse_function_definition(&mut self) -> Result<StatementDefinitionFunc, Error>
    {

        let loc = self.next(None)?.unwrap().loc; // skip kw

        let name = self.expect(TokenType::Identifier)?.unwrap();
        self.expect(TokenType::GroupOpen)?;

        let mut params: Vec<StatementDefinitionVar> = Vec::new();

        while(self.peek(None)?.unwrap().t_type != TokenType::GroupClose)
        {
            if(!params.is_empty()) { self.expect(TokenType::ListSeperator)?; }
            params.push(self.parse_variable_definition(false, StatementDefinitionVarType::Let)?);
        }
        self.next(None)?; // skip TokenType::GroupClose

        let rtype = if(self.peek(None)?.unwrap().t_type == TokenType::TypeIndicator)
        {
            self.expect(TokenType::TypeIndicator)?;
            Some(self.parse_expression()?)
        }
        else { None };

        self.expect(TokenType::BlockOpen)?;

        let params = params;
        let mut content: Vec<Statement> = Vec::new();

        while(self.peek(None)?.unwrap().t_type != TokenType::BlockClose)
        {
            content.push(self.parse_statement()?);
        }
        self.next(None)?; // skip TokenType::BlockClose

        let name  = ( name .raw, name .loc );

        Ok(StatementDefinitionFunc 
        {
            loc,
            name,
            rtype,
            params,
            content,
        })

    }

    fn parse_struct(&mut self) -> Result<StatementDefinitionStruct, Error>
    {

        let loc = self.next(None)?.unwrap().loc; // skip kw

        let name = self.expect(TokenType::Identifier)?.unwrap();

        self.expect(TokenType::BlockOpen)?;

        let mut members: Vec<Statement> = Vec::new();

        while(self.peek(None)?.unwrap().t_type != TokenType::BlockClose)
        {
            members.push(self.parse_statement()?);
        }
        self.next(None)?; // skip TokenType::BlockClose

        let name    = ( name   .raw, name   .loc );

        Ok(StatementDefinitionStruct
        {
            loc,
            name,
            members,
        })

    }

    fn parse_return(&mut self) -> Result<StatementReturn, Error>
    {
        let loc = self.next(None)?.unwrap().loc; // skip kw
        let value = if(check_eof!(self, self.peek(None))?.t_type == TokenType::LineTermination) { None }
            else
            {
                Some(self.parse_expression()?)
            };
        Ok(StatementReturn
        {
            loc,
            value,
        })
    }

    fn parse_expression_literal(&mut self) -> Result<StatementExpression, Error>
    {
        if(self.peek(Some(true))?.unwrap().t_type == TokenType::GroupOpen)
        {
            self.next(Some(true))?;
            let e = self.parse_expression()?;
            self.expect(TokenType::GroupClose)?;
            Ok(e)
        }
        else
        {
            let tok = self.expect(TokenType::Identifier)?.unwrap();
            Ok(StatementExpression::Literal(
                StatementExpressionLiteral
                {
                    loc: tok.loc,
                    value: tok.raw,
                }
            ))
        }
    }

    fn parse_expression_object_access(&mut self) -> Result<StatementExpression, Error>
    {

        let mut object = self.parse_expression_literal()?;

        while(self.peek(Some(true))?.unwrap().t_type == TokenType::ObjectAccess)
        {
            let loc = self.next(Some(true))?.unwrap().loc;
            object = StatementExpression::ObjectAccess(StatementExpressionObjectAccess {
                loc,
                object: Box::new(object),
                member: Box::new(self.parse_expression_literal()?),
            });
        }

        Ok(object)

    }

    fn parse_expression_function_call(&mut self) -> Result<StatementExpression, Error>
    {

        let loc = self.peek(None)?.unwrap().loc.clone();

        let mut name = self.parse_expression_object_access()?;

        while(self.peek(Some(true))?.unwrap().t_type == TokenType::GroupOpen)
        {
            self.next(Some(true))?;
            let mut args: Vec<StatementExpression> = Vec::new();
            while(self.peek(Some(true))?.unwrap().t_type != TokenType::GroupClose)
            {
                args.push(self.parse_expression()?);
                if(self.peek(Some(true))?.unwrap().t_type != TokenType::GroupClose)
                {
                    self.expect(TokenType::ListSeperator)?;
                }
            }
            self.next(None)?;
            name = StatementExpression::FunctionCall(StatementExpressionFunctionCall {
                loc: loc.clone(),
                name: Box::new(name),
                args,
            });
        }

        Ok(name)

    }

    fn parse_expression_unary(&mut self) -> Result<StatementExpression, Error>
    {

        let loc = self.peek(None)?.unwrap().loc.clone();

        Ok(if(self.peek(Some(true))?.unwrap().t_type == TokenType::Operator && operators_unary.contains(&self.peek(Some(true))?.unwrap().raw.as_ref()))
        {
            StatementExpression::Unary(StatementExpressionUnary{
                loc,
                operator: self.next(Some(true))?.unwrap().raw,
                expr: Box::new(self.parse_expression_unary()?),
            })
        }
        else
        {
            self.parse_expression_function_call()?
        })

    }

    fn parse_expression_binary(&mut self, level: i8) -> Result<StatementExpression, Error>
    {

        let loc = self.peek(None)?.unwrap().loc.clone();

        if(level <= -1) { return self.parse_expression_unary(); }

        let mut expr1 = self.parse_expression_binary(level - 1)?;

        while(self.peek(Some(true))?.unwrap().t_type == TokenType::Operator && operators_binary[level as usize].contains(&self.peek(Some(true))?.unwrap().raw.as_ref()))
        {
            expr1 = StatementExpression::Binary(StatementExpressionBinary{
                loc: loc.clone(),
                operator: self.next(Some(true))?.unwrap().raw,
                expr1: Box::new(expr1),
                expr2: Box::new(self.parse_expression_binary(level - 1)?),
            });
        }

        Ok(expr1)

    }

    pub fn parse_expression(&mut self) -> Result<StatementExpression, Error>
    {
        self.parse_expression_binary((operators_binary.len() - 1) as i8)
    }

    fn parse_confitional(&mut self) -> Result<StatementConditional, Error>
    {

        let loc = self.peek(None)?.unwrap().loc.clone();

        let if_type = (self.peek(None)?.unwrap().t_type == TokenType::KeywordIf);

        self.next(None)?; // skip kw

        self.expect(TokenType::GroupOpen)?;

        let condition = self.parse_expression()?;

        self.expect(TokenType::GroupClose)?;

        let mut content: Vec<Statement> = Vec::new();

        if(self.peek(None)?.unwrap().t_type == TokenType::BlockOpen)
        {
            self.next(None)?;
            while(self.peek(None)?.unwrap().t_type != TokenType::BlockClose)
            {
                content.push(self.parse_statement()?);
            }
            self.next(None)?; // skip TokenType::BlockClose
        }
        else
        {
            content.push(self.parse_statement()?);
        }

        let mut r#else: Vec<Statement> = Vec::new();

        if(self.peek(Some(true))?.unwrap().t_type == TokenType::KeywordElse)
        {
            self.next(Some(true))?;
            if(self.peek(None)?.unwrap().t_type == TokenType::BlockOpen)
            {
                self.next(None)?;
                while(self.peek(None)?.unwrap().t_type != TokenType::BlockClose)
                {
                    r#else.push(self.parse_statement()?);
                }
                self.next(None)?; // skip TokenType::BlockClose
            }
            else
            {
                r#else.push(self.parse_statement()?);
            }
        }

        let r#type = if(if_type) { StatementConditionalType::If } else { StatementConditionalType::While };

        Ok(StatementConditional
        {
            loc,
            r#type,
            condition,
            content,
            r#else,
        })
        
    }

    fn __check_eof<T: Borrow<Token>>(&self, t: Result<Option<T>, Error>) -> Result<T, Error>
    {
        match t?
        {
            Some(t) => Ok(t),
            None => Err(error_in!((self.tokenizer.preproc.get_loc()), "Unexpected EOF!")),
        }
    }

    pub fn parse_statement(&mut self) -> Result<Statement, Error>
    {
        
        let mut modifiers: Vec<Modifier> = Vec::new();

        loop
        {
            let (stm, mo) = self._parse_statement(modifiers.clone())?;
            if let Some(m) = mo
            {
                modifiers.push(m);
            }
            else if let Some(stm) = stm
            {
                return Ok(stm);
            }
            else
            {
                return Err(error_in!((self.tokenizer.preproc.get_loc()), "Unexpected EOF!"));
            }
        }

    }
    fn _parse_statement(&mut self, mod_list: Vec<Modifier>) -> Result<(Option<Statement>, Option<Modifier>), Error>
    {

        while(self.peek(Some(true))?.unwrap_or(&Token { t_type: TokenType::Error, loc: Location::new(), raw: String::new() }).t_type  == TokenType::LineTermination) { self.next(None)?; }

        let mut line_termination_needed = true;

        if(self.peek(Some(true))?.is_none()) { return Ok((None, None)); }

        if(self.peek(None)?.unwrap().t_type == TokenType::ModifierIndicator)
        {
            
            self.next(None)?;

            let ident = self.expect(TokenType::Identifier)?.unwrap();
            let mut args: Vec<String> = Vec::new();

            if(self.peek(None)?.unwrap().t_type == TokenType::GroupOpen)
            {
                self.next(None)?; // skip GroupOpen
                loop
                {
                    let ident = self.expect(TokenType::Identifier)?.unwrap();
                    args.push(ident.raw);
                    if(check_eof!(self, self.peek(None))?.t_type == TokenType::GroupClose)
                    { break; }
                    self.expect(TokenType::ListSeperator)?;
                }
                self.next(None)?; // skip GroupClose
            }

            let m = Modifier
            {
                loc: ident.loc,
                name: ident.raw,
                args,
            };

            return Ok((None, Some(m)));

        }

        let statement: Statement = match self.peek(None)?.unwrap().t_type
        {
            TokenType::KeywordStructDef => 
            {
                line_termination_needed = false;
                Statement::DefinitionStruct(mod_list, self.parse_struct()?)
            },
            TokenType::KeywordFuncDef => 
            {
                line_termination_needed = false;
                Statement::DefinitionFunc(mod_list, self.parse_function_definition()?)
            },
            TokenType::KeywordVarDef => Statement::DefinitionVar(mod_list, self.parse_variable_definition(true, StatementDefinitionVarType::Var)?),
            TokenType::KeywordConDef => Statement::DefinitionVar(mod_list, self.parse_variable_definition(true, StatementDefinitionVarType::Con)?),
            TokenType::KeywordLetDef => Statement::DefinitionVar(mod_list, self.parse_variable_definition(true, StatementDefinitionVarType::Let)?),
            TokenType::KeywordReturn => 
            {
                if(!mod_list.is_empty())
                {
                    return Err(error_in!((&mod_list.last().unwrap().loc), "Return does not accept modifiers!"));
                }
                Statement::Return(mod_list, self.parse_return()?)
            },

            TokenType::KeywordIf | TokenType::KeywordWhile =>
            {
                if(!mod_list.is_empty())
                {
                    return Err(error_in!((&mod_list.last().unwrap().loc), "Conditionals do not accept modifiers!"));
                }
                line_termination_needed = false;
                Statement::Conditional(mod_list, self.parse_confitional()?)
            }

            TokenType::Identifier | TokenType::GroupOpen | TokenType::Operator =>
            {
                if(!mod_list.is_empty())
                {
                    return Err(error_in!((&mod_list.last().unwrap().loc), "Expressions do not accept modifiers!"));
                }
                Statement::Expression(mod_list, self.parse_expression()?)
            }

            _ => 
            {
                panic!("your windows will bluescreen in 50 seconds (statement magic token:{:?})", self.peek(None).unwrap_or_default());
            },
        };

        if(line_termination_needed) { self.expect(TokenType::LineTermination)?; }

        //println!("-------------\n{}", ColoredString::from(format!("{:?}", statement)).red());

        Ok((Some(statement), None))

    }

    pub fn parse(&mut self) -> Result<Vec<Statement>, Error>
    {

        let mut statements: Vec<Statement> = Vec::new();

        while(!self.tokenizer.end_of_tokens())
        {
            statements.push(self.parse_statement()?);
        }

        Ok(statements)

    }

}
