use wario::vm::{ExternFunction, Instruction, Machine, ModuleFunction};
use wario::wasm::{BlockType, FuncIdx, LabelIdx, LocalIdx, MemArg};

fn main() {
    // int i = 0;
    // while (true) {
    //   push(i)
    //   call(print)
    //   i++;
    // }

    let code = vec![
        Instruction::I32Const(0),
        Instruction::I32Store(MemArg {
            align: 0,
            offset: 0,
        }),
        Instruction::Loop(
            BlockType::Empty,
            vec![
                Instruction::I32Load(MemArg {
                    align: 0,
                    offset: 0,
                }),
                Instruction::Call(FuncIdx(1)),
                Instruction::I32Load(MemArg {
                    align: 0,
                    offset: 0,
                }),
                Instruction::Call(FuncIdx(0)),
                Instruction::I32Store(MemArg {
                    align: 0,
                    offset: 0,
                }),
            ],
        ),
    ];

    // fn increment(value: i32) {
    //   if value == 80 {
    //     0
    //   }
    //   else {
    //     value + 1
    //   }
    // }
    let move_player = ModuleFunction {
        param_count: 1,
        code: vec![
            Instruction::LocalGet(LocalIdx(0)),
            Instruction::I32Const(80),
            Instruction::I32Eq,
            Instruction::Block(
                BlockType::Empty,
                vec![
                    Instruction::Block(
                        BlockType::Empty,
                        vec![
                            Instruction::BranchIf(LabelIdx(0)),
                            Instruction::LocalGet(LocalIdx(0)),
                            Instruction::I32Const(1),
                            Instruction::I32Add,
                            Instruction::Branch(LabelIdx(1)),
                        ],
                    ),
                    Instruction::I32Const(0),
                ],
            ),
        ],
    };

    let display_player = ExternFunction {
        param_count: 1,
        fun: Box::new(|args: &[i32]| {
            println!("{} B-)", " ".repeat(args[0] as usize));
            None
        }),
    };

    let module_functions = vec![move_player];
    let mut extern_functions = vec![display_player];
    let mut locals = vec![];

    let mut machine = Machine::new();
    machine.debugging = false;

    machine.invoke(&code, &module_functions, &mut extern_functions, &mut locals);
}
