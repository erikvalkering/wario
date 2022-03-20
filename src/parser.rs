use std::convert::TryInto;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};

use super::wasm::*;

#[derive(Debug)]
enum ParseErr {
    Err(String),
    Eof,
}

type ParseResult<T> = std::result::Result<T, ParseErr>;
pub type Result<T> = std::result::Result<T, String>;

trait Parse: Sized {
    fn parse(file: &mut File) -> ParseResult<Self>;
}

impl<T: Parse> Parse for Vec<T> {
    fn parse(file: &mut File) -> ParseResult<Self> {
        let n = u32::parse(file)?;

        let mut result_type = vec![];
        for _ in 0..n {
            result_type.push(Parse::parse(file)?);
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

impl Parse for usize {
    fn parse(file: &mut File) -> ParseResult<Self> {
        Ok(u32::parse(file)? as usize)
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

impl Parse for Preamble {
    fn parse(file: &mut File) -> ParseResult<Self> {
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

impl Parse for RefType {
    fn parse(file: &mut File) -> ParseResult<Self> {
        let result = match u8::parse(file)? {
            0x70 => Self::FuncRef,
            elem_type => return Err(ParseErr::Err(format!("Invalid RefType: {}", elem_type))),
        };

        Ok(result)
    }
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

impl Parse for TableType {
    fn parse(file: &mut File) -> ParseResult<Self> {
        let result = Self {
            elem_type: Parse::parse(file)?,
            limits: Parse::parse(file)?,
        };

        Ok(result)
    }
}

impl Parse for MemType {
    fn parse(file: &mut File) -> ParseResult<Self> {
        Ok(Self {
            limits: Parse::parse(file)?,
        })
    }
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

impl Parse for GlobalType {
    fn parse(file: &mut File) -> ParseResult<Self> {
        Ok(Self {
            value_type: Parse::parse(file)?,
            mutability: Parse::parse(file)?,
        })
    }
}

impl Parse for ImportDescriptor {
    fn parse(file: &mut File) -> ParseResult<Self> {
        Ok(match u8::parse(file)? {
            0x00 => Self::Func(Parse::parse(file)?),
            0x01 => Self::Table(Parse::parse(file)?),
            0x02 => Self::Memory(Parse::parse(file)?),
            0x03 => Self::Global(Parse::parse(file)?),
            id => {
                return Err(ParseErr::Err(format!(
                    "Invalid import descriptor type: {}",
                    id
                )))
            }
        })
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

impl Parse for Import {
    fn parse(file: &mut File) -> ParseResult<Self> {
        Ok(Self {
            module: Parse::parse(file)?,
            name: Parse::parse(file)?,
            descriptor: Parse::parse(file)?,
        })
    }
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

impl Parse for MemArg {
    fn parse(file: &mut File) -> ParseResult<Self> {
        Ok(Self {
            align: Parse::parse(file)?,
            offset: Parse::parse(file)?,
        })
    }
}

impl Parse for Vec<Instruction> {
    fn parse(file: &mut File) -> ParseResult<Self> {
        let mut result = vec![];

        loop {
            let opcode = u8::parse(file)?;

            let instruction = match opcode {
                0x05 => break, // else
                0x0B => break, // end

                // Control instructions
                0x00 => Instruction::Unreachable,
                0x02 => Instruction::Block(Parse::parse(file)?, Parse::parse(file)?),
                0x03 => Instruction::Loop(Parse::parse(file)?, Parse::parse(file)?),
                0x04 => Instruction::If(
                    Parse::parse(file)?,
                    Parse::parse(file)?,
                    Parse::parse(file)?,
                ),
                0x0C => Instruction::Branch(Parse::parse(file)?),
                0x0D => Instruction::BranchIf(Parse::parse(file)?),
                0x0F => Instruction::Return,
                0x10 => Instruction::Call(Parse::parse(file)?),

                // Variable instructions
                0x20 => Instruction::LocalGet(Parse::parse(file)?),
                0x21 => Instruction::LocalSet(Parse::parse(file)?),
                0x23 => Instruction::GlobalGet(Parse::parse(file)?),
                0x24 => Instruction::GlobalSet(Parse::parse(file)?),

                // Memory instructions
                0x28 => Instruction::I32Load(Parse::parse(file)?),
                0x36 => Instruction::I32Store(Parse::parse(file)?),

                // Numeric instructions
                0x41 => Instruction::I32Const(Parse::parse(file)?),
                0x44 => Instruction::F64Const(Parse::parse(file)?),
                0x46 => Instruction::I32Eq,
                0x4A => Instruction::I32GtSigned,
                0x63 => Instruction::F64Lt,
                0x64 => Instruction::F64Gt,
                0x66 => Instruction::F64Ge,
                0x6A => Instruction::I32Add,
                0x6B => Instruction::I32Sub,
                0x6C => Instruction::I32Mul,
                0xA0 => Instruction::F64Add,
                0xA1 => Instruction::F64Sub,
                0xA2 => Instruction::F64Mul,
                0xA3 => Instruction::F64Div,

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

        Ok(result)
    }
}

impl Parse for Global {
    fn parse(file: &mut File) -> ParseResult<Self> {
        Ok(Self {
            global_type: Parse::parse(file)?,
            expression: Parse::parse(file)?,
        })
    }
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

impl Parse for Export {
    fn parse(file: &mut File) -> ParseResult<Self> {
        Ok(Self {
            name: Parse::parse(file)?,
            descriptor: Parse::parse(file)?,
        })
    }
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

impl Module {
    pub fn parse(file: &mut File) -> Result<Module> {
        let preamble = match Parse::parse(file) {
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
