pub mod bytecode;
pub mod compiler;
pub mod vm;

use std::io::{stdout, LineWriter};
use user_interface::{RuloxImplementation, RunResult as UiRunResult};

pub struct RuloxVm {}

impl RuloxVm {
    pub fn new() -> RuloxVm {
        RuloxVm {}
    }
}

impl RuloxImplementation for RuloxVm {
    fn run(&mut self, source: &str) -> UiRunResult {
        let chunk = compiler::compile(source).unwrap();
        let stdout = stdout();
        let handle = stdout.lock();
        let mut writer = LineWriter::new(handle);
        let _ = bytecode::disassemble(&chunk, "Test", &mut writer).unwrap();
        UiRunResult::Ok
    }
}

#[cfg(test)]
mod tests {
    use vm::*;

    proptest! {
    #[test]
    fn doesnt_crash(ref input in "\\PC*") {
        let mut ruloxvm = RuloxVm::new();
        ruloxvm.run(input)
    }
    }
}
