//! Defines a compiler for brainfuck programs.

use crate::runner::Runner;

#[derive(Debug)]
enum Instruction {
    Inc,
    Dec,
    Shl,
    Shr,
    Read,
    Write,
    Repeat(Vec<Instruction>),
}

#[derive(Debug)]
/// A compiled brainfuck program.
pub struct Program(Vec<Instruction>);

impl Program {
    /// Compiles a brainfuck program.
    pub fn new(source: &str) -> Result<Program, &'static str> {
        let mut all_lists: Vec<Vec<Instruction>> = Vec::new();
        let mut current_list: Vec<Instruction> = Vec::new();

        for char in source.chars() {
            match char {
                '+' => current_list.push(Instruction::Inc),
                '-' => current_list.push(Instruction::Dec),
                '<' => current_list.push(Instruction::Shl),
                '>' => current_list.push(Instruction::Shr),
                ',' => current_list.push(Instruction::Read),
                '.' => current_list.push(Instruction::Write),

                '[' => {
                    let sub_instruction_list: Vec<Instruction> = Vec::new();
                    all_lists.push(current_list);
                    current_list = sub_instruction_list;
                }

                ']' => {
                    let sub_instruction_list = current_list;

                    let Some(mut last_instruction_list) = all_lists.pop() else {
                        return Err("unmatched closing bracket");
                    };

                    last_instruction_list.push(Instruction::Repeat(sub_instruction_list));
                    current_list = last_instruction_list;
                }

                _ => {}
            };
        }

        if !all_lists.is_empty() {
            Err("unmatched opening bracket")
        } else {
            Ok(Program(current_list))
        }
    }

    /// Runs this program on a given runner.
    pub fn run_on<const N: usize>(&self, runner: &mut Runner<N>) {
        fn run<const N: usize>(list: &Vec<Instruction>, runner: &mut Runner<N>) {
            for instruction in list {
                match instruction {
                    Instruction::Inc => runner.inc(),
                    Instruction::Dec => runner.dec(),
                    Instruction::Shl => runner.shl(),
                    Instruction::Shr => runner.shr(),
                    Instruction::Read => runner.read(),
                    Instruction::Write => runner.write(),
                    Instruction::Repeat(list) => runner.repeat(|runner| run(list, runner)),
                }
            }
        }

        run(&self.0, runner)
    }

    /// Runs this program on a new runner.
    pub fn run<const N: usize>(&self, input: &[u8]) -> Runner<N> {
        let mut runner = Runner::new(input);
        self.run_on(&mut runner);
        runner
    }
}
