use colored::{ColoredString, Colorize};

use std::sync::Arc;
use squire::instructions::Error;
pub type Location = squire::instructions::SourceLocation;
use squire::error_in;

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
    value: StatementExpression
}
#[derive(Clone)]
pub struct StatementDefinitionVar
{
    name: String,
    r#type: String,
    value: Option<StatementExpression>,
}

impl std::fmt::Debug for StatementDefinitionVar
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result 
    {
        let name = ColoredString::from("\"".to_owned() + &self.name + "\"").bright_green();
        let r#type = ColoredString::from("\"".to_owned() + &self.r#type + "\"").bright_green();
        let value = match &self.value { Some(e) => format!("{:?}", e), None => format!("None")};
        write!(f, "{} {{ {}: {}, {}: {}, {}: {} }}", "DefinitionVar".green(), "name".cyan(), name, "type".cyan(), r#type, "value".cyan(), value)
    }
}

#[derive(Debug, Clone)]
pub struct StatementDefinitionFunc
{
    name: String,
    rtype: String,
    params: Vec<StatementDefinitionVar>,
    content: Vec<Statement>,
}
#[derive(Clone)]
pub struct StatementDefinitionStruct
{
    name: String,
    members: Vec<Statement>,
}

impl std::fmt::Debug for StatementDefinitionStruct
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result 
    {
        let name = ColoredString::from("\"".to_owned() + &self.name + "\"").bright_green();
        let mut _members: Vec<String> = Vec::new();
        self.members.iter().for_each(|m| _members.push(format!("{:?}", m)));
        let members = format!("{} {} {}", "[".bright_green(), _members.join(", "), "]".bright_green());
        write!(f, "{} {{ {}: {}, {}: {} }}", "DefinitionStruct".green(), "name".cyan(), name, "members".cyan(), members)
    }
}

#[derive(Clone)]
pub struct StatementExpressionLiteral
{
    value: String,
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
    object: Box<StatementExpression>,
    member: Box<StatementExpression>,
}
#[derive(Debug, Clone)]
pub struct StatementExpressionFunctionCall
{
    name: Box<StatementExpression>,
    args: Vec<StatementExpression>,
}
#[derive(Debug, Clone)]
pub struct StatementExpressionUnary
{
    operator: String,
    expr: Box<StatementExpression>,
}
#[derive(Debug, Clone)]
pub struct StatementExpressionBinary
{
    operator: String,
    expr1: Box<StatementExpression>,
    expr2: Box<StatementExpression>,
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
pub struct StatementConditionalIf
{
    condition: StatementExpression,
    content: Vec<Statement>,
    r#else: Vec<Statement>,
}
#[derive(Debug, Clone)]
pub struct StatementConditionalWhile
{
    condition: StatementExpression,
    content: Vec<Statement>,
    r#else: Vec<Statement>,
}
#[derive(Debug, Clone)]
pub enum StatementConditional
{
    If    (StatementConditionalIf),
    While (StatementConditionalWhile),
}
#[derive(Clone)]
pub enum Statement
{
    Return           (StatementReturn),
    DefinitionVar    (StatementDefinitionVar),
    DefinitionFunc   (StatementDefinitionFunc),
    DefinitionStruct (StatementDefinitionStruct),
    Expression       (StatementExpression),
    Conditional      (StatementConditional),
}

impl std::fmt::Debug for Statement
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result 
    {
        match self
        {
            Statement::Return           (s) => write!(f, "{:?}", s),
            Statement::DefinitionVar    (s) => write!(f, "{:?}", s),
            Statement::DefinitionFunc   (s) => write!(f, "{:?}", s),
            Statement::DefinitionStruct (s) => write!(f, "{:?}", s),
            Statement::Expression       (s) => write!(f, "{:?}", s),
            Statement::Conditional      (s) => write!(f, "{:?}", s),
        }
    }
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

    fn parse_variable_definition(&mut self, skip_kw:bool) -> Result<StatementDefinitionVar, Error>
    {

        if(skip_kw)
        {
            self.next(None)?; // skip kw       
        }

        let name = self.expect(TokenType::Identifier)?.unwrap().raw;
        self.expect(TokenType::TypeIndicator)?;
        let r#type = self.expect(TokenType::Identifier)?.unwrap().raw;
        let mut value: Option<StatementExpression> = None;

        if(self.peek(None)?.unwrap().t_type == TokenType::Operator && self.peek(None)?.unwrap().raw == "=")
        {
            self.next(None)?;
            value = Some(self.parse_expression()?);
        }

        Ok(StatementDefinitionVar 
        {
            name,
            r#type,
            value,            
        })

    }

    fn parse_function_definition(&mut self) -> Result<StatementDefinitionFunc, Error>
    {

        self.next(None)?; // skip kw

        let name = self.expect(TokenType::Identifier)?.unwrap().raw;
        self.expect(TokenType::GroupOpen)?;

        let mut params: Vec<StatementDefinitionVar> = Vec::new();

        while(self.peek(None)?.unwrap().t_type != TokenType::GroupClose)
        {
            if(!params.is_empty()) { self.expect(TokenType::ListSeperator)?; }
            params.push(self.parse_variable_definition(false)?);
        }
        self.next(None)?; // skip TokenType::GroupClose

        self.expect(TokenType::TypeIndicator)?;
        let rtype = self.expect(TokenType::Identifier)?.unwrap().raw;

        self.expect(TokenType::BlockOpen)?;

        let params = params;
        let mut content: Vec<Statement> = Vec::new();

        while(self.peek(None)?.unwrap().t_type != TokenType::BlockClose)
        {
            content.push(self.parse_statement()?);
        }
        self.next(None)?; // skip TokenType::BlockClose

        Ok(StatementDefinitionFunc 
        {
            name,
            rtype,
            params,
            content,
        })

    }

    fn parse_struct(&mut self) -> Result<StatementDefinitionStruct, Error>
    {

        self.next(None)?; // skip kw

        let name = self.expect(TokenType::Identifier)?.unwrap().raw;

        self.expect(TokenType::BlockOpen)?;

        let mut members: Vec<Statement> = Vec::new();

        while(self.peek(None)?.unwrap().t_type != TokenType::BlockClose)
        {
            members.push(self.parse_statement()?);
        }
        self.next(None)?; // skip TokenType::BlockClose

        Ok(StatementDefinitionStruct
        {
            name,
            members
        })

    }

    fn parse_return(&mut self) -> Result<StatementReturn, Error>
    {
        self.next(None)?; // skip kw
        Ok(StatementReturn
        {
            value: self.parse_expression()?,
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
            Ok(StatementExpression::Literal(
                StatementExpressionLiteral
                {
                    value: self.expect(TokenType::Identifier)?.unwrap().raw,
                }
            ))
        }
    }

    fn parse_expression_object_access(&mut self) -> Result<StatementExpression, Error>
    {

        let mut object = self.parse_expression_literal()?;

        while(self.peek(Some(true))?.unwrap().t_type == TokenType::ObjectAccess)
        {
            self.next(Some(true))?;
            object = StatementExpression::ObjectAccess(StatementExpressionObjectAccess {
                object: Box::new(object),
                member: Box::new(self.parse_expression_literal()?),
            });
        }

        Ok(object)

    }

    fn parse_expression_function_call(&mut self) -> Result<StatementExpression, Error>
    {

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
                name: Box::new(name),
                args,
            });
        }

        Ok(name)

    }

    fn parse_expression_unary(&mut self) -> Result<StatementExpression, Error>
    {

        Ok(if(self.peek(Some(true))?.unwrap().t_type == TokenType::Operator && operators_unary.contains(&self.peek(Some(true))?.unwrap().raw.as_ref()))
        {
            StatementExpression::Unary(StatementExpressionUnary{
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

        if(level <= -1) { return self.parse_expression_unary(); }

        let mut expr1 = self.parse_expression_binary(level - 1)?;

        while(self.peek(Some(true))?.unwrap().t_type == TokenType::Operator && operators_binary[level as usize].contains(&self.peek(Some(true))?.unwrap().raw.as_ref()))
        {
            expr1 = StatementExpression::Binary(StatementExpressionBinary{
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

        Ok(if(if_type)
        {
            StatementConditional::If(StatementConditionalIf{
                condition,
                content,
                r#else,
            })
        }
        else
        {
            StatementConditional::While(StatementConditionalWhile{
                condition,
                content,
                r#else,
            })
        })

    }

    pub fn parse_statement(&mut self) -> Result<Statement, Error>
    {

        while(self.peek(Some(true))?.unwrap_or(&Token { t_type: TokenType::Error, loc: Location::new(), raw: String::new() }).t_type  == TokenType::LineTermination) { self.next(None)?; }

        let mut line_termination_needed = true;

        let statement: Statement = match self.peek(Some(true))?.unwrap_or(&Token { t_type: TokenType::Error, loc: Location::new(), raw: String::new() }).t_type
        {
            TokenType::KeywordStructDef => 
            {
                line_termination_needed = false;
                Statement::DefinitionStruct(self.parse_struct()?)
            },
            TokenType::KeywordFuncDef => 
            {
                line_termination_needed = false;
                Statement::DefinitionFunc(self.parse_function_definition()?)
            },
            TokenType::KeywordVarDef => 
            {
                Statement::DefinitionVar(self.parse_variable_definition(true)?)
            },
            TokenType::KeywordReturn => 
            {
                Statement::Return(self.parse_return()?)
            },

            TokenType::KeywordIf | TokenType::KeywordWhile =>
            {
                line_termination_needed = false;
                Statement::Conditional(self.parse_confitional()?)
            }

            TokenType::Identifier | TokenType::GroupOpen | TokenType::Operator =>
            {
                Statement::Expression(self.parse_expression()?)
            }

            _ => 
            {
                panic!("your windows will bluescreen in 50 seconds (statement magic token:{:?})", self.peek(None)?);
            },
        };

        if(line_termination_needed) { self.expect(TokenType::LineTermination)?; }

        println!("-------------\n{}", ColoredString::from(format!("{:?}", statement)).red());

        Ok(statement)

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
