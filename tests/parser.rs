use std::convert::TryInto;
use std::fs::File;
use std::io::Read;

#[derive(Debug)]
enum ParseErr {
    Err(String),
    Eof,
}

type ParseResult<T> = std::result::Result<T, ParseErr>;
type Result<T> = std::result::Result<T, String>;

fn parse_u8_array(file: &mut File, size: usize) -> ParseResult<Vec<u8>> {
    let mut buf = vec![0; size];

    match file.read(&mut buf) {
        Err(err) => Err(ParseErr::Err(format!("Unable to read data: {}", err))),
        Ok(s) if s == size => Ok(buf),
        Ok(0) => Err(ParseErr::Eof),
        Ok(s) => Err(ParseErr::Err(format!(
            "Unable to read data: expected size to be read: {} actual size read: {}",
            size, s
        ))),
    }
}

fn parse_u8(file: &mut File) -> ParseResult<u8> {
    Ok(parse_u8_array(file, 1)?[0])
}

fn parse_leb128(file: &mut File) -> ParseResult<u32> {
    let mut result = 0u32;

    let mut shift = 0;
    loop {
        let value = parse_u8(file)?;

        result |= (value as u32 & 0x7f) << shift;

        if value & 0x80 == 0 {
            break;
        }

        shift += 7;
    }

    Ok(result)
}

fn parse_u32(file: &mut File) -> ParseResult<u32> {
    parse_leb128(file)
}

struct Preamble {
    magic: [u8; 4],
    version: [u8; 4],
}

impl fmt::Debug for Preamble {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use std::str;
        f.debug_struct("Preamble")
            .field("magic", &str::from_utf8(&self.magic).unwrap())
            .field("version", &format!("{:?}", self.version))
            .finish()
    }
}

impl Preamble {
    fn parse(file: &mut File) -> ParseResult<Preamble> {
        let magic = parse_u8_array(file, 4)?;
        if &magic != b"\0asm" {
            return Err(ParseErr::Err("Invalid magic value".to_owned()));
        }

        let version = parse_u8_array(file, 4)?;
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

impl ValueType {
    fn parse(file: &mut File) -> ParseResult<Self> {
        let value_type = parse_u8(file)?;

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

fn parse_result_type(file: &mut File) -> ParseResult<Vec<ValueType>> {
    let n = parse_u32(file)?;

    let mut result_type = vec![];
    for _ in 0..n {
        result_type.push(ValueType::parse(file)?);
    }

    Ok(result_type)
}

#[derive(Debug)]
struct FuncType {
    parameter_types: Vec<ValueType>,
    result_types: Vec<ValueType>,
}

impl FuncType {
    fn parse(file: &mut File) -> ParseResult<Self> {
        let marker = parse_u8(file)?;
        if marker != 0x60 {
            return Err(ParseErr::Err(format!(
                "Invalid marker found for FuncType: {}",
                marker
            )));
        }

        Ok(FuncType {
            parameter_types: parse_result_type(file)?,
            result_types: parse_result_type(file)?,
        })
    }
}

fn parse_types(file: &mut File) -> ParseResult<Vec<FuncType>> {
    let n = parse_u32(file)?;

    let mut func_types = vec![];
    for _ in 0..n {
        func_types.push(FuncType::parse(file)?);
    }

    Ok(func_types)
}

#[derive(Debug)]
struct TypeIdx(u32);

impl TypeIdx {
    fn parse(file: &mut File) -> ParseResult<Self> {
        Ok(Self(parse_u32(file)?))
    }
}

#[derive(Debug)]
enum ElemType {
    FuncRef,
}

impl ElemType {
    fn parse(file: &mut File) -> ParseResult<Self> {
        let result = match parse_u8(file)? {
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

impl Limits {
    fn parse(file: &mut File) -> ParseResult<Self> {
        let has_max = parse_u8(file)? == 1;

        let result = Self {
            min: parse_u32(file)?,
            max: if has_max {
                Some(parse_u32(file)?)
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

impl TableType {
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

impl MemType {
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

impl Mutability {
    fn parse(file: &mut File) -> ParseResult<Self> {
        Ok(match parse_u8(file)? {
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

impl GlobalType {
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

impl ImportDescriptor {
    fn parse(file: &mut File) -> ParseResult<Self> {
        Ok(match parse_u8(file)? {
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
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Name {
    fn parse(file: &mut File) -> ParseResult<Self> {
        let n = parse_u32(file)?;

        let mut result = vec![];
        for _ in 0..n {
            result.push(parse_u8(file)?);
        }

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
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:?}.{:?}: {:?}",
            self.module, self.name, self.descriptor
        )
    }
}

impl Import {
    fn parse(file: &mut File) -> ParseResult<Self> {
        Ok(Import {
            module: Name::parse(file)?,
            name: Name::parse(file)?,
            descriptor: ImportDescriptor::parse(file)?,
        })
    }
}

fn parse_imports(file: &mut File) -> ParseResult<Vec<Import>> {
    let n = parse_u32(file)?;

    let mut result = vec![];
    for _ in 0..n {
        result.push(Import::parse(file)?);
    }

    Ok(result)
}

#[derive(Debug)]
enum Section {
    Custom,
    Type(Vec<FuncType>),
    Import(Vec<Import>),
    Function,
    Table,
    Memory,
    Global,
    Export,
    Start,
    Element,
    Code,
    Data,
}

impl Section {
    fn parse(file: &mut File) -> ParseResult<Section> {
        let id = parse_u8(file)?;
        let size = parse_u32(file)?;

        let section = match id {
            00 => Section::Custom,
            01 => Section::Type(parse_types(file)?),
            02 => Section::Import(parse_imports(file)?),
            03 => Section::Function,
            04 => Section::Table,
            05 => Section::Memory,
            06 => Section::Global,
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
            _ => {
                let _contents = parse_u8_array(file, size as usize).unwrap();
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
        };

        for section in parse_sections(file)? {
            match section {
                Section::Type(types) => module.types = types,
                Section::Import(imports) => module.imports = imports,
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
