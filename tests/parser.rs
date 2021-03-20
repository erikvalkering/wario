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

#[derive(Debug)]
struct Preamble {
    magic: [u8; 4],
    version: [u8; 4],
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
enum Section {
    Custom,
    Type,
    Import,
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
            01 => Section::Type,
            02 => Section::Import,
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

        let _contents = parse_u8_array(file, size as usize).unwrap();

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
struct FuncType;

#[derive(Debug)]
struct Module {
    preamble: Preamble,
    types: Option<Vec<FuncType>>,
}

impl Module {
    fn parse(file: &mut File) -> Result<Module> {
        let preamble = match Preamble::parse(file) {
            Ok(x) => x,
            Err(ParseErr::Err(err)) => return Err(err),
            Err(ParseErr::Eof) => return Err("Unexpected end of file detected".to_owned()),
        };

        let module = Module {
            preamble,
            types: None,
        };

        for section in parse_sections(file)? {
            println!("{:?}", section);
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
    println!("{:?}", module);

    Ok(())
}
