use std::io::{Error, LineWriter, Write};
use vm::bytecode::{disassemble_instruction, BinaryOp, Chunk, OpCode, Value};

#[derive(Debug)]
pub enum RuntimeError {
    TracingError(Error),
    StackUnderflow,
    // The following out of bound errors have been found using
    // property based testing.
    // Unless I can statically determine that a chunk it is well-formed
    // they should be there, with a runtime penalty of bound checking
    // at every instruction
    InstructionOutOfBound,
    ValueOutOfBound,
}

struct Vm<'a> {
    chunk: &'a Chunk,
    program_counter: usize,
    stack: Vec<Value>,
}

impl<'a> Vm<'a> {
    fn new(chunk: &'a Chunk) -> Vm<'a> {
        Vm {
            chunk: chunk,
            program_counter: 0,
            stack: vec![],
        }
    }

    fn pop(&mut self) -> Result<Value, RuntimeError> {
        if let Some(value) = self.stack.pop() {
            Ok(value)
        } else {
            Err(RuntimeError::StackUnderflow)
        }
    }

    /// Interprets the next instruction.
    /// The execution of this function have some side effects including:
    ///  * update of the program counter to have it point to the next
    ///    instruction
    ///  * mutate the state of the stack
    ///
    /// This function true if there are other instructions left to execute
    /// or false if we're done interpreting the chunk.
    fn interpret_next(&mut self) -> Result<bool, RuntimeError> {
        self.program_counter += 1;
        if self.program_counter >= self.chunk.instruction_count() {
            return Err(RuntimeError::InstructionOutOfBound);
        }
        match self.chunk.get(self.program_counter - 1) {
            OpCode::Return => {
                let value = self.stack.pop();
                println!{"{:?}", value};
                // Temporarily changed the meaning
                return Ok(false);
            }
            OpCode::Constant(offset) => {
                if offset >= self.chunk.values_count() {
                    return Err(RuntimeError::ValueOutOfBound);
                }
                self.stack.push(self.chunk.get_value(offset))
            }
            OpCode::Negate => {
                let op = try!(self.pop());
                self.stack.push(-op);
            }
            OpCode::Binary(ref operator) => {
                // Note the order!
                // Op2 is the topmost element of the stack,
                // Op1 is the second topmost element
                let op2 = try!(self.pop());
                let op1 = try!(self.pop());
                self.stack.push(match operator {
                    &BinaryOp::Add => op1 + op2,
                    &BinaryOp::Subtract => op1 - op2,
                    &BinaryOp::Multiply => op1 * op2,
                    &BinaryOp::Divide => op1 / op2,
                })
            }
        };
        Ok(true)
    }
    fn trace<T>(&mut self, out: &mut LineWriter<T>) -> Result<(), Error>
    where
        T: Write,
    {
        try!(write!(out, "Stack: "));
        for value in self.stack.iter() {
            try!(write!(out, "[ {} ]", value));
        }
        try!(writeln!(out));
        if self.program_counter < self.chunk.instruction_count() {
            disassemble_instruction(&self.chunk.get(self.program_counter), self.chunk, out)
        } else {
            //TODO: is this really ok?
            Ok(())
        }
    }
}

pub fn interpret(chunk: &Chunk) -> Result<(), RuntimeError> {
    let mut vm = Vm::new(chunk);
    while try!{vm.interpret_next()} {}
    Ok(())
}

pub fn trace<T>(chunk: &Chunk, writer: &mut LineWriter<T>) -> Result<(), RuntimeError>
where
    T: Write,
{
    let mut vm = Vm::new(chunk);
    while {
        try!{vm.trace(writer).map_err(RuntimeError::TracingError)};
        try!{vm.interpret_next()}
    } {}
    Ok(())
}

#[cfg(test)]
mod tests {
    use proptest::collection::*;
    use proptest::num::*;
    use proptest::prelude::*;
    use proptest::strategy::*;
    use std::io::*;
    use vm::bytecode::*;
    use vm::*;

    fn arb_constants(max_constants: usize) -> VecStrategy<f64::Any> {
        prop::collection::vec(any::<Value>(), 0..max_constants)
    }

    fn arb_instruction(max_offset: usize) -> BoxedStrategy<OpCode> {
        prop_oneof![
            (0..max_offset).prop_map(OpCode::Constant),
            Just(OpCode::Return),
            Just(OpCode::Negate),
            prop_oneof![
                Just(BinaryOp::Add),
                Just(BinaryOp::Subtract),
                Just(BinaryOp::Multiply),
                Just(BinaryOp::Divide),
            ].prop_map(OpCode::Binary),
        ].boxed()
    }

    fn arb_instructions(
        max_offset: usize,
        max_instructions: usize,
    ) -> VecStrategy<BoxedStrategy<OpCode>> {
        prop::collection::vec(arb_instruction(max_offset), 0..max_instructions)
    }

    prop_compose!{
        fn arb_chunk(max_offset : usize, max_instructions : usize)
            (constants in arb_constants(max_offset),
             instructions in arb_instructions(max_offset, max_instructions)) -> Chunk{
            let mut chunk = Chunk::new();
            for constant in constants{
                let _offset = chunk.add_constant(constant);
            }
            let mut line = 0;
            for instruction in instructions{
                chunk.add_instruction(instruction, line);
                line = line + 1;
            }
            chunk
        }
    }

    proptest! {
    #[test]
    fn interpret_doesnt_crash(ref chunk in arb_chunk(10, 20)) {
        let _ = vm::interpret(chunk);
    }
    }

    proptest! {
    #[test]
    fn trace_doesnt_crash(ref chunk in arb_chunk(10, 20)) {
        let mut writer = LineWriter::new(sink());
        let _ = vm::trace(chunk, &mut writer);
    }
    }

    proptest! {
    #[test]
    fn disassemble_doesnt_crash(ref chunk in arb_chunk(10, 20)) {
        let mut writer = LineWriter::new(sink());
        let _ = disassemble(chunk, "Test", &mut writer);
    }
    }
}
