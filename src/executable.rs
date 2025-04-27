#![allow(unused_parens)]
#![allow(dead_code)]

pub mod __internal
{

    use crate::{debug::DebugSymbol, instructions::{helpers::HeaderConstructor, SourceLocation}};

    #[derive(Debug, Clone)]
    pub struct Label
    {
        pub name: String,
        pub fileloc: SourceLocation,
        pub pos: i64,
    }
    impl PartialEq for Label
    {
        fn eq(&self, other: &Self) -> bool { self.name == other.name }
    }

    #[derive(Debug, Clone)]
    pub struct LabelRequest
    {
        pub name: String,
        pub loc: SourceLocation,
        pub pos: u32,
    }

    #[derive(Debug, PartialEq, Clone, Copy)]
    pub enum Section
    {
        None,
        Code,
        Data,
    }

    #[derive(Debug, Clone)]
    pub struct SectionData
    {
        pub section: Section,
        pub data: Vec<u8>,
    }
    impl SectionData
    {
        pub fn len     (&self) -> usize { self.data.len      () }
        pub fn is_empty(&self) -> bool  { self.data.is_empty () }
    }

    #[derive(Debug, Clone)]
    pub struct SectionFormat
    {
        pub section: SectionData,
        pub labels: Vec<Label>,
        pub exposed_labels: Vec<Label>,
        pub requested_labels: Vec<LabelRequest>,
        pub symbols: Vec<DebugSymbol>,
    }

    #[derive(Debug)]
    pub struct Format
    {
        pub sections: Vec<SectionFormat>,
        pub external_labels: Vec<Label>,
        pub header: Option<HeaderConstructor>,
    }
    impl Format
    {
        pub fn len(&self) -> usize
        { 
            let mut len = 0;
            for s in &self.sections
            {
                len += s.section.len();
            }
            len
        }
        pub fn is_empty(&self) -> bool
        { 
            self.len() == 0
        }
    }

}

#[derive(Debug)]
pub struct Executable
{
    pub section_header: [u8; 32],
    pub section_code: Vec<u8>,
    pub section_data: Vec<u8>,
}
