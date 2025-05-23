#![allow(unused_parens)]
use crossterm::event::{self, Event, KeyCode, KeyModifiers}; 
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};

use std::io::{stdout, Write};
use std::ops::Div;
use std::time::Duration;
use std::collections::HashMap;

use _instruction_conversion::bytes_to_ins;
use erebos::error;
use erebos::instructions::{*, helpers::*};
use crate::fs::FS;
use crate::ray::RAY;



const RAM_PAGE_SIZE : usize = 0x10000_usize;
const RAM_PAGE_COUNT: usize = (0x100000000_usize.div_ceil(RAM_PAGE_SIZE));

#[allow(non_camel_case_types)]
pub struct RAM
{
    list: [Option<usize>; RAM_PAGE_COUNT],
    pages: Vec<[u8; RAM_PAGE_SIZE]>,
}
impl Default for RAM
{
    fn default() -> Self { RAM::new() }
}
impl RAM
{

    pub fn new() -> Self
    {
        Self
        {
            list: [None; RAM_PAGE_COUNT],
            pages: Vec::new(),
        }
    }

    fn split_index(i: u32) -> (usize, usize)
    {
        let i = i as usize;
        let in_page_index = i % RAM_PAGE_SIZE;
        let page_index = (i - in_page_index).div(RAM_PAGE_SIZE);
        (page_index, in_page_index)
    }
    fn get_page(&mut self, i: usize) -> &mut [u8; RAM_PAGE_SIZE]
    {
        if(self.list[i].is_none())
        {
            self.pages.push( [0; RAM_PAGE_SIZE] );
            self.list[i] = Some( self.pages.len() - 1 );
        }
        &mut self.pages[self.list[i].unwrap()]
    }
    fn get_page_safe(&self, i: usize) -> Option<&[u8; RAM_PAGE_SIZE]>
    {
        Some(&self.pages[self.list[i]?])
    }

    pub fn get_safe(&self, i: u32) -> u8
    {

        let (page_index, i) = Self::split_index(i);

        let page = self.get_page_safe(page_index);

        if let Some(page) = page { page[i] }
        else { 0 }

    }
    pub fn get(&mut self, i: u32) -> u8
    {

        let (page_index, i) = Self::split_index(i);

        let page = self.get_page(page_index);

        page[i]

    }
    pub fn set(&mut self, i: u32, v: u8)
    {
        let _i = i;

        let (page_index, i) = Self::split_index(i);

        let page = self.get_page(page_index);

        page[i] = v;

    }

}

#[repr(u8)]
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum InterruptID
{
    None = 0x00,
    UserModeViolation = 0x01,
    Syscall = 0x02,
#[allow(non_camel_case_types)] __Err_Highest,
}
impl From<InterruptID> for u8
{
    fn from(value: InterruptID) -> Self
    {
        unsafe { std::mem::transmute(value) }
    }
}
impl TryFrom<u8> for InterruptID
{
    type Error = Error;
    fn try_from(value: u8) -> Result<Self, Error>
    {
        if(value >= (InterruptID::__Err_Highest as u8)) { return Err(error!("Cannot cast {:#x} to Interrupt ID because it is too big!", value)) }
        unsafe { Ok(std::mem::transmute::<u8, InterruptID>(value)) }
    }
}

#[allow(dead_code)]
pub struct MemoryMap
{
    id: u32,
    map: HashMap<u32, u32>,
}
impl MemoryMap
{
    pub fn from(id: u32, adr: u32, size: u32, dst: u32) -> Self
    {
        let mut map: HashMap<u32, u32> = HashMap::new();
        for i in 0..size
        {
            map.insert(adr + i, dst + i);
        }
        Self
        {
            id,
            map,
        }
    }
}

pub struct Interrupt
{
    pub id: InterruptID,
    pub state: InterruptState,
}
impl Interrupt
{
    pub fn new(id: InterruptID, state: InterruptState) -> Self
    {
        Self
        {
            id,
            state,
        }
    }
}
#[derive(Debug, Clone)]
pub struct InterruptState
{
    pub registers: [u32; 13],
    pub flags: u8,
    pub instruction_pointer: u32,
    pub stack_pointer: u32,
    pub user_mode: bool,
    pub  sub_mode: bool,
}

pub struct VM
{

    /// only 13 spaces bc last 3 registers arent stored here
    pub registers: [u32; 13],
    pub memory: RAM,

    code_section: Vec<u8>,
    section_mode: bool,

    user_mode: bool,
     sub_mode: bool,

    running: bool,
    pub flags: u8,
    
    pub instruction_pointer: u32,
    pub stack_pointer: u32,

    stack_position: u32,
    stack_size: u32,

    debug_print: bool,

    io_device: u16,
    fs: FS,

    interrupt: Option<Interrupt>,
    interrupt_location: u32,

    memory_maps: Vec<MemoryMap>,
    memory_mapping_suspended: bool,

    ray: RAY,

}
impl Default for VM
{
    fn default() -> Self { VM::new() }
}
impl VM
{

    pub fn new() -> Self
    {

        Self
        {
            
            registers: [0; 13],
            memory: RAM::new(),
            
            code_section: Vec::new(),
            section_mode: false,

            user_mode: false,
             sub_mode: false,
            
            instruction_pointer: 0,
            stack_pointer: 0x8000,
            stack_position: 0x8000,
            stack_size: 0x3000,
            
            running: false,
            flags: 0,
            
            debug_print: false,

            io_device: 0,
            fs: FS::new(),

            interrupt: None,
            interrupt_location: 0,

            memory_maps: Vec::new(),
            memory_mapping_suspended: false,

            ray: RAY::new(),

        }

    }

    pub fn mem_map(&self, adr: u32) -> u32
    {
        if(!self.memory_mapping_suspended)
        {
            for m in &self.memory_maps
            {
                if(m.map.contains_key(&adr))
                {
                    return *m.map.get(&adr).unwrap();
                }
            }
        }
        adr
    }

    pub fn get_reg(&self, reg:IRRegister) -> u32
    {
        match reg
        {
            IRRegister::RZ  => 0, 
            IRRegister::RIP => self.instruction_pointer, 
            IRRegister::RSP => self.stack_pointer, 
            _ => self.registers[reg as usize],
        }
    }
    pub fn set_reg(&mut self, val: u32, reg:IRRegister)
    {
        match reg
        {
            IRRegister::RZ  => {},
            IRRegister::RIP => self.instruction_pointer = val, 
            IRRegister::RSP => self.stack_pointer       = val, 
            _ => self.registers[reg as usize] = val,
        };
    }

    fn fetch_byte(&mut self) -> Result<u8, Error>
    {
        let b = if(self.section_mode)
        {
            if(self.instruction_pointer as usize + 1 >= self.code_section.len())
            {
                return Err(error!("Codeoverflow!"));
            }
            
            self.code_section[self.instruction_pointer as usize]
        }
        else
        {
            self.memory.get(self.instruction_pointer)
        };
        let p = self.instruction_pointer.overflowing_add(1);
        if(p.1)
        {
            return Err(error!("Overflowing rip!"));
        }
        self.instruction_pointer = p.0;
        Ok(b)
    }

    fn stack_push(&mut self, v: u8) -> Result<(), Error>
    {
        if(self.stack_pointer as u64 >= 0xFFFF || self.stack_pointer >= self.stack_position + self.stack_size)
        {
            Err(error!("Stackoverflow!"))
        }
        else
        {
            self.memory.set(self.stack_pointer, v);
            self.stack_pointer += 1;
            Ok(())
        }
    }
    fn stack_push16(&mut self, v: u16) -> Result<(), Error>
    {
        if(self.stack_pointer >= 0xFFFE || self.stack_pointer + 1 >= self.stack_position + self.stack_size)
        {
            Err(error!("Stackoverflow!"))
        }
        else
        {
            let v = u16_2_u8(v);
            self.memset(self.stack_pointer, v.0)?;
            self.stack_pointer += 1;
            self.memset(self.stack_pointer, v.1)?;
            self.stack_pointer += 1;
            Ok(())
        }
    }
    fn stack_push32(&mut self, v: u32) -> Result<(), Error>
    {
        if(self.stack_pointer >= 0xFFFC || self.stack_pointer + 3 >= self.stack_position + self.stack_size)
        {
            Err(error!("Stackoverflow!"))
        }
        else
        {
            let v = u32_2_u8(v);
            self.memset(self.stack_pointer, v.0)?;
            self.stack_pointer += 1;
            self.memset(self.stack_pointer, v.1)?;
            self.stack_pointer += 1;
            self.memset(self.stack_pointer, v.2)?;
            self.stack_pointer += 1;
            self.memset(self.stack_pointer, v.3)?;
            self.stack_pointer += 1;
            Ok(())
        }
    }
    fn stack_pop(&mut self) -> Result<u8, Error>
    {
        if(self.stack_pointer <= self.stack_position)
        {
            Err(error!("Stackunderflow!"))
        }
        else
        {
            self.stack_pointer -= 1;
            Ok(self.memory.get(self.stack_pointer))
        }
    }
    fn stack_pop16(&mut self) -> Result<u16, Error>
    {
        if(self.stack_pointer - 1 <= self.stack_position)
        {
            Err(error!("Stackunderflow!"))
        }
        else
        {
            self.stack_pointer -= 1;
            let a = self.memory.get(self.stack_pointer);
            self.stack_pointer -= 1;
            let b = self.memory.get(self.stack_pointer);
            Ok(u8_2_u16((b, a)))
        }
    }
    fn stack_pop32(&mut self) -> Result<u32, Error>
    {
        if(self.stack_pointer - 3 <= self.stack_position)
        {
            Err(error!("Stackunderflow!"))
        }
        else
        {
            self.stack_pointer -= 1;
            let a = self.memory.get(self.stack_pointer);
            self.stack_pointer -= 1;
            let b = self.memory.get(self.stack_pointer);
            self.stack_pointer -= 1;
            let c = self.memory.get(self.stack_pointer);
            self.stack_pointer -= 1;
            let d = self.memory.get(self.stack_pointer);
            Ok(u8_2_u32((d, c, b, a)))
        }
    }

    pub fn memset(&mut self, adr: u32, v: u8) -> Result<(), Error>
    {
        self.memory.set(self.mem_map(adr), v);
        Ok(())
    }
    fn memset16(&mut self, adr: u32, v: u16) -> Result<(), Error>
    {
        let adr = self.mem_map(adr);
        if(adr as u64 >= 0xFFFF)
        {
            return Err(error!("Cannot memset outside of ram range!"))
        }
        let v = u16_2_u8(v);
        self.memory.set(adr    , v.0);
        self.memory.set(adr + 1, v.1);
        Ok(())
    }
    fn memset32(&mut self, adr: u32, v: u32) -> Result<(), Error>
    {
        let adr = self.mem_map(adr);
        if(adr as u64 >= 0xFFFF)
        {
            println!("{adr:#010x}");
            return Err(error!("Cannot memset outside of ram range!"))
        }
        let v = u32_2_u16(v);
        self.memset16(adr    , v.0)?;
        self.memset16(adr + 2, v.1)?;
        Ok(())
    }
    pub fn memget(&mut self, adr: u32) -> Result<u8, Error>
    { Ok(self.memory.get(self.mem_map(adr))) }
    fn memget16(&mut self, adr: u32) -> Result<u16, Error>
    { 
        let adr = self.mem_map(adr);
        if(adr as u64 >= 0xFFFF)
        {
            return Err(error!("Cannot memget outside of ram range!"))
        }
        let a = self.memory.get(adr    );
        let b = self.memory.get(adr + 1);
        Ok(u8_2_u16((a, b)))
    }
    fn memget32(&mut self, adr: u32) -> Result<u32, Error>
    { 
        let adr = self.mem_map(adr);
        if(adr as u64 >= 0xFFFF)
        {
            return Err(error!("Cannot memget outside of ram range!"))
        }
        let a = self.memget16(adr    )?;
        let b = self.memget16(adr + 2)?;
        Ok(u16_2_u32((a, b)))
    }
    fn memget32_safe(&self, adr: u32) -> Result<u32, Error>
    { 
        let adr = self.mem_map(adr);
        if(adr as u64 >= 0xFFFF)
        {
            return Err(error!("Cannot memget outside of ram range!"))
        }
        let a = self.memory.get_safe(adr    );
        let b = self.memory.get_safe(adr + 1);
        let c = self.memory.get_safe(adr + 2);
        let d = self.memory.get_safe(adr + 3);
        Ok(u8_2_u32((a, b, c, d)))
    }

    fn set_flag(&mut self, flag: u8, status: bool) -> Result<(), Error>
    {
        if(status)
        {
            self.flags |= flag;
        }
        else
        {
            self.flags = !((!self.flags) | flag);
        }
        Ok(())
    }

    fn __execute_alu_instruction2(&mut self, ins: _IRALUInstruction2) -> Result<(), Error>
    {

        let m = match &ins
        {
            _IRALUInstruction2::NOT(m) => m,
            _IRALUInstruction2::CMP(m) => m,
        };

        let left = match &m.0
        {
            IRInstructionModifier::Register (r) => self.get_reg  (*r),
            IRInstructionModifier::Memory   (a) => self.memget32 (*a)?,
            _ => return Err(error!("INVALID ALU2 ARGUMENT {:?}", m)),
        };
        
        let value = match ins
        {
            _IRALUInstruction2::NOT(_) => !left,
            _IRALUInstruction2::CMP(_) =>
            {

                let right = match &m.1
                {
                    IRInstructionModifier::Register (r) => self.get_reg  (*r),
                    IRInstructionModifier::Memory   (a) => self.memget32 (*a)?,
                    _ => return Err(error!("INVALID CMP ARGUMENT {:?}", m)),
                };

                self.set_flag(FLAG_E, false)?;
                self.set_flag(FLAG_A, false)?;
                self.set_flag(FLAG_B, false)?;

                match left.cmp(&right)
                {
                    std::cmp::Ordering::Equal   => self.set_flag(FLAG_E, true)?,
                    std::cmp::Ordering::Less    => self.set_flag(FLAG_B, true)?,
                    std::cmp::Ordering::Greater => self.set_flag(FLAG_A, true)?,
                }

                return Ok(());

            },
        };

        match &m.1
        {
            IRInstructionModifier::Register (r) => self.set_reg  (value, *r),
            IRInstructionModifier::Memory   (a) => self.memset32 (*a, value)?,
            _ => return Err(error!("INVALID ALU2 ARGUMENT {:?}", m)),
        };

        Ok(())

    }
    fn __execute_alu_instruction3(&mut self, ins: _IRALUInstruction3) -> Result<(), Error>
    {

        let m = match &ins
        {
            _IRALUInstruction3:: ADD(m) => m,
            _IRALUInstruction3:: SUB(m) => m,
            _IRALUInstruction3:: MUL(m) => m,
            _IRALUInstruction3:: DIV(m) => m,
            _IRALUInstruction3:: MOD(m) => m,
            _IRALUInstruction3:: AND(m) => m,
            _IRALUInstruction3::  OR(m) => m,
            _IRALUInstruction3:: XOR(m) => m,
            _IRALUInstruction3:: SHL(m) => m,
            _IRALUInstruction3:: SHR(m) => m,
            _IRALUInstruction3::NAND(m) => m,
            _IRALUInstruction3:: NOR(m) => m,
        };

        let r = match &m
        {
            Some(m) => match &m.1
                {
                    IRInstructionModifier::Register (r) => self.get_reg  (*r),
                    IRInstructionModifier::Memory   (a) => self.memget32 (*a)?,
                    _ => return Err(error!("INVALID ALU3 ARGUMENT {:?}", m.1)),
                },
            None =>
            {
                self.stack_pop32()?
            },
        };

        let l = match &m
        {
            Some(m) => match &m.0
                {
                    IRInstructionModifier::Register (r) => self.get_reg  (*r),
                    IRInstructionModifier::Memory   (a) => self.memget32 (*a)?,
                    _ => return Err(error!("INVALID ALU3 ARGUMENT {:?}", m.0)),
                },
            None =>
            {
                self.stack_pop32()?
            },
        };

        let v = match &ins
        {
            _IRALUInstruction3:: ADD(_) => l.overflowing_add(r).0,
            _IRALUInstruction3:: SUB(_) => l.overflowing_sub(r).0,
            _IRALUInstruction3:: MUL(_) => l.overflowing_mul(r).0,
            _IRALUInstruction3:: DIV(_) => l.checked_div(r).unwrap_or(0),
            _IRALUInstruction3:: MOD(_) => if(r == 0) { 0 } else { l % r },
            _IRALUInstruction3:: AND(_) => l & r,
            _IRALUInstruction3::  OR(_) => l | r,
            _IRALUInstruction3:: XOR(_) => l ^ r,
            _IRALUInstruction3:: SHL(_) => l.overflowing_shl(r).0,
            _IRALUInstruction3:: SHR(_) => l.overflowing_shr(r).0,
            _IRALUInstruction3::NAND(_) => !(l & r),
            _IRALUInstruction3:: NOR(_) => !(l | r),
        };

        match &m
        {
            Some(m) => match &m.2
                {
                    IRInstructionModifier::Register (r) => self.set_reg  (v, *r),
                    IRInstructionModifier::Memory   (a) => self.memset32 (*a, v)?,
                    _ => return Err(error!("INVALID ALU3 ARGUMENT {:?}", m.2)),
                },
            None =>
            {
                self.stack_push32(v)?
            },
        };

        Ok(())

    }
    fn execute_alu_instruction(&mut self, ins: IRALUInstruction) -> Result<(), Error>
    {
        match ins
        {
            IRALUInstruction::Simple (i) => self.__execute_alu_instruction2(i),
            IRALUInstruction::Complex(i) => self.__execute_alu_instruction3(i),
        }
    }

    fn execute_instruction(&mut self, ins: IRInstruction) -> Result<(), Error>
    {

        match ins
        {
            IRInstruction::NOP | IRInstruction::DATA(_) => {},
            IRInstruction::HLT => if(self.validate_kernel_mode(true)?) { self.running = false },
            IRInstruction::CLF => self.flags = 0,
            IRInstruction::PSHFLG => self.stack_push(self.flags)?,
            IRInstruction::POPFLG => self.flags = self.stack_pop()?,
            IRInstruction::INT(i) => self.send_interrupt((i as u8).try_into()?)?,

            IRInstruction::LEA(r) =>
            {
                let v = self.get_reg(r);
                let v = self.mem_map(v);
                self.set_reg(v, r);
            },

            IRInstruction::INC(r) =>
            {
                let v = self.get_reg(r);
                let v = v.overflowing_add(1).0;
                self.set_reg(v, r);
            },
            IRInstruction::DEC(r) =>
            {
                let v = self.get_reg(r);
                let v = v.overflowing_sub(1).0;
                self.set_reg(v, r);
            },

            IRInstruction::DBG => 
            {
                let mut msg = String::new();
                loop
                {
                    let b = self.fetch_byte()?;
                    if(b == 0) { break; }
                    msg.push(b as char);
                }
                print!("{msg}");
                if(false)
                {
                    let ptr = self.get_reg(IRRegister::RA);
                    for i in 0..10
                    {
                        print!("{:#04x} ", self.memget(ptr + i).unwrap());
                    }
                    println!();
                }
                if(false)
                {
                    println!("{:#x}", self.stack_pointer);
                    for i in 0..((self.stack_pointer - self.stack_position)/2)
                    {
                        print!("{:#x}  ", self.memget16(self.stack_position + i*2)?);
                    }
                    println!();
                }
                else
                {
                    println!("Register dump:");
                    for r in self.registers
                    {
                        print!("{:#06x} ", r);
                    }
                    print!("{:#06x} ", self.instruction_pointer);
                    print!("{:#06x} ", self.      stack_pointer);
                    println!();
                }
            }, 
            
            IRInstruction::SER_OUT(r) => 
            {

                if(self.validate_kernel_mode(false)?)
                {
                    
                    let c = self.get_reg(r) as u8 as char;

                    print!("{c}");
                    if let Err(e) = stdout().flush()
                    {
                        return Err(Error::fromio(e));
                    }

                }

            },
            IRInstruction::SER_IN (r) => 
            {

                if(self.validate_kernel_mode(false)?)
                {

                    enable_raw_mode().unwrap();

                    while let Ok(true) = event::poll(Duration::from_millis(1)) 
                    {
                        _ = event::read();
                    }

                    let c = loop 
                    {
                        if let Event::Key(k) = event::read().unwrap()
                        {
                            let ctrl = matches!(k.modifiers, KeyModifiers::CONTROL);
                            match k.code
                            {
                                KeyCode::Enter => break '\n',
                                KeyCode::Backspace => break '\x08',
                                KeyCode::Char('c') =>
                                {
                                    if(ctrl)
                                    {
                                        self.running = false;
                                        return Ok(());
                                    }
                                },
                                KeyCode::Esc =>
                                {
                                    //self.running = false;
                                    //return Ok(());
                                },
                                _ => {},
                            }
                            let Some(c) = k.code.as_char()
                            else { continue; };
                            break c;
                        }
                    } as u8;

                    while let Ok(true) = event::poll(Duration::from_millis(1)) 
                    {
                        _ = event::read();
                    }

                    disable_raw_mode().unwrap();

                    self.set_reg(c as u32, r);

                }

            },
            IRInstruction::SER_IO(i) =>  if(self.validate_kernel_mode(false)?) { self._io_execute_instruction(i)? } ,

            IRInstruction::MOV(w, m) => 
            {

                let value = match m.0
                {
                    IRInstructionModifier::Register(r) => self.get_reg(r),
                    IRInstructionModifier::Memory(a) =>
                    {
                        match w 
                        {
                            IRInstructionWidth::B32 => self.memget32 (a)?,
                            IRInstructionWidth::B16 => self.memget16 (a)? as u32,
                            IRInstructionWidth::B8  => self.memget   (a)? as u32,
                        }
                    },
                    IRInstructionModifier::RegisterAddress(r) => 
                    {
                        let v = self.get_reg(r);
                        match w 
                        {
                            IRInstructionWidth::B32 => self.memget32 (v)?,
                            IRInstructionWidth::B16 => self.memget16 (v)? as u32,
                            IRInstructionWidth::B8  => self.memget   (v)? as u32,
                        }
                    },
                    IRInstructionModifier::MemoryAddress(a) =>
                    {
                        let v = match w 
                        {
                            IRInstructionWidth::B32 => self.memget32 (a)?,
                            IRInstructionWidth::B16 => self.memget16 (a)? as u32,
                            IRInstructionWidth::B8  => self.memget   (a)? as u32,
                        };
                        match w 
                        {
                            IRInstructionWidth::B32 => self.memget32 (v)?,
                            IRInstructionWidth::B16 => self.memget16 (v)? as u32,
                            IRInstructionWidth::B8  => self.memget   (v)? as u32,
                        }
                    },
                    IRInstructionModifier::Immediate(i) => i,
                    //_ => return Err(Error::new(ErrorKind::Other, format!("INVALID MOV ARGUMENT {:?}", m.0))),
                };

                match m.1
                {
                    IRInstructionModifier::Register(r) => self.set_reg(value, r),
                    IRInstructionModifier::Memory(a) =>
                    {
                        match w 
                        {
                            IRInstructionWidth::B32 => self.memset32 (a, (value         )       )?,
                            IRInstructionWidth::B16 => self.memset16 (a, (value & 0xFFFF) as u16)?,
                            IRInstructionWidth::B8  => self.memset   (a, (value & 0x00FF) as u8 )?,
                        }
                    },
                    IRInstructionModifier::RegisterAddress(r) => 
                    {
                        let v = self.get_reg(r);
                        match w 
                        {
                            IRInstructionWidth::B32 => self.memset32 (v, (value         )       )?,
                            IRInstructionWidth::B16 => self.memset16 (v, (value & 0xFFFF) as u16)?,
                            IRInstructionWidth::B8  => self.memset   (v, (value & 0x00FF) as u8 )?,
                        }
                    },
                    IRInstructionModifier::MemoryAddress(a) =>
                    {
                        let v = match w 
                        {
                            IRInstructionWidth::B32 => self.memget32 (a)?,
                            IRInstructionWidth::B16 => self.memget16 (a)? as u32,
                            IRInstructionWidth::B8  => self.memget   (a)? as u32,
                        };
                        match w 
                        {
                            IRInstructionWidth::B32 => self.memset32 (v, (value         )       )?,
                            IRInstructionWidth::B16 => self.memset16 (v, (value & 0xFFFF) as u16)?,
                            IRInstructionWidth::B8  => self.memset   (v, (value & 0x00FF) as u8 )?,
                        }
                    },
                    _ => return Err(error!("INVALID MOV ARGUMENT {:?}", m.1)),
                };

            },

            IRInstruction::PSH(w, m) => 
            {

                let value = match m
                {
                    IRInstructionModifier::Register(r) => self.get_reg(r),
                    IRInstructionModifier::Memory(a) =>
                    {
                        match w 
                        {
                            IRInstructionWidth::B32 => self.memget32 (a)?,
                            IRInstructionWidth::B16 => self.memget16 (a)? as u32,
                            IRInstructionWidth::B8  => self.memget   (a)? as u32,
                        }
                    },
                    IRInstructionModifier::Immediate(i) => i,
                    _ => return Err(error!("INVALID PSH ARGUMENT {:?}", m)),
                };
                
                match w 
                {
                    IRInstructionWidth::B32 => self.stack_push32 ((value         ) as u32)?,
                    IRInstructionWidth::B16 => self.stack_push16 ((value & 0xFFFF) as u16)?,
                    IRInstructionWidth::B8  => self.stack_push   ((value & 0x00FF) as u8 )?,
                }

            },
            IRInstruction::POP(w, m) =>
            {
                
                let value = match w
                {
                    IRInstructionWidth::B32 => self.stack_pop32 ()?,
                    IRInstructionWidth::B16 => self.stack_pop16 ()? as u32,
                    IRInstructionWidth::B8  => self.stack_pop   ()? as u32,
                };

                match m
                {
                    IRInstructionModifier::Register(r) => self.set_reg(value, r),
                    IRInstructionModifier::Memory(a) =>
                    {
                        match w 
                        {
                            IRInstructionWidth::B32 => self.memset32 (a,  value                 )?,
                            IRInstructionWidth::B16 => self.memset16 (a, (value & 0xFFFF) as u16)?,
                            IRInstructionWidth::B8  => self.memset   (a, (value & 0x00FF) as u8 )?,
                        }
                    },
                    _ => return Err(error!("INVALID POP ARGUMENT {:?}", m)),
                };

            },

            IRInstruction::JMP(m) => 
            {
                self.instruction_pointer = self.mem_map(match m
                {
                    IRInstructionModifier::Register(r) => self.get_reg(r),
                    IRInstructionModifier::Memory(a) => self.memget32_safe(a)?,
                    IRInstructionModifier::Immediate(i) => i,
                    _ => return Err(error!("INVALID JMP ARGUMENT {:?}", m)),
                });
            },
            IRInstruction::JIF(m, f) => 
            {
                if((self.flags & f) != 0)
                {
                    self.instruction_pointer = self.mem_map(match m
                    {
                        IRInstructionModifier::Register(r) => self.get_reg(r),
                        IRInstructionModifier::Memory(a) => self.memget32_safe(a)?,
                        IRInstructionModifier::Immediate(i) => i,
                        _ => return Err(error!("INVALID JIF ARGUMENT {:?}", m)),
                    });
                }
            },
            IRInstruction::CAL(m) =>
            {
                self.stack_push32(self.instruction_pointer)?;
                self.instruction_pointer = self.mem_map(match m
                {
                    IRInstructionModifier::Register(r) => self.get_reg(r),
                    IRInstructionModifier::Memory(a)   => self.memget32_safe(a)?,
                    IRInstructionModifier::Immediate(i) => i,
                    _ => return Err(error!("INVALID CAL ARGUMENT {:?}", m)),
                });
            },
            IRInstruction::RET => self.instruction_pointer = self.stack_pop32()?,

            IRInstruction::ALU(ins) => self.execute_alu_instruction(ins)?,
        };

        Ok(())

    }

    fn execute_next_instruction(&mut self) -> Result<(), Error>
    {

        if(self.debug_print) { print!("[{:#010x}]", self.instruction_pointer); }

        let ins = bytes_to_ins(|| self.fetch_byte())?;

        if(self.debug_print) { println!(" Executing {:?}", ins); }

        self.execute_instruction(ins)

    }

    pub fn run(&mut self) -> Result<(), Error>
    {

        self.running = true;

        while(self.running)
        {
            self.execute_next_instruction()?
        }

        Ok(())

    }



    pub fn load(&mut self, data: Vec<u8>, pos: u32) -> Result<(), Error>
    {

        if(self.section_mode)
        {

            if((data.len() + (pos as usize)) > 0x10000)
            {
                return Err(error!("Cannot load data at addresses higher that {:#x}! (attempting to load {:#x} bytes with offset {:#x})", 0x10000, data.len(), pos));
            }
    
            self.code_section = data;
    
        }
        else
        {

            if((data.len() + (pos as usize)) > 0x10000)
            {
                return Err(error!("Loading {} into ram at pos {:#x} overflows ram!", data.len(), pos));
            }

            for (i, item) in data.iter().enumerate()
            { self.memset(pos + (i as u32), *item)?; }

        }

        Ok(())

    }

    pub fn load_executable(&mut self, exe: Vec<u8>) -> Result<(), Error>
    {

        if(exe.len() < 32)
        {
            return Err(error!("Executable cannot have less than 32 bytes!"));
        }

        let mut data: Vec<u8> = Vec::new();
        let mut header: [u8; 32] = [0; 32];
        
        for i in 0..exe.len()
        {
            if(i < 32)
            {
                header[i] = exe[i];
            }
            else
            {
                data.push(exe[i]);
            }
        }

        let header = IRBinaryHeader::deserialize(header);

        match header
        {
            IRBinaryHeader::V_0000(v0000) =>
            {
                self.load(data, 0)?;
                self.stack_position = v0000.stack_adr;
                self.stack_pointer  = v0000.stack_adr;
                self.stack_size     = v0000.stack_size;
                self.instruction_pointer = v0000.entry_point;
                //TODO: smth with the flags lol
            },
        }

        if(self.stack_position < 0x100)
        {
            self.stack_position = 0x100;
            self.stack_pointer  = 0x100;
        }

        if(self.stack_size < 512)
        {
            self.stack_size = 512;
        }
        
        Ok(())

    }

    pub fn enable_debug_print (&mut self) { self.debug_print  = true; }
    pub fn enable_section_mode(&mut self) { self.section_mode = true; }





    fn _io_execute_instruction(&mut self, ins: u32) -> Result<(), Error>
    {
        
        if(ins >= 0xF0)
        {
            match ins
            {

                0xF0 =>
                {
                    self.io_device = self.get_reg(IRRegister::RA) as u16;
                    Ok(())
                },
                0xF1 => self._io_execute_instruction_rl( self.get_reg(IRRegister::RA) ),
                
                _ => unreachable!(),

            }
        }
        else
        {        
            match self.io_device
            {
                0x0000 => self._io_execute_instruction_fs(ins),
                0x0001 => self._io_execute_instruction_ih(ins),
                0x0002 => self._io_execute_instruction_mm(ins),
                _ => unreachable!(),
            }
        }

    }
    fn _io_execute_instruction_fs(&mut self, ins: u32) -> Result<(), Error>
    {

        match ins
        {

            0x00 => // Reindex()
            {
                self.fs.Reindex()?;
            },
            0x01 => // GetFiles()
            {
                self.set_reg(self.fs.GetFiles()? as u32, IRRegister::RA);
            },
            0x02 => // CreateFile()
            {
                
                let mut name_ptr = self.get_reg(IRRegister::RA);
                let mut name = String::new();

                loop
                {
                    let c = self.memget(name_ptr)?;
                    name_ptr += 1;
                    if(c == 0)
                    {
                        break;
                    }
                    name.push(c as char);
                }

                let result = self.fs.CreateFile(name)? as u32;
                self.set_reg(result, IRRegister::RB);

            },
            0x03 => // DeleteFile()
            {
                let index = self.get_reg(IRRegister::RA) as u8;
                let result = self.fs.DeleteFile(index)? as u32;
                self.set_reg(result, IRRegister::RB);
            },
            0x04 => // FileExists()
            {
                
                let mut name_ptr = self.get_reg(IRRegister::RA);
                let mut name = String::new();

                loop
                {
                    let c = self.memget(name_ptr)?;
                    name_ptr += 1;
                    if(c == 0)
                    {
                        break;
                    }
                    name.push(c as char);
                }

                let result = self.fs.FileExists(name)? as u32;
                self.set_reg(result, IRRegister::RB);

            }
            0x05 => // GetSupDir()
            {
                
                let mut name_ptr = self.get_reg(IRRegister::RA);
                let mut name = String::new();
                
                let dts_ptr = self.get_reg(IRRegister::RB);

                loop
                {
                    let c = self.memget(name_ptr)?;
                    name_ptr += 1;
                    if(c == 0)
                    {
                        break;
                    }
                    name.push(c as char);
                }

                let (result, path) = self.fs.GetSupDir(name)?;
                self.set_reg(result as u32, IRRegister::RC);

                if let Some(path) = path
                {
                    let mut ptr = dts_ptr;
                    for c in path.chars()
                    {
                        self.memset(ptr, c as u8)?;
                        ptr += 1;
                    }
                }

            }
            0x0E => // QuickRead()
            {
                
                let mut name_ptr = self.get_reg(IRRegister::RA);
                let mut name = String::new();
                
                let dts_ptr = self.get_reg(IRRegister::RB);

                loop
                {
                    let c = self.memget(name_ptr)?;
                    name_ptr += 1;
                    if(c == 0)
                    {
                        break;
                    }
                    name.push(c as char);
                }

                let (result, bytes) = self.fs.QuickRead(name)?;
                self.set_reg(result as u32, IRRegister::RD);

                if let Some(bytes) = bytes
                {
                    self.set_reg(bytes.len() as u32, IRRegister::RC);
                    let mut ptr = dts_ptr;
                    for b in bytes
                    {
                        self.memset(ptr, b)?;
                        ptr += 1;
                    }
                }

            }
            0x0F => // SetRoot()
            {
                
                let mut path_ptr = self.get_reg(IRRegister::RA);
                let mut path = String::new();

                loop
                {
                    let c = self.memget(path_ptr)?;
                    path_ptr += 1;
                    if(c == 0)
                    {
                        break;
                    }
                    path.push(c as char);
                }

                let result = self.fs.SetRoot(path)?;
                self.set_reg(result as u32, IRRegister::RB);

            }



            0x10 => // GetFileName()
            {

                let index = self.get_reg(IRRegister::RA) as u8;
                let ptr   = self.get_reg(IRRegister::RB);

                let (result, name) = self.fs.GetFileName(index)?;
                self.set_reg(result as u32, IRRegister::RD);

                if let Some(name) = name
                {

                    let mut i: u32 = 0;

                    for c in name.chars()
                    {
                        self.memset(i + ptr, c as u8)?;
                        i += 1;
                    }

                    // i == length of the string
                    self.set_reg(i, IRRegister::RC);

                }

            },
            0x11 => // SetFileName()
            {

                let index = self.get_reg(IRRegister::RA) as u8;

                let mut name_ptr = self.get_reg(IRRegister::RB);
                let mut name = String::new();

                loop
                {
                    let c = self.memget(name_ptr)?;
                    name_ptr += 1;
                    if(c == 0)
                    {
                        break;
                    }
                    name.push(c as char);
                }

                let result = self.fs.SetFileName(index, name)?;
                self.set_reg(result as u32, IRRegister::RC);

            },
            0x12 => // GetFileLength()
            {
                
                let index = self.get_reg(IRRegister::RA) as u8;

                let (result, length) = self.fs.GetFileLength(index)?;
                self.set_reg(result as u32, IRRegister::RC);

                if let Some(length) = length
                {
                    self.set_reg(length, IRRegister::RB);
                }

            },
            0x13 => // SetFileLength()
            {
                
                let index  = self.get_reg(IRRegister::RA) as u8;
                let length = self.get_reg(IRRegister::RB);

                let result = self.fs.SetFileLength(index, length)?;
                self.set_reg(result as u32, IRRegister::RC);

            },



            0x20 => // ReadFile()
            {

                let index = self.get_reg(IRRegister::RA) as u8;
                let ptr   = self.get_reg(IRRegister::RB);

                let (result, buffer) = self.fs.ReadFile(index)?;
                self.set_reg(result as u32, IRRegister::RC);

                let mut i: u32 = 0;

                if let Some(buffer) = buffer
                {

                    for b in buffer
                    {
                        self.memset(i + ptr, b)?;
                        i += 1;
                    }

                }

                self.set_reg(i, IRRegister::RD);

            },
            0x21 => // ReadFileAt()
            {

                let index = self.get_reg(IRRegister::RA) as u8;
                let ptr   = self.get_reg(IRRegister::RB);
                let pos   = self.get_reg(IRRegister::RC);

                let (result, val) = self.fs.ReadFileAt(index, pos)?;
                self.set_reg(result as u32, IRRegister::RD);

                if let Some(val) = val
                {
                    self.memset(ptr, val)?;
                }

            }
            0x22 => // WrtieFile()
            {

                let index = self.get_reg(IRRegister::RA) as u8;
                let len   = self.get_reg(IRRegister::RB);
                let ptr   = self.get_reg(IRRegister::RC);

                let mut buffer: Vec<u8> = Vec::new();
                
                for i in 0..len
                {
                    buffer.push(self.memget(i + ptr)?);
                }

                let result = self.fs.WriteFile(index, buffer)?;
                self.set_reg(result as u32, IRRegister::RD);

            },
            0x23 => // WrtieFileAt()
            {

                let index = self.get_reg(IRRegister::RA) as u8;
                let ptr   = self.get_reg(IRRegister::RB);
                let pos   = self.get_reg(IRRegister::RC);

                let val = self.memget(ptr)?;

                let result = self.fs.WriteFileAt(index, pos, val)?;
                self.set_reg(result as u32, IRRegister::RD);

            },



            _ =>
            {
                return Err(error!("FileSystem[TM]: {:#x} is not a fs function!", ins));
            }

        }
    
        Ok(())

    }
    fn _io_execute_instruction_ih(&mut self, ins: u32) -> Result<(), Error>
    {

        match ins
        {

            0x00 => // GetInterruptID()
            {
                let id: InterruptID = 
                    if let Some(int) = &self.interrupt
                    {
                        int.id
                    }
                    else 
                    {
                        InterruptID::None
                    };
                self.set_reg(id as u8 as u32, IRRegister::RA);
            },
            0x01 => // SetUserMode()
            {
                self.user_mode = true;
            },
            0x02 => // SetInterruptHandlerLocation()
            {
                let loc = self.get_reg(IRRegister::RA);
                self.interrupt_location = loc;
            },
            0x03 => // ResolveInterrupt()
            {
                self.resolve_interrupt()?;
            },
            0x04 => // RemoveInterrupt()
            {

                let i = self.instruction_pointer;
                let s = self.      stack_pointer;

                self.resolve_interrupt()?;

                self.instruction_pointer = i;
                self.      stack_pointer = s;
                self.user_mode = false;
                self. sub_mode = false;

            },
            0x05 => // SetUserMode()
            {
                self.sub_mode = true;
            },     
            0x06 => // ResolveInterruptNoRSP()
            {
                let rsp = self.stack_pointer;
                self.resolve_interrupt()?;
                self.stack_pointer = rsp;
            },       
            _ => unreachable!("{ins}"),

        }

        Ok(())

    }
    fn _io_execute_instruction_mm(&mut self, ins: u32) -> Result<(), Error>
    {
        
        match ins
        {

            0x00 => // SuspendMapping()
            {
                self.memory_mapping_suspended = true;
            },
            0x01 => // ResumeMapping()
            {
                self.memory_mapping_suspended = false;
            },
            0x02 => // SetMap()
            {

                let adr = self.get_reg(IRRegister::RA);
                let len = self.get_reg(IRRegister::RB);
                let dst = self.get_reg(IRRegister::RC);

                let ID = self.memory_maps.len() as u32 + 1;

                self.memory_maps.push(MemoryMap::from(ID, adr, len, dst));

                self.set_reg(ID, IRRegister::RD);

            },
            0x03 => // RmvMap()
            {

                let ID = self.get_reg(IRRegister::RA);

                for (i,m) in self.memory_maps.iter().enumerate()
                {
                    if(m.id == ID)
                    {
                        self.memory_maps.remove(i);
                        break;
                    }
                };

            },
            
            _ => unreachable!(),

        }

        Ok(())

    }
    fn _io_execute_instruction_rl(&mut self, ins: u32) -> Result<(), Error>
    {
        
        match ins
        {

            0x00 => // WindowShouldClose()
            {
                let b = self.ray.WindowShouldClose()?;
                self.stack_push(b)?;
            },
            0x01 => // BeginDrawing()
            {
                self.ray.BeginDrawing()?;
            },
            0x02 => // EndDrawing()
            {
                self.ray.EndDrawing()?;
            },
            
            0x03 => // DrawRectangle()
            {

                let ca = self.stack_pop()?;
                let cb = self.stack_pop()?;
                let cg = self.stack_pop()?;
                let cr = self.stack_pop()?;

                let color = RAY::rgba(cr, cg, cb, ca);

                let h = self.stack_pop32()?;
                let w = self.stack_pop32()?;
                let y = self.stack_pop32()?;
                let x = self.stack_pop32()?;

                self.ray.DrawRectange(x, y, w, h, color)?;
                
            },
            0x04 => // DrawFPS()
            {
                let y = self.stack_pop32()?;
                let x = self.stack_pop32()?;
                self.ray.DrawFPS(x, y)?;
            },
            0x05 => // ClearBackground()
            {

                let ca = self.stack_pop()?;
                let cb = self.stack_pop()?;
                let cg = self.stack_pop()?;
                let cr = self.stack_pop()?;

                let color = RAY::rgba(cr, cg, cb, ca);

                self.ray.ClearBackground(color)?;
                
            },
            0x06 => // DrawText()
            {
                let mut ptr = self.stack_pop32()?;
                let mut text = String::new();

                loop
                {
                    let c = self.memget(ptr)?;
                    if(c == 0)
                    {
                        break;
                    }
                    ptr += 1;
                    text.push(c as char);
                }

                let ca = self.stack_pop()?;
                let cb = self.stack_pop()?;
                let cg = self.stack_pop()?;
                let cr = self.stack_pop()?;

                let color = RAY::rgba(cr, cg, cb, ca);

                let s = self.stack_pop32()?;
                let y = self.stack_pop32()?;
                let x = self.stack_pop32()?;

                self.ray.DrawText(x, y, s, color, text)?;
                
            },

            0xED => // SetTargetFPS()
            {
                let fps = self.stack_pop32()?;
                self.ray.SetTargetFPS(fps)?;
            },
            0xEE => // CloseWindow()
            {
                self.ray.CloseWindow()?;
            },
            0xEF => // OpenWindow()
            {
                
                let r = self.stack_pop()?;
                let mut ptr = self.stack_pop32()?;
                let mut title = String::new();

                loop
                {
                    let c = self.memget(ptr)?;
                    ptr += 1;
                    if(c == 0)
                    {
                        break;
                    }
                    title.push(c as char);
                }
                
                let h = self.stack_pop32()?;
                let w = self.stack_pop32()?;

                self.ray.OpenWindow(w, h, title, r != 0)?;

            },

            0xD0 => // IsWindowResized()
            {
                self.stack_push(self.ray.IsWindowResized()?)?;
            },
            0xD1 => // GetWindowWidth()
            {
                self.stack_push32(self.ray.GetWindowWidth()?)?;
            },
            0xD2 => // GetWindowHeight()
            {
                self.stack_push32(self.ray.GetWindowHeight()?)?;
            },
            
            _ => unreachable!(),

        }

        Ok(())

    }
    

    fn validate_kernel_mode(&mut self, sub_mode_valid: bool) -> Result<bool, Error>
    {
        if(self.user_mode || (self.sub_mode && sub_mode_valid))
        {
            self.send_interrupt(InterruptID::UserModeViolation)?;
            Ok(false)
        }
        else
        {
            Ok(true)
        }
    }

    pub fn send_interrupt(&mut self, id: InterruptID) -> Result<(), Error>
    {

         if(id == InterruptID::None)
         {
            return Err(error!("Cannot send empty Interrupt ID!"));
         }

         let state = self.get_interrupt_state();

         self.user_mode = false;
         self. sub_mode = false;

         self.instruction_pointer = self.interrupt_location;

         self.interrupt = Some(Interrupt::new(id, state));

         Ok(())

    }
    pub fn resolve_interrupt(&mut self) -> Result<(), Error>
    {

        if let Some(int) = &self.interrupt
        {
            self.restore_interrupt_state(int.state.clone());
            Ok(())
        }
        else
        {
            Err(error!("Cannot resolve interrupt; No interrupt present!"))
        }

    }

    fn get_interrupt_state(&self) -> InterruptState
    {
        InterruptState
        {
            registers: self.registers,
            flags: self.flags,
            instruction_pointer: self.instruction_pointer,
            stack_pointer: self.stack_pointer,
            user_mode: self.user_mode,
             sub_mode: self. sub_mode,
        }
    }
    fn restore_interrupt_state(&mut self, state: InterruptState)
    {
        self.registers = state.registers;
        self.flags = state.flags;
        self.instruction_pointer = state.instruction_pointer;
        self.stack_pointer = state.stack_pointer;
        self.user_mode = state.user_mode;
        self. sub_mode = state. sub_mode;
    }

}

