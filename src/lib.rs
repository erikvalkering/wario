#[derive(Debug)]
pub enum Instruction {
    Const(i32),
    Load(usize),
    Store(usize),
    Add,
    Sub,
    Mul,
    LocalGet(usize),
    Call(usize),
    // TODO: br, br_if, loop, return
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
        let args = machine
            .stack
            .split_off(machine.stack.len() - self.param_count);

        machine.execute(&self.code, module_functions, extern_functions, args);
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
}

impl Machine {
    pub fn new() -> Self {
        Machine {
            stack: Vec::new(),
            memory: vec![0; 10],
        }
    }

    pub fn execute(
        self: &mut Self,
        code: &Vec<Instruction>,
        module_functions: &Vec<ModuleFunction>,
        extern_functions: &mut Vec<ExternFunction>,
        locals: Vec<i32>,
    ) {
        for instruction in code {
            println!("> {:?}", instruction);
            println!("  locals: {:?}", locals);

            match *instruction {
                Instruction::Const(value) => self.stack.push(value),

                // TODO: Load/Store indirect (maybe to support arrays? first implement loops and conditionals?)
                Instruction::Load(address) => self.stack.push(self.memory[address]),
                Instruction::Store(address) => self.memory[address] = self.stack.pop().unwrap(),

                Instruction::Add => {
                    let right = self.stack.pop().unwrap();
                    let left = self.stack.pop().unwrap();
                    self.stack.push(left + right);
                }

                Instruction::Sub => {
                    let right = self.stack.pop().unwrap();
                    let left = self.stack.pop().unwrap();
                    self.stack.push(left - right);
                }

                Instruction::Mul => {
                    let right = self.stack.pop().unwrap();
                    let left = self.stack.pop().unwrap();
                    self.stack.push(left * right);
                }

                // TODO: Indirect addressing to support arrays?
                // TODO: LocalSet?
                Instruction::LocalGet(address) => self.stack.push(locals[address]),

                Instruction::Call(function_index) => {
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
            }

            println!("  stack: {:?}", self.stack);
            println!("  memory: {:?}", self.memory);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constant() {
        let code = vec![Instruction::Const(42)];

        let module_functions = vec![];
        let mut extern_functions = vec![];
        let locals = vec![];

        let mut machine = Machine::new();
        assert_eq!(machine.stack, vec![]);

        machine.execute(&code, &module_functions, &mut extern_functions, locals);

        assert_eq!(machine.stack, vec![42]);
    }

    #[test]
    fn load() {
        let code = vec![Instruction::Load(0)];

        let module_functions = vec![];
        let mut extern_functions = vec![];
        let locals = vec![];

        let mut machine = Machine::new();
        assert_eq!(machine.stack, vec![]);

        machine.memory[0] = 42;
        machine.execute(&code, &module_functions, &mut extern_functions, locals);

        assert_eq!(machine.stack, vec![42]);
    }

    #[test]
    fn store() {
        let code = vec![Instruction::Store(0)];

        let module_functions = vec![];
        let mut extern_functions = vec![];
        let locals = vec![];

        let mut machine = Machine::new();

        machine.stack = vec![42];
        machine.execute(&code, &module_functions, &mut extern_functions, locals);

        assert_eq!(machine.stack, vec![]);
        assert_eq!(machine.memory[0], 42);
    }

    #[test]
    fn add() {
        let a = 1;
        let b = 2;

        let code = vec![
            Instruction::Const(a),
            Instruction::Const(b),
            Instruction::Add,
        ];

        let module_functions = vec![];
        let mut extern_functions = vec![];
        let locals = vec![];

        let mut machine = Machine::new();

        machine.execute(&code, &module_functions, &mut extern_functions, locals);

        assert_eq!(machine.stack, vec![a + b]);
    }

    #[test]
    fn sub() {
        let a = 1;
        let b = 2;

        let code = vec![
            Instruction::Const(a),
            Instruction::Const(b),
            Instruction::Sub,
        ];

        let module_functions = vec![];
        let mut extern_functions = vec![];
        let locals = vec![];

        let mut machine = Machine::new();

        machine.execute(&code, &module_functions, &mut extern_functions, locals);

        assert_eq!(machine.stack, vec![a - b]);
    }

    #[test]
    fn mul() {
        let a = 2;
        let b = 3;

        let code = vec![
            Instruction::Const(a),
            Instruction::Const(b),
            Instruction::Mul,
        ];

        let module_functions = vec![];
        let mut extern_functions = vec![];
        let locals = vec![];

        let mut machine = Machine::new();

        machine.execute(&code, &module_functions, &mut extern_functions, locals);

        assert_eq!(machine.stack, vec![a * b]);
    }

    #[test]
    fn localget() {
        let code = vec![Instruction::LocalGet(0)];

        let module_functions = vec![];
        let mut extern_functions = vec![];
        let locals = vec![42];

        let mut machine = Machine::new();

        machine.execute(&code, &module_functions, &mut extern_functions, locals);

        assert_eq!(machine.stack, vec![42]);
    }

    #[test]
    fn call_module_function() {
        let code = vec![Instruction::Call(0)];

        let function = ModuleFunction {
            param_count: 0,
            code: vec![Instruction::Const(42)],
        };

        let module_functions = vec![function];
        let mut extern_functions = vec![];
        let locals = vec![];

        let mut machine = Machine::new();

        machine.execute(&code, &module_functions, &mut extern_functions, locals);

        assert_eq!(machine.stack, vec![42]);
    }

    #[test]
    fn call_module_function_with_args() {
        let a = 5;
        let b = 3;

        let code = vec![
            Instruction::Const(a),
            Instruction::Const(b),
            Instruction::Call(0),
        ];

        let function = ModuleFunction {
            param_count: 2,
            code: vec![
                Instruction::LocalGet(0),
                Instruction::LocalGet(1),
                Instruction::Sub,
            ],
        };

        let module_functions = vec![function];
        let mut extern_functions = vec![];
        let locals = vec![];

        let mut machine = Machine::new();

        machine.execute(&code, &module_functions, &mut extern_functions, locals);

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
            let locals = vec![];

            let mut machine = Machine::new();

            machine.execute(&code, &module_functions, &mut extern_functions, locals);
        }

        assert_eq!(function_was_called, true);
    }

    #[test]
    fn call_extern_function_with_args() {
        let a = 5;
        let b = 3;

        let code = vec![
            Instruction::Const(a),
            Instruction::Const(b),
            Instruction::Call(0),
        ];
        let function = ExternFunction {
            param_count: 2,
            fun: Box::new(|args: &[i32]| Some(args[0] - args[1])),
        };

        let module_functions = vec![];
        let mut extern_functions = vec![function];
        let locals = vec![];

        let mut machine = Machine::new();

        machine.execute(&code, &module_functions, &mut extern_functions, locals);

        assert_eq!(machine.stack, vec![a - b]);
    }

    #[test]
    fn simple_break() {
        let code = vec![
            Instruction::Const(42),
            Instruction::Break(0),
            Instruction::Const(43),
            Instruction::Const(44),
        ];

        let module_functions = vec![];
        let mut extern_functions = vec![];
        let locals = vec![];

        let mut machine = Machine::new();

        machine.execute(&code, &module_functions, &mut extern_functions, locals);

        assert_eq!(machine.stack, vec![42]);
    }
} // mod tests
