use std::collections::HashMap;

#[derive(Debug)]
pub struct Interpreter {
    pub codes: HashMap<i64, i64>,
    position: i64,
    pub is_running: bool,
    pub last_output: i64,
    pub has_outputted: bool,
    relative_base: i64,
    pub joystick: i64,
    pub outputs: Vec<i64>
}

#[derive(Debug, PartialEq, Clone)]
pub enum Mode {
    Position,
    Immediate,
    Relative
}

#[derive(Clone, Debug, PartialEq)]
pub enum OpCode {
    Add(Mode, Mode, Mode),
    Multiply(Mode, Mode, Mode),
    Halt,
    Input(Mode),
    Output(Mode),
    JumpIfTrue(Mode, Mode),
    JumpIfFalse(Mode, Mode),
    LessThan(Mode, Mode, Mode),
    Equals(Mode, Mode, Mode),
    AdjustBase(Mode),
    Noop
}

impl Interpreter {
    pub fn new(codes: Vec<i64>) -> Interpreter {
        let mut code_dict: HashMap<i64, i64> = HashMap::new();

        for (i, c) in codes.iter().enumerate() {
            code_dict.insert(i as i64, *c as i64);
        }

        Interpreter {
            codes: code_dict,
            position: 0,
            is_running: true,
            last_output: 0,
            has_outputted: false,
            relative_base: 0,
            joystick: 0,
            outputs: Vec::new()
        }
    }

    pub fn step(&mut self) {
        let op = self.parse_opcode(self.fetch(self.position));

        if op == OpCode::Halt {
            self.is_running = false;
            return;
        }

        match op {
            OpCode::Add(p1_mode, p2_mode, p3_mode) => {
                let operand_1 = self.get_operand(self.position + 1, p1_mode);
                let operand_2 = self.get_operand(self.position + 2, p2_mode);
                let result = operand_1 + operand_2;
                self.put(self.position + 3, result, p3_mode);

                self.position += 4
            }
            OpCode::Multiply(p1_mode, p2_mode, p3_mode) => {
                let operand_1 = self.get_operand(self.position + 1, p1_mode);
                let operand_2 = self.get_operand(self.position + 2, p2_mode);
                let result = operand_1 * operand_2;
                self.put(self.position + 3, result, p3_mode);

                self.position += 4
            }
            OpCode::Halt => self.position += 1,
            OpCode::Input(p1_mode) => {
                self.put(self.position + 1, self.joystick, p1_mode);

                self.position += 2;
            }
            OpCode::Output(fetch_mode) => {
                let output = self.get_operand(self.position + 1, fetch_mode);
                self.last_output = output;

                self.has_outputted = true;

                self.outputs.push(output);

                self.position += 2;
            },
            OpCode::JumpIfTrue(p1_mode, p2_mode) => {
                let comparison = self.get_operand(self.position + 1, p1_mode);
                let to_jump = self.get_operand(self.position + 2, p2_mode);

                if comparison != 0 {
                    self.position = to_jump;
                } else {
                    self.position += 3;
                }
            },
            OpCode::JumpIfFalse(p1_mode, p2_mode) => {
                let comparison = self.get_operand(self.position + 1, p1_mode);
                let to_jump = self.get_operand(self.position + 2, p2_mode);

                if comparison == 0 {
                    self.position = to_jump;
                } else {
                    self.position += 3;
                }
            },
            OpCode::LessThan(p1_mode, p2_mode, p3_mode) => {
                let comparison_1 = self.get_operand(self.position + 1, p1_mode);
                let comparison_2 = self.get_operand(self.position + 2, p2_mode);

                if comparison_1 < comparison_2 {
                    self.put(self.position + 3, 1, p3_mode)
                } else {
                    self.put(self.position + 3, 0, p3_mode)
                }

                self.position += 4;
            }
            OpCode::Equals(p1_mode, p2_mode, p3_mode) => {
                let comparison_1 = self.get_operand(self.position + 1, p1_mode);
                let comparison_2 = self.get_operand(self.position + 2, p2_mode);

                if comparison_1 == comparison_2 {
                    self.put(self.position + 3, 1, p3_mode)
                } else {
                    self.put(self.position + 3, 0, p3_mode)
                }

                self.position += 4;
            },
            OpCode::AdjustBase(p1_mode) => {
                let arg = self.get_operand(self.position + 1, p1_mode);

                self.relative_base += arg;

                self.position += 2;
            },
            OpCode::Noop => {
                self.position += 1;
            }
        };
    }

    fn get_operand(&self, pos: i64, mode: Mode) -> i64 {
        match mode {
            Mode::Immediate => self.fetch(pos),
            Mode::Position => self.fetch(self.fetch(pos)),
            Mode::Relative => {
                self.fetch(self.relative_base + self.fetch(pos))
            }
        }
    }

    fn put(&mut self, pos: i64, data: i64, mode: Mode) {
        match mode {
            Mode::Position => {
                self.codes.insert(self.fetch(pos), data);
            },
            Mode::Relative => {
                self.codes.insert(self.relative_base + self.fetch(pos), data);
            }
            _ => panic!("Writing data may only be position or relative")
        }
    }

    fn get_digits(&self, number: i64) -> (Mode, Mode, Mode, i64) {
        let mut digits: Vec<i64> = number
            .to_string()
            .chars()
            .filter_map(|x| x.to_digit(10))
            .map(|x| (x as i64))
            .collect();

        digits.reverse();

        while digits.len() < 5 {
            digits.push(0);
        }

        digits.reverse();

        (
            self.parse_mode(digits[2]),
            self.parse_mode(digits[1]),
            self.parse_mode(digits[0]),
            digits[3] * 10 + digits[4],
        )
    }

    pub fn fetch(&self, pos: i64) -> i64 {
        *self.codes.get(&pos).unwrap_or(&0)
    }

    fn parse_mode(&self, mode: i64) -> Mode {
        match mode {
            0 => Mode::Position,
            1 => Mode::Immediate,
            2 => Mode::Relative,
            _ => Mode::Position
        }
    }

    fn parse_opcode(&self, op: i64) -> OpCode {
        let op = self.get_digits(op);

        match op {
            (p1_mode, p2_mode, p3_mode, 1) => OpCode::Add(p1_mode, p2_mode, p3_mode),
            (p1_mode, p2_mode, p3_mode, 2) => OpCode::Multiply(p1_mode, p2_mode, p3_mode),
            (p1_mode, _, _, 3) => OpCode::Input(p1_mode),
            (p1_mode, _, _, 4) => OpCode::Output(p1_mode),
            (p1_mode, p2_mode, _, 5) => OpCode::JumpIfTrue(p1_mode, p2_mode),
            (p1_mode, p2_mode, _, 6) => OpCode::JumpIfFalse(p1_mode, p2_mode),
            (p1_mode, p2_mode, p3_mode, 7) => OpCode::LessThan(p1_mode, p2_mode, p3_mode),
            (p1_mode, p2_mode, p3_mode, 8) => OpCode::Equals(p1_mode, p2_mode, p3_mode),
            (p1_mode, _, _, 9) => OpCode::AdjustBase(p1_mode),
            (_, _, _, 99) => OpCode::Halt,
            (_, _, _, 0) => OpCode::Noop,
            _ => panic!("Unimplemented opcode: {:?}", op),
        }
    }
}
