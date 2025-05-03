use std::fs::read_to_string;
use std::iter::Peekable;
use std::sync::Arc;

use erebos::instructions::{Error, SourceLocation};
pub type Location = erebos::instructions::SourceLocation;

#[derive(Debug)]
pub struct Preprocessor
{
    pub source: Peekable<std::vec::IntoIter<(char, SourceLocation)>>,
    pub file: Arc<str>,
    pub last_loc: Location,
}
impl Preprocessor
{

    fn preproc_magic(src: String, file: Arc<str>) -> Peekable<std::vec::IntoIter<(char, SourceLocation)>>
    {

        let mut linev: Vec<(char, SourceLocation)> = Vec::new();
        let mut lines: Vec<Vec<(char, SourceLocation)>> = Vec::new();

        let mut line  : i64 = 1;
        let mut column: i64 = 1;

        for c in src.chars()
        {
            if(c == '\n')
            {
                lines.push(linev);
                linev = Vec::new();
                line += 1;
                column = 1;
            }
            else
            {
                column += 1;
                linev.push((c, SourceLocation::from(line, column, file.clone())));
            }
        }
        
        let mut out: Vec<(char, SourceLocation)> = Vec::new();

        for l in lines
        {
            
            if(l.is_empty()) { continue; }

            let mut start = 0;
            let mut end = l.len() - 1;

            for (i, c) in l.iter().enumerate()
            {
                start = i;
                if(!c.0.is_whitespace())
                {
                    break;
                }
            }

            if(start == end && l[end].0.is_whitespace()) { continue; }

            for (i, c) in l.iter().enumerate().rev()
            {
                if(!c.0.is_whitespace())
                {
                    end = i;
                    break;
                }
            }

            out.extend_from_slice(&l[start..=end]);

        }

        out.into_iter().peekable()

    }

    pub fn file(file: Arc<str>) -> Result<Self, Error>
    {
        Ok(Self
        {
            source: match read_to_string(file.to_string())
            {
                Ok(s) => Preprocessor::preproc_magic(s, file.clone()),
                Err(e) => return Err(Error::fromio(e)),
            },
            file,
            last_loc: Location::default(),
        })
    }

    pub fn get_loc(&self) -> Location
    { self.last_loc.clone() }

    pub fn peek(&mut self) -> Option<&(char, Location)>
    { 
        if(self.source.peek()?.0 == '\r')
        {
            self.next();
            self.peek()
        }
        else
        {
            self.source.peek()
        }
    }
#[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> Option<(char, Location)>
    {
        let n = self.source.next()?;
        self.last_loc = n.1.clone();
        Some(n)
    }

}
