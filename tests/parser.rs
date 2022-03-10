use std::convert::TryInto;
use std::fmt;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};

#[derive(Debug)]
enum ParseErr {
    Err(String),
    Eof,
}

type ParseResult<T> = std::result::Result<T, ParseErr>;
type Result<T> = std::result::Result<T, String>;

trait Parse: Sized {
    fn parse(file: &mut File) -> ParseResult<Self>;
}

impl<T: Parse> Parse for Vec<T> {
    fn parse(file: &mut File) -> ParseResult<Self> {
        let n = u32::parse(file)?;
        // println!("vec len: {}", n);

        let mut result_type = vec![];
        for _ in 0..n {
            result_type.push(T::parse(file)?);
        }

        Ok(result_type)
    }
}

impl<const SIZE: usize> Parse for [u8; SIZE] {
    fn parse(file: &mut File) -> ParseResult<Self> {
        let mut buf = [0; SIZE];

        match file.read(&mut buf) {
            Err(err) => Err(ParseErr::Err(format!("Unable to read data: {}", err))),
            Ok(s) if s == SIZE => Ok(buf),
            Ok(0) => Err(ParseErr::Eof),
            Ok(s) => Err(ParseErr::Err(format!(
                "Unable to read data: expected size to be read: {} actual size read: {}",
                SIZE, s
            ))),
        }
    }
}

impl Parse for u8 {
    fn parse(file: &mut File) -> ParseResult<Self> {
        Ok(<[u8; 1]>::parse(file)?[0])
    }
}

fn parse_leb128_u32(file: &mut File) -> ParseResult<u32> {
    let mut result = 0u32;

    let mut shift = 0;
    loop {
        let value = u8::parse(file)?;

        result |= (value as u32 & 0x7f) << shift;

        if value & 0x80 == 0 {
            break;
        }

        shift += 7;
    }

    Ok(result)
}

fn parse_leb128_i32(file: &mut File) -> ParseResult<i32> {
    let mut result = 0i32;

    let mut value;
    let mut shift = 0;
    loop {
        value = u8::parse(file)?;

        result |= (value as i32 & 0x7f) << shift;

        if value & 0x80 == 0 {
            break;
        }

        shift += 7;
    }

    if value & 0x40 != 0 {
        result |= !0 << shift;
    }

    Ok(result)
}

impl Parse for u32 {
    fn parse(file: &mut File) -> ParseResult<Self> {
        parse_leb128_u32(file)
    }
}

impl Parse for i32 {
    fn parse(file: &mut File) -> ParseResult<Self> {
        parse_leb128_i32(file)
    }
}

impl Parse for f64 {
    fn parse(file: &mut File) -> ParseResult<Self> {
        Ok(f64::from_le_bytes(
            <[u8; std::mem::size_of::<f64>()]>::parse(file)?,
        ))
    }
}

struct Preamble {
    magic: [u8; 4],
    version: [u8; 4],
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

impl Preamble {
    fn parse(file: &mut File) -> ParseResult<Preamble> {
        let magic = <[u8; 4]>::parse(file)?;
        if &magic != b"\0asm" {
            return Err(ParseErr::Err("Invalid magic value".to_owned()));
        }

        let version = <[u8; 4]>::parse(file)?;
        if version != [1, 0, 0, 0] {
            return Err(ParseErr::Err("Invalid version".to_owned()));
        };

        Ok(Preamble {
            magic: magic.try_into().unwrap(),
            version: version.try_into().unwrap(),
        })
    }
}

#[derive(Debug, Copy, Clone)]
enum ValueType {
    I32,
    I64,
    F32,
    F64,
}

impl Parse for ValueType {
    fn parse(file: &mut File) -> ParseResult<Self> {
        let value_type = u8::parse(file)?;

        match value_type {
            0x7f => Ok(ValueType::I32),
            0x7e => Ok(ValueType::I64),
            0x7d => Ok(ValueType::F32),
            0x7c => Ok(ValueType::F64),
            _ => Err(ParseErr::Err(format!(
                "Invalid value type encountered: {}",
                value_type
            ))),
        }
    }
}

struct FuncType {
    parameter_types: Vec<ValueType>,
    result_types: Vec<ValueType>,
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

impl Parse for FuncType {
    fn parse(file: &mut File) -> ParseResult<Self> {
        let marker = u8::parse(file)?;
        if marker != 0x60 {
            return Err(ParseErr::Err(format!(
                "Invalid marker found for FuncType: {}",
                marker
            )));
        }

        Ok(FuncType {
            parameter_types: Parse::parse(file)?,
            result_types: Parse::parse(file)?,
        })
    }
}

struct TypeIdx(u32);
struct FuncIdx(u32);
struct TableIdx(u32);
struct MemIdx(u32);
struct GlobalIdx(u32);
struct LocalIdx(u32);
struct LabelIdx(u32);

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

impl Parse for TypeIdx {
    fn parse(file: &mut File) -> ParseResult<Self> {
        Ok(Self(Parse::parse(file)?))
    }
}

impl Parse for FuncIdx {
    fn parse(file: &mut File) -> ParseResult<Self> {
        Ok(Self(Parse::parse(file)?))
    }
}

impl Parse for TableIdx {
    fn parse(file: &mut File) -> ParseResult<Self> {
        Ok(Self(Parse::parse(file)?))
    }
}

impl Parse for MemIdx {
    fn parse(file: &mut File) -> ParseResult<Self> {
        Ok(Self(Parse::parse(file)?))
    }
}

impl Parse for GlobalIdx {
    fn parse(file: &mut File) -> ParseResult<Self> {
        Ok(Self(Parse::parse(file)?))
    }
}

impl Parse for LocalIdx {
    fn parse(file: &mut File) -> ParseResult<Self> {
        Ok(Self(Parse::parse(file)?))
    }
}

impl Parse for LabelIdx {
    fn parse(file: &mut File) -> ParseResult<Self> {
        Ok(Self(Parse::parse(file)?))
    }
}

#[derive(Debug)]
enum ElemType {
    FuncRef,
}

impl Parse for ElemType {
    fn parse(file: &mut File) -> ParseResult<Self> {
        let result = match u8::parse(file)? {
            0x70 => Self::FuncRef,
            elem_type => return Err(ParseErr::Err(format!("Invalid ElemType: {}", elem_type))),
        };

        Ok(result)
    }
}

#[derive(Debug)]
struct Limits {
    min: u32,
    max: Option<u32>,
}

impl Parse for Limits {
    fn parse(file: &mut File) -> ParseResult<Self> {
        let has_max = u8::parse(file)? == 1;

        let result = Self {
            min: u32::parse(file)?,
            max: if has_max {
                Some(u32::parse(file)?)
            } else {
                None
            },
        };

        Ok(result)
    }
}

#[derive(Debug)]
struct TableType {
    elem_type: ElemType,
    limits: Limits,
}

impl Parse for TableType {
    fn parse(file: &mut File) -> ParseResult<Self> {
        let result = Self {
            elem_type: ElemType::parse(file)?,
            limits: Limits::parse(file)?,
        };

        Ok(result)
    }
}

#[derive(Debug)]
struct MemType {
    limits: Limits,
}

impl Parse for MemType {
    fn parse(file: &mut File) -> ParseResult<Self> {
        Ok(Self {
            limits: Limits::parse(file)?,
        })
    }
}

#[derive(Debug)]
enum Mutability {
    Constant,
    Variable,
}

impl Parse for Mutability {
    fn parse(file: &mut File) -> ParseResult<Self> {
        Ok(match u8::parse(file)? {
            0x00 => Self::Constant,
            0x01 => Self::Variable,
            mutability => return Err(ParseErr::Err(format!("Invalid mutability: {}", mutability))),
        })
    }
}

#[derive(Debug)]
struct GlobalType {
    value_type: ValueType,
    mutability: Mutability,
}

impl Parse for GlobalType {
    fn parse(file: &mut File) -> ParseResult<Self> {
        Ok(Self {
            value_type: ValueType::parse(file)?,
            mutability: Mutability::parse(file)?,
        })
    }
}

#[derive(Debug)]
enum ImportDescriptor {
    Func(TypeIdx),
    Table(TableType),
    Memory(MemType),
    Global(GlobalType),
}

impl Parse for ImportDescriptor {
    fn parse(file: &mut File) -> ParseResult<Self> {
        Ok(match u8::parse(file)? {
            0x00 => Self::Func(TypeIdx::parse(file)?),
            0x01 => Self::Table(TableType::parse(file)?),
            0x02 => Self::Memory(MemType::parse(file)?),
            0x03 => Self::Global(GlobalType::parse(file)?),
            id => {
                return Err(ParseErr::Err(format!(
                    "Invalid import descriptor type: {}",
                    id
                )))
            }
        })
    }
}

struct Name(String);

impl fmt::Debug for Name {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Parse for Name {
    fn parse(file: &mut File) -> ParseResult<Self> {
        let result = Parse::parse(file)?;

        let result = match String::from_utf8(result) {
            Ok(result) => result,
            Err(err) => return Err(ParseErr::Err(format!("Invalid UTF8 string: {}", err))),
        };

        Ok(Name(result))
    }
}

struct Import {
    module: Name,
    name: Name,
    descriptor: ImportDescriptor,
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

impl Parse for Import {
    fn parse(file: &mut File) -> ParseResult<Self> {
        Ok(Self {
            module: Parse::parse(file)?,
            name: Parse::parse(file)?,
            descriptor: Parse::parse(file)?,
        })
    }
}

#[derive(Debug)]
enum BlockType {
    Empty,
}

impl Parse for BlockType {
    fn parse(file: &mut File) -> ParseResult<Self> {
        let id = u8::parse(file)?;

        Ok(match id {
            0x40 => BlockType::Empty,
            _ => panic!("Unsupported blocktype: {}", id),
        })
    }
}

#[derive(Debug)]
enum Instruction {
    // Control instructions
    Unreachable,
    Block(BlockType, Expression),
    Loop(BlockType, Expression),
    BranchIf(LabelIdx),
    Return,
    Call(FuncIdx),

    // Variable instructions
    LocalGet(LocalIdx),
    LocalSet(LocalIdx),
    GlobalSet(GlobalIdx),

    // Numeric instructions
    I32Const(i32),
    F64Const(f64),
    I32GtSigned,
    I32Sub,
    F64Mul,
}

#[derive(Debug)]
struct Expression(Vec<Instruction>);

impl Parse for Expression {
    fn parse(file: &mut File) -> ParseResult<Self> {
        let mut result = vec![];


        loop {
            let opcode = u8::parse(file)?;

            let instruction = match opcode {
                0x0B => break,

                // Control instructions
                0x00 => Instruction::Unreachable,
                0x02 => Instruction::Block(Parse::parse(file)?, Parse::parse(file)?),
                0x03 => Instruction::Loop(Parse::parse(file)?, Parse::parse(file)?),
                0x0D => Instruction::BranchIf(Parse::parse(file)?),
                0x0F => Instruction::Return,
                0x10 => Instruction::Call(Parse::parse(file)?),

                // Variable instructions
                0x20 => Instruction::LocalGet(Parse::parse(file)?),
                0x21 => Instruction::LocalSet(Parse::parse(file)?),
                0x24 => Instruction::GlobalSet(Parse::parse(file)?),

                // Numeric instructions
                0x41 => Instruction::I32Const(Parse::parse(file)?),
                0x44 => Instruction::F64Const(Parse::parse(file)?),
                0x4A => Instruction::I32GtSigned,
                0x6B => Instruction::I32Sub,
                0xA2 => Instruction::F64Mul,

                _ => panic!(
                    "
                    Unsupported opcode found: {0:#04X} (stream pos = {1} ({1:#04X})).
                    Decoded instructions so far: {2:?}
                    ",
                    opcode,
                    file.stream_position().unwrap() - 1,
                    result,
                ),
            };

            result.push(instruction);
        }

        Ok(Self(result))
    }
}

#[derive(Debug)]
struct Global {
    global_type: GlobalType,
    expression: Expression,
}

impl Parse for Global {
    fn parse(file: &mut File) -> ParseResult<Self> {
        Ok(Self {
            global_type: Parse::parse(file)?,
            expression: Parse::parse(file)?,
        })
    }
}

#[derive(Debug)]
enum ExportDescriptor {
    Func(FuncIdx),
    Table(TableIdx),
    Memory(MemIdx),
    Global(GlobalIdx),
}

impl Parse for ExportDescriptor {
    fn parse(file: &mut File) -> ParseResult<Self> {
        Ok(match u8::parse(file)? {
            0x00 => Self::Func(Parse::parse(file)?),
            0x01 => Self::Table(Parse::parse(file)?),
            0x02 => Self::Memory(Parse::parse(file)?),
            0x03 => Self::Global(Parse::parse(file)?),
            id => {
                return Err(ParseErr::Err(format!(
                    "Invalid export descriptor type: {}",
                    id
                )))
            }
        })
    }
}

#[derive(Debug)]
struct Export {
    name: Name,
    descriptor: ExportDescriptor,
}

impl Parse for Export {
    fn parse(file: &mut File) -> ParseResult<Self> {
        Ok(Self {
            name: Parse::parse(file)?,
            descriptor: Parse::parse(file)?,
        })
    }
}

#[derive(Debug)]
struct Code {
    locals: Vec<ValueType>,
    body: Expression,
}

struct Locals {
    n: u32,
    t: ValueType,
}

impl Parse for Locals {
    fn parse(file: &mut File) -> ParseResult<Self> {
        Ok(Self {
            n: Parse::parse(file)?,
            t: Parse::parse(file)?,
        })
    }
}

impl Parse for Code {
    fn parse(file: &mut File) -> ParseResult<Self> {
        let _size = u32::parse(file)?;
        let start = file.stream_position().unwrap();

        let locals = Vec::<Locals>::parse(file)?
            .iter()
            .flat_map(|local| vec![local.t; local.n as usize])
            .collect();

        let body = Parse::parse(file)?;

        let stop = file.stream_position().unwrap();
        assert_eq!(_size, (stop - start) as u32);

        Ok(Self { locals, body })
    }
}

#[derive(Debug)]
enum Section {
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

impl Parse for Section {
    fn parse(file: &mut File) -> ParseResult<Self> {
        let id = u8::parse(file)?;
        let size = u32::parse(file)?;
        let start = file.stream_position().unwrap();

        let section = match id {
            00 => Section::Custom,
            01 => Section::Type(Parse::parse(file)?),
            02 => Section::Import(Parse::parse(file)?),
            03 => Section::Function(Parse::parse(file)?),
            04 => Section::Table,
            05 => Section::Memory(Parse::parse(file)?),
            06 => Section::Global(Parse::parse(file)?),
            07 => Section::Export(Parse::parse(file)?),
            08 => Section::Start,
            09 => Section::Element,
            10 => Section::Code(Parse::parse(file)?),
            11 => Section::Data,
            _ => return Err(ParseErr::Err(format!("Found unknown section id: {}", id))),
        };

        match section {
            Section::Type(_) => {}
            Section::Import(_) => {}
            Section::Function(_) => {}
            Section::Memory(_) => {}
            Section::Global(_) => {}
            Section::Export(_) => {}
            Section::Code(_) => {}
            _ => {
                file.seek(SeekFrom::Current(size as i64)).unwrap();
            }
        }

        let stop = file.stream_position().unwrap();

        assert_eq!(size, (stop - start) as u32);

        Ok(section)
    }
}

fn parse_sections(file: &mut File) -> Result<Vec<Section>> {
    let mut sections = Vec::new();

    loop {
        match Section::parse(file) {
            Ok(section) => sections.push(section),
            Err(ParseErr::Eof) => break,
            Err(ParseErr::Err(err)) => return Err(err),
        }
    }

    Ok(sections)
}

#[derive(Debug)]
struct Module {
    preamble: Preamble,
    types: Vec<FuncType>,
    imports: Vec<Import>,
    functions: Vec<TypeIdx>,
    memories: Vec<Limits>,
    globals: Vec<Global>,
    exports: Vec<Export>,
    codes: Vec<Code>,
}

impl Module {
    fn parse(file: &mut File) -> Result<Module> {
        let preamble = match Preamble::parse(file) {
            Ok(x) => x,
            Err(ParseErr::Err(err)) => return Err(err),
            Err(ParseErr::Eof) => return Err("Unexpected end of file detected".to_owned()),
        };

        let mut module = Module {
            preamble,
            types: vec![],
            imports: vec![],
            functions: vec![],
            memories: vec![],
            globals: vec![],
            exports: vec![],
            codes: vec![],
        };

        for section in parse_sections(file)? {
            match section {
                Section::Type(types) => module.types = types,
                Section::Import(imports) => module.imports = imports,
                Section::Function(functions) => module.functions = functions,
                Section::Memory(memories) => module.memories = memories,
                Section::Global(globals) => module.globals = globals,
                Section::Export(exports) => module.exports = exports,
                Section::Code(codes) => module.codes = codes,
                section => println!("Section {:?} not implemented yet, skipping", section),
            }
        }

        Ok(module)
    }
}

#[test]
fn parse_wasm() -> Result<()> {
    use std::path::PathBuf;
    let path: PathBuf = [env!("CARGO_MANIFEST_DIR"), "tests", "input.wasm"]
        .iter()
        .collect();

    let file = File::open(path);
    let mut file = match file {
        Ok(file) => file,
        Err(err) => return Err(format!("Unable to open file: {}", err)),
    };

    let module = Module::parse(&mut file)?;
    println!("{:#?}", module);

    Ok(())
}
