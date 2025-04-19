#![allow(unused_parens)]

use std::{io::{stdout, Write}, time::Duration};
use crossterm::{event::{self, Event, KeyCode}, terminal::{disable_raw_mode, enable_raw_mode}};

use _instruction_conversion::bytes_to_ins;
use squire::instructions::*;
use helpers::*;

use squire::error;

pub struct VM
{

    /// only 13 spaces bc last 3 registers arent stored here
    pub registers: [u16; 13],
    pub memory: [u8; 0x10000],

    code_section: Vec<u8>,
    section_mode: bool,

    running: bool,
    pub flags: u8,
    
    pub instruction_pointer: u16,
    pub stack_pointer: u16,

    stack_position: u16,
    stack_size: u16,

    debug_print: bool,

}
impl VM
{

    pub fn new() -> Self
    {
        Self
        {
            
            registers: [0; 13],
            memory: [0; 0x10000],

            code_section: Vec::new(),
            section_mode: false,

            instruction_pointer: 0,
            stack_pointer: 0x8000,
            stack_position: 0x8000,
            stack_size: 0x3000,

            running: false,
            flags: 0,

            debug_print: false,

        }
    }

    fn get_reg(&self, reg:IRRegister) -> u16
    {
        match reg
        {
            IRRegister::RZ  => 0, 
            IRRegister::RIP => self.instruction_pointer, 
            IRRegister::RSP => self.stack_pointer, 
            _ => self.registers[reg as usize],
        }
    }
    fn set_reg(&mut self, val: u16, reg:IRRegister)
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
            self.memory[self.instruction_pointer as usize]
        };
        self.instruction_pointer += 1;
        Ok(b)
    }

    fn stack_push(&mut self, v: u8) -> Result<(), Error>
    {
        if(self.stack_pointer >= 0xFFFF || self.stack_pointer >= self.stack_position + self.stack_size)
        {
            Err(error!("Stackoverflow!"))
        }
        else
        {
            self.memory[self.stack_pointer as usize] = v;
            self.stack_pointer += 1;
            Ok(())
        }
    }
    fn stack_push16(&mut self, v: u16) -> Result<(), Error>
    {
        if(self.stack_pointer >= 0xFFFE || self.stack_pointer >= self.stack_position + self.stack_size - 1)
        {
            Err(error!("Stackoverflow!"))
        }
        else
        {
            let v = u16_2_u8(v);
            self.memory[self.stack_pointer as usize] = v.0;
            self.stack_pointer += 1;
            self.memory[self.stack_pointer as usize] = v.1;
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
            Ok(self.memory[self.stack_pointer as usize])
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
            let a = self.memory[self.stack_pointer as usize];
            self.stack_pointer -= 1;
            let b = self.memory[self.stack_pointer as usize];
            Ok(u8_2_u16((b, a)))
        }
    }

    fn memset(&mut self, adr: u16, v: u8) -> Result<(), Error>
    {
        self.memory[adr as usize] = v;
        Ok(())
    }
    fn memset16(&mut self, adr: u16, v: u16) -> Result<(), Error>
    {
        if(adr >= 0xFFFF)
        {
            return Err(error!("Cannot memset outside of ram range!"))
        }
        let v = u16_2_u8(v);
        self.memory[ adr      as usize] = v.0;
        self.memory[(adr + 1) as usize] = v.1;
        Ok(())
    }
    fn memget(&self, adr: u16) -> Result<u8, Error>
    { Ok(self.memory[adr as usize]) }
    fn memget16(&self, adr: u16) -> Result<u16, Error>
    { 
        if(adr >= 0xFFFF)
        {
            return Err(error!("Cannot memget outside of ram range!"))
        }
        let a = self.memory[ adr      as usize];
        let b = self.memory[(adr + 1) as usize];
        Ok(u8_2_u16((a, b)))
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
            IRInstructionModifier::Memory   (a) => self.memget16 (*a)?,
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
                    IRInstructionModifier::Memory   (a) => self.memget16 (*a)?,
                    _ => return Err(error!("INVALID CMP ARGUMENT {:?}", m)),
                };

                self.set_flag(FLAG_E, false)?;
                self.set_flag(FLAG_A, false)?;
                self.set_flag(FLAG_B, false)?;

                     if(left == right) { self.set_flag(FLAG_E, true)?; }
                else if(left >  right) { self.set_flag(FLAG_A, true)?; }
                else if(left <  right) { self.set_flag(FLAG_B, true)?; }

                return Ok(());

            },
        };

        match &m.1
        {
            IRInstructionModifier::Register (r) => self.set_reg  (value, *r),
            IRInstructionModifier::Memory   (a) => self.memset16 (*a, value)?,
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
                    IRInstructionModifier::Memory   (a) => self.memget16 (*a)?,
                    _ => return Err(error!("INVALID ALU3 ARGUMENT {:?}", m.1)),
                },
            None =>
            {
                self.stack_pop16()?
            },
        };

        let l = match &m
        {
            Some(m) => match &m.0
                {
                    IRInstructionModifier::Register (r) => self.get_reg  (*r),
                    IRInstructionModifier::Memory   (a) => self.memget16 (*a)?,
                    _ => return Err(error!("INVALID ALU3 ARGUMENT {:?}", m.0)),
                },
            None =>
            {
                self.stack_pop16()?
            },
        };

        let v = match &ins
        {
            _IRALUInstruction3:: ADD(_) => l.overflowing_add(r).0,
            _IRALUInstruction3:: SUB(_) => l.overflowing_sub(r).0,
            _IRALUInstruction3:: MUL(_) => l.overflowing_mul(r).0,
            _IRALUInstruction3:: DIV(_) => match l.checked_div(r) { Some(s) => s, None => 0 },
            _IRALUInstruction3:: MOD(_) => if(r == 0) { 0 } else { l % r },
            _IRALUInstruction3:: AND(_) => l & r,
            _IRALUInstruction3::  OR(_) => l | r,
            _IRALUInstruction3:: XOR(_) => l ^ r,
            _IRALUInstruction3:: SHL(_) => l.overflowing_shl(r.into()).0,
            _IRALUInstruction3:: SHR(_) => l.overflowing_shr(r.into()).0,
            _IRALUInstruction3::NAND(_) => !(l & r),
            _IRALUInstruction3:: NOR(_) => !(l | r),
        };

        match &m
        {
            Some(m) => match &m.2
                {
                    IRInstructionModifier::Register (r) => self.set_reg  (v, *r),
                    IRInstructionModifier::Memory   (a) => self.memset16 (*a, v)?,
                    _ => return Err(error!("INVALID ALU3 ARGUMENT {:?}", m.2)),
                },
            None =>
            {
                self.stack_push16(v)?
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

    fn execute_next_instruction(&mut self) -> Result<(), Error>
    {

        let ins = bytes_to_ins(|| self.fetch_byte())?;

        if(self.debug_print) { println!("Executing {:?}", ins); }

        self.execute_instruction(ins)

    }

    fn execute_instruction(&mut self, ins: IRInstruction) -> Result<(), Error>
    {

        match ins
        {
            IRInstruction::NOP => {},
            IRInstruction::HLT => self.running = false,
            IRInstruction::CLF => self.flags = 0,
            IRInstruction::RAY(_) => {},
            IRInstruction::DBG => 
            {
                if(true)
                {
                    println!("{:#x}", self.stack_pointer);
                    for i in 0..((self.stack_pointer - self.stack_position)/2)
                    {
                        print!("{:#x}  ", self.memget16(self.stack_position + i*2)?);
                    }
                    println!("");
                }
                else
                {
                    println!("Register dump:");
                    for r in self.registers
                    {
                        print!("{:#x} ", r);
                    }
                    println!("");
                }
            }, 
            
            IRInstruction::SER_OUT(r) => 
            {
                
                let c = self.get_reg(r) as u8 as char;

                print!("{c}");
                match stdout().flush()
                {
                    Err(e) => return Err(Error::IO(e)),
                    _ => {}
                };

            },
            IRInstruction::SER_IN (r) => 
            {

                enable_raw_mode().unwrap();

                while let Ok(true) = event::poll(Duration::from_millis(1)) 
                {
                    _ = event::read();
                }

                let c = loop 
                {
                    match event::read().unwrap()
                    {
                        Event::Key(k) =>
                        {
                            match k.code
                            {
                                KeyCode::Enter => break '\n',
                                KeyCode::Backspace => break '\x08',
                                KeyCode::Esc =>
                                {
                                    self.running = false;
                                    return Ok(());
                                },
                                _ => {},
                            }
                            let Some(c) = k.code.as_char()
                            else { continue; };
                            break c;
                        }
                        _ => {}
                    }
                } as u8;

                while let Ok(true) = event::poll(Duration::from_millis(1)) 
                {
                    _ = event::read();
                }

                disable_raw_mode().unwrap();

                self.set_reg(c as u16, r);

            },

            IRInstruction::MOV(w, m) => 
            {

                let value = match m.0
                {
                    IRInstructionModifier::Register(r) => self.get_reg(r),
                    IRInstructionModifier::Memory(a) =>
                    {
                        match w 
                        {
                            IRInstructionWidth::B16 => self.memget16(a)?,
                            IRInstructionWidth::B8  => self.memget(a)? as u16,
                        }
                    },
                    IRInstructionModifier::RegisterAddress(r) => 
                    {
                        let v = self.get_reg(r);
                        match w 
                        {
                            IRInstructionWidth::B16 => self.memget16(v)?,
                            IRInstructionWidth::B8  => self.memget(v)? as u16,
                        }
                    },
                    IRInstructionModifier::MemoryAddress(a) =>
                    {
                        let v = match w 
                        {
                            IRInstructionWidth::B16 => self.memget16(a)?,
                            IRInstructionWidth::B8  => self.memget(a)? as u16,
                        };
                        match w 
                        {
                            IRInstructionWidth::B16 => self.memget16(v)?,
                            IRInstructionWidth::B8  => self.memget(v)? as u16,
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
                            IRInstructionWidth::B16 => self.memset16(a, value)?,
                            IRInstructionWidth::B8  => self.memset(a, (value & 0xFF) as u8)?,
                        }
                    },
                    IRInstructionModifier::RegisterAddress(r) => 
                    {
                        let v = self.get_reg(r);
                        match w 
                        {
                            IRInstructionWidth::B16 => self.memset16(v, value)?,
                            IRInstructionWidth::B8  => self.memset(v, (value & 0xFF) as u8)?,
                        }
                    },
                    IRInstructionModifier::MemoryAddress(a) =>
                    {
                        let v = match w 
                        {
                            IRInstructionWidth::B16 => self.memget16(a)?,
                            IRInstructionWidth::B8  => self.memget(a)? as u16,
                        };
                        match w 
                        {
                            IRInstructionWidth::B16 => self.memset16(v, value)?,
                            IRInstructionWidth::B8  => self.memset(v, (value & 0xFF) as u8)?,
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
                            IRInstructionWidth::B16 => self.memget16(a)?,
                            IRInstructionWidth::B8  => self.memget(a)? as u16,
                        }
                    },
                    IRInstructionModifier::Immediate(i) => i,
                    _ => return Err(error!("INVALID PSH ARGUMENT {:?}", m)),
                };
                
                match w 
                {
                    IRInstructionWidth::B16 => self.stack_push16(value)?,
                    IRInstructionWidth::B8  => self.stack_push((value & 0xFF) as u8)?,
                }

            },
            IRInstruction::POP(w, m) =>
            {
                
                let value: u16 = match w
                {
                    IRInstructionWidth::B16 => self.stack_pop16()?,
                    IRInstructionWidth::B8  => self.stack_pop  ()? as u16,
                };

                match m
                {
                    IRInstructionModifier::Register(r) => self.set_reg(value, r),
                    IRInstructionModifier::Memory(a) =>
                    {
                        match w 
                        {
                            IRInstructionWidth::B16 => self.memset16(a, value)?,
                            IRInstructionWidth::B8  => self.memset(a, (value & 0xFF) as u8)?,
                        }
                    },
                    _ => return Err(error!("INVALID POP ARGUMENT {:?}", m)),
                };

            },

            IRInstruction::JMP(m) => 
            {
                self.instruction_pointer = match m
                {
                    IRInstructionModifier::Register(r) => self.get_reg(r),
                    IRInstructionModifier::Memory(a) => self.memget16(a)?,
                    IRInstructionModifier::Immediate(i) => i,
                    _ => return Err(error!("INVALID JMP ARGUMENT {:?}", m)),
                };
            },
            IRInstruction::JIF(m, f) => 
            {
                if((self.flags & f) != 0)
                {
                    self.instruction_pointer = match m
                    {
                        IRInstructionModifier::Register(r) => self.get_reg(r),
                        IRInstructionModifier::Memory(a) => self.memget16(a)?,
                        IRInstructionModifier::Immediate(i) => i,
                        _ => return Err(error!("INVALID JIF ARGUMENT {:?}", m)),
                    };
                }
            },
            IRInstruction::CAL(m) =>
            {
                self.stack_push16(self.instruction_pointer)?;
                self.instruction_pointer = match m
                {
                    IRInstructionModifier::Register(r) => self.get_reg(r),
                    IRInstructionModifier::Memory(a) => self.memget16(a)?,
                    IRInstructionModifier::Immediate(i) => i,
                    _ => return Err(error!("INVALID CAL ARGUMENT {:?}", m)),
                };
            },
            IRInstruction::RET => self.instruction_pointer = self.stack_pop16()?,

            IRInstruction::ALU(ins) => self.execute_alu_instruction(ins)?,
        };

        Ok(())

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

    pub fn load(&mut self, data: Vec<u8>, pos: u16) -> Result<(), Error>
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

            for i in 0..data.len()
            { self.memset(pos + (i as u16), data[i])?; }

        }

        Ok(())

    }

    pub fn enable_debug_print (&mut self) { self.debug_print  = true; }
    pub fn enable_section_mode(&mut self) { self.section_mode = true; }

}
