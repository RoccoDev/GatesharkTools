/*
 *    Copyright 2019 RoccoDev
 *
 *    Licensed under the Apache License, Version 2.0 (the "License");
 *    you may not use this file except in compliance with the License.
 *    You may obtain a copy of the License at
 *
 *        http://www.apache.org/licenses/LICENSE-2.0
 *
 *    Unless required by applicable law or agreed to in writing, software
 *    distributed under the License is distributed on an "AS IS" BASIS,
 *    WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 *    See the License for the specific language governing permissions and
 *    limitations under the License.
*/

use crate::cheat::{Cheat, Instruction};
use crate::cheat::Opcode::{self, *};
use crate::check::CheckResult::Error;

macro_rules! err_if {
    ($assertion:expr, $err:expr) => {
        if $assertion {
            return CheckResult::Error($err.0, $err.1.to_owned());
        }
    };
}

pub mod checks;
mod errors;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum CheckResult {
    Pass,
    Warning(String),
    Error(usize, String)
}

pub struct DetailedResult {
    pub res_type: CheckResult,
    pub cheat_line: usize
}

impl DetailedResult {
    pub fn new(res_type: CheckResult, line: usize) -> DetailedResult {
        DetailedResult {
            res_type,
            cheat_line: line
        }
    }
}

pub trait Checker {
    fn check(&self, instr: Opcode, block_a: &str, block_b: &str) -> CheckResult;
}

pub struct AlwaysPassChecker;
impl Checker for AlwaysPassChecker {
    #[inline(always)]
    fn check(&self, instr: Opcode, block_a: &str, block_b: &str) -> CheckResult {
        CheckResult::Pass
    }
}

pub fn get_checker(opcode: Opcode) -> Box<dyn Checker> {
    match opcode {
        WriteWord | WriteShort | WriteByte => Box::new(checks::WriteChecker),
        Reset | EndCond | SetOffsetPtr => Box::new(checks::ResetChecker),
        Repeat | EndRepeat | SetOffsetImmediate | AddToDxData | SetDxData | CopyDxByte | CopyDxShort | CopyDxWord
        | LoadDxByte | LoadDxShort | LoadDxWord | AddOffset | BtnCode => Box::new(checks::ZeroAfterOpcodeChecker),
        EqWord | LtWord | GtWord | NeWord | PatchCode | MemoryCopy |EqShort | LtShort
        | GtShort | NeShort => Box::new(AlwaysPassChecker)
    }
}

fn check_instruction_pre(instruction: &Instruction, results: &mut Vec<CheckResult>) {
    if instruction.block_a.len() != 8 {
        results.push(CheckResult::Error(errors::WRONG_LENGTH_A.0, errors::WRONG_LENGTH_A.1.to_owned()));
    }
    if instruction.block_b.len() != 8 {
        results.push(CheckResult::Error(errors::WRONG_LENGTH_B.0, errors::WRONG_LENGTH_B.1.to_owned()));
    }
    if i64::from_str_radix(&instruction.block_a, 16).is_err() {
        results.push(CheckResult::Error(errors::INVALID_HEX_A.0, errors::INVALID_HEX_A.1.to_owned()));
    }
    if i64::from_str_radix(&instruction.block_b, 16).is_err() {
        results.push(CheckResult::Error(errors::INVALID_HEX_B.0, errors::INVALID_HEX_B.1.to_owned()));
    }
}

pub fn check_cheat(cheat: &Cheat) -> (CheckResult, Vec<DetailedResult>) {
    let mut results = vec![];
    for (line, instr) in cheat.instructions.iter().enumerate() {
        let mut current = vec![];
        check_instruction_pre(instr, &mut current);
        let result = instr.checker.check(instr.opcode, &instr.block_a, &instr.block_b);
        current.push(result);

        for res in current {
            results.push(DetailedResult::new(res, line));
        }
    }
    let mut final_res = CheckResult::Pass;
    if results.iter().filter(|r| if let CheckResult::Error(_, _) = (**r).res_type {true} else {false})
        .count() > 0 {
        final_res = CheckResult::Error(0, "Instruction compiled with errors.".to_owned());
    }
    else if results.iter().filter(|r| if let CheckResult::Warning(_) = (**r).res_type {true} else {false})
        .count() > 0 {
        final_res = CheckResult::Warning("Instruction compiled with warnings.".to_owned());
    }
    (final_res, results)
}

#[cfg(test)]
mod tests {
    use crate::cheat::{Cheat, Descriptor, Instruction, Opcode};
    use crate::check::{check_cheat, CheckResult, get_checker};

    #[test]
    pub fn pass_check() {
        assert_eq!(CheckResult::Pass, check("0AF2CD18", "CFF2AD4C"));
    }

    #[test]
    pub fn fail_check() {
        assert_ne!(CheckResult::Pass, check("0GZA7F9C", "A4B8LF7J8L82JK"));
    }

    fn check<'a>(block_a: &str, block_b: &str) -> CheckResult {
        let instruction = Instruction {
            opcode: Opcode::WriteWord,
            block_a: block_a.to_string(),
            block_b: block_b.to_string(),
            checker: get_checker(Opcode::WriteWord)
        };
        let cheat = Cheat {
            descriptor: Descriptor {
                name: "[Test Cheat]".to_string()
            },
            instructions: vec![instruction]
        };
        let (result, _) = check_cheat(&cheat);
        result
    }
}