#![allow(unused_parens)]
use crate::instructions::*;
use crate::instructions::helpers::*;
use std::fs::read;

macro_rules! error
{
    ($($arg:tt)*) => 
    { 
        crate::instructions::Error::from(format!($($arg)*))
    }
}

#[derive(Debug, Clone)]
pub struct DebugSymbol
{
    pub loc: SourceLocation,
    pub pos: u32
}
impl DebugSymbol
{
    pub fn new(loc: SourceLocation, pos: u32) -> Self
    {
        Self
        {
            loc,
            pos,
        }
    }
    pub fn from_stream(mut fetch: impl FnMut() -> Result<u8, Error>) -> Result<Self, Error>
    {
        
        let pos  = u8_2_u32(( fetch()?, fetch()?, fetch()?, fetch()? ));
        let col  = u8_2_u32(( fetch()?, fetch()?, fetch()?, fetch()? ));
        let line = u8_2_u32(( fetch()?, fetch()?, fetch()?, fetch()? ));

        let mut file = String::new();

        loop
        {
            let b = fetch()?;
            if(b == 0) { break; }
            file.push(b as char);
        }

        Ok(Self
        {
            pos,
            loc: SourceLocation::from(line as i64, col as i64, file.into())
        })

    }
    pub fn from_bytes(bytes: Vec<u8>) -> Result<Self, Error>
    {

        if(bytes.len() < 0x0E)
        {
            return Err(error!("Not enough bytes to create DebugSymbol!"));
        }

        let pos  = u8_2_u32(( bytes[0x00], bytes[0x01], bytes[0x02], bytes[0x03] ));
        let col  = u8_2_u32(( bytes[0x04], bytes[0x05], bytes[0x06], bytes[0x07] ));
        let line = u8_2_u32(( bytes[0x08], bytes[0x09], bytes[0x0A], bytes[0x0B] ));

        let mut cntr = 0x0C;
        let mut file = String::new();

        while(bytes[cntr] != 0)
        {
            if(cntr >= bytes.len())
            {
                return Err(error!("File name was not 0 terminated!"));
            }
            file.push(bytes[cntr] as char);
            cntr += 1;
        }

        Ok(Self
        {
            pos,
            loc: SourceLocation::from(line as i64, col as i64, file.into())
        })

    }

    pub fn to_line(&self) -> String
    {
        format!("[{:#010x}] {}", self.pos, self.loc)
    }
    pub fn to_bytes(&self) -> Vec<u8>
    {

        let mut bytes: Vec<u8> = Vec::new();

        let pos  = u32_2_u8(self.pos);
        let col  = u32_2_u8(self.loc.column as u32);
        let line = u32_2_u8(self.loc.line   as u32);

        bytes.push(pos.0);
        bytes.push(pos.1);
        bytes.push(pos.2);
        bytes.push(pos.3);

        bytes.push(col.0);
        bytes.push(col.1);
        bytes.push(col.2);
        bytes.push(col.3);

        bytes.push(line.0);
        bytes.push(line.1);
        bytes.push(line.2);
        bytes.push(line.3);

        for c in self.loc.file.chars()
        {
            bytes.push(c as u8);
        }

        bytes.push(0);

        bytes

    }
}

pub struct DebugInfoProvider
{
    pub symbols: Vec<DebugSymbol>,
}
impl DebugInfoProvider
{

    pub fn from_file(file: String) -> Result<Self, Error>
    {
        
        let bytes = match read(&file)
        {
            Ok(b) => b,
            Err(e) => return Err(Error::fromio(e)),
        };

        if(bytes[0] != 0xFF || bytes[1] != 0xFF)
        {
            return Err(error!("'{}' is not a valid debug symbol file! [did you maybe provide the human readable version?]", file));
        }

        let mut ptr = 2;

        let mut symbols: Vec<DebugSymbol> = Vec::new();

        loop
        {
            symbols.push(DebugSymbol::from_stream(|| {
                if(ptr >= bytes.len())
                {
                    return Err(error!("Expected more bytes when parsing debug symbol file!"));
                }
                ptr += 1;
                Ok(bytes[ptr - 1])
            })?);
            if(ptr >= bytes.len()) { break; }
        }

        Ok(Self
        {
            symbols
        })

    }

    pub fn get_location(&self, pos: u32) -> Option<SourceLocation>
    {

        if(self.symbols.is_empty()) { return None; }
        
        for i in 0..(self.symbols.len() - 1)
        {
            let c = &self.symbols[i    ];
            let n = &self.symbols[i + 1];
            if(c.pos <= pos && n.pos > pos)
            {
                return Some(c.loc.clone());
            }
        }

        Some(self.symbols.last()?.loc.clone())

    }

}
