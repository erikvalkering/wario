#[derive(Debug)]
pub enum Instruction {
    I32Const(i32),           // TODO: can be replaced with wasm::Instruction
    I32Load(usize),          // TODO: can be replaced with wasm::Instruction
    I32Store(usize),         // TODO: can be replaced with wasm::Instruction
    I32Add,                  // TODO: can be replaced with wasm::Instruction
    I32Sub,                  // TODO: can be replaced with wasm::Instruction
    I32Mul,                  // TODO: can be replaced with wasm::Instruction
    I32Eq,                   // TODO: can be replaced with wasm::Instruction
    LocalGet(usize),         // TODO: can be replaced with wasm::Instruction
    Call(usize),             // TODO: can be replaced with wasm::Instruction
    Return,                  // TODO: can be replaced with wasm::Instruction
    Branch(usize),           // TODO: can be replaced with wasm::Instruction
    BranchIf(usize),         // TODO: can be replaced with wasm::Instruction
    Block(Vec<Instruction>), // TODO: can be replaced with wasm::Instruction
    Loop(Vec<Instruction>),  // TODO: can be replaced with wasm::Instruction
}

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

// TODO: Can be replaced with wasm::FuncType and wasm::Code (combine those into Function struct)
pub struct ModuleFunction {
    pub param_count: usize,
    pub code: Vec<Instruction>,
}

impl ModuleFunction {
    fn call(
        &self,
        machine: &mut Machine,
        module_functions: &Vec<ModuleFunction>,
        extern_functions: &mut Vec<ExternFunction>,
    ) {
        // pop param_count parameters off the stack
        let mut args = machine
            .stack
            .split_off(machine.stack.len() - self.param_count);

        machine.execute(&self.code, module_functions, extern_functions, &mut args);
    }
}

pub struct ExternFunction<'a> {
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

    pub fn execute(
        self: &mut Self,
        code: &Vec<Instruction>,
        module_functions: &Vec<ModuleFunction>,
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
                Instruction::I32Load(address) => self.stack.push(self.memory[*address]),
                Instruction::I32Store(address) => self.memory[*address] = self.stack.pop().unwrap(),

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
                Instruction::LocalGet(address) => self.stack.push(locals[*address]),

                Instruction::Call(function_index) => {
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
                Instruction::Branch(level) => return Some(ControlFlow::Branch(*level)),
                Instruction::BranchIf(level) => {
                    let condition = self.stack.pop().unwrap();

                    if condition != 0 {
                        return Some(ControlFlow::Branch(*level));
                    }
                }

                Instruction::Block(block_code) => {
                    match self.execute(block_code, module_functions, extern_functions, locals) {
                        None => {}

                        Some(ControlFlow::Return) => return Some(ControlFlow::Return),
                        Some(ControlFlow::Branch(level)) => {
                            if level > 0 {
                                return Some(ControlFlow::Branch(level - 1));
                            }
                        }
                    }
                }

                Instruction::Loop(loop_code) => loop {
                    match self.execute(loop_code, module_functions, extern_functions, locals) {
                        None => {}

                        Some(ControlFlow::Return) => return Some(ControlFlow::Return),
                        Some(ControlFlow::Branch(level)) => {
                            if level > 0 {
                                return Some(ControlFlow::Branch(level - 1));
                            }
                        }
                    }
                },
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
    use super::*;

    #[test]
    fn constant() {
        let code = vec![Instruction::I32Const(42)];

        let module_functions = vec![];
        let mut extern_functions = vec![];
        let mut locals = vec![];

        let mut machine = Machine::new();
        assert_eq!(machine.stack, vec![]);

        machine.execute(&code, &module_functions, &mut extern_functions, &mut locals);

        assert_eq!(machine.stack, vec![42]);
    }

    #[test]
    fn load() {
        let code = vec![Instruction::I32Load(0)];

        let module_functions = vec![];
        let mut extern_functions = vec![];
        let mut locals = vec![];

        let mut machine = Machine::new();
        assert_eq!(machine.stack, vec![]);

        machine.memory[0] = 42;
        machine.execute(&code, &module_functions, &mut extern_functions, &mut locals);

        assert_eq!(machine.stack, vec![42]);
    }

    #[test]
    fn store() {
        let code = vec![Instruction::I32Store(0)];

        let module_functions = vec![];
        let mut extern_functions = vec![];
        let mut locals = vec![];

        let mut machine = Machine::new();

        machine.stack = vec![42];
        machine.execute(&code, &module_functions, &mut extern_functions, &mut locals);

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

        machine.execute(&code, &module_functions, &mut extern_functions, &mut locals);

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

        machine.execute(&code, &module_functions, &mut extern_functions, &mut locals);

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

        machine.execute(&code, &module_functions, &mut extern_functions, &mut locals);

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

        machine.execute(&code, &module_functions, &mut extern_functions, &mut locals);

        assert_eq!(machine.stack, vec![(a == b) as i32, (b == c) as i32]);
    }

    #[test]
    fn localget() {
        let code = vec![Instruction::LocalGet(0)];

        let module_functions = vec![];
        let mut extern_functions = vec![];
        let mut locals = vec![42];

        let mut machine = Machine::new();

        machine.execute(&code, &module_functions, &mut extern_functions, &mut locals);

        assert_eq!(machine.stack, vec![42]);
    }

    #[test]
    fn call_module_function() {
        let code = vec![Instruction::Call(0)];

        let function = ModuleFunction {
            param_count: 0,
            code: vec![Instruction::I32Const(42)],
        };

        let module_functions = vec![function];
        let mut extern_functions = vec![];
        let mut locals = vec![];

        let mut machine = Machine::new();

        machine.execute(&code, &module_functions, &mut extern_functions, &mut locals);

        assert_eq!(machine.stack, vec![42]);
    }

    #[test]
    fn call_module_function_with_args() {
        let a = 5;
        let b = 3;

        let code = vec![
            Instruction::I32Const(a),
            Instruction::I32Const(b),
            Instruction::Call(0),
        ];

        let function = ModuleFunction {
            param_count: 2,
            code: vec![
                Instruction::LocalGet(0),
                Instruction::LocalGet(1),
                Instruction::I32Sub,
            ],
        };

        let module_functions = vec![function];
        let mut extern_functions = vec![];
        let mut locals = vec![];

        let mut machine = Machine::new();

        machine.execute(&code, &module_functions, &mut extern_functions, &mut locals);

        assert_eq!(machine.stack, vec![a - b]);
    }

    #[test]
    fn call_extern_function() {
        let code = vec![Instruction::Call(0)];

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

            machine.execute(&code, &module_functions, &mut extern_functions, &mut locals);
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
            Instruction::Call(0),
        ];
        let function = ExternFunction {
            param_count: 2,
            fun: Box::new(|args: &[i32]| Some(args[0] - args[1])),
        };

        let module_functions = vec![];
        let mut extern_functions = vec![function];
        let mut locals = vec![];

        let mut machine = Machine::new();

        machine.execute(&code, &module_functions, &mut extern_functions, &mut locals);

        assert_eq!(machine.stack, vec![a - b]);
    }

    #[test]
    fn return_statement() {
        let code = vec![
            Instruction::I32Const(42),
            Instruction::Block(vec![
                Instruction::Return,
                Instruction::I32Const(43),
                Instruction::I32Const(44),
            ]),
            Instruction::I32Const(45),
        ];

        let module_functions = vec![];
        let mut extern_functions = vec![];
        let mut locals = vec![];

        let mut machine = Machine::new();

        machine.execute(&code, &module_functions, &mut extern_functions, &mut locals);

        assert_eq!(machine.stack, vec![42]);
    }

    #[test]
    fn simple_break() {
        let code = vec![
            Instruction::I32Const(42),
            Instruction::Branch(0),
            Instruction::I32Const(43),
            Instruction::I32Const(44),
        ];

        let module_functions = vec![];
        let mut extern_functions = vec![];
        let mut locals = vec![];

        let mut machine = Machine::new();

        machine.execute(&code, &module_functions, &mut extern_functions, &mut locals);

        assert_eq!(machine.stack, vec![42]);
    }

    #[test]
    fn nested_break_single() {
        let code = vec![
            Instruction::I32Const(42),
            Instruction::Block(vec![
                Instruction::Branch(0),
                Instruction::I32Const(43),
                Instruction::I32Const(44),
            ]),
            Instruction::I32Const(45),
        ];

        let module_functions = vec![];
        let mut extern_functions = vec![];
        let mut locals = vec![];

        let mut machine = Machine::new();

        machine.execute(&code, &module_functions, &mut extern_functions, &mut locals);

        assert_eq!(machine.stack, vec![42, 45]);
    }

    #[test]
    fn nested_break_double() {
        let code = vec![
            Instruction::I32Const(42),
            Instruction::Block(vec![
                Instruction::Branch(1),
                Instruction::I32Const(43),
                Instruction::I32Const(44),
            ]),
            Instruction::I32Const(45),
        ];

        let module_functions = vec![];
        let mut extern_functions = vec![];
        let mut locals = vec![];

        let mut machine = Machine::new();

        machine.execute(&code, &module_functions, &mut extern_functions, &mut locals);

        assert_eq!(machine.stack, vec![42]);
    }

    #[test]
    fn simple_break_if() {
        let code = vec![
            Instruction::I32Const(0),
            Instruction::BranchIf(0),
            Instruction::I32Const(42),
            Instruction::I32Const(1),
            Instruction::BranchIf(0),
            Instruction::I32Const(45),
        ];

        let module_functions = vec![];
        let mut extern_functions = vec![];
        let mut locals = vec![];

        let mut machine = Machine::new();

        machine.execute(&code, &module_functions, &mut extern_functions, &mut locals);

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
            Instruction::I32Store(0),
            Instruction::Loop(vec![
                Instruction::I32Load(0),
                Instruction::I32Const(4),
                Instruction::I32Eq,
                Instruction::BranchIf(1),
                Instruction::I32Const(42),
                Instruction::I32Load(0),
                Instruction::I32Const(1),
                Instruction::I32Add,
                Instruction::I32Store(0),
            ]),
        ];

        let module_functions = vec![];
        let mut extern_functions = vec![];
        let mut locals = vec![];

        let mut machine = Machine::new();

        machine.execute(&code, &module_functions, &mut extern_functions, &mut locals);

        assert_eq!(machine.stack, vec![42, 42, 42, 42]);
    }
} // mod tests
