#![allow(unused_parens)]

use std::sync::Arc;
use std::io::{BufReader, Read};
use std::fs::File;
use std::collections::HashMap;
use squire::instructions::_instruction_conversion::bytes_to_ins;
use squire::instructions::*;
use squire::error;

pub struct DASM
{
    source: Vec<u8>,
    last: Option<u8>,
    position: usize,
    end_of_file: bool,
    labels: HashMap<u16, String>,
}
impl DASM
{

    pub fn new(src: Arc<str>) -> Result<Self, Error>
    {
        let file = match File::open(src.to_string())
        {
            Ok(f) => f,
            Err(e) => return Err(Error::fromio(e)),
        };
        let mut buffer: Vec<u8> = Vec::new();
        let mut reader = BufReader::new(file);
        if let Err(e) = reader.read_to_end(&mut buffer)
        {
            return Err(Error::fromio(e));
        }
        Ok(Self
        {
            source: buffer,
            last: None,
            position: 0,
            end_of_file: false,
            labels: HashMap::new(),
        })
    }

    fn peek(&mut self) -> Option<&u8>
    {
        if(self.last.is_none())
        {
            if(self.position >= self.source.len())
            {
                self.end_of_file = true;
                return None;
            }
            let c = self.source[self.position];
            self.position += 1;
            self.last = Some(c);
        }
        self.last.as_ref()
    }
    fn next(&mut self) -> Option<u8>
    {
        self.peek();
        self.last.take()
    }

    fn get_label_for(&mut self, adr: u16) -> &String
    {
        if(self.labels.contains_key(&adr))
        {
            self.labels.get(&adr).unwrap()
        }
        else
        {
            let label = format!("_label_{adr:#06x}");
            self.labels.insert(adr, label);
            self.get_label_for(adr)
        }
    }
    fn parse_mod(&mut self, m: IRInstructionModifier, allow_label: bool) -> (String, String)
    {
        let p = match m
        {
            IRInstructionModifier::Register        (r) => ( "r" , DASM::reg_to_str(r) ),
            IRInstructionModifier::RegisterAddress (r) => ( "ra", DASM::reg_to_str(r) ),
            IRInstructionModifier::Memory          (m) => ( "m" , if (allow_label) { self.get_label_for(m).to_string() } else { format!("{m:#06x}") } ),
            IRInstructionModifier::Immediate       (m) => ( "i" , if (allow_label) { self.get_label_for(m).to_string() } else { format!("{m:#06x}") } ),
            IRInstructionModifier::MemoryAddress   (m) => ( "ma", if (allow_label) { self.get_label_for(m).to_string() } else { format!("{m:#06x}") } ),
        };
        (p.0.to_string(), p.1)
    }
    fn parse_mod2(&mut self, m: IRInstructionModifier2, allow_label: bool) -> (String, String, String)
    {
        let a = self.parse_mod(m.0, allow_label);
        let b = self.parse_mod(m.1, allow_label);
        ( format!("{}{}", a.0, b.0), a.1, b.1 )
    }
    fn parse_mod_alu(&mut self, m: IRALUInstructionModifier3) -> (String, String, String, String)
    {
        if let Some(m) = m
        {
            let a = self.parse_mod(m.0, false);
            let b = self.parse_mod(m.1, false);
            let c = self.parse_mod(m.2, false);
            ( format!("{}{}{}", a.0, b.0, c.0), a.1, b.1, c.1 )
        }
        else
        {
            ( "s".to_string(), String::new(), String::new(), String::new() )
        }
    }
    fn parse_width(w: IRInstructionWidth) -> String
    {
        match w
        {
            IRInstructionWidth::B16 => String::new(),
            IRInstructionWidth::B8  => "b".to_string(),
        }
    }
    fn reg_to_str(r: IRRegister) -> String
    {
        match r
        {
                IRRegister::RA  => "ra", 
                IRRegister::RB  => "rb", 
                IRRegister::RC  => "rc", 
                IRRegister::RD  => "rd", 
                IRRegister::R1  => "r1", 
                IRRegister::R2  => "r2", 
                IRRegister::R3  => "r3", 
                IRRegister::R4  => "r4", 
                IRRegister::R5  => "r5", 
                IRRegister::R6  => "r6", 
                IRRegister::R7  => "r7", 
                IRRegister::R8  => "r8", 
                IRRegister::R9  => "r9", 
                IRRegister::RZ  => "rz", 
                IRRegister::RIP => "rip", 
                IRRegister::RSP => "rsp", 

        }.to_string()
    }
    fn _parse_alu2(&mut self, ins: _IRALUInstruction2) -> String
    {
        match ins
        {
            _IRALUInstruction2::NOT(m) =>
            {
                let m = self.parse_mod2(m, true);
                format!("not{} {}, {}", m.0, m.1, m.2)
            },
            _IRALUInstruction2::CMP(m) =>
            {
                let m = self.parse_mod2(m, true);
                format!("not{} {}, {}", m.0, m.1, m.2)
            },
        }
    }
    fn _parse_alu3(&mut self, ins: _IRALUInstruction3) -> String
    {

        let (ins, m) = match ins
        {
             _IRALUInstruction3:: ADD(m) => ( "add", m ),
             _IRALUInstruction3:: SUB(m) => ( "sub", m ),
             _IRALUInstruction3:: MUL(m) => ( "mul", m ),
             _IRALUInstruction3:: DIV(m) => ( "div", m ),
             _IRALUInstruction3:: MOD(m) => ( "mod", m ),
             _IRALUInstruction3:: AND(m) => ( "and", m ),
             _IRALUInstruction3::  OR(m) => (  "or", m ),
             _IRALUInstruction3:: XOR(m) => ( "xor", m ),
             _IRALUInstruction3:: SHL(m) => ( "shl", m ),
             _IRALUInstruction3:: SHR(m) => ( "shr", m ),
             _IRALUInstruction3::NAND(m) => ("nand", m ),
             _IRALUInstruction3:: NOR(m) => ( "nor", m ),
        };

        let m = if m.is_some()
        {
            let m = self.parse_mod_alu(m);
            ( m.0, format!("{}, {}, {}", m.1, m.2, m.3))
        }
        else
        {
            ( "s".to_string(), String::new() )
        };

        format!("{}{} {}", ins, m.0, m.1)
        
    }
    fn _parse_alu(&mut self, ins: IRALUInstruction) -> String
    {
        match ins
        {
            IRALUInstruction::Simple  (i) => self._parse_alu2(i),
            IRALUInstruction::Complex (i) => self._parse_alu3(i),
        }
    }
    fn ir_to_line(&mut self, ir: IRInstruction, loc: u16) -> String
    {
        let ins = match ir
        {
            IRInstruction::NOP => "nop".to_string(),
            IRInstruction::HLT => "hlt".to_string(),
            IRInstruction::CLF => "clf".to_string(),
            IRInstruction::RET => "ret".to_string(),
            IRInstruction::DBG => "dbg".to_string(),
            IRInstruction::SER_OUT(r) => format!("__out {}", DASM::reg_to_str(r)),
            IRInstruction::SER_IN (r) => format!("__in {}", DASM::reg_to_str(r)),
            IRInstruction::INC(r) => format!("inc {}", DASM::reg_to_str(r)),
            IRInstruction::DEC(r) => format!("dec {}", DASM::reg_to_str(r)),
            IRInstruction::SER_IO(imm) => format!("__io {imm:#04x}"),
            IRInstruction::PSHFLG => "psgflg".to_string(),
            IRInstruction::POPFLG => "popflg".to_string(),
            IRInstruction::INT(imm) => format!("int {imm:#06x}"),
            IRInstruction::MOV(w, m2) =>
            {
                let w = DASM::parse_width(w);
                let m = self.parse_mod2(m2, true);
                format!("{}mov{} {}, {}", w, m.0, m.1, m.2)
            },
            IRInstruction::PSH(w, m) =>
            {
                let m = self.parse_mod(m, true);
                let w = DASM::parse_width(w);
                format!("{}psh{} {}", w, m.0, m.1)
            },
            IRInstruction::POP(w, m) =>
            {
                let m = self.parse_mod(m, true);
                let w = DASM::parse_width(w);
                format!("{}pop{} {}", w, m.0, m.1)
            },
            IRInstruction::JMP(m) =>
            {
                let m = self.parse_mod(m, true);
                format!("jmp{} {}", m.0, m.1)
            },
            IRInstruction::JIF(m, f) =>
            {
                let m = self.parse_mod(m, true);
                let mut flags = String::new();
                if(f & FLAG_C != 0) { flags.push('C'); }
                if(f & FLAG_Z != 0) { flags.push('Z'); }
                if(f & FLAG_B != 0) { flags.push('B'); }
                if(f & FLAG_A != 0) { flags.push('A'); }
                if(f & FLAG_E != 0) { flags.push('E'); }
                format!("jif{} {}, {}", m.0, m.1, flags)
            },
            IRInstruction::CAL(m) =>
            {
                let m = self.parse_mod(m, true);
                format!("cal{} {}", m.0, m.1)
            },
            IRInstruction::ALU(ins) => self._parse_alu(ins),
        };
        format!("[{loc:#06x}] {ins}")
    }

    fn get_line(&mut self, loc: u16) -> (Result<String, Error>, Vec<u8>)
    {
        let mut read_bytes: Vec<u8> = Vec::new();
        let ir = match bytes_to_ins(||
            {
                let c = self.next();
                match c
                {
                    Some(c) => 
                    {
                        read_bytes.push(c);
                        Ok(c)
                    },
                    None => Err(Error::from("Expected another byte but EOF reached!")),
                }
            })
        {
            Ok(ir) => ir,
            Err(e) => return ( Err(e), read_bytes ),
        };
        ( Ok(self.ir_to_line(ir, loc)), read_bytes )
    }

    pub fn disassemble(&mut self) -> Result<String, Error>
    {

        if(self.source.len() < 32)
        {
            return Err(error!("Binary doesnt include header!"));
        }

        let mut output = "-----HEADER-----\n".to_string();
        for _ in 0..2
        {
            for _ in 0..16
            {
                let c = self.next().unwrap();
                output.push_str(format!("{c:#04x} ").as_str());
            }
            output.push('\n');
        }

        output.push_str("-----DATA-----\n");

        let mut lines: Vec<(String, u16)> = Vec::new();

        loop
        {
            let location = self.position as u16 - 32;
            match self.get_line(location)
            {
                (Ok(s),_) => lines.push(( format!("{}", s), location )),
                (Err(e),b) =>
                {
                    if(self.end_of_file) { break; }
                    else
                    {
                        let mut bytes = String::new();
                        for b in b
                        {
                            bytes.push_str(format!("{b:#04x} ").as_str());
                        }
                        lines.push(( format!("{} => Error: {}", bytes, e), location ));
                    }
                },
            }
        }

        for i in 0..lines.len()
        {
            
            let c = &lines[i];

            for l in &self.labels
            {
                if(*l.0 == c.1) { output.push_str(format!("{}:\n", l.1).as_str()); }
                else if(*l.0 >= c.1)
                {
                    if(i + 1 >= lines.len())
                    {
                        output.push_str(format!("{}:\n", l.1).as_str());
                    }
                    else
                    {
                        let n = &lines[i + 1];
                        if(*l.0 < n.1)
                        {
                            output.push_str(format!("{}:\n", l.1).as_str());
                        }
                        else
                        {
                            continue;
                        }
                    }
                }
            }

            output.push_str(format!("{}\n", c.0).as_str());

        }

        Ok(output)

    }

}

