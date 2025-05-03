use std::cell::RefCell;
use std::os::linux::raw::stat;
use std::rc::{Rc, Weak};
use std::sync::Arc;
use crate::parse::*;
use erebos::{error, error_in};
use erebos::instructions::{*, IIR::*};

#[derive(Debug)]
pub enum EvalStorage
{
    Register(IRRegister),
    Location(Option<IRImmediate>),
    Stack,
}
impl EvalStorage
{
    pub fn get_reg(&self) -> IRRegister
    {
        match self
        {
            Self::Register(r) => *r,
            _ => panic!("NOT A REGISTER!"),
        }
    }
}

pub mod types
{

    use super::*;

    #[derive(Debug, PartialEq, Clone)]
    pub enum EvalTypeInternal
    {
        U8, U16, U32,
        Bool,
        Void,
        Never,
    }
    impl std::fmt::Display for EvalTypeInternal
    {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
        {
            write!(f, "{}", match self
            {
                EvalTypeInternal::U8    => "U8",
                EvalTypeInternal::U16   => "U16",
                EvalTypeInternal::U32   => "U32",
                EvalTypeInternal::Bool  => "Bool",
                EvalTypeInternal::Void  => "Void",
                EvalTypeInternal::Never => "Never",
            })
        }
    }
    impl EvalTypeInternal
    {
    
        pub fn has_trait(&self, _trait: EvalTypeTrait) -> bool
        {
            
            if matches!(self, Self::Void ) { return  true; }
            if matches!(self, Self::Never) { return false; }

            match _trait
            {
                EvalTypeTrait::Any => true,
                EvalTypeTrait::Number  => matches!(self, Self::U32 | Self::U16 | Self::U8),
                EvalTypeTrait::Boolean => matches!(self, Self::Bool),
            }

        }

        pub fn fits_type(&self, val: &EvalValue) -> bool
        {
            if(matches!(self, Self::Void )) { return  true; }
            if(matches!(self, Self::Never)) { return false; }
            match val
            {
                EvalValue::Boolean(_, _) => matches!(self, Self::Bool),
                EvalValue::String(_, _) => todo!(),
                EvalValue::Number(_, _) => matches!(self, Self::U32 | Self::U16 | Self::U8 ),

                EvalValue::Symbol(s, _) =>
                {
                    match s
                    {
                        EvalSymbol::Variable(v) => v.borrow().r#type.borrow().is_internal(self),
                        _ => false
                    }
                }
                EvalValue::Complex(c, _) =>
                {
                    match c
                    {
                        EvalValueComplex::FunctionCall(f, _) => f.upgrade().as_ref().unwrap().borrow().r#type.borrow().is_internal(self),
                        EvalValueComplex::OpUnary(op, a) =>
                        {
                            let fits = match op
                            {
                                | EvalValueComplexOpUnary::POS
                                | EvalValueComplexOpUnary::NEG
                                | EvalValueComplexOpUnary::INV => matches!(self, Self::U32 | Self::U16 | Self::U8 ),
                                | EvalValueComplexOpUnary::NOT => matches!(self, Self::Bool),
                                | EvalValueComplexOpUnary::PTR
                                | EvalValueComplexOpUnary::REF => false
                            };
                            if(fits)
                            { self.fits_type(a) }
                            else { false }
                        }
                        EvalValueComplex::OpBinary(op, a, b) =>
                        {
                            match op
                            {
                                | EvalValueComplexOpBinary::ADD
                                | EvalValueComplexOpBinary::SUB
                                | EvalValueComplexOpBinary::MUL
                                | EvalValueComplexOpBinary::DIV
                                | EvalValueComplexOpBinary::MOD
                                | EvalValueComplexOpBinary::AND
                                | EvalValueComplexOpBinary::OR 
                                | EvalValueComplexOpBinary::XOR
                                | EvalValueComplexOpBinary::SHL
                                | EvalValueComplexOpBinary::SHR => 
                                    matches!(self, Self::U32 | Self::U16 | Self::U8 )
                                    && self.fits_type(a)
                                    && self.fits_type(b),

                                | EvalValueComplexOpBinary::EQU
                                | EvalValueComplexOpBinary::NEQU
                                | EvalValueComplexOpBinary::LT
                                | EvalValueComplexOpBinary::LTE
                                | EvalValueComplexOpBinary::GT
                                | EvalValueComplexOpBinary::GTE => 
                                    matches!(self, Self::Bool),

                                | EvalValueComplexOpBinary::BAND
                                | EvalValueComplexOpBinary::BOR => 
                                    matches!(self, Self::Bool)
                                    && self.fits_type(a)
                                    && self.fits_type(b),

                                | EvalValueComplexOpBinary::NONE => false,
                                | EvalValueComplexOpBinary::ASSIGN(_) => todo!()
                            }
                        }
                        EvalValueComplex::Reference(_) => todo!()
                    }
                }
            }
        }
        pub fn get_size(&self) -> u8
        {
            match *self
            {
                EvalTypeInternal::Never => 0,
                EvalTypeInternal::Void  => 4,
                EvalTypeInternal::Bool  => 1,
                EvalTypeInternal::U8    => 1, 
                EvalTypeInternal::U16   => 2, 
                EvalTypeInternal::U32   => 4,
            }
        }
        pub fn get_type(name: &str) -> Option<Self>
        {
            match name.to_ascii_uppercase().as_str()
            {
                "NEVER" => Some(EvalTypeInternal::Never),
                "VOID"  => Some(EvalTypeInternal::Void ),
                "BOOL"  => Some(EvalTypeInternal::Bool ),
                "U8"    => Some(EvalTypeInternal::U8   ),
                "U16"   => Some(EvalTypeInternal::U16  ),
                "U32"   => Some(EvalTypeInternal::U32  ),
                _ => None,
            }
        }
        pub fn default_value(&self) -> EvalValue
        {
            match *self
            {
                | EvalTypeInternal::Void
                | EvalTypeInternal::U8  
                | EvalTypeInternal::U16 
                | EvalTypeInternal::U32  => EvalValue::Number(0, SourceLocation::default()),
                | EvalTypeInternal::Bool => EvalValue::Boolean(false, SourceLocation::default()),
                | EvalTypeInternal::Never => panic!("Never does not have a default value!"),
            }
        }
    
    }
    
    #[derive(Debug, PartialEq, Clone)]
    pub struct EvalTypeDecl {}
    impl std::fmt::Display for EvalTypeDecl
    {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
        {
            write!(f, "[CUSTOM TYPE]")
        }
    }
    
    #[derive(Debug, Clone)]
    pub enum EvalType 
    {
        Internal ( EvalTypeInternal ),
        Custom   ( EvalTypeDecl     ),
        Pointer  ( Rc<RefCell<EvalType>> ),
    }
    impl std::fmt::Display for EvalType
    {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
        {
            match self
            {
                EvalType::Internal (i) => write!(f, "{i}"),
                EvalType::Custom   (c) => write!(f, "{c}"),
                EvalType::Pointer  (p) => write!(f, "{p:?}"),
            }
        }
        }
    impl EvalType
    {

        pub fn has_trait(&self, _trait: EvalTypeTrait) -> bool
        {
            match self
            {
                Self::Internal(i) => i.has_trait(_trait),
                Self::Custom(_) => false,
                Self::Pointer(_) => todo!()
            }
        }

        pub fn fits_type(&self, val: &EvalValue) -> bool
        {
            match self
            {
                Self::Internal(i) => i.fits_type(val),
                _ => false,
            }
        }

        pub fn get_size(&self) -> u8
        {
            match self
            {
                EvalType::Internal (_t) => _t.get_size(),
                EvalType::Custom   (_t) => panic!("Custom types are not supported yet!"),
                EvalType::Pointer  (_t) => 4,
            }
        }
        pub fn can_cast_to   (&self, _other: &Self) -> bool { self == _other }
        pub fn can_cast_from (&self, _other: &Self) -> bool { _other.can_cast_to(self) }

        pub fn default_value(&self) -> EvalValue
        {
            match self
            {
                EvalType::Internal (_t) => _t.default_value(),
                EvalType::Custom   (_t) => panic!("Custom types are not supported yet!"),
                EvalType::Pointer  (_ ) => EvalValue::Number(0, SourceLocation::default()),
            }
        }

        pub fn is_internal(&self, other: &EvalTypeInternal) -> bool
        {
            match self
            {
                Self::Internal(i) => other == i,
                _ => false,
            }
        }
    
    }
    impl PartialEq for EvalType
    {
        fn eq(&self, other: &Self) -> bool
        {
            if(matches!(self, Self::Internal(EvalTypeInternal::Never)) || matches!(other, Self::Internal(EvalTypeInternal::Never)))
            { return false; }
            if(matches!(self, Self::Internal(EvalTypeInternal::Void )) || matches!(other, Self::Internal(EvalTypeInternal::Void )))
            { return true; }
            match self
            {
                Self::Custom   (c0) => match other { Self::Custom   (c1) => c0 == c1, _ => false },
                Self::Pointer  (p0) => match other { Self::Pointer  (p1) => p0 == p1, _ => false },
                Self::Internal (i0) => match other { Self::Internal (i1) => i0 == i1, _ => false },
            }
        }
    }



    #[derive(Debug, PartialEq, Clone, Copy)]
    pub enum EvalTypeTrait
    {
        Any,
        Number,
        Boolean,
    }
    impl EvalTypeTrait
    {

        pub fn fits_trait(&self, val: &EvalValue) -> bool
        {

            if(matches!(self, Self::Any)) { return true; }
            match val
            {
                EvalValue::Boolean(_, _) => matches!(self, Self::Boolean),
                EvalValue::String(_, _) => todo!(),
                EvalValue::Number(_, _) => matches!(self, Self::Number ),

                EvalValue::Symbol(s, _) =>
                {
                    match s
                    {
                        EvalSymbol::Variable(v) => v.borrow().r#type.borrow().has_trait(*self),
                        _ => false
                    }
                }
                EvalValue::Complex(c, _) =>
                {
                    match c
                    {
                        EvalValueComplex::FunctionCall(f, _) => f.upgrade().as_ref().unwrap().borrow().r#type.borrow().has_trait(*self),
                        EvalValueComplex::OpUnary(op, a) =>
                        {
                            let fits = match op
                            {
                                | EvalValueComplexOpUnary::POS
                                | EvalValueComplexOpUnary::NEG
                                | EvalValueComplexOpUnary::INV => matches!(self, Self::Number ),
                                | EvalValueComplexOpUnary::NOT => matches!(self, Self::Boolean),
                                | EvalValueComplexOpUnary::PTR
                                | EvalValueComplexOpUnary::REF => false
                            };
                            if(fits)
                            { self.fits_trait(a) }
                            else { false }
                        }
                        EvalValueComplex::OpBinary(op, a, b) =>
                        {
                            match op
                            {
                                | EvalValueComplexOpBinary::ADD
                                | EvalValueComplexOpBinary::SUB
                                | EvalValueComplexOpBinary::MUL
                                | EvalValueComplexOpBinary::DIV
                                | EvalValueComplexOpBinary::MOD
                                | EvalValueComplexOpBinary::AND
                                | EvalValueComplexOpBinary::OR 
                                | EvalValueComplexOpBinary::XOR
                                | EvalValueComplexOpBinary::SHL
                                | EvalValueComplexOpBinary::SHR => 
                                    matches!(self, Self::Number )
                                    && self.fits_trait(a)
                                    && self.fits_trait(b),

                                | EvalValueComplexOpBinary::EQU
                                | EvalValueComplexOpBinary::NEQU
                                | EvalValueComplexOpBinary::LT
                                | EvalValueComplexOpBinary::LTE
                                | EvalValueComplexOpBinary::GT
                                | EvalValueComplexOpBinary::GTE => 
                                    matches!(self, Self::Boolean),

                                | EvalValueComplexOpBinary::BAND
                                | EvalValueComplexOpBinary::BOR => 
                                    matches!(self, Self::Boolean)
                                    && self.fits_trait(a)
                                    && self.fits_trait(b),

                                | EvalValueComplexOpBinary::NONE => false,
                                | EvalValueComplexOpBinary::ASSIGN(_) => todo!()
                            }
                        }
                        EvalValueComplex::Reference(_) => todo!()
                    }
                }
            }
        }

    }

    #[derive(Debug, Clone)]
    pub enum TypeOrTrait
    {
        Type (Rc<RefCell<EvalType>>),
        Trait(EvalTypeTrait),
    }
    impl TypeOrTrait
    {

        pub fn presedence<'a>(a: &'a TypeOrTrait, b: &'a TypeOrTrait) -> &'a TypeOrTrait
        {

            if let Self::Trait(EvalTypeTrait::Any) = a { return b; }
            if let Self::Trait(EvalTypeTrait::Any) = b { return a; }

            if let Self::Type(t) = a
            { if(t.borrow().is_internal(&EvalTypeInternal::Void)) { return b; } }
            if let Self::Type(t) = b
            { if(t.borrow().is_internal(&EvalTypeInternal::Void)) { return a; } }
            
            if(matches!(a, Self::Type(_)) && matches!(b, Self::Trait(_)))
            { a }
            else  if(matches!(b, Self::Type(_)) && matches!(a, Self::Trait(_)))
            { b }
            else if(matches!(a, Self::Trait(_)) && matches!(b, Self::Trait(_)))
            {
                if let Self::Trait(ta) = a
                {
                    if(matches!(ta, EvalTypeTrait::Any)) { b }
                    else { a }
                }
                else { unreachable!() }
            }
            else if(matches!(a, Self::Type(_)) && matches!(b, Self::Type(_)))
            {
                if let Self::Type(ta) = a
                {
                    if(matches!(*ta.borrow(), EvalType::Internal(EvalTypeInternal::Void))) { b }
                    else { a }
                }
                else { unreachable!() }
            }
            else { unreachable!() }

        }

        pub fn equ(&self, other: &TypeOrTrait) -> bool
        {
            match self
            {
                Self::Type(t0) => match other
                {
                    Self::Type(t1) => t0 == t1,
                    Self::Trait(t1) => t0.borrow().has_trait(*t1),
                },
                Self::Trait(t0) => match other
                {
                    Self::Type(t1) => t1.borrow().has_trait(*t0),
                    Self::Trait(t1) => t0 == t1,
                },
            }
        }
        pub fn fits(&self, val: &EvalValue) -> bool
        {
            match self
            {
                Self::Type  (t) => t.borrow().fits_type  (val),
                Self::Trait (t) => t.fits_trait (val),
            }
        }

    }

}
use types::*;

 #[derive(Debug)]
 pub struct EvalVariable
 {
    pub id: u32,
    pub name: String,
    pub unique_name: String,
    pub r#type: Rc<RefCell<EvalType>>,
    pub storage: EvalStorage,
    pub value: Option<StatementExpression>,
    pub initializer: Option<EvalValue>,
    pub constant: bool,
    pub parent: Weak<RefCell<Scope>>,
 }
 impl PartialEq for EvalVariable
 {
    fn eq(&self, other: &Self) -> bool { self.id == other.id }
 }
 #[derive(Debug)]
 pub struct EvalFunction
{
    pub id: u32,
    pub name: String,
    pub unique_name: String,
    pub r#type: Rc<RefCell<EvalType>>,
    pub params: Vec<EvalVariable>,
    pub parent: Weak<RefCell<Scope>>,
    pub scope: Rc<RefCell<Scope>>,
}
impl PartialEq for EvalFunction
{
    fn eq(&self, other: &Self) -> bool { self.id == other.id }
}

#[allow(clippy::enum_variant_names)]
enum EvalVarDefContext
{
    StaticDef,
    LocalDef,
    ParamDef,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ScopeType
{
    Sequential,
    Static,
    None,
}
#[derive(Debug)]
pub struct Scope
{
    
    pub id: u32,
    pub statement: Option<Statement>,
    pub r#type: ScopeType,
    pub children:  Vec< Rc <RefCell<Scope>>>,
    pub parent: Option<Weak<RefCell<Scope>>>,

    pub variables: Vec<Rc<RefCell<EvalVariable>>>,
    pub functions: Vec<Rc<RefCell<EvalFunction>>>,
    pub typedecls: Vec<Rc<RefCell<EvalTypeDecl>>>,

    pub self_variable: Option<Rc<RefCell<EvalVariable>>>,
    pub self_function: Option<Rc<RefCell<EvalFunction>>>,
    pub self_typedecl: Option<Rc<RefCell<EvalTypeDecl>>>,

}
impl Scope
{

    pub fn new(id: u32, statement: Statement, parent: Weak<RefCell<Scope>>, r#type: ScopeType) -> Self
    {
        Self
        {
            id,
            statement: Some(statement),
            r#type,
            children: Vec::new(),
            parent: Some(parent),
            variables: Vec::new(),
            functions: Vec::new(),
            typedecls: Vec::new(),
            self_function: None,
            self_variable: None,
            self_typedecl: None,
        }
    }
    pub fn new_none       (id: u32, statement: Statement, parent: Weak<RefCell<Scope>>) -> Self { Self::new(id, statement, parent, ScopeType::None       ) }
    pub fn new_sequential (id: u32, statement: Statement, parent: Weak<RefCell<Scope>>) -> Self { Self::new(id, statement, parent, ScopeType::Sequential ) }
    pub fn new_static     (id: u32, statement: Statement, parent: Weak<RefCell<Scope>>) -> Self { Self::new(id, statement, parent, ScopeType::Static     ) }
    
}

pub mod eval_value
{

    use super::*;

    #[derive(Debug, Clone)]
    pub enum EvalValueComplexOpBinary
    {
        NONE,
        ADD, SUB, MUL, DIV, MOD,
        AND, OR, XOR,
        SHL, SHR,
        EQU, NEQU, LT, GT, LTE, GTE,
        BAND, BOR,
        ASSIGN(Box<EvalValueComplexOpBinary>),
    }
    #[derive(Debug, Clone, Copy)]
    pub enum EvalValueComplexOpUnary
    {
        POS, NEG,
        INV, NOT,
        PTR, REF,
    }
    #[derive(Debug, Clone)]
    pub enum EvalValueComplex
    {
        Reference(Weak<RefCell<EvalVariable>>),
        FunctionCall(Weak<RefCell<EvalFunction>>, Vec<EvalValue>),
        OpBinary(EvalValueComplexOpBinary, Box<EvalValue>, Box<EvalValue>),
        OpUnary(EvalValueComplexOpUnary , Box<EvalValue>),
    }
    impl EvalValueComplex
    {
        pub fn is_const(&self) -> bool
        {
            match self
            {
                Self::Reference    (_      ) => true,
                Self::FunctionCall (_, _   ) => false,
                Self::OpBinary     (_, a, b) => a.is_const() && b.is_const(),
                Self::OpUnary      (_, a   ) => a.is_const(),
            }
        }
    }
    #[derive(Debug, Clone)]
    pub enum EvalValue
    {
        String  ( String          , SourceLocation ),
        Number  ( u32             , SourceLocation ),
        Boolean ( bool            , SourceLocation ),
        Symbol  ( EvalSymbol      , SourceLocation ),
        Complex ( EvalValueComplex, SourceLocation ),
    }
    impl EvalValue
    {
        pub fn is_const(&self) -> bool
        {
            match self
            {
                | Self::Boolean (_,_)
                | Self::String  (_,_)
                | Self::Number  (_,_) => true,
                Self::Symbol    (v,_) => v.is_const(), 
                Self::Complex   (v,_) => v.is_const(),
            }
        }
        pub fn loc(&self) -> SourceLocation
        {
            match self
            {
                Self::String  (_,loc) => loc.clone(),
                Self::Number  (_,loc) => loc.clone(),
                Self::Boolean (_,loc) => loc.clone(),
                Self::Symbol  (_,loc) => loc.clone(),
                Self::Complex (_,loc) => loc.clone(),
            }
        }
    }

    #[derive(Debug, Clone)]
    pub enum EvalSymbol
    {
        Function (Rc<RefCell<EvalFunction>>),
        Variable (Rc<RefCell<EvalVariable>>),
        Type     (Rc<RefCell<EvalType    >>),
    }
    impl EvalSymbol
    {
        pub fn is_const(&self) -> bool
        {
            match self
            {
                Self::Variable(v) => v.borrow().constant,
                _ => true
            }
        }
    }



    #[derive(Debug)]
    pub struct EvalValueTyped
    {
        pub type_or_trait: TypeOrTrait,
        pub val: EvalValue,
    }
    impl EvalValueTyped
    {
        
        pub fn from_either(val: EvalValue, type_or_trait : TypeOrTrait) -> Self
        {
            Self
            {
                val,
                type_or_trait,
            }
        }
        pub fn from_type (val: EvalValue, _type : Rc<RefCell<EvalType>>) -> Self
        {
            Self
            {
                val,
                type_or_trait: TypeOrTrait::Type(_type),
            }
        }
        pub fn from_trait(val: EvalValue, _trait: EvalTypeTrait) -> Self
        {
            Self
            {
                val,
                type_or_trait: TypeOrTrait::Trait(_trait),
            }
        }

    }

}
use eval_value::*;

pub mod step
{
    use std::rc::Rc;
    use std::cell::RefCell;
    use erebos::instructions::*;
    use erebos::error_in;
    use super::*;


    #[derive(Clone)]
    pub enum Storage
    {
        TMP(u32),
        REG(IRRegister),
        LET(Rc<RefCell<EvalVariable>>),
        VAR(Rc<RefCell<EvalVariable>>),
        VAL(EvalValue),
    }
    impl std::fmt::Debug for Storage
    {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
        {
            match self
            {
                Storage::TMP(v) => write!(f, "TMP({v})"),
                Storage::REG(r) => write!(f, "REG({r:?})"),
                Storage::LET(v) => write!(f, "LET(\"{}\":{:?})", v.borrow().name, v.borrow().storage.get_reg()),
                Storage::VAR(v) => write!(f, "VAR(\"{}\")", v.borrow().name),
                Storage::VAL(v) => write!(f, "VAL({v:?})"),
            }
        }
    }

    pub fn filter_steps(steps: Vec<CompleteStep>) -> Vec<CompleteStep>
    {
        let mut filtered: Vec<CompleteStep> = Vec::new();
        for s in steps
        {
            if(!matches!(s.1, Step::Value))
            {
                filtered.push(s);
            }
        }
        filtered
    }
    pub fn value_to_steps(val: &EvalValue, can_assign: bool, filter: bool) -> Result<Vec<CompleteStep>, Error>
    { 
        // _value_to_steps(val, can_assign, &mut 1)
        let steps = _value_to_steps(val, can_assign, &mut 1)?;
        if(filter)
        {
            Ok(filter_steps(steps))
        }
        else
        {
            Ok(steps)
        }
    }
    fn _value_to_steps(val: &EvalValue, can_assign: bool, tmp_id_counter: &mut u32) -> Result<Vec<CompleteStep>, Error>
    {

        let mut steps: Vec<CompleteStep> = Vec::new();

        match val
        {
            | EvalValue::Boolean (_, _)
            | EvalValue::String  (_, _)
            | EvalValue::Number  (_, _)
                =>
            {
                let store = Storage::VAL(val.clone());
                steps.push((store, Step::Value));

            }
            EvalValue::Symbol(s, l) =>
            {
                match s
                {
                    EvalSymbol::Function (_) => return Err(error_in!(l, "Cannot assign function!")),
                    EvalSymbol::Type     (_) => return Err(error_in!(l, "Cannot assign type!")),
                    EvalSymbol::Variable (v) => 
                    {
                        let store = match v.borrow().storage
                        {
                            EvalStorage::Register(_) => Storage::LET(v.clone()),
                                                  _  => Storage::VAR(v.clone()),
                        };
                        steps.push((store, Step::Value));
                    },
                }
            },
            EvalValue::Complex(c, l) => 
            {
                match c
                {
                    EvalValueComplex::OpBinary(op, a, b) =>
                    {

                        let (op, do_assign) = match op
                        {
                            EvalValueComplexOpBinary::ASSIGN(op) => (&**op, true),
                            _ => (op, false),
                        };

                        let (a, b) = (&**a, &**b);

                        let ra = Evaluator::eval_type_check(a)?;
                        let rb = Evaluator::eval_type_check(b)?;

                        let mut a_steps = _value_to_steps(&ra.val, false, tmp_id_counter)?;
                        let mut b_steps = _value_to_steps(&rb.val, false, tmp_id_counter)?;

                        let a_store = a_steps.last().unwrap().0.clone();
                        let b_store = b_steps.last().unwrap().0.clone();

                        steps.append(&mut b_steps);
                        steps.append(&mut a_steps);

                        if(do_assign)
                        {

                            if(!can_assign)
                            {
                                return Err(error_in!(l, "Nested assignments are not allowed!"));
                            }

                            match &a_store
                            {
                                Storage::VAL(_) => return Err(error_in!(l, "Cannot assign to constant value!")),
                                Storage::VAR(v) =>
                                {
                                    if(v.borrow().constant)
                                    {
                                        return Err(error_in!(l, "Cannot assign to constant!"));
                                    }
                                },
                                _ => {},
                            };

                            if(!ra.type_or_trait.equ(&rb.type_or_trait))
                            {
                                return Err(error_in!((l), "Cannot assign! Types dont match. [left: {:?}, right: {:?}]", ra.type_or_trait, rb.type_or_trait));
                            }

                        }

                        if(matches!(op, EvalValueComplexOpBinary::NONE))
                        {
                            if(do_assign)
                            {
                                steps.push((a_store.clone(), Step::Assignment(b_store.clone())));
                            }
                            else
                            {
                                return Err(error_in!((l), "NONE OPERAND"));
                            }
                        }
                        else
                        {
                            *tmp_id_counter += 1;
                            let store = Storage::TMP(*tmp_id_counter);
                            steps.push((store.clone(), Step::BinOP(op.clone(), a_store.clone(), b_store.clone())));
                            if(do_assign)
                            {
                                steps.push((a_store.clone(), Step::Assignment(store)));
                            }
                        }

                    },
                    EvalValueComplex::OpUnary(op, a) =>
                    {
                        let a = &**a;
                        let mut a_steps = _value_to_steps(a, false, tmp_id_counter)?;
                        steps.append(&mut a_steps);
                        let a_store = &a_steps.last().unwrap().0;
                        *tmp_id_counter += 1;
                        let store = Storage::TMP(*tmp_id_counter);
                        steps.push((store, Step::UnOP(*op, a_store.clone())));
                    },
                    _ => todo!(),
                }
            }
        }

        Ok(steps)

    }

    #[derive(Debug, Clone)]
    pub enum Step
    {
        Value,
        Assignment(Storage),
        BinOP(EvalValueComplexOpBinary, Storage, Storage),
         UnOP(EvalValueComplexOpUnary , Storage),
        Call(Rc<RefCell<EvalFunction>>, Vec<Storage>),
        Instruction(IIRStatement),
    }
    pub type CompleteStep = (Storage, Step);

}

pub struct Evaluator
{
    parser: Parser,
    statement: Option<Statement>,
    id_counter: u32,
    entry_point: Option<Rc<RefCell<EvalFunction>>>,
}
impl Evaluator
{

    pub fn file(file: Arc<str>) -> Result<Self, Error>
    {
        Ok(Self
        {
            parser: Parser::file(file)?,
            statement: None,
            id_counter: 1,
            entry_point: None,
        })
    }

    fn peek(&mut self) -> Result<Option<&Statement>, Error>
    {
        if(self.statement.is_none()) { self.statement = Some(self.parser.parse_statement()?); }
        Ok(self.statement.as_ref())
    }
    fn next(&mut self) -> Result<Option<Statement>, Error>
    {
        self.peek()?;
        Ok(self.statement.take())
    }

    pub fn get_scope(get_id: &mut impl FnMut()->u32, statement: Statement, parent: Weak<RefCell<Scope>>) -> Result<(Rc<RefCell<Scope>>, Option<Rc<RefCell<Scope>>>), Error>
    {

        match &statement
        {
            Statement::Return           (_, _) => Ok((Rc::new(RefCell::new(Scope::new_none(get_id(), statement, parent))), None)),
            Statement::DefinitionVar    (_, _) => Ok((Rc::new(RefCell::new(Scope::new_none(get_id(), statement, parent))), None)),
            Statement::Expression       (_, _) => Ok((Rc::new(RefCell::new(Scope::new_none(get_id(), statement, parent))), None)),
            Statement::DefinitionFunc   (_, s) => 
            {

                let content = s.content.clone();
                let scope = Scope::new_sequential(get_id(), statement, parent);
                let  ptr = Rc::new(RefCell::new(scope));
                let _ptr = Rc::downgrade(&ptr);

                for s in content
                {
                    unsafe
                    {
                        let child = Self::get_scope(get_id, s, _ptr.clone())?;
                        (*ptr.as_ptr()).children.push(child.0);
                        if let Some(child) = child.1
                        {
                            (*ptr.as_ptr()).children.push(child);
                        }
                    }
                }

                Ok((ptr, None))

            },
            Statement::DefinitionStruct (_, s) =>
            {

                let content = s.members.clone();
                let scope = Scope::new_static(get_id(), statement, parent);
                let  ptr = Rc::new(RefCell::new(scope));
                let _ptr = Rc::downgrade(&ptr);

                for s in content
                {
                    unsafe
                    {
                        let child = Self::get_scope(get_id, s, _ptr.clone())?;
                        (*ptr.as_ptr()).children.push(child.0);
                        if let Some(child) = child.1
                        {
                            (*ptr.as_ptr()).children.push(child);
                        }
                    }
                }

                Ok((ptr, None))

            },
            Statement::Conditional      (m, s) =>
            {

                let content = s.content.clone();
                let _else   = s.r#else .clone();
                
                let scope = Scope::new_sequential(get_id(), statement.clone(), parent.clone());
                let  ptr = Rc::new(RefCell::new(scope));
                let _ptr = Rc::downgrade(&ptr);

                for s in content
                {
                    unsafe
                    {
                        let child = Self::get_scope(get_id, s, _ptr.clone())?;
                        (*ptr.as_ptr()).children.push(child.0);
                        if let Some(child) = child.1
                        {
                            (*ptr.as_ptr()).children.push(child);
                        }
                    }
                }

                let mut ptr2: Option<Rc<RefCell<Scope>>> = None;
                
                if(!_else.is_empty())
                {
                    let mut s = s.clone();
                    s.r#type = StatementConditionalType::Else;
                    let statement = Statement::Conditional(m.to_vec(), s);
                    let scope = Scope::new_static(get_id(), statement, parent);
                    let  ptr = Rc::new(RefCell::new(scope));
                    let _ptr = Rc::downgrade(&ptr);

                    for s in _else
                    {
                        unsafe
                        {
                            let child = Self::get_scope(get_id, s, _ptr.clone())?;
                            (*ptr.as_ptr()).children.push(child.0);
                            if let Some(child) = child.1
                            {
                                (*ptr.as_ptr()).children.push(child);
                            }
                        }
                    }
                    ptr2 = Some(ptr);
                }

                Ok((ptr, ptr2))

            },
        }

    }
    pub fn get_scope_tree(&mut self) -> Result<Rc<RefCell<Scope>>, Error>
    {

        let globalScope = Scope
        {
            id: 0,
            children: Vec::new(),
            parent: None,
            statement: None,
            r#type: ScopeType::Static,
            variables: Vec::new(),
            functions: Vec::new(),
            typedecls: Vec::new(),
            self_function: None,
            self_variable: None,
            self_typedecl: None,
        };
        let  scope = Rc::new(RefCell::new(globalScope));
        let _scope = Rc::downgrade(&scope);

        let statements = self.parser.parse()?;

        for stm in statements
        {
            unsafe
            {
                let child = Self::get_scope(&mut ||self.get_id(), stm, _scope.clone())?;
                (*scope.as_ptr()).children.push(child.0);
                if let Some(child) = child.1
                {
                    (*scope.as_ptr()).children.push(child);
                }
            }
        }

        Ok(scope)

    }

    fn get_id(&mut self) -> u32
    {
        self.id_counter += 1;
        self.id_counter
    }

    fn get_unique_name(&mut self, scope: Rc<RefCell<Scope>>) -> String
    {
        let pre = Self::ir_get_prefix(scope);
        let id = self.get_id();
        format!("{pre}__{id:#06x}")
    }

    fn get_type(&mut self, exp: &StatementExpression, scope: Rc<RefCell<Scope>>) -> Result<Rc<RefCell<EvalType>>, Error>
    {
        let val = Self::eval_expression(exp, scope)?;
        let val = Self::calc_const_expr(&val, 0, val.loc())?;
        if let EvalValue::Symbol(EvalSymbol::Type(t), _) = val { Ok(t) }
        else
        {
            Err(error_in!((exp.loc()), "Is not a type!"))
        }
    }
    fn get_storage_specifier(spec: &str, loc: &Location) -> Result<EvalStorage, Error>
    {

        if(spec.starts_with("mem"))
        {
        
            let mut location: Option<u32> = None;
            if(spec.len() > 3)
            {
                if(spec.chars().nth(3) == Some('('))
                {
                    
                    if(!spec.ends_with(')'))
                    {
                        return Err(error_in!(loc, "Expected closing ')' at end of storage specifier!"));
                    }          
                    let mut num = spec[4..(spec.len() - 1)].to_string();
                    if(num.is_empty())
                    {
                        return Err(error_in!(loc, "Expected location inside mem(...)!"));
                    }
                    let mut base = 10;
                    if(num.starts_with('0'))
                    {
                             if(num.starts_with("0x")) { base = 16; num = num[2..].to_string(); }
                        else if(num.starts_with("0o")) { base =  2; num = num[2..].to_string(); }
                        else if(num.starts_with("0b")) { base =  8; num = num[2..].to_string(); }
                        else if(num.starts_with("0d")) { base = 10; num = num[2..].to_string(); }
                    }

                    location = Some(match u32::from_str_radix(&num, base)
                    {
                        Ok(n) => n,
                        Err(_) => return Err(error_in!(loc, "Could not parse number! [if this was not supposed to be a number, prefix it with a non-numeric character]")),
                    });

                }
                else
                {
                    return Err(error_in!(loc, "Invalid character '{}' in storage specifier!", spec.chars().nth(3).unwrap()));
                }
            }
        
            return Ok(EvalStorage::Location(location));

        }
  
        let register = match spec.to_ascii_uppercase().as_str()
        {
            "RA"  => Some( IRRegister::RA  ),
            "RB"  => Some( IRRegister::RB  ),
            "RC"  => Some( IRRegister::RC  ),
            "RD"  => Some( IRRegister::RD  ),
            "R1"  => Some( IRRegister::R1  ),
            "R2"  => Some( IRRegister::R2  ),
            "R3"  => Some( IRRegister::R3  ),
            "R4"  => Some( IRRegister::R4  ),
            "R5"  => Some( IRRegister::R5  ),
            "R6"  => Some( IRRegister::R6  ),
            "R7"  => Some( IRRegister::R7  ),
            "R8"  => Some( IRRegister::R8  ),
            "R9"  => Some( IRRegister::R9  ),
            "RZ"  => Some( IRRegister::RZ  ),
            "RIP" => Some( IRRegister::RIP ),
            "RSP" => Some( IRRegister::RSP ),
              _   => None,
        };

        if let Some(reg) = register
        {
            Ok(EvalStorage::Register(reg))
        }
        else
        {
            Err(error_in!(loc, "'{spec}' is not a valid storage specifier!"))
        }

    }
    fn eval_var(&mut self, scope: Rc<RefCell<Scope>>, stm: &StatementDefinitionVar, context: EvalVarDefContext) -> Result<EvalVariable, Error>
    {

        let name = stm.name.0.clone();
        let r#type = if let Some(t) = &stm.r#type { self.get_type(t, scope.clone())? } else { panic!() };
        let storage = Self::get_storage_specifier(&stm.storage.0, &stm.storage.1)?;
        let value = stm.value.clone();

        match context
        {
            EvalVarDefContext::StaticDef =>
            {
                if(stm.vtype == StatementDefinitionVarType::Let)
                {
                    return Err(error_in!((&stm.loc), "Cannot define register variable in static scope!"));
                }
                if(!matches!(storage, EvalStorage::Location(_)))
                {
                    return Err(error_in!((stm.loc.clone()), "Can only define static storages here!"))
                }
            },
            EvalVarDefContext::ParamDef =>
            {
                if(!matches!(storage, EvalStorage::Register(_)))
                {
                    return Err(error_in!((stm.loc.clone()), "Only register storage allowed in function parameters!"))
                }
            },
            EvalVarDefContext::LocalDef => 
            {
                if(stm.vtype != StatementDefinitionVarType::Let)
                {
                    return Err(error_in!((&stm.loc), "Cannot define static variable in function body!"));
                }
            },
        }

        Ok(EvalVariable
        {
            id: self.get_id(),
            unique_name: self.get_unique_name(scope.clone()),
            name,
            r#type,
            storage,
            value,
            initializer: None,
            constant: stm.vtype == StatementDefinitionVarType::Con,
            parent: scope.borrow().parent.clone().unwrap(),
        })

    }
    fn eval_func(&mut self, scope: Rc<RefCell<Scope>>, stm: &StatementDefinitionFunc) -> Result<EvalFunction, Error>
    {

        let name = stm.name.0.clone();
        let r#type = if let Some(t) = &stm.rtype { self.get_type(t, scope.clone())? } else { Rc::new(RefCell::new(EvalType::Internal(EvalTypeInternal::Never))) };

        let params = stm.params
            .iter()
            .map(|p|self.eval_var(scope.clone(), p, EvalVarDefContext::ParamDef))
            .collect::<Result<Vec<EvalVariable>, Error>>()?;

        Ok(EvalFunction
        {
            id: self.get_id(),
            unique_name: self.get_unique_name(scope.clone()),
            name,
            r#type,
            params,
            parent: scope.borrow().parent.clone().unwrap(),
            scope: scope.clone()
        })

    }
    pub fn eval_scope(&mut self, scope: &Rc<RefCell<Scope>>) -> Result<(), Error>
    {
        
        let parent_is_static_scope = match scope.borrow().r#type
        {
            ScopeType::None => unreachable!(),
            ScopeType::Sequential => false,
            ScopeType::Static     => true,
        };
        
        let len = scope.borrow().children.len();
        for i in 0..len
        {
    
            let b = scope.borrow();
            let c = &b.children[i];
            let a = c.borrow();
            let stm = match &a.statement
            {
                Some(s) => s,
                None => return Err(error!("FATAL: somehow empty statement in scope {:?}", a)),
            };
            
            match a.r#type
            {
                
                ScopeType::None =>
                {
                    
                    match stm
                    {
                        Statement::Return(_, r)     => 
                        {
                            
                            if(parent_is_static_scope)
                            {
                                return Err(error_in!((&r.loc), "Expressions are not allowed in static scopes!"));
                            }

                            let func = Self::eval_get_parent_func(c.clone(), r.loc.clone())?;
                            drop(a);
                            c.borrow_mut().self_function = Some(func);

                        },
                        Statement::Expression(_, e) => 
                        {
                            if(parent_is_static_scope)
                            {
                                return Err(error_in!((e.loc()), "Expressions are not allowed in static scopes!"));
                            }
                        },
                        Statement::DefinitionVar(_, s) => 
                        {
                            let context = 
                                if(parent_is_static_scope)
                                     { EvalVarDefContext::StaticDef }
                                else { EvalVarDefContext::LocalDef  };
                            let v = self.eval_var(c.clone(), s, context)?;
                            let v0 = Rc::new(RefCell::new(v));
                            drop(a);
                            c.borrow_mut().self_variable = Some(v0.clone());
                            drop(b);
                            scope.borrow_mut().variables.push(v0);
                        },
                        _ => unreachable!()
                    }

                },
                
                ScopeType::Sequential =>
                {

                    match stm
                    {
                        Statement::DefinitionFunc(m, s) =>
                        {

                            let mut set_entry_point = false;

                            for m in m
                            {
                                match m.name.as_str()
                                {
                                    "entry" => 
                                    {
                                        if let Some(e) = &self.entry_point
                                        {
                                            let loc = e.borrow().scope.borrow().statement.as_ref().unwrap().loc();
                                            return Err(error_in!((&m.loc), "Redefinition of entry point! (Already defined here: {loc})"));
                                        }
                                        set_entry_point = true;
                                    },
                                    _ => return Err(error_in!((&m.loc), "Unknown modifier '{}'!", m.name)),
                                }
                            }

                            let func = self.eval_func(c.clone(), s)?;
                            let f = Rc::new(RefCell::new(func));
                            drop(a);
                            c.borrow_mut().self_function = Some(f.clone());
                            drop(b);
                            if(set_entry_point) { self.entry_point = Some(f.clone()); }
                            scope.borrow_mut().functions.push(f);

                        },
                        Statement::Conditional(_, _) => { drop(a); drop(b); },
                        _ => unreachable!()
                    }

                    let child = scope.borrow().children[i].clone();
                    self.eval_scope(&child)?;
                    
                },

                _ => unreachable!(),

            }

        }

        Ok(())

    }
    
    fn eval_get_parent_func(scope: Rc<RefCell<Scope>>, l: Location) -> Result<Rc<RefCell<EvalFunction>>, Error>
    {

        let mut scope = scope;

        loop
        {

            let c = scope.borrow();
            let stm = c.statement.as_ref().unwrap();

            if matches!(stm, Statement::DefinitionFunc(_, _))
            {
                return Ok(c.self_function.as_ref().unwrap().clone());
            }

            let par = c.parent.clone();
            drop(c);
            match par
            {
                Some(p) => scope = p.upgrade().unwrap(),
                None => break,
            }

        }

        Err(error_in!(l, "Could not find a parent function!"))

    }
    fn eval_step_handle_initializers(scope: Rc<RefCell<Scope>>) -> Result<(), Error>
    {

        let stype = scope.borrow().r#type;
        match stype
        {
            ScopeType::None => Ok(()),
            ScopeType::Sequential | ScopeType::Static =>
            {

                if(stype == ScopeType::Static)
                {
                    for v in &scope.borrow().variables
                    {

                        let var = v.borrow();
                        let value = var.value.as_ref().unwrap();
                        
                        let init = if let Some(init) = &var.initializer { init.clone() }
                            else { Self::eval_expression(value, scope.clone())? };

                        if(!init.is_const())
                        {
                            return Err(error_in!((value.loc()), "Variable '{}' is expected to be constant but its initializer is not!", var.name));
                        }

                        drop(var);
                        v.borrow_mut().initializer = Some(init);

                    }
                }

                for c in &scope.borrow().children
                {
                    Self::eval_step_handle_initializers(c.clone())?;
                }
                
                Ok(())

            },
        }

    }
    fn eval_step_calc_initializers(scope: Rc<RefCell<Scope>>) -> Result<(), Error>
    {

        for v in &scope.borrow().variables
        {

            let init = v.borrow().initializer.clone();

            let init = Some(
                if let Some(init) = &init
                {
                    Self::calc_const_expr(init, 0, init.loc())?
                }
                else if let Some(value) = &v.borrow().value
                {
                    Self::eval_expression(value, scope.clone())?
                }
                else
                {
                    v.borrow().r#type.borrow().default_value()
                }
            );
            v.borrow_mut().initializer = init;

        }

        for c in &scope.borrow().children
        {
            Self::eval_step_calc_initializers(c.clone())?;
        }
        
        Ok(())

    }
    
    fn ir_get_prefix(scope: Rc<RefCell<Scope>>) -> String
    {
        let sc = scope.borrow();
        let id = sc.id;
        let name = match sc.statement.as_ref()
        {
            Some(s) => match s
            {
                Statement::Return     (_, _) => "ret".to_string(),
                Statement::Expression (_, _) => "exp".to_string(),
                Statement::Conditional(_, c) =>
                {
                    match c.r#type
                    {
                        StatementConditionalType::If    => "if"   .to_string(),
                        StatementConditionalType::Else  => "else" .to_string(),
                        StatementConditionalType::While => "while".to_string(),
                    }
                },
                Statement::DefinitionStruct (_, s) => format!("s_{}", s.name.0),
                Statement::DefinitionFunc   (_, f) => format!("f_{}", f.name.0),
                Statement::DefinitionVar    (_, v) => format!("v_{}", v.name.0),
            },
            None => String::new(),
        };
        let par = &sc.parent;
        if(par.is_none())
        {
            name
        }
        else
        {
            format!("{}___{name}_{id:#06x}", Self::ir_get_prefix(par.as_ref().unwrap().upgrade().unwrap()))
        }
    }
    pub fn ir_steps_sequ_scope(&mut self, scope: Rc<RefCell<Scope>>) -> Result<Vec<step::CompleteStep>, Error>
    {

        let sc = scope.borrow();

        let mut statements: Vec<step::CompleteStep> = Vec::new();

        let mut __cond: Option<StatementConditional> = None;
        
        let len = sc.children.len();
        for i in 0..len
        {
    
            
            let c = &sc.children[i];
            let a = c.borrow();
            let stm = a.statement.as_ref().unwrap();

            let mut __cond_else: Option<StatementConditional> = None;

            match stm
            {
                Statement::Expression(_, e) =>
                {
                    let val = Self::eval_expression(e, c.clone())?;
                    let steps = step::value_to_steps(&val, true, true)?;
                    statements.extend(steps);
                },
                Statement::Return(_, r) =>
                {

                    if(a.self_function.as_ref().unwrap().borrow().r#type.borrow().is_internal(&EvalTypeInternal::Never) && r.value.is_some())
                    {
                        return Err(error_in!((&r.loc), "Function does not have a return type!"));
                    }

                    if let Some(value) = &r.value
                    {
                        let val = Self::eval_expression(value, c.clone())?;
                        let typed = Self::eval_type_check(&val)?;

                        if(!typed.type_or_trait.equ(&TypeOrTrait::Type(a.self_function.as_ref().unwrap().borrow().r#type.clone())))
                        {
                            return Err(error_in!((&r.loc), "Return value does not fit function type!"));
                        }

                        let steps = step::value_to_steps(&val, false, false)?;
                        let str = steps.last().unwrap().0.clone();

                        statements.extend(step::filter_steps(steps));
                        statements.push((step::Storage::REG(IRRegister::R9), step::Step::Assignment(str)));
                    }
                    else if(
                        !a.self_function.as_ref().unwrap().borrow().r#type.borrow().is_internal(&EvalTypeInternal::Void ) && 
                        !a.self_function.as_ref().unwrap().borrow().r#type.borrow().is_internal(&EvalTypeInternal::Never)
                    )
                    {
                        return Err(error_in!((&r.loc), "Function expected return value!"));
                    }
                    statements.push((step::Storage::TMP(0), step::Step::Instruction(IIRStatement::Instruction(IIRInstruction::RET))));

                },
                Statement::DefinitionVar(_, _) =>
                {

                    let v = a.self_variable.as_ref().unwrap();

                    match v.borrow().storage
                    {
                        EvalStorage::Register(_) => {},
                        _ => return Err(error_in!((stm.loc()), "Local var can only be stored in register!")),
                    }

                    let init =  if let Some(init) = &v.borrow().initializer
                    {
                        init.clone()
                    }
                    else
                    {
                        let init = Self::eval_expression(v.borrow().value.as_ref().unwrap(), c.clone())?;
                        v.borrow_mut().initializer = Some(init.clone());
                        init
                    };
                    
                    let steps = step::value_to_steps(&init, false, false)?;
                    let str = steps.last().unwrap().0.clone();
                    statements.extend(step::filter_steps(steps));
                    statements.push((step::Storage::LET(v.clone()), step::Step::Assignment(str)));

                },
                Statement::Conditional(_, c) =>
                {
                    match c.r#type
                    {
                        StatementConditionalType::Else => __cond_else = Some(c.clone()),
                        _ => __cond = Some(c.clone()),
                    }
                },
                _ => todo!(),
            }

            if let Some(cond) = &__cond
            {

                let else_mode = __cond_else.is_some();
                let loop_type = matches!(cond.r#type, StatementConditionalType::While);
            
                let _name = self.get_unique_name(a.parent.as_ref().unwrap().upgrade().as_ref().unwrap().clone());
                let name1 = format!("{_name}__{}{}", if(loop_type) { "while" } else { "if" }, self.get_id());
                let name2 = format!("{_name}__else{}", self.get_id());


                let val = Self::eval_expression(&cond.condition, c.clone())?;
                let typed = Self::eval_type_check(&val)?;

                if(!typed.type_or_trait.equ(&TypeOrTrait::Trait(EvalTypeTrait::Boolean)))
                {
                    return Err(error_in!((cond.condition.loc()), "Expected boolean as conditional selector!"));
                }

                let steps = step::value_to_steps(&val, false, false)?;

                let cond_pos = steps.last().unwrap().0.clone();

                let steps = step::filter_steps(steps);

                statements.push((step::Storage::TMP(0), step::Step::Instruction(IIRStatement::Label(format!("{name1}__start")))));

                statements.extend(steps);

                statements.push((step::Storage::REG(IRRegister::R9), step::Step::Assignment(cond_pos)));
                statements.push((step::Storage::TMP(0), step::Step::Instruction(IIRStatement::Instruction(
                    IIRInstruction::ALU(IIRALUInstruction::Simple(_IIRALUInstruction2::CMP((
                        IIRInstructionModifier::Register(IRRegister::R9),
                        IIRInstructionModifier::Register(IRRegister::RZ)
                    ))))
                ))));
                statements.push((step::Storage::TMP(0), step::Step::Instruction(IIRStatement::Instruction(
                    IIRInstruction::JIF(IIRInstructionModifier::Immediate(IIRImmediate::Label(
                        if(else_mode)
                        {
                            format!("{name2}__start")
                        }
                        else
                        {
                            format!("{name1}__end")
                        }
                    )), FLAG_E)
                ))));

                statements.extend(self.ir_steps_sequ_scope(c.clone())?);

                if(loop_type)
                {
                    statements.push((step::Storage::TMP(0), step::Step::Instruction(IIRStatement::Instruction(
                        IIRInstruction::JMP(IIRInstructionModifier::Immediate(IIRImmediate::Label(format!("{name1}__start"))))
                    ))));
                }

                statements.push((step::Storage::TMP(0), step::Step::Instruction(IIRStatement::Label(format!("{name1}__end")))));

                __cond = None;

            }

        }

        Ok(statements)

    }

    

    pub fn eval_type_check(exp: &EvalValue) -> Result<EvalValueTyped, Error>
    {

        let val = match exp
        {

            EvalValue::Boolean (_, _) => EvalValueTyped::from_trait(exp.clone(), EvalTypeTrait::Boolean),
            EvalValue::Number  (_, _) => EvalValueTyped::from_trait(exp.clone(), EvalTypeTrait::Number ),
            EvalValue::String  (_, _) => todo!(),

            EvalValue::Symbol(s, _) =>
            {
                match s
                {
                    EvalSymbol::Type(t) => EvalValueTyped::from_type(exp.clone(), t.clone()),
                    EvalSymbol::Function(_) => todo!(),
                    EvalSymbol::Variable(v) => EvalValueTyped::from_type(exp.clone(), v.borrow().r#type.clone()),
                }
            },

            EvalValue::Complex(c, l) => 
            {
                match c
                {
                    EvalValueComplex::OpUnary(op, a) =>
                    {

                        let _trait = match op
                        {
                            | EvalValueComplexOpUnary::POS
                            | EvalValueComplexOpUnary::NEG
                            | EvalValueComplexOpUnary::INV
                                => Some(EvalTypeTrait::Number),
                            EvalValueComplexOpUnary::NOT
                                => Some(EvalTypeTrait::Boolean),
                            _ => None,
                        };

                        if let Some(_trait) = _trait
                        {
                            let res = Self::eval_type_check(a)?;
                            if(!res.type_or_trait.equ(&TypeOrTrait::Trait(_trait)))
                            {
                                return Err(error_in!((l), "Expected {:?}!", _trait));
                            }
                            EvalValueTyped::from_either(exp.clone(), res.type_or_trait)
                        }
                        else
                        {
                            match op
                            {
                                EvalValueComplexOpUnary::PTR =>
                                {
                                    if let EvalValue::Symbol(EvalSymbol::Type(t), _) = &**a
                                    {
                                        let t = EvalType::Pointer(t.clone());
                                        EvalValueTyped::from_type(exp.clone(), Rc::new(RefCell::new(t)))
                                    }
                                    else
                                    {
                                        return Err(error_in!((a.loc()), "Expected to be a type!"));
                                    }
                                },
                                EvalValueComplexOpUnary::REF => todo!(),
                                _ => unreachable!()
                            }
                        }
                    },
                    EvalValueComplex::OpBinary(op, a, b) =>
                    {

                        let (in_trait, out_trait) = match op
                        {
                            | EvalValueComplexOpBinary::ADD
                            | EvalValueComplexOpBinary::SUB
                            | EvalValueComplexOpBinary::MUL
                            | EvalValueComplexOpBinary::DIV
                            | EvalValueComplexOpBinary::MOD
                            | EvalValueComplexOpBinary::AND
                            | EvalValueComplexOpBinary::OR 
                            | EvalValueComplexOpBinary::XOR
                            | EvalValueComplexOpBinary::SHL
                            | EvalValueComplexOpBinary::SHR
                                => (EvalTypeTrait::Number, EvalTypeTrait::Number),

                            | EvalValueComplexOpBinary::EQU
                            | EvalValueComplexOpBinary::NEQU
                            | EvalValueComplexOpBinary::LT
                            | EvalValueComplexOpBinary::LTE
                            | EvalValueComplexOpBinary::GT
                            | EvalValueComplexOpBinary::GTE
                                => (EvalTypeTrait::Number, EvalTypeTrait::Boolean),

                            | EvalValueComplexOpBinary::BAND
                            | EvalValueComplexOpBinary::BOR
                                => (EvalTypeTrait::Boolean, EvalTypeTrait::Boolean),

                            _ => todo!()
                        };

                        let res_a = Self::eval_type_check(a)?;
                        let res_b = Self::eval_type_check(b)?;
                        if(!res_a.type_or_trait.equ(&TypeOrTrait::Trait(in_trait)))
                        {
                            return Err(error_in!((a.loc()), "Expected {:?}!", in_trait));
                        }
                        if(!res_b.type_or_trait.equ(&TypeOrTrait::Trait(in_trait)))
                        {
                            return Err(error_in!((b.loc()), "Expected {:?}!", in_trait));
                        }
                        
                        let out = TypeOrTrait::presedence(&res_a.type_or_trait, &res_b.type_or_trait);

                        if(!out.equ(&TypeOrTrait::Trait(out_trait)))
                        {
                            return Err(error_in!((l), "Resulting Type does not match expected Trait!"));
                        }

                        EvalValueTyped::from_either(exp.clone(), out.clone())
                            
                    },
                    _ => todo!()
                }
            },

        };

        Ok(val)

    }
    
    fn calc_const_expr(exp: &EvalValue, depth: u32, depth_start: Location) -> Result<EvalValue, Error>
    {

        if(!exp.is_const())
        {
            return Err(error_in!((exp.loc()), "Expected constant expression!"));
        }

        let threshold = 200;
        if(depth >= threshold)
        {
            return Err(error_in!((depth_start), "Exceeded {threshold} iterations at constant evalutation! (Maybe you have a circular dependency?)"))
        }

        match exp
        {
            | EvalValue::String  (_, _) 
            | EvalValue::Number  (_, _)
            | EvalValue::Boolean (_, _)
                => Ok(exp.clone()),
            EvalValue::Symbol(s, _) =>
            {
                match s
                {
                    | EvalSymbol::Type     (_)
                    | EvalSymbol::Function (_)
                        => Ok(exp.clone()),
                    EvalSymbol::Variable(v) =>
                    {
                        let var = v.borrow();
                        let val = if let Some(init) = &var.initializer
                        {
                            match init
                            {
                                | EvalValue::String  (_, _) 
                                | EvalValue::Number  (_, _)
                                | EvalValue::Boolean (_, _)
                                    => init.clone(),
                                _ =>
                                {
                                    let init = init.clone();
                                    drop(var);
                                    Self::calc_const_expr(&init, depth + 1, depth_start)?
                                },
                            }
                        }
                            else { var.r#type.borrow().default_value() };
                        Ok(val)
                    },
                }
            },
            EvalValue::Complex(c, _) => 
            {
                match c
                {
                    EvalValueComplex::FunctionCall(_, _) => todo!(), //FUTURE: handle constant function calls
                    EvalValueComplex::OpUnary(op, v) =>
                    {
                        let val = Self::calc_const_expr(v, depth + 1, depth_start)?;
                        let val = match op
                        {
                            EvalValueComplexOpUnary::POS =>
                            { 
                                if(!matches!(val, EvalValue::Number(_, _)))
                                {
                                    return Err(error_in!((val.loc()), "Is not a number!"));
                                }
                                val
                            },
                            EvalValueComplexOpUnary::NEG =>
                            { 
                                let (v,l) = match val
                                {
                                    EvalValue::Number(n, l) => (n,l),
                                    _ => return Err(error_in!((val.loc()), "Is not a number!")),
                                };
                                let v = (-(v as i64)) as u32;
                                EvalValue::Number(v, l)
                            },
                            EvalValueComplexOpUnary::INV =>
                            { 
                                let (v,l) = match val
                                {
                                    EvalValue::Number(n, l) => (n,l),
                                    _ => return Err(error_in!((val.loc()), "Is not a number!")),
                                };
                                let v = !v;
                                EvalValue::Number(v, l)
                            },
                            EvalValueComplexOpUnary::NOT =>
                            { 
                                let (v,l) = match val
                                {
                                    EvalValue::Boolean(b, l) => (b,l),
                                    _ => return Err(error_in!((val.loc()), "Is not a boolean!")),
                                };
                                let v = !v;
                                EvalValue::Boolean(v, l)
                            },
                            EvalValueComplexOpUnary::PTR =>
                            { 
                                let (t, l) = if let EvalValue::Symbol(EvalSymbol::Type(t), l) = val
                                {
                                    (t, l)
                                }
                                else
                                {
                                    return Err(error_in!((val.loc()), "Is not a type! {:?}", exp));
                                };
                                EvalValue::Symbol(EvalSymbol::Type(Rc::new(RefCell::new(EvalType::Pointer(t)))), l)
                            },
                            EvalValueComplexOpUnary::REF =>
                            {
                                let (v, l) = if let EvalValue::Symbol(EvalSymbol::Variable(v), l) = &**v
                                {
                                    (v, l)
                                }
                                else
                                {
                                    return Err(error_in!((exp.loc()), "Is not a variable!"));
                                };
                                if(!matches!(v.borrow().storage, EvalStorage::Location(_)))
                                {
                                    return Err(error_in!((exp.loc()), "Can only take a reference to a location in ram!"));
                                }
                                EvalValue::Complex(EvalValueComplex::Reference(Rc::downgrade(v)), l.clone())
                            },
                        };
                        Ok(val)
                    },
                    EvalValueComplex::OpBinary(op, _a, _b) =>
                    {
                        let a = Self::calc_const_expr(_a, depth + 1, depth_start.clone())?;
                        let b = Self::calc_const_expr(_b, depth + 1, depth_start.clone())?;
                        let val = match op
                        {
                            | EvalValueComplexOpBinary::ADD
                            | EvalValueComplexOpBinary::SUB
                            | EvalValueComplexOpBinary::MUL
                            | EvalValueComplexOpBinary::DIV
                            | EvalValueComplexOpBinary::MOD
                            | EvalValueComplexOpBinary::AND
                            | EvalValueComplexOpBinary::OR
                            | EvalValueComplexOpBinary::XOR
                            | EvalValueComplexOpBinary::SHL
                            | EvalValueComplexOpBinary::SHR
                                =>
                            {
                                let (a, l) = match a
                                {
                                    EvalValue::Number(n, l) => (n, l),
                                    _ => return Err(error_in!((_a.loc()), "Is not a number!")),
                                };
                                let b = match b
                                {
                                    EvalValue::Number(n, _) => n,
                                    _ => return Err(error_in!((_b.loc()), "Is not a number! {b:?}")),
                                };
                                let v = match op
                                {
                                    EvalValueComplexOpBinary::ADD => a.overflowing_add(b).0,
                                    EvalValueComplexOpBinary::SUB => a.overflowing_sub(b).0,
                                    EvalValueComplexOpBinary::MUL => a.overflowing_mul(b).0,
                                    EvalValueComplexOpBinary::DIV => 
                                    {
                                        if let Some(n) = a.checked_div(b) { n }
                                        else { return Err(error_in!((_b.loc()), "Second operand cannot be zero!")) }
                                    },
                                    EvalValueComplexOpBinary::MOD =>
                                    {
                                        if(b == 0) { return Err(error_in!((_b.loc()), "Second operand cannot be zero!")) }
                                        a % b
                                    },
                                    EvalValueComplexOpBinary::AND => a & b,
                                    EvalValueComplexOpBinary::OR  => a | b,
                                    EvalValueComplexOpBinary::XOR => a ^ b,
                                    EvalValueComplexOpBinary::SHL => a.overflowing_shl(b).0,
                                    EvalValueComplexOpBinary::SHR => a.overflowing_shr(b).0,
                                    _ => unreachable!()
                                };
                                EvalValue::Number(v, l)
                            },
                            | EvalValueComplexOpBinary::EQU
                            | EvalValueComplexOpBinary::NEQU
                            | EvalValueComplexOpBinary::LT
                            | EvalValueComplexOpBinary::LTE
                            | EvalValueComplexOpBinary::GT
                            | EvalValueComplexOpBinary::GTE
                                =>
                            {
                                let (a,l) = match a
                                {
                                    EvalValue::Number(n, l) => (n,l),
                                    _ => return Err(error_in!((a.loc()), "Is not a number!")),
                                };
                                let b = match b
                                {
                                    EvalValue::Number(n, _) => n,
                                    _ => return Err(error_in!((b.loc()), "Is not a number!")),
                                };
                                let v = match op
                                {
                                    EvalValueComplexOpBinary::EQU  => a == b,
                                    EvalValueComplexOpBinary::NEQU => a != b,
                                    EvalValueComplexOpBinary::LT   => a <  b,
                                    EvalValueComplexOpBinary::LTE  => a <= b,
                                    EvalValueComplexOpBinary::GT   => a >  b,
                                    EvalValueComplexOpBinary::GTE  => a >= b,
                                    _ => unreachable!()
                                };
                                EvalValue::Boolean(v, l)
                            },
                            | EvalValueComplexOpBinary::BAND
                            | EvalValueComplexOpBinary::BOR
                                =>
                            {
                                let (a,l) = match a
                                {
                                    EvalValue::Boolean(n, l) => (n,l),
                                    _ => return Err(error_in!((a.loc()), "Is not a boolean!")),
                                };
                                let b = match b
                                {
                                    EvalValue::Boolean(n,_l) => n,
                                    _ => return Err(error_in!((b.loc()), "Is not a boolean!")),
                                };
                                let v = match op
                                {
                                    EvalValueComplexOpBinary::BAND => a && b,
                                    EvalValueComplexOpBinary::BOR  => a || b,
                                    _ => unreachable!()
                                };
                                EvalValue::Boolean(v, l)
                            },
                            EvalValueComplexOpBinary::ASSIGN(_) => return Err(error_in!((exp.loc()), "Assignments are not constant!")),
                            EvalValueComplexOpBinary::NONE => return Err(error_in!((exp.loc()), "NONE OPERATION")),
                        };
                        Ok(val)
                    },
                    EvalValueComplex::Reference(_) => todo!(),
                }
            },
        }

    }

    fn value_fits_type(_value: &EvalValue, _vtype: &EvalType) -> bool { true }

    pub fn eval_expression(exp: &StatementExpression, scope: Rc<RefCell<Scope>>) -> Result<EvalValue, Error>
    {
        match exp
        {
            StatementExpression::Literal(l) =>
            {

                if(l.value == "true" || l.value == "false")
                {
                    Ok(EvalValue::Boolean(l.value == "true", l.loc.clone()))
                }
                else if(l.value.chars().nth(0).unwrap().is_numeric())
                {

                    let mut num = l.value.as_str();

                    let base = 
                        if(l.value.chars().nth(0).unwrap() == '0' && l.value.len() > 2)
                        {
                            if(l.value.chars().nth(1).unwrap().is_numeric()) { 10 }
                            else
                            {
                                match l.value.chars().nth(1).unwrap()
                                {
                                    'x' => { num = &num[2..]; 16 },
                                    'o' => { num = &num[2..];  8 },
                                    'b' => { num = &num[2..];  2 },
                                        _  => 10,
                                }
                            }
                        }
                        else { 10 };
                    
                    let num = match u32::from_str_radix(num, base)
                    {
                        Ok(n) => n,
                        Err(_) => return Err(error_in!((&l.loc), "Could not parse number! [if this was not supposed to be a number, prefix it with a non-numeric character]")),
                    };

                    Ok(EvalValue::Number(num, l.loc.clone()))

                }
                else if(l.value.chars().nth(0).unwrap() == '"')
                {
                    Ok(EvalValue::String(l.value.as_str()[1..].to_string(), l.loc.clone()))
                }
                else
                {

                    if let Some(t) = EvalTypeInternal::get_type(&l.value)
                    {
                        return Ok(EvalValue::Symbol(EvalSymbol::Type(Rc::new(RefCell::new(EvalType::Internal(t)))), exp.loc()))
                    }
                    
                    let mut s = scope;

                    loop
                    {

                        //FUTURE: search for types

                        for f in &s.borrow().functions
                        {
                            if(f.borrow().name == l.value)
                            {
                                return Ok(EvalValue::Symbol(EvalSymbol::Function(f.clone()), l.loc.clone()))
                            }
                        }

                        for v in &s.borrow().variables
                        {
                            if(v.borrow().name == l.value)
                            {
                                return Ok(EvalValue::Symbol(EvalSymbol::Variable(v.clone()), l.loc.clone()))
                            }
                        }

                        let __s = s.borrow();
                        if(__s.parent.is_none()) { break; }
                        drop(__s);

                        let cell = s.borrow().parent.as_ref().unwrap().upgrade().unwrap();
                        s = cell;

                    }
                    
                    // nothing found
                    Err(error_in!((&l.loc), "The symbol '{}' doesnt exist!", l.value))

                }
                
            },
            StatementExpression::FunctionCall(f) =>
            {

                let _val = Self::eval_expression(&f.name, scope.clone())?;
                let _f = if let EvalValue::Symbol(EvalSymbol::Function(f), _) = _val
                    { f }
                else 
                {
                    //FUTURE: function call returns function?
                    return Err(error_in!((&f.loc), "'{:?}' is not a function!", f.name));
                };
                let func = _f.borrow();

                if(func.params.len() != f.args.len())
                {
                    let loc = match f.args.last()
                    {
                        Some(l) => l.loc(),
                        None    => f.loc.clone(),
                    };
                    return Err(error_in!(loc, "'{}' expects {} arguments, {} given!", func.name, func.params.len(), f.args.len()));
                }

                let mut values: Vec<EvalValue> = Vec::new();
                for (i, a) in f.args.iter().enumerate()
                {

                    let value = Self::eval_expression(a, scope.clone())?;

                    if(!Self::value_fits_type(&value, &func.params[i].r#type.borrow()))
                    {
                        return Err(error_in!((a.loc()), "'{}' expects argument {} to be of type {}!", func.name, i+1, func.params[i].r#type.borrow()));
                    }

                    values.push(value);

                }

                //FUTURE: function constant? args constant? eval at compilation

                Ok(EvalValue::Complex(EvalValueComplex::FunctionCall(Rc::downgrade(&_f), values), f.loc.clone()))

            },
            StatementExpression::Binary(b) =>
            {

                let op = match b.operator.as_str()
                {

                    "==" => EvalValueComplexOpBinary::EQU,
                    "!=" => EvalValueComplexOpBinary::NEQU,
                    "<"  => EvalValueComplexOpBinary::LT,
                    ">"  => EvalValueComplexOpBinary::GT,
                    "<=" => EvalValueComplexOpBinary::LTE,
                    ">=" => EvalValueComplexOpBinary::GTE,
                    "&&" => EvalValueComplexOpBinary::BAND,
                    "||" => EvalValueComplexOpBinary::BOR,

                    mut op => 
                    {

                        let assign_op = op.ends_with('=');
                        if(assign_op) { op = &op[0..op.len()-1]; }

                        let op = match op
                        {
                            ""   => EvalValueComplexOpBinary::NONE,
                            "+"  => EvalValueComplexOpBinary::ADD,
                            "-"  => EvalValueComplexOpBinary::SUB,
                            "*"  => EvalValueComplexOpBinary::MUL,
                            "/"  => EvalValueComplexOpBinary::DIV,
                            "%"  => EvalValueComplexOpBinary::MOD,
                            "&"  => EvalValueComplexOpBinary::AND,
                            "|"  => EvalValueComplexOpBinary::OR,
                            "^"  => EvalValueComplexOpBinary::XOR,
                            "<<" => EvalValueComplexOpBinary::SHL,
                            ">>" => EvalValueComplexOpBinary::SHR,
                            _ => unreachable!()
                        };

                        if(assign_op)
                        {
                            EvalValueComplexOpBinary::ASSIGN(Box::new(op))
                        }
                        else { op }
                        
                    },

                };

                let A = Self::eval_expression(&b.expr1, scope.clone())?;
                let B = Self::eval_expression(&b.expr2, scope.clone())?;

                Ok(EvalValue::Complex(EvalValueComplex::OpBinary(op, Box::new(A), Box::new(B)), b.loc.clone()))

            },
            StatementExpression::Unary(u) =>
            {

                let A = Self::eval_expression(&u.expr, scope)?;
                
                let op = match u.operator.as_str()
                {
                    "+" => EvalValueComplexOpUnary::POS, 
                    "-" => EvalValueComplexOpUnary::NEG,
                    "~" => EvalValueComplexOpUnary::INV, 
                    "!" => EvalValueComplexOpUnary::NOT,
                    "*" => EvalValueComplexOpUnary::PTR, 
                    "&" => EvalValueComplexOpUnary::REF,
                    _ => unreachable!()
                };

                Ok(EvalValue::Complex(EvalValueComplex::OpUnary(op, Box::new(A)), u.loc.clone()))

            },
            StatementExpression::ObjectAccess(_) => todo!(),
        }
    }

    pub fn generate_ir(&mut self) -> Result<Rc<RefCell<Scope>>, Error>
    {

        let globalScope = self.get_scope_tree()?;
        self.eval_scope(&globalScope)?;

        if let Some(e) = &self.entry_point
        {
            if(e.borrow().parent.upgrade().unwrap().borrow().id != globalScope.borrow().id)
            {
                return Err(error_in!((e.borrow().scope.borrow().statement.as_ref().unwrap().loc()), "Entry point may only be defined in global scope!"));
            }
        }
        else
        {
            return Err(error!("No entry point defined! [Hint: Put @entry before a function in global scope to set it as entry]"));
        }

        Self::eval_step_handle_initializers (globalScope.clone())?;
        Self::eval_step_calc_initializers   (globalScope.clone())?;

        Ok(globalScope)

    }

}
