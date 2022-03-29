use super::wasm::{Func, FuncIdx, Instruction, LabelIdx, LocalIdx};

#[derive(Debug)]
pub enum ControlFlow {
    Return,
    Branch(usize),
}

// TODO: add all four datatypes: i32, i64, f32, f64
// TODO: load/store should have offset
// TODO: memory.size
// TODO: memory.grow
// TODO: local.tee
// TODO: local.drop
// TODO: select
// TODO: br_table
// TODO: wasm parser (into Module)
// TODO: memory initialization
// TODO: obtain exported functions to find entry point(s)
// TODO: what about local memory, like the call frame
//       For example:
//
//       auto foo() {
//         int a[20];
//         for (auto &e : a)
//           e = 0.0;
//       }
//
//       Will this set the size of the local memory to 20?

impl Func {
    fn call(
        &self,
        machine: &mut Machine,
        module_functions: &Vec<Func>,
        extern_functions: &mut Vec<ExternFunction>,
    ) {
        // pop param_count parameters off the stack
        let mut args = machine
            .stack
            .split_off(machine.stack.len() - self.ftype.parameter_types.len());

        machine.invoke(
            &self.code.body,
            module_functions,
            extern_functions,
            &mut args,
        );
    }
}

pub struct ExternFunction<'a> {
    // TODO: replace param_count with a FuncType
    pub param_count: usize,
    pub fun: Box<dyn FnMut(&[i32]) -> Option<i32> + 'a>,
}

impl<'a> ExternFunction<'a> {
    fn call(&mut self, machine: &mut Machine) {
        let args = machine
            .stack
            .split_off(machine.stack.len() - self.param_count);

        if let Some(result) = (self.fun)(&args) {
            machine.stack.push(result)
        }
    }
}

pub struct Machine {
    pub stack: Vec<i32>,
    pub memory: Vec<i32>,
    pub debugging: bool,
}

impl Machine {
    pub fn new() -> Self {
        Machine {
            stack: Vec::new(),
            memory: vec![0; 10],
            debugging: true,
        }
    }

    pub fn invoke(
        self: &mut Self,
        code: &Vec<Instruction>,
        module_functions: &Vec<Func>,
        extern_functions: &mut Vec<ExternFunction>,
        locals: &mut Vec<i32>,
    ) -> Option<ControlFlow> {
        for instruction in code {
            if self.debugging {
                println!("> {:?}", instruction);
                println!("  locals: {:?}", locals);
            }

            match instruction {
                Instruction::I32Const(value) => self.stack.push(*value),

                // TODO: Load/Store indirect (maybe to support arrays? first implement loops and conditionals?)
                Instruction::I32Load(memarg) => self.stack.push(self.memory[memarg.offset]),
                Instruction::I32Store(memarg) => {
                    self.memory[memarg.offset] = self.stack.pop().unwrap()
                }

                Instruction::I32Add => {
                    let right = self.stack.pop().unwrap();
                    let left = self.stack.pop().unwrap();
                    self.stack.push(left + right);
                }

                Instruction::I32Sub => {
                    let right = self.stack.pop().unwrap();
                    let left = self.stack.pop().unwrap();
                    self.stack.push(left - right);
                }

                Instruction::I32Mul => {
                    let right = self.stack.pop().unwrap();
                    let left = self.stack.pop().unwrap();
                    self.stack.push(left * right);
                }

                Instruction::I32Eq => {
                    let right = self.stack.pop().unwrap();
                    let left = self.stack.pop().unwrap();
                    self.stack.push((left == right) as i32);
                }

                // TODO: Indirect addressing to support arrays?
                // TODO: LocalSet?
                Instruction::LocalGet(LocalIdx(address)) => self.stack.push(locals[*address]),

                Instruction::Call(FuncIdx(function_index)) => {
                    let function_index = *function_index;

                    if function_index < module_functions.len() {
                        module_functions[function_index].call(
                            self,
                            module_functions,
                            extern_functions,
                        )
                    } else {
                        let function_index = function_index - module_functions.len();
                        extern_functions[function_index].call(self)
                    }
                }

                Instruction::Return => return Some(ControlFlow::Return),
                Instruction::Branch(LabelIdx(level)) => return Some(ControlFlow::Branch(*level)),
                Instruction::BranchIf(LabelIdx(level)) => {
                    let condition = self.stack.pop().unwrap();

                    if condition != 0 {
                        return Some(ControlFlow::Branch(*level));
                    }
                }

                Instruction::Block(_, block_code) => {
                    match self.invoke(block_code, module_functions, extern_functions, locals) {
                        None => {}

                        Some(ControlFlow::Return) => return Some(ControlFlow::Return),
                        Some(ControlFlow::Branch(level)) => {
                            if level > 0 {
                                return Some(ControlFlow::Branch(level - 1));
                            }
                        }
                    }
                }

                Instruction::Loop(_, loop_code) => loop {
                    match self.invoke(loop_code, module_functions, extern_functions, locals) {
                        None => {}

                        Some(ControlFlow::Return) => return Some(ControlFlow::Return),
                        Some(ControlFlow::Branch(level)) => {
                            if level > 0 {
                                return Some(ControlFlow::Branch(level - 1));
                            }
                        }
                    }
                },

                _ => panic!("Unsupported instruction encountered: {:?}", instruction),
            }

            if self.debugging {
                println!("  stack: {:?}", self.stack);
                println!("  memory: {:?}", self.memory);
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use crate::vm::{ExternFunction, Machine};
    use crate::wasm::{
        BlockType, Code, Func, FuncIdx, FuncType, Instruction, LabelIdx, LocalIdx, MemArg, NumType,
        ValueType,
    };

    #[test]
    fn constant() {
        let code = vec![Instruction::I32Const(42)];

        let module_functions = vec![];
        let mut extern_functions = vec![];
        let mut locals = vec![];

        let mut machine = Machine::new();
        assert_eq!(machine.stack, vec![]);

        machine.invoke(&code, &module_functions, &mut extern_functions, &mut locals);

        assert_eq!(machine.stack, vec![42]);
    }

    #[test]
    fn load() {
        let code = vec![Instruction::I32Load(MemArg {
            align: 0,
            offset: 0,
        })];

        let module_functions = vec![];
        let mut extern_functions = vec![];
        let mut locals = vec![];

        let mut machine = Machine::new();
        assert_eq!(machine.stack, vec![]);

        machine.memory[0] = 42;
        machine.invoke(&code, &module_functions, &mut extern_functions, &mut locals);

        assert_eq!(machine.stack, vec![42]);
    }

    #[test]
    fn store() {
        let code = vec![Instruction::I32Store(MemArg {
            align: 0,
            offset: 0,
        })];

        let module_functions = vec![];
        let mut extern_functions = vec![];
        let mut locals = vec![];

        let mut machine = Machine::new();

        machine.stack = vec![42];
        machine.invoke(&code, &module_functions, &mut extern_functions, &mut locals);

        assert_eq!(machine.stack, vec![]);
        assert_eq!(machine.memory[0], 42);
    }

    #[test]
    fn add() {
        let a = 1;
        let b = 2;

        let code = vec![
            Instruction::I32Const(a),
            Instruction::I32Const(b),
            Instruction::I32Add,
        ];

        let module_functions = vec![];
        let mut extern_functions = vec![];
        let mut locals = vec![];

        let mut machine = Machine::new();

        machine.invoke(&code, &module_functions, &mut extern_functions, &mut locals);

        assert_eq!(machine.stack, vec![a + b]);
    }

    #[test]
    fn sub() {
        let a = 1;
        let b = 2;

        let code = vec![
            Instruction::I32Const(a),
            Instruction::I32Const(b),
            Instruction::I32Sub,
        ];

        let module_functions = vec![];
        let mut extern_functions = vec![];
        let mut locals = vec![];

        let mut machine = Machine::new();

        machine.invoke(&code, &module_functions, &mut extern_functions, &mut locals);

        assert_eq!(machine.stack, vec![a - b]);
    }

    #[test]
    fn mul() {
        let a = 2;
        let b = 3;

        let code = vec![
            Instruction::I32Const(a),
            Instruction::I32Const(b),
            Instruction::I32Mul,
        ];

        let module_functions = vec![];
        let mut extern_functions = vec![];
        let mut locals = vec![];

        let mut machine = Machine::new();

        machine.invoke(&code, &module_functions, &mut extern_functions, &mut locals);

        assert_eq!(machine.stack, vec![a * b]);
    }

    #[test]
    fn eq() {
        let a = 2;
        let b = 3;
        let c = 3;

        let code = vec![
            Instruction::I32Const(a),
            Instruction::I32Const(b),
            Instruction::I32Eq,
            Instruction::I32Const(b),
            Instruction::I32Const(c),
            Instruction::I32Eq,
        ];

        let module_functions = vec![];
        let mut extern_functions = vec![];
        let mut locals = vec![];

        let mut machine = Machine::new();

        machine.invoke(&code, &module_functions, &mut extern_functions, &mut locals);

        assert_eq!(machine.stack, vec![(a == b) as i32, (b == c) as i32]);
    }

    #[test]
    fn localget() {
        let code = vec![Instruction::LocalGet(LocalIdx(0))];

        let module_functions = vec![];
        let mut extern_functions = vec![];
        let mut locals = vec![42];

        let mut machine = Machine::new();

        machine.invoke(&code, &module_functions, &mut extern_functions, &mut locals);

        assert_eq!(machine.stack, vec![42]);
    }

    #[test]
    fn call_module_function() {
        let code = vec![Instruction::Call(FuncIdx(0))];

        let function = Func {
            ftype: FuncType {
                parameter_types: vec![],
                result_types: vec![ValueType::NumType(NumType::I32)],
            },
            code: Code {
                locals: vec![],
                body: vec![Instruction::I32Const(42)],
            },
        };

        let module_functions = vec![function];
        let mut extern_functions = vec![];
        let mut locals = vec![];

        let mut machine = Machine::new();

        machine.invoke(&code, &module_functions, &mut extern_functions, &mut locals);

        assert_eq!(machine.stack, vec![42]);
    }

    #[test]
    fn call_module_function_with_args() {
        let a = 5;
        let b = 3;

        let code = vec![
            Instruction::I32Const(a),
            Instruction::I32Const(b),
            Instruction::Call(FuncIdx(0)),
        ];

        let function = Func {
            ftype: FuncType {
                // TODO: simplify ValueType::NumType(NumType::I32) -> NumType::I32
                parameter_types: vec![
                    ValueType::NumType(NumType::I32),
                    ValueType::NumType(NumType::I32),
                ],
                result_types: vec![ValueType::NumType(NumType::I32)],
            },
            code: Code {
                locals: vec![],
                body: vec![
                    Instruction::LocalGet(LocalIdx(0)),
                    Instruction::LocalGet(LocalIdx(1)),
                    Instruction::I32Sub,
                ],
            },
        };

        let module_functions = vec![function];
        let mut extern_functions = vec![];
        let mut locals = vec![];

        let mut machine = Machine::new();

        machine.invoke(&code, &module_functions, &mut extern_functions, &mut locals);

        assert_eq!(machine.stack, vec![a - b]);
    }

    #[test]
    fn call_extern_function() {
        let code = vec![Instruction::Call(FuncIdx(0))];

        let mut function_was_called = false;
        {
            let function = ExternFunction {
                param_count: 0,
                fun: Box::new(|_: &[i32]| {
                    function_was_called = true;
                    None
                }),
            };

            let module_functions = vec![];
            let mut extern_functions = vec![function];
            let mut locals = vec![];

            let mut machine = Machine::new();

            machine.invoke(&code, &module_functions, &mut extern_functions, &mut locals);
        }

        assert_eq!(function_was_called, true);
    }

    #[test]
    fn call_extern_function_with_args() {
        let a = 5;
        let b = 3;

        let code = vec![
            Instruction::I32Const(a),
            Instruction::I32Const(b),
            Instruction::Call(FuncIdx(0)),
        ];

        let function = ExternFunction {
            param_count: 2,
            fun: Box::new(|args: &[i32]| Some(args[0] - args[1])),
        };

        let module_functions = vec![];
        let mut extern_functions = vec![function];
        let mut locals = vec![];

        let mut machine = Machine::new();

        machine.invoke(&code, &module_functions, &mut extern_functions, &mut locals);

        assert_eq!(machine.stack, vec![a - b]);
    }

    #[test]
    fn return_statement() {
        let code = vec![
            Instruction::I32Const(42),
            Instruction::Block(
                BlockType::Empty,
                vec![
                    Instruction::Return,
                    Instruction::I32Const(43),
                    Instruction::I32Const(44),
                ],
            ),
            Instruction::I32Const(45),
        ];

        let module_functions = vec![];
        let mut extern_functions = vec![];
        let mut locals = vec![];

        let mut machine = Machine::new();

        machine.invoke(&code, &module_functions, &mut extern_functions, &mut locals);

        assert_eq!(machine.stack, vec![42]);
    }

    #[test]
    fn simple_break() {
        let code = vec![
            Instruction::I32Const(42),
            Instruction::Branch(LabelIdx(0)),
            Instruction::I32Const(43),
            Instruction::I32Const(44),
        ];

        let module_functions = vec![];
        let mut extern_functions = vec![];
        let mut locals = vec![];

        let mut machine = Machine::new();

        machine.invoke(&code, &module_functions, &mut extern_functions, &mut locals);

        assert_eq!(machine.stack, vec![42]);
    }

    #[test]
    fn nested_break_single() {
        let code = vec![
            Instruction::I32Const(42),
            Instruction::Block(
                BlockType::Empty,
                vec![
                    Instruction::Branch(LabelIdx(0)),
                    Instruction::I32Const(43),
                    Instruction::I32Const(44),
                ],
            ),
            Instruction::I32Const(45),
        ];

        let module_functions = vec![];
        let mut extern_functions = vec![];
        let mut locals = vec![];

        let mut machine = Machine::new();

        machine.invoke(&code, &module_functions, &mut extern_functions, &mut locals);

        assert_eq!(machine.stack, vec![42, 45]);
    }

    #[test]
    fn nested_break_double() {
        let code = vec![
            Instruction::I32Const(42),
            Instruction::Block(
                BlockType::Empty,
                vec![
                    Instruction::Branch(LabelIdx(1)),
                    Instruction::I32Const(43),
                    Instruction::I32Const(44),
                ],
            ),
            Instruction::I32Const(45),
        ];

        let module_functions = vec![];
        let mut extern_functions = vec![];
        let mut locals = vec![];

        let mut machine = Machine::new();

        machine.invoke(&code, &module_functions, &mut extern_functions, &mut locals);

        assert_eq!(machine.stack, vec![42]);
    }

    #[test]
    fn simple_break_if() {
        let code = vec![
            Instruction::I32Const(0),
            Instruction::BranchIf(LabelIdx(0)),
            Instruction::I32Const(42),
            Instruction::I32Const(1),
            Instruction::BranchIf(LabelIdx(0)),
            Instruction::I32Const(45),
        ];

        let module_functions = vec![];
        let mut extern_functions = vec![];
        let mut locals = vec![];

        let mut machine = Machine::new();

        machine.invoke(&code, &module_functions, &mut extern_functions, &mut locals);

        assert_eq!(machine.stack, vec![42]);
    }

    #[test]
    fn loop_statement() {
        // int i = 0;
        // while (true) {
        //   if (i == 4) break;
        //   "push 42"
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
                    Instruction::I32Const(4),
                    Instruction::I32Eq,
                    Instruction::BranchIf(LabelIdx(1)),
                    Instruction::I32Const(42),
                    Instruction::I32Load(MemArg {
                        align: 0,
                        offset: 0,
                    }),
                    Instruction::I32Const(1),
                    Instruction::I32Add,
                    Instruction::I32Store(MemArg {
                        align: 0,
                        offset: 0,
                    }),
                ],
            ),
        ];

        let module_functions = vec![];
        let mut extern_functions = vec![];
        let mut locals = vec![];

        let mut machine = Machine::new();

        machine.invoke(&code, &module_functions, &mut extern_functions, &mut locals);

        assert_eq!(machine.stack, vec![42, 42, 42, 42]);
    }
} // mod tests
