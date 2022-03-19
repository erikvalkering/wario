use std::fmt;

pub struct Preamble {
    pub magic: [u8; 4],
    pub version: [u8; 4],
}

impl fmt::Debug for Preamble {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use std::str;
        f.debug_struct("Preamble")
            .field("magic", &str::from_utf8(&self.magic).unwrap())
            .field("version", &format!("{:?}", self.version))
            .finish()
    }
}

#[derive(Debug, Copy, Clone)]
pub enum ValueType {
    I32,
    I64,
    F32,
    F64,
}

pub struct FuncType {
    pub parameter_types: Vec<ValueType>,
    pub result_types: Vec<ValueType>,
}

impl fmt::Debug for FuncType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "FuncType: {:?} -> {:?}",
            self.parameter_types, self.result_types
        )
    }
}

pub struct TypeIdx(pub usize);
pub struct FuncIdx(pub usize);
pub struct TableIdx(pub usize);
pub struct MemIdx(pub usize);
pub struct GlobalIdx(pub usize);
pub struct LocalIdx(pub usize);
pub struct LabelIdx(pub usize);

impl fmt::Debug for TypeIdx {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "TypeIdx({:?})", self.0)
    }
}

impl fmt::Debug for FuncIdx {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "FuncIdx({:?})", self.0)
    }
}

impl fmt::Debug for TableIdx {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "TableIdx({:?})", self.0)
    }
}

impl fmt::Debug for MemIdx {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "MemIdx({:?})", self.0)
    }
}

impl fmt::Debug for GlobalIdx {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "GlobalIdx({:?})", self.0)
    }
}

impl fmt::Debug for LocalIdx {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "LocalIdx({:?})", self.0)
    }
}

impl fmt::Debug for LabelIdx {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "LabelIdx({:?})", self.0)
    }
}

#[derive(Debug)]
pub enum ElemType {
    FuncRef,
}

#[derive(Debug)]
pub struct Limits {
    pub min: u32,
    pub max: Option<u32>,
}

#[derive(Debug)]
pub struct TableType {
    pub elem_type: ElemType,
    pub limits: Limits,
}

#[derive(Debug)]
pub struct MemType {
    pub limits: Limits,
}

#[derive(Debug)]
pub enum Mutability {
    Constant,
    Variable,
}

#[derive(Debug)]
pub struct GlobalType {
    pub value_type: ValueType,
    pub mutability: Mutability,
}

#[derive(Debug)]
pub enum ImportDescriptor {
    Func(TypeIdx),
    Table(TableType),
    Memory(MemType),
    Global(GlobalType),
}

pub struct Name(pub String);

impl fmt::Debug for Name {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub struct Import {
    pub module: Name,
    pub name: Name,
    pub descriptor: ImportDescriptor,
}

impl fmt::Debug for Import {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{:?}.{:?}: {:?}",
            self.module, self.name, self.descriptor
        )
    }
}

#[derive(Debug)]
pub enum BlockType {
    Empty,
}

#[derive(Debug)]
pub struct MemArg {
    pub align: usize,
    pub offset: usize,
}

#[derive(Debug)]
pub enum Instruction {
    // Control instructions
    Unreachable,
    Block(BlockType, Vec<Instruction>),
    Loop(BlockType, Vec<Instruction>),
    If(BlockType, Vec<Instruction>, Vec<Instruction>),
    Branch(LabelIdx),
    BranchIf(LabelIdx),
    Return,
    Call(FuncIdx),

    // Variable instructions
    LocalGet(LocalIdx),
    LocalSet(LocalIdx),
    GlobalGet(GlobalIdx),
    GlobalSet(GlobalIdx),

    // Memory instructions
    I32Load(MemArg),
    I32Store(MemArg),

    // Numeric instructions
    I32Const(i32),
    F64Const(f64),
    I32Eq,
    I32GtSigned,
    F64Lt,
    F64Gt,
    F64Ge,
    I32Add,
    I32Sub,
    I32Mul,
    F64Add,
    F64Sub,
    F64Mul,
    F64Div,
}

#[derive(Debug)]
pub struct Global {
    pub global_type: GlobalType,
    pub expression: Vec<Instruction>,
}

#[derive(Debug)]
pub enum ExportDescriptor {
    Func(FuncIdx),
    Table(TableIdx),
    Memory(MemIdx),
    Global(GlobalIdx),
}

#[derive(Debug)]
pub struct Export {
    pub name: Name,
    pub descriptor: ExportDescriptor,
}

#[derive(Debug)]
pub struct Code {
    pub locals: Vec<ValueType>,
    pub body: Vec<Instruction>,
}

pub struct Locals {
    pub n: u32,
    pub t: ValueType,
}

#[derive(Debug)]
pub enum Section {
    Custom,
    Type(Vec<FuncType>),
    Import(Vec<Import>),
    Function(Vec<TypeIdx>),
    Table,
    Memory(Vec<Limits>),
    Global(Vec<Global>),
    Export(Vec<Export>),
    Start,
    Element,
    Code(Vec<Code>),
    Data,
}

#[derive(Debug)]
pub struct Module {
    pub preamble: Preamble,
    pub types: Vec<FuncType>,
    pub imports: Vec<Import>,
    pub functions: Vec<TypeIdx>,
    pub memories: Vec<Limits>,
    pub globals: Vec<Global>,
    pub exports: Vec<Export>,
    pub codes: Vec<Code>,
    // TODO: Add funcs component (see section 2.5.3 from spec)
}
