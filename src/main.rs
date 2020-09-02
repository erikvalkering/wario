#[derive(Debug)]
enum Instruction {
    Const(i32),
    Load(usize),
    Store(usize),
    Add,
    Mul,
    LocalGet(usize),
    Call(usize),
}

enum Function {
    ModuleFunction(ModuleFunction),
    ImportFunction(ImportFunction),
}

impl Function {
    fn call(&self, machine: &mut Machine, functions: &Vec<Function>) {
        match self {
            Function::ModuleFunction(function) => function.call(machine, functions),
            Function::ImportFunction(function) => function.call(machine, functions),
        }
    }
}

struct ModuleFunction {
    param_count: usize,
    code: Vec<Instruction>,
}

impl ModuleFunction {
    fn call(&self, machine: &mut Machine, functions: &Vec<Function>) {
        // pop param_count parameters off the stack
        let fargs = machine.stack.split_off(machine.stack.len() - self.param_count);

        machine.interpret(&self.code, &functions, fargs);
    }
}

struct ImportFunction {
    // param_count: usize,
    fun: fn () -> (),
}

impl ImportFunction {
    fn call(&self, _machine: &Machine, _functions: &Vec<Function>) {
        (self.fun)();
    }
}

struct Machine {
    stack: Vec<i32>,
    memory: Vec<i32>,
}

impl Machine {
    fn new() -> Self {
        Machine{
            stack: Vec::new(),
            memory: vec![0; 10],
        }
    }

    fn interpret(self: &mut Self, code: &Vec<Instruction>, functions: &Vec<Function>, locals: Vec<i32>) {
        for instruction in code {
            println!("> {:?}", instruction);
            println!("  locals: {:?}", locals);

            match *instruction {
                Instruction::Const(value) => self.stack.push(value),

                Instruction::Load(address) => self.stack.push(self.memory[address]),
                Instruction::Store(address) => self.memory[address] = self.stack.pop().unwrap(),

                Instruction::Add => {
                    let right = self.stack.pop().unwrap();
                    let left = self.stack.pop().unwrap();
                    self.stack.push(left + right);
                }

                Instruction::Mul => {
                    let right = self.stack.pop().unwrap();
                    let left = self.stack.pop().unwrap();
                    self.stack.push(left * right);
                }

                Instruction::LocalGet(address) => self.stack.push(locals[address]),

                Instruction::Call(function_index) => {
                    let function = &functions[function_index];
                    function.call(self, functions);
                }
            }

            println!("  stack: {:?}", self.stack);
            println!("  memory: {:?}", self.memory);
        }
    }
}

#[test]
fn test_const() {
    let code = vec![
        Instruction::Const(42),
    ];

    let functions = vec![];
    let locals = vec![];

    let mut machine = Machine::new();
    assert_eq!(machine.stack, vec![]);

    machine.interpret(&code, &functions, locals);

    assert_eq!(machine.stack, vec![42]);
}

#[test]
fn test_load() {
    let code = vec![
        Instruction::Load(0),
    ];

    let functions = vec![];
    let locals = vec![];

    let mut machine = Machine::new();
    assert_eq!(machine.stack, vec![]);

    machine.memory[0] = 42;
    machine.interpret(&code, &functions, locals);

    assert_eq!(machine.stack, vec![42]);
}

#[test]
fn test_store() {
    let code = vec![
        Instruction::Store(0),
    ];

    let functions = vec![];
    let locals = vec![];

    let mut machine = Machine::new();

    machine.stack = vec![42];
    machine.interpret(&code, &functions, locals);

    assert_eq!(machine.stack, vec![]);
    assert_eq!(machine.memory[0], 42);
}

#[test]
fn test_add() {
    let a = 1;
    let b = 2;

    let code = vec![
        Instruction::Const(a),
        Instruction::Const(b),
        Instruction::Add,
    ];

    let functions = vec![];
    let locals = vec![];

    let mut machine = Machine::new();

    machine.interpret(&code, &functions, locals);

    assert_eq!(machine.stack, vec![a + b]);
}

#[test]
fn test_mul() {
    let a = 2;
    let b = 3;

    let code = vec![
        Instruction::Const(a),
        Instruction::Const(b),
        Instruction::Mul,
    ];

    let functions = vec![];
    let locals = vec![];

    let mut machine = Machine::new();

    machine.interpret(&code, &functions, locals);

    assert_eq!(machine.stack, vec![a * b]);
}

#[test]
fn test_localget() {
    let code = vec![
        Instruction::LocalGet(0),
    ];

    let functions = vec![];
    let locals = vec![42];

    let mut machine = Machine::new();

    machine.interpret(&code, &functions, locals);

    assert_eq!(machine.stack, vec![42]);
}

#[test]
fn test_call() {
    let code = vec![
        Instruction::Call(0),
    ];

    let function = ModuleFunction{
        param_count: 0,
        code: vec![
            Instruction::Const(42),
        ]
    };

    let functions = vec![Function::ModuleFunction(function)];
    let locals = vec![];

    let mut machine = Machine::new();

    machine.interpret(&code, &functions, locals);

    assert_eq!(machine.stack, vec![42]);
}

#[test]
fn test_import() {
    let code = vec![
        Instruction::Call(0),
    ];

    let function = ImportFunction{
        fun: || println!("Ha!")
    };

    let functions = vec![Function::ImportFunction(function)];
    let locals = vec![];

    let mut machine = Machine::new();

    machine.interpret(&code, &functions, locals);

    assert_eq!(machine.stack, vec![42]);
}

#[test]
fn test_complex() {
    let add_function = ModuleFunction{
        param_count: 2,
        code: vec![
            Instruction::LocalGet(0),
            Instruction::LocalGet(1),
            Instruction::Add,
        ]
    };

    let functions = vec![Function::ModuleFunction(add_function)];

    let x_address = 0;
    let y_address = 1;
    let z_address = 2;
    let add_function_address = 0;

    let code: Vec<Instruction> = vec![
        Instruction::Const(1),
        Instruction::Const(2),
        Instruction::Add,
        Instruction::Store(x_address),

        Instruction::Const(3),
        Instruction::Const(4),
        Instruction::Add,
        Instruction::Store(y_address),

        Instruction::Const(5),
        Instruction::Const(6),
        Instruction::Call(add_function_address),
        Instruction::Store(z_address),

        Instruction::Load(x_address),
        Instruction::Load(y_address),
        Instruction::Add,

        Instruction::Load(z_address),
        Instruction::Mul,
    ];

    let locals = vec![];

    let mut machine = Machine::new();
    machine.interpret(&code, &functions, locals);

    assert_eq!(machine.stack, vec![110])
}

fn main() {}
