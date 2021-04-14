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

fn parse_leb128(file: &mut File) -> ParseResult<u32> {
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

impl Parse for u32 {
    fn parse(file: &mut File) -> ParseResult<Self> {
        parse_leb128(file)
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

#[derive(Debug)]
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

impl fmt::Debug for TypeIdx {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "TypeIdx({:?})", self.0)
    }
}

impl Parse for TypeIdx {
    fn parse(file: &mut File) -> ParseResult<Self> {
        Ok(Self(u32::parse(file)?))
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
enum Instruction {
    // TODO: add real instructions
    Nop,
}

#[derive(Debug)]
struct Global {
    global_type: GlobalType,
    expression: Vec<Instruction>,
}

impl Parse for Global {
    fn parse(file: &mut File) -> ParseResult<Self> {
        Ok(Self {
            global_type: Parse::parse(file)?,
            expression: {
                let mut expression = vec![];
                while u8::parse(file)? != 0x0B {
                    expression.push(Instruction::Nop)
                }
                expression
            },
        })
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
    Export,
    Start,
    Element,
    Code,
    Data,
}

impl Section {
    fn parse(file: &mut File) -> ParseResult<Section> {
        let id = u8::parse(file)?;
        let size = u32::parse(file)?;

        let section = match id {
            00 => Section::Custom,
            01 => Section::Type(Parse::parse(file)?),
            02 => Section::Import(Parse::parse(file)?),
            03 => Section::Function(Parse::parse(file)?),
            04 => Section::Table,
            05 => Section::Memory(Parse::parse(file)?),
            06 => Section::Global(Parse::parse(file)?),
            07 => Section::Export,
            08 => Section::Start,
            09 => Section::Element,
            10 => Section::Code,
            11 => Section::Data,
            _ => return Err(ParseErr::Err(format!("Found unknown section id: {}", id))),
        };

        match section {
            Section::Type(_) => {}
            Section::Import(_) => {}
            Section::Function(_) => {}
            Section::Memory(_) => {}
            Section::Global(_) => {}
            _ => {
                file.seek(SeekFrom::Current(size as i64)).unwrap();
            }
        }

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
    funcs: Vec<TypeIdx>,
    mems: Vec<Limits>,
    glob: Vec<Global>,
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
            funcs: vec![],
            mems: vec![],
            glob: vec![],
        };

        for section in parse_sections(file)? {
            match section {
                Section::Type(types) => module.types = types,
                Section::Import(imports) => module.imports = imports,
                Section::Function(funcs) => module.funcs = funcs,
                Section::Memory(mems) => module.mems = mems,
                Section::Global(glob) => module.glob = glob,
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
