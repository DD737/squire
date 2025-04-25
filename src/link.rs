#![allow(unused_parens)]

pub mod __link
{

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
    
    use colored::Colorize;

    use crate::instructions::helpers::u16_2_u8;
    use crate::instructions::{Error, IRBinaryHeader, __IRBinaryHeader};
    use crate::executable::{Executable, __internal::*};

    pub struct Linker
    {
        pub formats: Vec<Format>,
    }
    impl Linker
    {
        
        pub fn formats(formats: Vec<Format>) -> Self
        {
            Self
            {
                formats,
            }
        }

        fn format_get_sections(mut f: Format) -> (Format, Option<SectionFormat>, Option<SectionFormat>)
        {

            let mut code_format: Option<SectionFormat> = None;
            let mut data_format: Option<SectionFormat> = None;

            for s in &mut f.sections
            {
                match &s.section.section
                {
                    &Section::None   => unreachable!(),
                    &Section::Code   => code_format = Some(s.clone()),
                    &Section::Data   => data_format = Some(s.clone()),
                }
            }

            (f, code_format, data_format)

        }

        pub fn executable_to_bytes(exe: Executable, parse_header: bool) -> Vec<u8>
        {

            let mut bytes: Vec<u8> = Vec::new();

            if(parse_header)
            {
                bytes.extend(exe.section_header);
            }

            bytes.extend(exe.section_code);
            bytes.extend(exe.section_data);

            bytes

        }

        pub fn link(&mut self) -> Result<Executable, Error>
        {

            let mut binary_header: Option<IRBinaryHeader> = None;

            let mut code_section: Vec<u8> = Vec::new();
            let mut data_section: Vec<u8> = Vec::new();

            let mut intermediate_list: Vec<
            (
                Format, 
                Option<SectionFormat>, usize, 
                Option<SectionFormat>, usize,
            )> = Vec::new();

            let mut any_have_headr_constructor = false;

            let mut code_off = 0;
            let mut data_off = 0;
            for f in self.formats.drain(..)
            {

                if let Some(_) = f.header
                {
                    if(any_have_headr_constructor)
                    {
                        return Err(error!("FATAL: Multiple binary header defined!"));
                    }
                    any_have_headr_constructor = true;
                }
                
                let c_o = code_off;
                let d_o = data_off;

                let (f, code, data) = Linker::format_get_sections(f);

                if let Some(code) = &code
                {
                    code_off += code.section.len();
                }
                if let Some(data) = &data
                {
                    data_off += data.section.len();
                }

                intermediate_list.push((
                    f,
                    code, c_o,
                    data, d_o
                ));

            }

            if(!any_have_headr_constructor)
            {
                println!("{} No binary header defined.", "Notice: ".cyan());
            }

            let mut code_label_index: Vec<Label> = Vec::new();
            let mut data_label_index: Vec<Label> = Vec::new();
            
            for l in &mut intermediate_list
            {

                if let Some(code_format) = &mut l.1
                {
                    for label in &mut code_format.labels
                    {
                        label.pos += l.2 as i32;
                    }
                    'a: for exp in &code_format.exposed_labels
                    {
                        for label in &mut code_format.labels
                        {
                            if(label.name == exp.name)
                            {
                                code_label_index.push(label.clone());
                                continue 'a;
                            }
                        }
                        return Err(error_in!((&exp.fileloc), "There is no label '{}' that can be exposed!", exp.name))
                    }
                }
                
                if let Some(data_format) = &mut l.3
                {
                    for label in &mut data_format.labels
                    {
                        label.pos += l.4 as i32;
                    }
                    'a: for exp in &data_format.exposed_labels
                    {
                        for label in &mut data_format.labels
                        {
                            if(label.name == exp.name)
                            {
                                data_label_index.push(label.clone());
                                continue 'a;
                            }
                        }
                        return Err(error_in!((&exp.fileloc), "There is no label '{}' that can be exposed!", exp.name))
                    }
                }

            }
            for l in &mut intermediate_list
            {
                'a: for ext in &mut l.0.external_labels
                {

                    for l in &code_label_index
                    {
                        if(l.name == ext.name)
                        {
                            ext.pos = l.pos;
                            continue 'a;
                        }
                    } 

                    for l in &data_label_index
                    {
                        if(l.name == ext.name)
                        {
                            ext.pos = l.pos;
                            continue 'a;
                        }
                    }    

                    println!("{:?}", ext);
                    println!("{:?}", code_label_index);
                    println!("{:?}", data_label_index);

                    return Err(error_in!((&ext.fileloc), "There is no exposed label with the name '{}'!", ext.name))

                }
            }
            for l in &mut intermediate_list
            {
                if let Some(code_format) = &mut l.1
                {
                    code_section.extend(code_format.section.data.drain(..));
                }
                if let Some(data_format) = &mut l.3
                {
                    data_section.extend(data_format.section.data.drain(..));
                }
            }
            for l in &mut intermediate_list
            {

                if let Some(code_format) = &l.1
                {

                    for req in &code_format.requested_labels
                    {

                        let mut _label: Option<Label> = None;

                        for label in &code_format.labels
                        {
                            if(label.name == req.name)
                            {
                                _label = Some(label.clone());
                                break;
                            }
                        }
                        
                        if(_label.is_none())
                        {
                            if let Some(data_format) = &l.3
                            {
                                for label in &data_format.labels
                                {
                                    if(label.name == req.name)
                                    {
                                        let mut lab = label.clone();
                                        lab.pos += code_section.len() as i32;
                                        _label = Some(lab);
                                        break;
                                    }
                                }
                            }
                        }

                        if(_label.is_none())
                        {

                            for label in &code_label_index
                            {
                                if(label.name == req.name)
                                {
                                    _label = Some(label.clone());
                                    break;
                                }
                            }

                        }

                        if(_label.is_none())
                        {

                            for label in &data_label_index
                            {
                                if(label.name == req.name)
                                {
                                    let mut lab = label.clone();
                                    lab.pos += code_section.len() as i32;
                                    _label = Some(lab);
                                    break;
                                }
                            }

                        }
                        
                        let label = match _label
                        {
                            Some(l) => l,
                            None => panic!("couldnt find label for request {:?}", req),
                        };

                        let adr = req.pos as usize + l.2;
                        let v = u16_2_u8(label.pos as u16);

                        code_section[adr + 0] = v.0;
                        code_section[adr + 1] = v.1;

                    }

                }

                if let Some(data_format) = &l.3
                {

                    for req in &data_format.requested_labels
                    {

                        let mut _label: Option<Label> = None;

                        for label in &data_format.labels
                        {
                            if(label.name == req.name)
                            {
                                _label = Some(label.clone());
                                break;
                            }
                        }
                        
                        if(_label.is_none())
                        {
                            if let Some(code_format) = &l.1
                            {
                                for label in &code_format.labels
                                {
                                    if(label.name == req.name)
                                    {
                                        let mut lab = label.clone();
                                        lab.pos -= code_section.len() as i32;
                                        _label = Some(lab);
                                        break;
                                    }
                                }
                            }
                        }

                        if(_label.is_none())
                        {

                            for label in &data_label_index
                            {
                                if(label.name == req.name)
                                {
                                    _label = Some(label.clone());
                                    break;
                                }
                            }

                        }

                        if(_label.is_none())
                        {

                            for label in &code_label_index
                            {
                                if(label.name == req.name)
                                {
                                    let mut lab = label.clone();
                                    lab.pos -= code_section.len() as i32;
                                    _label = Some(label.clone());
                                    break;
                                }
                            }

                        }
                        
                        let mut label = match _label
                        {
                            Some(l) => l,
                            None => panic!("couldnt find label for request {:?}", req),
                        };

                        label.pos += code_section.len() as i32;

                        let adr = req.pos as usize + l.4;
                        let v = u16_2_u8(label.pos as u16);

                        data_section[adr + 0] = v.0;
                        data_section[adr + 1] = v.1;

                    }

                }

                if let Some(header) = &mut l.0.header
                {

                    if let Some(entry) = &header.entry
                    {
    
                        let mut _label: Option<Label> = None;

                        if let Some(code_format) = &l.1
                        {
                            for label in &code_format.labels
                            {
                                if(label.name == entry.name)
                                {
                                    _label = Some(label.clone());
                                    break;
                                }
                            }
                        }

                        if(_label.is_none())
                        {
                            if let Some(data_format) = &l.3
                            {
                                for label in &data_format.labels
                                {
                                    if(label.name == entry.name)
                                    {
                                        let mut lab = label.clone();
                                        lab.pos += code_section.len() as i32;
                                        _label = Some(lab);
                                        break;
                                    }
                                }
                            }
                        }

                        if(_label.is_none())
                        {

                            for label in &code_label_index
                            {
                                if(label.name == entry.name)
                                {
                                    _label = Some(label.clone());
                                    break;
                                }
                            }

                        }

                        if(_label.is_none())
                        {

                            for label in &data_label_index
                            {
                                if(label.name == entry.name)
                                {
                                    let mut lab = label.clone();
                                    lab.pos += code_section.len() as i32;
                                    _label = Some(lab);
                                    break;
                                }
                            }

                        }
                        
                        let label = match _label
                        {
                            Some(l) => l,
                            None => panic!("couldnt find label for request {:?}", entry),
                        };

                        let v = label.pos;

                        header.set_straight_entry(v as u16, entry.fileloc.clone())?;

                        binary_header = Some(header.finalize()?);

                    }

                }

            }

            let section_header = if let Some(header) = binary_header { header.serialize() } else { [0; 32] };

            Ok(Executable
            {
                section_header,
                section_code: code_section,
                section_data: data_section,
            })

        }

    }

}

pub type Linker = __link::Linker;
