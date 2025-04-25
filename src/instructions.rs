#![allow(unused_parens)]
#![allow(non_camel_case_types)]
#![allow(dead_code)]

use std::sync::Arc;

use helpers::u8_2_u16;

pub const FLAG_E:u8 =  1;
pub const FLAG_A:u8 =  2;
pub const FLAG_B:u8 =  4;
pub const FLAG_Z:u8 =  8;
pub const FLAG_C:u8 = 16;

pub mod errors
{
    use std::{borrow::Borrow, fmt::Display};

    use super::SourceLocation;

    #[derive(Debug)]
    pub struct InternalError
    {
        loc: SourceLocation,
        message: String,
    }
    impl Display for InternalError
    {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
        {
            write!(f, "{}: {}", self.loc, self.message)
        }
    }

    #[derive(Debug)]
    pub enum Error
    {
        Internal(InternalError),
        Basic(String),
        IO(std::io::Error),
    }
    impl Display for Error
    {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
        {
            match self
            {
                Error::Internal(e) => write!(f, "{e}"),
                Error::Basic(e) => write!(f, "{e}"),
                Error::IO(e) => write!(f, "IO Error: {e}"),
            }
        }
    }
    impl Error
    {
        pub fn from<T: ToString>(message: T) -> Self
        {
            Error::Basic(message.to_string())
        }
        pub fn fromin<T: ToString, U: Borrow<SourceLocation>>(message: T, loc: U) -> Self
        {
            Error::Internal(InternalError { message: message.to_string(), loc: loc.borrow().clone() })
        }
        pub fn fromio(err: std::io::Error) -> Self
        {
            Error::IO(err)
        }
    }

}

pub type Error = errors::Error;

macro_rules! error
{
    ($($arg:tt)*) => 
    { 
        crate::instructions::Error::from(format!($($arg)*))
        //std::io::Error::new(std::io::ErrorKind::Other, format!($($arg)*)) 
    }
}
macro_rules! error_in
{
    ($loc:tt,$($arg:tt)*) => 
    { 
        crate::instructions::Error::fromin(format!($($arg)*), $loc)
        //std::io::Error::new(std::io::ErrorKind::Other, format!($($arg)*)) 
    }
}

#[derive(Clone)]
pub struct SourceLocation
{
    pub line: i32,
    pub column: i32,
    pub file: Arc<str>,
}
impl Default for SourceLocation
{
    fn default() -> Self { Self::new() }
}
impl SourceLocation
{
    pub fn new() -> Self
    {
        Self
        {
            line: 1,
            column: 1,
            file: Arc::default(),
        }
    }
    pub fn from(line: i32, column: i32, file: Arc<str>) -> Self
    {
        Self
        {
            line,
            column,
            file,
        }
    }
}
impl std::fmt::Display for SourceLocation
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result 
    {
        write!(f, "({}:{}:{})", self.file, self.line, self.column)
    }
}
impl std::fmt::Debug for SourceLocation
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result 
    {
        write!(f, "({}:{}:{})", self.file, self.line, self.column)
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum IRRegister
{
    RA=0, 
    RB, RC, RD, 
    R1, R2, R3, 
    R4, R5, R6, 
    R7, R8, R9, 
    RZ, 
    RIP, 
    RSP, 
}
impl TryFrom<u8> for IRRegister
{
    type Error = Error;
    fn try_from(value: u8) -> Result<Self, Self::Error> 
    {
        if value <= 15 { unsafe { Ok(std::mem::transmute::<u8, IRRegister>(value)) } }
        else { Err(error!("Cannot convert {} to Register!", value)) }
    }
}
pub type IRImmediate = u16;

#[derive(Debug, Clone)]
pub enum IRInstructionModifier
{
    Register(IRRegister),
    Memory(IRImmediate),
    RegisterAddress(IRRegister),
    MemoryAddress(IRImmediate),
    Immediate(IRImmediate),
}

pub type IRInstructionModifier2 = (IRInstructionModifier, IRInstructionModifier);
pub type IRInstructionModifier3 = (IRInstructionModifier, IRInstructionModifier, IRInstructionModifier);
/// inexistence implies stack mode
pub type IRALUInstructionModifier3 = Option<IRInstructionModifier3>; 

#[derive(Debug, Clone)]
pub enum IRInstructionWidth
{
    B8  =  8,
    B16 = 16,
}

pub type IRJIFFlags = u8;

#[derive(Debug, Clone)]
pub enum _IRALUInstruction2
{

    NOT(IRInstructionModifier2),
    CMP(IRInstructionModifier2),

}

#[derive(Debug, Clone)]
pub enum _IRALUInstruction3
{

    ADD(IRALUInstructionModifier3),
    SUB(IRALUInstructionModifier3),
    MUL(IRALUInstructionModifier3),
    DIV(IRALUInstructionModifier3),
    MOD(IRALUInstructionModifier3),

    AND(IRALUInstructionModifier3),
     OR(IRALUInstructionModifier3),
    XOR(IRALUInstructionModifier3),
    SHL(IRALUInstructionModifier3),
    SHR(IRALUInstructionModifier3),
   NAND(IRALUInstructionModifier3),
    NOR(IRALUInstructionModifier3),

}

#[derive(Debug, Clone)]
pub enum IRALUInstruction
{
    Simple (_IRALUInstruction2),
    Complex(_IRALUInstruction3),
}

#[derive(Debug, Clone)]
pub enum IRInstruction
{
    
    NOP, // no op
    HLT, // halt
    CLF, // cleaf flags
    SER_OUT(IRRegister),
    SER_IN (IRRegister),
    SER_IO(IRImmediate),
    PSHFLG,
    POPFLG,
    INT(IRImmediate),
    DBG, // debug instruction

    // every mov instruction
    MOV(IRInstructionWidth, IRInstructionModifier2), 

    // every push instruction
    PSH(IRInstructionWidth, IRInstructionModifier),
    // every pop instruction
    POP(IRInstructionWidth, IRInstructionModifier),

    JMP(IRInstructionModifier), // jump
    JIF(IRInstructionModifier, IRJIFFlags), // jump
    CAL(IRInstructionModifier), // call
    RET,

    INC(IRRegister),
    DEC(IRRegister),
    // alu instructions
    ALU(IRALUInstruction),

}

pub trait __IRBinaryHeader
{
    fn serialize(&self) -> [u8; 32];
    fn deserialize(bytes: [u8; 32]) -> Self;
}

#[allow(non_snake_case)]
pub mod _IRBinaryHeader
{
    use helpers::{u16_2_u8, u8_2_u16};

    use super::*;

    struct Serializer
    {
        pub bytes: [u8; 32],
        pub pos: usize,
    }
    impl Serializer
    {
        pub fn new() -> Self
        {
            Self
            {
                bytes: [0; 32],
                pos: 0,
            }
        }
        pub fn put8(&mut self, b: u8) -> &mut Self
        {
            self.bytes[self.pos] = b;
            self.pos += 1;
            self
        }
        pub fn put16(&mut self, w: u16) -> &mut Self
        {
            let b = u16_2_u8(w);
            self.put8(b.0).put8(b.1)
        }
        pub fn finish(&mut self) -> [u8; 32]
        {
            for i in self.pos..32
            {
                self.bytes[i] = 0;
            }
            self.bytes
        }
    }

    struct Deserializer
    {
        pub bytes: [u8; 32],
        pub pos: usize,
    }
    impl Deserializer
    {
        pub fn new(bytes: [u8; 32]) -> Self
        {
            Self
            {
                bytes,
                pos: 0,
            }
        }
        pub fn skip(&mut self, n: usize) -> &mut Self
        {
            self.pos += n;
            self
        }
        pub fn get8(&mut self, b: &mut u8) -> &mut Self
        {
            *b = self.bytes[self.pos];
            self.pos += 1;
            self
        }
        pub fn get16(&mut self, w: &mut u16) -> &mut Self
        {
            let mut b0 = 0;
            let mut b1 = 0;
            self.get8(&mut b0).get8(&mut b1);
            *w = u8_2_u16((b0, b1));
            self
        }
    }

    #[derive(Debug)]
    pub struct _V_0000
    {
        pub entry_point: u16,
        pub stack_adr: u16,
        pub stack_size: u16,
        pub flags: u8,
    }
    impl __IRBinaryHeader for _V_0000
    {
        fn serialize(&self) -> [u8; 32] 
        {
            Serializer::new()
                .put16(0x0000)
                .put16(self.entry_point)
                .put16(self.stack_adr)
                .put16(self.stack_size)
                .put8(self.flags)
            .finish()
        }
        fn deserialize(bytes: [u8; 32]) -> Self
        {
            
            let mut entry_point: u16 = 0;
            let mut stack_adr: u16 = 0;
            let mut stack_size: u16 = 0;
            let mut flags: u8 = 0;

            Deserializer::new(bytes)
                .skip(2)
                .get16(&mut entry_point)
                .get16(&mut stack_adr)
                .get16(&mut stack_size)
                .get8(&mut flags);

            Self
            {
                entry_point,
                stack_adr,
                stack_size,                
                flags,
            }

        }
    }

}

#[derive(Debug)]
pub enum IRBinaryHeader
{
    V_0000(_IRBinaryHeader::_V_0000),
}
impl IRBinaryHeader
{
    pub fn version(ver: u16, bytes: [u8; 32]) -> Self
    {
        match ver
        {
            0x0000 => IRBinaryHeader::V_0000(_IRBinaryHeader::_V_0000::deserialize(bytes)),
            _ => panic!("FATAL INVALID BINARY HEADER VERSION {:#x}", ver),
        }
    }
}
impl __IRBinaryHeader for IRBinaryHeader
{
    fn serialize(&self) -> [u8; 32] 
    {
        match self
        {
            IRBinaryHeader::V_0000(v0000) => v0000.serialize(),
        }
    }
    fn deserialize(bytes: [u8; 32]) -> Self
    {
        let version = u8_2_u16(( bytes[0], bytes[1] ));
        IRBinaryHeader::version(version, bytes)
    }
}

pub mod helpers
{

    use super::{SourceLocation, Error, IRBinaryHeader, _IRBinaryHeader};
    use crate::executable::__internal::Label;

    pub fn u16_2_u8(i:u16) -> (u8,u8)
    {
        let l = ((i & 0xFF00) >> 8) as u8;
        let r = ( i & 0x00FF      ) as u8;
        (l, r)
    }
    pub fn u8_2_u16(b:(u8,u8)) -> u16
    { ((b.0 as u16) << 8) | (b.1 as u16) }
    pub fn parse_escape(str: String, loc: &SourceLocation) -> Result<String, Error>
    {
        
        let mut out = String::new();

        let mut str = str.chars();

        while let Some(c) = str.next()
        {
            if(c == '\\') 
            {
                let n = match str.next()
                {
                    Some(c) => c,
                    //None => return Err(Error::fromin("Expected character of escape!", loc)),
                    None => return Err(error_in!(loc, "Expected character of escape!")),
                };
                let c = match n
                {
                    'r' => '\r',
                    'n' => '\n',
                    't' => '\t',
                    '0' => '\0',
                    'b' => '\x08',
                    '\\' => '\\',
                    //_ => return Err(Error::fromin(format!("Invalid escape sequence! ( \\{} is not valid)", n), loc)),
                    _ => return Err(error_in!(loc, "Invalid escape sequence! ( \\{} is not valid)", n)),
                };
                out.push(c);
            }
            else { out.push(c); }
        }

        Ok(out)

    }

    #[derive(Debug, Clone)]
    pub struct HeaderConstructor
    {
        pub version: u16,
        pub constructing: bool,

        pub stack_pos: u16,
        pub stack_size: u16,
        pub flags: u8,
        pub entry: Option<Label>,
        pub _entry: u16,

        pub file_loc: String,
    }
    impl Default for HeaderConstructor
    {
        fn default() -> Self { Self::new() }
    }
    impl HeaderConstructor
    {

        pub fn new() -> Self
        {
            Self
            {
                version: 0x0000,
                constructing: false,

                stack_pos: 0,
                stack_size: 0,
                flags: 0,
                entry: None,
                _entry: 0,

                file_loc: String::new(),
            }
        }

        pub fn set_stack_pos(&mut self, pos: u16, loc: SourceLocation) -> Result<(), Error>
        {
            self.constructing = true;
            match self.version
            {
                0x0000 | 0x0001
                    => self.stack_pos = pos,
                _ =>  return Err(error_in!(loc, "Cannot set stack location in header version {:#x}!", self.version)),
            };
            Ok(())
        }
        pub fn set_stack_size(&mut self, size: u16, loc: SourceLocation) -> Result<(), Error>
        {
            self.constructing = true;
            match self.version
            {
                0x0000 | 0x0001
                    => self.stack_size = size,
                _ =>  return Err(error_in!(loc, "Cannot set stack size in header version {:#x}!", self.version)),
            };
            Ok(())
        }
        pub fn set_flags(&mut self, flags: u8, loc: SourceLocation) -> Result<(), Error>
        {
            self.constructing = true;
            match self.version
            {
                0x0000 | 0x0001
                    => self.flags = flags,
                _ =>  return Err(error_in!(loc, "Cannot set flags in header version {:#x}!", self.version)),
            };
            Ok(())
        }
        pub fn set_straight_entry(&mut self, entry: u16, loc: SourceLocation) -> Result<(), Error>
        {
            self.constructing = true;
            match self.version
            {
                0x0000 | 0x0001
                    => self._entry = entry,
                _ =>  return Err(error_in!(loc, "Cannot set entry point in header version {:#x}!", self.version)),
            };
            Ok(())
        }
        pub fn set_entry(&mut self, entry: Label, loc: SourceLocation) -> Result<(), Error>
        {
            self.constructing = true;
            match self.version
            {
                0x0000 | 0x0001
                    => self.entry = Some(entry),
                _ =>  return Err(error_in!(loc, "Cannot set entry point in header version {:#x}!", self.version)),
            };
            Ok(())
        }

        pub fn set_file_loc(&mut self, loc_str: String, loc: SourceLocation) -> Result<(), Error>
        {
            self.constructing = true;
            match self.version
            {
                0x0001 => self.file_loc = loc_str,
                _ =>  return Err(error_in!(loc, "Cannot set entry point in header version {:#x}!", self.version)),
            };
            Ok(())
        }

        pub fn finalize(&mut self) -> Result<IRBinaryHeader, Error>
        {

            if(self.stack_size < 1024)
            {
                self.stack_size = 1024;
            }

            if(self.stack_pos < 0x1000)
            {
                self.stack_pos = 0x1000;
            }

            Ok(match self.version
            {
                0x0000 => 
                {
                    IRBinaryHeader::V_0000(_IRBinaryHeader::_V_0000{
                        entry_point: self._entry,
                        flags: self.flags,
                        stack_adr: self.stack_pos,
                        stack_size: self.stack_size,
                    })
                },
                _ => return Err(error!("Invalid binary header version: {:#x}", self.version)),
            })

        }

    }

}

pub mod _instruction_conversion
{

    use super::*;
    use super::helpers::*;

    pub fn reg_to_byte(reg: IRRegister) -> u8
    { reg as u8 }
    fn combine_regs(ra: u8, rb: u8) -> u8
    { ((ra) << 4) | (rb) }

    fn _alu_ins_to_bytes2(ins: _IRALUInstruction2, mut push: impl FnMut(u8) -> Result<(), Error>) -> Result<(), Error>
    {
        
        let (main, m) = match ins
        {
            _IRALUInstruction2::NOT(m) => (0x60, m),
            _IRALUInstruction2::CMP(m) => (0x64, m),
        };

        match m.0
        {
            IRInstructionModifier::Register(r) =>
            {
                let reg = reg_to_byte(r);
                match m.1
                {
                    IRInstructionModifier::Register(r) =>
                    {
                        push(main)?;
                        push(combine_regs(reg, reg_to_byte(r)))?;
                    },
                    IRInstructionModifier::Memory(m) =>
                    {
                        push(main + 1)?;
                        push(reg)?;
                        let m = u16_2_u8(m);
                        push(m.0)?;
                        push(m.1)?;
                    }
                    _ => return Err(error!("Invalid ALU2 modifiers! {:?}", m)),
                }
            },
            IRInstructionModifier::Memory(m0) =>
            {
                let mem = u16_2_u8(m0);
                match m.1
                {
                    IRInstructionModifier::Register(r) =>
                    {
                        push(main + 2)?;
                        push(reg_to_byte(r))?;
                        push(mem.0)?;
                        push(mem.1)?;
                    },
                    IRInstructionModifier::Memory(m) =>
                    {
                        push(main + 3)?;
                        push(mem.0)?;
                        push(mem.1)?;
                        let m = u16_2_u8(m);
                        push(m.0)?;
                        push(m.1)?;
                    }
                    _ => return Err(error!("Invalid ALU2 modifiers! {:?}", m)),
                }
            },
            _ => return Err(error!("Invalid ALU2 modifiers! {:?}", m)),
        }

        Ok(())

    }

    fn _alu_ins_to_bytes3(ins: _IRALUInstruction3, mut push: impl FnMut(u8) -> Result<(), Error>) -> Result<(), Error>
    {
        
        let (main, m) = match ins
        {
            _IRALUInstruction3:: ADD(m) => (0x70, m),
            _IRALUInstruction3:: SUB(m) => (0x71, m),
            _IRALUInstruction3:: MUL(m) => (0x72, m),
            _IRALUInstruction3:: DIV(m) => (0x73, m),
            _IRALUInstruction3:: MOD(m) => (0x74, m),
            _IRALUInstruction3:: AND(m) => (0x75, m),
            _IRALUInstruction3::  OR(m) => (0x76, m),
            _IRALUInstruction3:: XOR(m) => (0x77, m),
            _IRALUInstruction3:: SHL(m) => (0x78, m),
            _IRALUInstruction3:: SHR(m) => (0x79, m),
            _IRALUInstruction3::NAND(m) => (0x7A, m),
            _IRALUInstruction3:: NOR(m) => (0x7B, m),
        };

        let m = match m
        {
            Some(m) => m,
            None =>
            {
                push(main)?;
                return Ok(());
            },
        };


        match m.0
        {
            IRInstructionModifier::Register(r) =>
            {
                let r0 = reg_to_byte(r);
                match m.1
                {
                    IRInstructionModifier::Register(r) =>
                    {
                        let r1 = reg_to_byte(r);
                        match m.2
                        {
                            IRInstructionModifier::Register(r) =>
                            {
                                push(main + 0x10)?;
                                push(combine_regs(r0, r1))?;
                                push(reg_to_byte(r))?;
                            },
                            IRInstructionModifier::Memory(m) =>
                            {
                                push(main + 0x20)?;
                                push(combine_regs(r0, r1))?;
                                let m = u16_2_u8(m);
                                push(m.0)?;
                                push(m.1)?;
                            }
                            _ => return Err(error!("Invalid ALU3 modifiers! {:?}", m)),
                        }
                    },
                    IRInstructionModifier::Memory(m1) =>
                    {
                        let m1 = u16_2_u8(m1);
                        match m.2
                        {
                            IRInstructionModifier::Register(r) =>
                            {
                                push(main + 0x30)?;
                                push(combine_regs(r0, reg_to_byte(r)))?;
                                push(m1.0)?;
                                push(m1.1)?;
                            },
                            IRInstructionModifier::Memory(m) =>
                            {
                                push(main + 0x40)?;
                                push(r0)?;
                                push(m1.0)?;
                                push(m1.1)?;
                                let m = u16_2_u8(m);
                                push(m.0)?;
                                push(m.1)?;
                            }
                            _ => return Err(error!("Invalid ALU3 modifiers! {:?}", m)),
                        }
                    }
                    _ => return Err(error!("Invalid ALU3 modifiers! {:?}", m)),
                }
            },
            IRInstructionModifier::Memory(m0) =>
            {
                let m0 = u16_2_u8(m0);
                match m.1
                {
                    IRInstructionModifier::Register(r) =>
                    {
                        let r1 = reg_to_byte(r);
                        match m.2
                        {
                            IRInstructionModifier::Register(r) =>
                            {
                                push(main + 0x50)?;
                                push(combine_regs(r1, reg_to_byte(r)))?;
                                push(m0.0)?;
                                push(m0.1)?;
                            },
                            IRInstructionModifier::Memory(m) =>
                            {
                                push(main + 0x60)?;
                                push(r1)?;
                                push(m0.0)?;
                                push(m0.1)?;
                                let m = u16_2_u8(m);
                                push(m.0)?;
                                push(m.1)?;
                            }
                            _ => return Err(error!("Invalid ALU3 modifiers! {:?}", m)),
                        }
                    },
                    IRInstructionModifier::Memory(m1) =>
                    {
                        let m1 = u16_2_u8(m1);
                        match m.2
                        {
                            IRInstructionModifier::Register(r) =>
                            {
                                push(main + 0x70)?;
                                push(reg_to_byte(r))?;
                                push(m0.0)?;
                                push(m0.1)?;
                                push(m1.0)?;
                                push(m1.1)?;
                            },
                            IRInstructionModifier::Memory(m) =>
                            {
                                push(main + 0x80)?;
                                push(m0.0)?;
                                push(m0.1)?;
                                push(m1.0)?;
                                push(m1.1)?;
                                let m = u16_2_u8(m);
                                push(m.0)?;
                                push(m.1)?;
                            }
                            _ => return Err(error!("Invalid ALU3 modifiers! {:?}", m)),
                        }
                    }
                    _ => return Err(error!("Invalid ALU3 modifiers! {:?}", m)),
                }
            },
            _ => return Err(error!("Invalid ALU3 modifiers! {:?}", m)),
        }        

        Ok(())

    }

    fn alu_ins_to_bytes(ins: IRALUInstruction, push: impl FnMut(u8) -> Result<(), Error>) -> Result<(), Error>
    {
        match ins
        {
            IRALUInstruction::Simple (ins) => _alu_ins_to_bytes2(ins, push),
            IRALUInstruction::Complex(ins) => _alu_ins_to_bytes3(ins, push),
        }
    }
    
    pub fn ins_to_bytes(ins: IRInstruction, mut push: impl FnMut(u8) -> Result<(), Error>) -> Result<(), Error>
    {

        match ins
        {
    
            IRInstruction::NOP => push(0x00)?,
            IRInstruction::HLT => push(0x01)?,
            IRInstruction::CLF => push(0x02)?,
            IRInstruction::DBG => push(0x0F)?,

            IRInstruction::SER_OUT(r) => { push(0x04)?; push(reg_to_byte(r))?; },
            IRInstruction::SER_IN (r) => { push(0x05)?; push(reg_to_byte(r))?; },
            IRInstruction::SER_IO (i) => { push(0x06)?; push( i as u8      )?; },

            IRInstruction::PSHFLG => { push(0x07)?; },
            IRInstruction::POPFLG => { push(0x08)?; },
        
            IRInstruction::INT(imm) => { push(0x0E)?; push(imm as u8)?; }

            IRInstruction::MOV(w, (l,r)) =>
            {

                match r 
                {
                    IRInstructionModifier::Immediate(_) => return Err(error!("Instruction mov doesnt accept immediates as second argument!")),
                    _ => {}
                }

                let mut main = match w 
                {
                    IRInstructionWidth::B8  => 0x20,
                    IRInstructionWidth::B16 => 0x10,
                };

                {

                    match l
                    {
                        IRInstructionModifier::Register(rl) => 
                        {
                            let reg = reg_to_byte(rl);
                            match r
                            {
                                IRInstructionModifier::Register(r) => 
                                {
                                    main += 0x00;
                                    push(main)?;
                                    push(combine_regs(reg, reg_to_byte(r)))?;
                                },
                                IRInstructionModifier::Memory(m) => 
                                {
                                    main += 0x02;
                                    push(main)?;
                                    push(reg)?;
                                    let m = u16_2_u8(m);
                                    push(m.0)?;
                                    push(m.1)?;
                                },
                                IRInstructionModifier::RegisterAddress(r) => 
                                {
                                    main += 0x04;
                                    push(main)?;
                                    push(combine_regs(reg, reg_to_byte(r)))?;
                                },
                                IRInstructionModifier::MemoryAddress(m) => 
                                {
                                    main += 0x06;
                                    push(main)?;
                                    push(reg)?;
                                    let m = u16_2_u8(m);
                                    push(m.0)?;
                                    push(m.1)?;
                                },
                                _ => {},
                            }
                        },
                        IRInstructionModifier::Memory(ml) => 
                        {
                            let m0 = u16_2_u8(ml);
                            match r
                            {
                                IRInstructionModifier::Register(r) => 
                                {
                                    main += 0x01;
                                    push(main)?;
                                    push(reg_to_byte(r))?;
                                    push(m0.0)?;
                                    push(m0.1)?;
                                },
                                IRInstructionModifier::Memory(m) => 
                                {
                                    main += 0x03;
                                    push(main)?;
                                    push(m0.0)?;
                                    push(m0.1)?;
                                    let m = u16_2_u8(m);
                                    push(m.0)?;
                                    push(m.1)?;
                                },
                                IRInstructionModifier::RegisterAddress(r) => 
                                {
                                    main += 0x05;
                                    push(main)?;
                                    push(reg_to_byte(r))?;
                                    push(m0.0)?;
                                    push(m0.1)?;
                                },
                                IRInstructionModifier::MemoryAddress(m) => 
                                {
                                    main += 0x07;
                                    push(main)?;
                                    push(m0.0)?;
                                    push(m0.1)?;
                                    let m = u16_2_u8(m);
                                    push(m.0)?;
                                    push(m.1)?;
                                },
                                _ => {},
                            }
                        },
                        IRInstructionModifier::RegisterAddress(rl) => 
                        {
                            let reg = reg_to_byte(rl);
                            match r
                            {
                                IRInstructionModifier::Register(r) => 
                                {
                                    main += 0x08;
                                    push(main)?;
                                    push(combine_regs(reg, reg_to_byte(r)))?;
                                },
                                IRInstructionModifier::Memory(m) => 
                                {
                                    main += 0x0A;
                                    push(main)?;
                                    push(reg)?;
                                    let m = u16_2_u8(m);
                                    push(m.0)?;
                                    push(m.1)?;
                                },
                                IRInstructionModifier::RegisterAddress (_) => return Err(error!("Instruction mov doesnt accept register dereferences as second argument!")),
                                IRInstructionModifier::MemoryAddress   (_) => return Err(error!("Instruction mov doesnt accept memory dereferences as second argument!")),
                                _ => {},
                            }
                        },
                        IRInstructionModifier::MemoryAddress(ml) => 
                        {
                            let m0 = u16_2_u8(ml);
                            match r
                            {
                                IRInstructionModifier::Register(r) => 
                                {
                                    main += 0x09;
                                    push(main)?;
                                    push(reg_to_byte(r))?;
                                    push(m0.0)?;
                                    push(m0.1)?;
                                },
                                IRInstructionModifier::Memory(m) => 
                                {
                                    main += 0x0B;
                                    push(main)?;
                                    push(m0.0)?;
                                    push(m0.1)?;
                                    let m = u16_2_u8(m);
                                    push(m.0)?;
                                    push(m.1)?;
                                },
                                IRInstructionModifier::RegisterAddress (_) => return Err(error!("Instruction mov doesnt accept register dereferences as second argument!")),
                                IRInstructionModifier::MemoryAddress   (_) => return Err(error!("Instruction mov doesnt accept memory dereferences as second argument!")),
                                _ => {},
                            }
                        },
                        IRInstructionModifier::Immediate(il) => 
                        {
                            let m0 = u16_2_u8(il);
                            match r
                            {
                                IRInstructionModifier::Register(r) => 
                                {
                                    main += 0x0C;
                                    push(main)?;
                                    push(reg_to_byte(r))?;
                                    push(m0.0)?;
                                    push(m0.1)?;
                                },
                                IRInstructionModifier::Memory(m) => 
                                {
                                    main += 0x0D;
                                    push(main)?;
                                    push(m0.0)?;
                                    push(m0.1)?;
                                    let m = u16_2_u8(m);
                                    push(m.0)?;
                                    push(m.1)?;
                                },
                                IRInstructionModifier::RegisterAddress (_) => return Err(error!("Instruction mov doesnt accept register dereferences as second argument!")),
                                IRInstructionModifier::MemoryAddress   (_) => return Err(error!("Instruction mov doesnt accept memory dereferences as second argument!")),
                                _ => {},
                            }
                        },
                    }

                }

            }, 
        
            IRInstruction::PSH(w, m) =>
            {
                
                let mut main = match w 
                {
                    IRInstructionWidth::B16 => 0x30,
                    IRInstructionWidth::B8  => 0x33,
                };

                {
                    match m
                    {
                        IRInstructionModifier::Register(r) => 
                        {
                            main += 0x00;
                            push(main)?;
                            push(reg_to_byte(r))?;
                        },
                        IRInstructionModifier::Memory(m) => 
                        {
                            main += 0x01;
                            push(main)?;
                            let m = u16_2_u8(m);
                            push(m.0)?;
                            push(m.1)?;
                        },
                        IRInstructionModifier::Immediate(i) =>
                        {
                            main += 0x01;
                            push(main)?;
                            let m = u16_2_u8(i);
                            push(m.0)?;
                            push(m.1)?;
                        },
                        IRInstructionModifier::RegisterAddress (_) => return Err(error!("Instruction psh doesnt accept register dereferences as second argument!")),
                        IRInstructionModifier::MemoryAddress   (_) => return Err(error!("Instruction psh doesnt accept memory dereferences as second argument!")),
                    }

                }

            },
            IRInstruction::POP(w, m) =>
            {
                
                let mut main = match w 
                {
                    IRInstructionWidth::B16 => 0x36,
                    IRInstructionWidth::B8  => 0x38,
                };

                {
                    match m
                    {
                        IRInstructionModifier::Register(r) => 
                        {
                            main += 0x00;
                            push(main)?;
                            push(reg_to_byte(r))?;
                        },
                        IRInstructionModifier::Memory(m) => 
                        {
                            main += 0x01;
                            push(main)?;
                            let m = u16_2_u8(m);
                            push(m.0)?;
                            push(m.1)?;
                        },
                        IRInstructionModifier::RegisterAddress (_) => return Err(error!("Instruction pop doesnt accept register dereferences as second argument!")),
                        IRInstructionModifier::MemoryAddress   (_) => return Err(error!("Instruction pop doesnt accept memory dereferences as second argument!")),
                        IRInstructionModifier::Immediate       (_) => return Err(error!("Instruction pop doesnt accept immediates as second argument!")),
                    }

                }

            },
        
            IRInstruction::JMP(m) =>
            {

                match m
                {
                    IRInstructionModifier::Register(r) =>
                    {
                        push(0x40)?;
                        push(reg_to_byte(r))?;
                    },
                    IRInstructionModifier::Memory(m) =>
                    {
                        push(0x41)?;
                        let m = u16_2_u8(m);
                        push(m.0)?;
                        push(m.1)?;
                    },
                    IRInstructionModifier::Immediate(i) =>
                    {
                        push(0x42)?;
                        let m = u16_2_u8(i);
                        push(m.0)?;
                        push(m.1)?;
                    },
                    _ => return Err(error!("Instruction jmp can only take registers, memory addresses and immediates!")),
                }

            },
            IRInstruction::JIF(m, f) =>
            {

                match m
                {
                    IRInstructionModifier::Register(r) =>
                    {
                        push(0x43)?;
                        push(reg_to_byte(r))?;
                    },
                    IRInstructionModifier::Memory(m) =>
                    {
                        push(0x44)?;
                        let m = u16_2_u8(m);
                        push(m.0)?;
                        push(m.1)?;
                    },
                    IRInstructionModifier::Immediate(i) =>
                    {
                        push(0x45)?;
                        let m = u16_2_u8(i);
                        push(m.0)?;
                        push(m.1)?;
                    },
                    _ => return Err(error!("Instruction jif can only take registers, memory addresses and immediates!")),
                }

                push(f)?;

            },
            IRInstruction::CAL(m) =>
            {

                match m
                {
                    IRInstructionModifier::Register(r) =>
                    {
                        push(0x46)?;
                        push(reg_to_byte(r))?;
                    },
                    IRInstructionModifier::Memory(m) =>
                    {
                        push(0x47)?;
                        let m = u16_2_u8(m);
                        push(m.0)?;
                        push(m.1)?;
                    },
                    IRInstructionModifier::Immediate(i) =>
                    {
                        push(0x48)?;
                        let m = u16_2_u8(i);
                        push(m.0)?;
                        push(m.1)?;
                    },
                    _ => return Err(error!("Instruction cal can only take registers, memory addresses and immediates!")),
                }

            },
            IRInstruction::RET => push(0x4F)?,
       
            IRInstruction::INC(r) => { push(0x6E)?; push(reg_to_byte(r))?; },
            IRInstruction::DEC(r) => { push(0x6F)?; push(reg_to_byte(r))?; },

            IRInstruction::ALU(alu_ins) => return alu_ins_to_bytes(alu_ins, push),

        };

        Ok(())

    }

    pub mod bytes_to_repr
    {

        use super::*;
    
        fn fetch_word(fetch: &mut impl FnMut() -> Result<u8, Error>) -> Result<u16, Error>
        { Ok(u8_2_u16((fetch()?, fetch()?))) }

        fn get_reg(fetch: &mut impl FnMut() -> Result<u8, Error>) -> Result<IRRegister, Error>
        { IRRegister::try_from(fetch()?) }
        fn get2reg(fetch: &mut impl FnMut() -> Result<u8, Error>) -> Result<(IRRegister, IRRegister), Error>
        {
            let b = fetch()?;
            let ba = (b >> 4);
            let bb = (b & 15);
            Ok((
                IRRegister::try_from(ba)?, 
                IRRegister::try_from(bb)?
            ))
        }

        pub fn bytes_to_ins(mut fetch: impl FnMut() -> Result<u8, Error>) -> Result<IRInstruction, Error>
        {

            let ins = fetch()?;

            match ins 
            {
                
                0x00 => return Ok(IRInstruction::NOP),
                0x01 => return Ok(IRInstruction::HLT),
                0x02 => return Ok(IRInstruction::CLF),
                0x06 => return Ok(IRInstruction::SER_IO(fetch()? as u16)),
                0x07 => return Ok(IRInstruction::PSHFLG),
                0x08 => return Ok(IRInstruction::POPFLG),
                0x0E => return Ok(IRInstruction::INT(fetch()? as u16)),
                0x0F => return Ok(IRInstruction::DBG),

                0x4F => return Ok(IRInstruction::RET),

                _ => {},

            };

            Ok(if(ins >= 0x70) // complex alu
            {

                let modifiers: IRALUInstructionModifier3 = match ((ins & 0xF0) >> 4)
                {
                    0x7 => None,
                    0x8 => 
                    {
                        let reg01 = get2reg(&mut fetch)?;
                        let reg2  = get_reg(&mut fetch)?;
                        Some((
                            IRInstructionModifier::Register(reg01.0),
                            IRInstructionModifier::Register(reg01.1),
                            IRInstructionModifier::Register(reg2   ),
                        ))
                    }, // rrr
                    0x9 => 
                    {
                        let reg01 = get2reg    (&mut fetch)?;
                        let mem   = fetch_word (&mut fetch)?;
                        Some((
                            IRInstructionModifier::Register (reg01.0),
                            IRInstructionModifier::Register (reg01.1),
                            IRInstructionModifier::Memory   (mem    ),
                        ))
                    }, // rrm
                    0xA => 
                    {
                        let reg01 = get2reg    (&mut fetch)?;
                        let mem   = fetch_word (&mut fetch)?;
                        Some((
                            IRInstructionModifier::Register (reg01.0),
                            IRInstructionModifier::Memory   (mem    ),
                            IRInstructionModifier::Register (reg01.1),
                        ))
                    }, // rmr
                    0xB => 
                    {
                        let reg  = get_reg    (&mut fetch)?;
                        let mem0 = fetch_word (&mut fetch)?;
                        let mem1 = fetch_word (&mut fetch)?;
                        Some((
                            IRInstructionModifier::Register (reg ),
                            IRInstructionModifier::Memory   (mem0),
                            IRInstructionModifier::Memory   (mem1),
                        ))
                    }, // rmm
                    0xC => 
                    {
                        let reg01 = get2reg    (&mut fetch)?;
                        let mem   = fetch_word (&mut fetch)?;
                        Some((
                            IRInstructionModifier::Memory   (mem    ),
                            IRInstructionModifier::Register (reg01.0),
                            IRInstructionModifier::Register (reg01.1),
                        ))
                    }, // mrr
                    0xD => 
                    {
                        let reg  = get_reg    (&mut fetch)?;
                        let mem0 = fetch_word (&mut fetch)?;
                        let mem1 = fetch_word (&mut fetch)?;
                        Some((
                            IRInstructionModifier::Memory   (mem0),
                            IRInstructionModifier::Register (reg ),
                            IRInstructionModifier::Memory   (mem1),
                        ))
                    }, // mrm
                    0xE => 
                    {
                        let reg  = get_reg    (&mut fetch)?;
                        let mem0 = fetch_word (&mut fetch)?;
                        let mem1 = fetch_word (&mut fetch)?;
                        Some((
                            IRInstructionModifier::Memory   (mem0),
                            IRInstructionModifier::Memory   (mem1),
                            IRInstructionModifier::Register (reg ),
                        ))
                    }, // mmr
                    0xF => 
                    {
                        let mem0 = fetch_word (&mut fetch)?;
                        let mem1 = fetch_word (&mut fetch)?;
                        let mem2 = fetch_word (&mut fetch)?;
                        Some((
                            IRInstructionModifier::Memory   (mem0),
                            IRInstructionModifier::Memory   (mem1),
                            IRInstructionModifier::Memory   (mem2),
                        ))
                    }, // mmm

                    _ => return Err(error!("FATAL: INVALID INSTRUCTION MODIFIER! ({:#x})", ((ins & 0xF0) >> 4))),
                };

                IRInstruction::ALU(IRALUInstruction::Complex(
                    match (ins & 0xF)
                    {

                        0x0 => _IRALUInstruction3:: ADD (modifiers),
                        0x1 => _IRALUInstruction3:: SUB (modifiers),
                        0x2 => _IRALUInstruction3:: MUL (modifiers),
                        0x3 => _IRALUInstruction3:: DIV (modifiers),
                        0x4 => _IRALUInstruction3:: MOD (modifiers),
                        0x5 => _IRALUInstruction3:: AND (modifiers),
                        0x6 => _IRALUInstruction3::  OR (modifiers),
                        0x7 => _IRALUInstruction3:: XOR (modifiers),
                        0x8 => _IRALUInstruction3:: SHL (modifiers),
                        0x9 => _IRALUInstruction3:: SHR (modifiers),
                        0xA => _IRALUInstruction3::NAND (modifiers),
                        0xB => _IRALUInstruction3:: NOR (modifiers),

                        _ => return Err(error!("FATAL: INVALID INSTRUCTION! ({:#x})", (ins & 0xF))),

                    }
                ))

            }
            else if(ins >= 0x60) // simple alu
            {

                match ins
                {
                    0x6E =>
                    {
                        let reg = get_reg(&mut fetch)?;
                        return Ok(IRInstruction::INC(reg));
                    },
                    0x6F =>
                    {
                        let reg = get_reg(&mut fetch)?;
                        return Ok(IRInstruction::DEC(reg));
                    },
                    _ => {},
                }
                
                let mut not_ins = true;

                let modifiers: IRInstructionModifier2 = match ins
                {

                    0x60 => 
                    {
                        let regs = get2reg(&mut fetch)?;
                        (
                            IRInstructionModifier::Register(regs.0),
                            IRInstructionModifier::Register(regs.1),
                        )
                    }, //not rr
                    0x61 => 
                    {
                        let reg = get_reg    (&mut fetch)?;
                        let mem = fetch_word (&mut fetch)?;
                        ( 
                            IRInstructionModifier::Memory   (mem),
                            IRInstructionModifier::Register (reg),
                        )
                    }, //not mr
                    0x62 => 
                    {
                        let reg = get_reg    (&mut fetch)?;
                        let mem = fetch_word (&mut fetch)?;
                        ( 
                            IRInstructionModifier::Register (reg),
                            IRInstructionModifier::Memory   (mem),
                        )
                    }, //not rm
                    0x63 => 
                    {
                        let mem0 = fetch_word (&mut fetch)?;
                        let mem1 = fetch_word (&mut fetch)?;
                        ( 
                            IRInstructionModifier::Memory   (mem0),
                            IRInstructionModifier::Memory   (mem1),
                        )
                    }, //not mm

                    0x64 => 
                    {
                        not_ins = false;
                        let regs = get2reg(&mut fetch)?;
                        (
                            IRInstructionModifier::Register(regs.0),
                            IRInstructionModifier::Register(regs.1),
                        )
                    }, //cmp rr
                    0x65 => 
                    {
                        not_ins = false;
                        let reg = get_reg    (&mut fetch)?;
                        let mem = fetch_word (&mut fetch)?;
                        ( 
                            IRInstructionModifier::Memory   (mem),
                            IRInstructionModifier::Register (reg),
                        )
                    }, //cmp mr
                    0x66 => 
                    {
                        not_ins = false;
                        let reg = get_reg    (&mut fetch)?;
                        let mem = fetch_word (&mut fetch)?;
                        ( 
                            IRInstructionModifier::Register (reg),
                            IRInstructionModifier::Memory   (mem),
                        )
                    }, //cmp rm
                    0x67 => 
                    {
                        not_ins = false;
                        let mem0 = fetch_word (&mut fetch)?;
                        let mem1 = fetch_word (&mut fetch)?;
                        ( 
                            IRInstructionModifier::Memory   (mem0),
                            IRInstructionModifier::Memory   (mem1),
                        )
                    }, //cmp mm

                    _ => return Err(error!("FATAL: INVALID INSTRUCTION MODIFIER! ({:#x})", ins)),

                };

                IRInstruction::ALU(IRALUInstruction::Simple(
                    if(not_ins)
                    {
                        _IRALUInstruction2::NOT(modifiers)
                    }
                    else
                    {
                        _IRALUInstruction2::CMP(modifiers)
                    }
                ))

            }
            else if(ins >= 0x40) // jump instructions
            {
                
                let mut jmp_ins = true;
                let mut jif_ins = true;

                let modifier = match ins
                {

                    0x40 => IRInstructionModifier::Register (get_reg(&mut fetch)?),
                    0x41 => IRInstructionModifier::Memory   (fetch_word(&mut fetch)?),
                    0x42 => IRInstructionModifier::Immediate(fetch_word(&mut fetch)?),

                    0x43 => 
                    {
                        jmp_ins = false;
                        IRInstructionModifier::Register (get_reg(&mut fetch)?)
                    },
                    0x44 => 
                    {
                        jmp_ins = false;
                        IRInstructionModifier::Memory   (fetch_word(&mut fetch)?)
                    },
                    0x45 => 
                    {
                        jmp_ins = false;
                        IRInstructionModifier::Immediate(fetch_word(&mut fetch)?)
                    },

                    0x46 => 
                    {
                        jmp_ins = false;
                        jif_ins = false;
                        IRInstructionModifier::Register (get_reg(&mut fetch)?)
                    },
                    0x47 => 
                    {
                        jmp_ins = false;
                        jif_ins = false;
                        IRInstructionModifier::Memory   (fetch_word(&mut fetch)?)
                    },
                    0x48 => 
                    {
                        jmp_ins = false;
                        jif_ins = false;
                        IRInstructionModifier::Immediate(fetch_word(&mut fetch)?)
                    },
                    
                    _ => return Err(error!("FATAL: INVALID INSTRUCTION MODIFIER! ({:#x})", ins)),

                };

                if(jmp_ins)
                {
                    IRInstruction::JMP(modifier)
                }
                else if(jif_ins)
                {
                    IRInstruction::JIF(modifier, fetch()?)
                }
                else
                {
                    IRInstruction::CAL(modifier)
                }

            }
            else if(ins >= 0x30) // stack instructions
            {
                
                let mut push_ins = true;
                let mut ins_width = IRInstructionWidth::B16;

                let modifier = match ins
                {

                    0x30 => IRInstructionModifier::Register  (get_reg(&mut fetch)?),
                    0x31 => IRInstructionModifier::Memory    (fetch_word(&mut fetch)?),
                    0x32 => IRInstructionModifier::Immediate (fetch_word(&mut fetch)?),
                    0x33 => 
                    {
                        ins_width = IRInstructionWidth::B8;
                        IRInstructionModifier::Register (get_reg(&mut fetch)?)
                    },
                    0x34 => 
                    {
                        ins_width = IRInstructionWidth::B8;
                        IRInstructionModifier::Memory   (fetch_word(&mut fetch)?)
                    },
                    0x35 => 
                    {
                        ins_width = IRInstructionWidth::B8;
                        IRInstructionModifier::Immediate(fetch_word(&mut fetch)?)
                    },

                    0x36 => 
                    {
                        push_ins = false;
                        IRInstructionModifier::Register (get_reg(&mut fetch)?)
                    },
                    0x37 => 
                    {
                        push_ins = false;
                        IRInstructionModifier::Memory   (fetch_word(&mut fetch)?)
                    },
                    0x38 => 
                    {
                        push_ins = false;
                        ins_width = IRInstructionWidth::B8;
                        IRInstructionModifier::Register (get_reg(&mut fetch)?)
                    },
                    0x39 => 
                    {
                        push_ins = false;
                        ins_width = IRInstructionWidth::B8;
                        IRInstructionModifier::Memory   (fetch_word(&mut fetch)?)
                    },
                    
                    _ => return Err(error!("FATAL: INVALID INSTRUCTION MODIFIER! ({:#x})", ins)),

                };

                if(push_ins)
                {
                    IRInstruction::PSH(ins_width, modifier)
                }
                else
                {
                    IRInstruction::POP(ins_width, modifier)
                }

            }
            else if(ins >= 0x10) // mov instructions
            {
                
                let width = if(ins >= 0x20) { IRInstructionWidth::B8 } else { IRInstructionWidth::B16 };

                let modifiers: IRInstructionModifier2 = match (ins & 0xF)
                {

                    0x0 => 
                    {
                        let regs = get2reg(&mut fetch)?;
                        (
                            IRInstructionModifier::Register(regs.0),
                            IRInstructionModifier::Register(regs.1),
                        )
                    }, //16mov rr
                    0x1 => 
                    {
                        let reg = get_reg    (&mut fetch)?;
                        let mem = fetch_word (&mut fetch)?;
                        ( 
                            IRInstructionModifier::Memory   (mem),
                            IRInstructionModifier::Register (reg),
                        )
                    }, //16mov mr
                    0x2 => 
                    {
                        let reg = get_reg    (&mut fetch)?;
                        let mem = fetch_word (&mut fetch)?;
                        ( 
                            IRInstructionModifier::Register (reg),
                            IRInstructionModifier::Memory   (mem),
                        )
                    }, //16mov rm
                    0x3 => 
                    {
                        let mem0 = fetch_word (&mut fetch)?;
                        let mem1 = fetch_word (&mut fetch)?;
                        ( 
                            IRInstructionModifier::Memory   (mem0),
                            IRInstructionModifier::Memory   (mem1),
                        )
                    }, //16mov mm
                    0x4 => 
                    {
                        let regs = get2reg(&mut fetch)?;
                        (
                            IRInstructionModifier::Register        (regs.0),
                            IRInstructionModifier::RegisterAddress (regs.1),
                        )
                    }, //16mov rra
                    0x5 => 
                    {
                        let reg = get_reg    (&mut fetch)?;
                        let mem = fetch_word (&mut fetch)?;
                        ( 
                            IRInstructionModifier::RegisterAddress (reg),
                            IRInstructionModifier::Memory          (mem),
                        )
                    }, //16mov mra
                    0x6 => 
                    {
                        let reg = get_reg    (&mut fetch)?;
                        let mem = fetch_word (&mut fetch)?;
                        ( 
                            IRInstructionModifier::Register      (reg),
                            IRInstructionModifier::MemoryAddress (mem),
                        )
                    }, //16mov rma
                    0x7 => 
                    {
                        let mem0 = fetch_word (&mut fetch)?;
                        let mem1 = fetch_word (&mut fetch)?;
                        ( 
                            IRInstructionModifier::Memory        (mem0),
                            IRInstructionModifier::MemoryAddress (mem1),
                        )
                    }, //16mov mma
                    0x8 => 
                    {
                        let regs = get2reg(&mut fetch)?;
                        (
                            IRInstructionModifier::RegisterAddress (regs.0),
                            IRInstructionModifier::Register        (regs.1),
                        )
                    }, //16mov rar
                    0x9 => 
                    {
                        let reg = get_reg    (&mut fetch)?;
                        let mem = fetch_word (&mut fetch)?;
                        ( 
                            IRInstructionModifier::MemoryAddress (mem),
                            IRInstructionModifier::Register      (reg),
                        )
                    }, //16mov mar
                    0xA => 
                    {
                        let reg = get_reg    (&mut fetch)?;
                        let mem = fetch_word (&mut fetch)?;
                        ( 
                            IRInstructionModifier::RegisterAddress (reg),
                            IRInstructionModifier::Memory          (mem),
                        )
                    }, //16mov ram
                    0xB => 
                    {
                        let mem0 = fetch_word (&mut fetch)?;
                        let mem1 = fetch_word (&mut fetch)?;
                        ( 
                            IRInstructionModifier::MemoryAddress (mem0),
                            IRInstructionModifier::Memory        (mem1),
                        )
                    }, //16mov mam
                    0xC => 
                    {
                        let reg = get_reg    (&mut fetch)?;
                        let mem = fetch_word (&mut fetch)?;
                        ( 
                            IRInstructionModifier::Immediate (mem),
                            IRInstructionModifier::Register  (reg),
                        )
                    }, //16mov ir
                    0xD => 
                    {
                        let mem0 = fetch_word (&mut fetch)?;
                        let mem1 = fetch_word (&mut fetch)?;
                        ( 
                            IRInstructionModifier::Immediate (mem0),
                            IRInstructionModifier::Memory    (mem1),
                        )
                    }, //16mov im

                    _ => return Err(error!("FATAL: INVALID INSTRUCTION MODIFIER! ({:#x})", (ins & 0xF))),

                };

                IRInstruction::MOV(width, modifiers)

            }
            else if(ins == 0x04 || ins == 0x05) // in/out ins
            {
                let reg = get_reg(&mut fetch)?;
                if(ins == 0x04)
                {
                    IRInstruction::SER_OUT(reg)
                }
                else
                {
                    IRInstruction::SER_IN(reg)
                }
            }
            else
            {
                return Err(error!("Error no instruction {:#04x}!", ins));
            }
            )

        }

    }

    pub fn bytes_to_ins(fetch: impl FnMut() -> Result<u8, Error>) -> Result<IRInstruction, Error>
    { bytes_to_repr::bytes_to_ins(fetch) }

}
