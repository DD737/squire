use std::fs::read_to_string;
use std::sync::Arc;

use squire::instructions::Error;
pub type Location = squire::instructions::SourceLocation;

#[derive(Debug)]
pub struct Preprocessor
{
    pub source: String,
    pub position: usize,
    pub last_char: Option<char>,

    pub file: Arc<str>,
    pub line: i32,
    pub column: i32,
}
impl Preprocessor
{

    fn preproc_magic(src: String) -> String
    {

        let mut out = String::new();

        for l in src.split('\n')
        {
            let l = l.trim();
            if(l.len() < 2 || &l[0..2] != "//") 
            { 
                out.push_str(l); 
            }
        }

        out

    }

    pub fn file(file: Arc<str>) -> Result<Self, Error>
    {
        Ok(Self
        {
            source: match read_to_string(file.to_string())
            {
                Ok(s) => Preprocessor::preproc_magic(s),
                Err(e) => return Err(Error::fromio(e)),
            },
            position: 0,
            last_char: None,

            file,
            line: 1,
            column: 1,
        })
    }

    pub fn get_loc(&self) -> Location
    { Location::from(self.line, self.column, self.file.clone()) }

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

    pub fn peek(&mut self) -> Option<(&char, Location)>
    { 
        if(self._peek()? == &'\r')
        {
            self.next();
            return self.peek();
        }
        else
        {
            let loc = self.get_loc();
            Some(( self._peek()?, loc )) 
        }
    }
    pub fn next(&mut self) -> Option<(char, Location)>
    {
        let c = self._next()?;
        if(c == '\n')
        {
            self.line += 1;
            self.column = 1;
        }
        else { self.column += 1; }
        Some(( c, self.get_loc() )) 
    }

}
