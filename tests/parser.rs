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
enum SectionID {
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

impl SectionID {
    fn parse(file: &mut File) -> ParseResult<SectionID> {
        let id = parse_u8(file)?;

        let id = match id {
            0 => SectionID::Custom,
            1 => SectionID::Type,
            2 => SectionID::Import,
            3 => SectionID::Function,
            4 => SectionID::Table,
            5 => SectionID::Memory,
            6 => SectionID::Global,
            7 => SectionID::Export,
            8 => SectionID::Start,
            9 => SectionID::Element,
            10 => SectionID::Code,
            11 => SectionID::Data,
            _ => return Err(ParseErr::Err(format!("Found unknown section id: {}", id))),
        };

        Ok(id)
    }
}

#[derive(Debug)]
struct Section {
    id: SectionID,
    size: u32,
    contents: Vec<u8>,
}

impl Section {
    fn parse(file: &mut File) -> ParseResult<Section> {
        let id = SectionID::parse(file)?;
        let size = parse_u32(file)?;

        let contents = parse_u8_array(file, size as usize).unwrap();

        Ok(Section { id, size, contents })
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
