#[derive(Debug)]
enum Instruction {
    Const(i32),
    Load(usize),
    Store(usize),
    Add,
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

    fn interpret(self: &mut Self, code: Vec<Instruction>) {
        for instruction in code {
            print!("> {:?}", instruction);

            match instruction {
                Instruction::Const(value) => self.stack.push(value),

                Instruction::Load(address) => self.stack.push(self.memory[address]),
                Instruction::Store(address) => self.memory[address] = self.stack.pop().unwrap(),

                Instruction::Add => {
                    let a = self.stack.pop().unwrap();
                    let b = self.stack.pop().unwrap();
                    self.stack.push(a + b);
                }
            }

            println!(" => {:?}", self.stack);
        }
    }
}

fn main() {
    let x_address = 0;
    let y_address = 1;

    let code: Vec<Instruction> = vec![
        Instruction::Const(1),
        Instruction::Const(2),
        Instruction::Add,
        Instruction::Store(x_address),

        Instruction::Const(3),
        Instruction::Const(4),
        Instruction::Add,
        Instruction::Store(y_address),

        Instruction::Load(x_address),
        Instruction::Load(y_address),
        Instruction::Add,
    ];

    let mut machine = Machine::new();
    machine.interpret(code);
}
