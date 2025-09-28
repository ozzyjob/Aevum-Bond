use crate::{BondError, BondResult};

/// Script execution result
#[derive(Debug, Clone, PartialEq)]
pub enum ScriptResult {
    /// Script executed successfully and returned true
    Success,
    /// Script executed successfully but returned false
    Failure,
    /// Script execution failed with an error
    Error(String),
}

/// Script execution context
#[derive(Debug, Clone)]
pub struct ScriptContext {
    /// Current block height
    pub block_height: u32,
    /// Current timestamp
    pub timestamp: u64,
    /// Transaction being validated
    pub tx_hash: Vec<u8>,
    /// Input index being validated
    pub input_index: u32,
}

/// Simple script interpreter (will be enhanced in Sprint 5)
/// This is a placeholder implementation for the programmable UTXO system
pub struct ScriptInterpreter {
    /// Maximum number of operations allowed
    max_ops: usize,
    /// Maximum stack size
    max_stack_size: usize,
}

/// Stack for script execution
#[derive(Debug, Clone)]
struct Stack {
    items: Vec<Vec<u8>>,
    max_size: usize,
}

impl ScriptInterpreter {
    /// Create a new script interpreter with default limits
    pub fn new() -> Self {
        Self {
            max_ops: 1000,       // Prevent infinite loops
            max_stack_size: 100, // Prevent memory exhaustion
        }
    }

    /// Create a script interpreter with custom limits
    pub fn with_limits(max_ops: usize, max_stack_size: usize) -> Self {
        Self {
            max_ops,
            max_stack_size,
        }
    }

    /// Execute a script with the given context
    pub fn execute(&self, script: &[u8], context: &ScriptContext) -> BondResult<ScriptResult> {
        let mut stack = Stack::new(self.max_stack_size);
        let mut pc = 0; // Program counter
        let mut op_count = 0;

        while pc < script.len() {
            // Check operation limit
            op_count += 1;
            if op_count > self.max_ops {
                return Ok(ScriptResult::Error("Operation limit exceeded".to_string()));
            }

            let opcode = script[pc];
            pc += 1;

            match self.execute_opcode(opcode, &mut stack, &mut pc, script, context)? {
                ScriptResult::Success => continue,
                ScriptResult::Failure => return Ok(ScriptResult::Failure),
                ScriptResult::Error(msg) => return Ok(ScriptResult::Error(msg)),
            }
        }

        // Script succeeds if stack has exactly one item that's "true"
        if stack.size() == 1 {
            let empty_vec = vec![];
            let top = stack.top().unwrap_or(&empty_vec);
            if self.is_true(top) {
                Ok(ScriptResult::Success)
            } else {
                Ok(ScriptResult::Failure)
            }
        } else {
            Ok(ScriptResult::Error(
                "Stack must have exactly one item at end".to_string(),
            ))
        }
    }

    /// Execute a single opcode
    fn execute_opcode(
        &self,
        opcode: u8,
        stack: &mut Stack,
        pc: &mut usize,
        script: &[u8],
        context: &ScriptContext,
    ) -> BondResult<ScriptResult> {
        match opcode {
            // Push operations (0x01-0x4b push 1-75 bytes)
            0x01..=0x4b => {
                let size = opcode as usize;
                if *pc + size > script.len() {
                    return Ok(ScriptResult::Error("Script truncated".to_string()));
                }

                let data = script[*pc..*pc + size].to_vec();
                *pc += size;

                stack.push(data)?;
                Ok(ScriptResult::Success)
            }

            // OP_0 (push empty array)
            0x00 => {
                stack.push(vec![])?;
                Ok(ScriptResult::Success)
            }

            // OP_1 (push number 1)
            0x51 => {
                stack.push(vec![1])?;
                Ok(ScriptResult::Success)
            }

            // OP_DUP (duplicate top stack item)
            0x76 => {
                if let Some(top) = stack.top() {
                    let dup = top.clone();
                    stack.push(dup)?;
                    Ok(ScriptResult::Success)
                } else {
                    Ok(ScriptResult::Error(
                        "Cannot duplicate empty stack".to_string(),
                    ))
                }
            }

            // OP_EQUAL (check if top two items are equal)
            0x87 => {
                if stack.size() < 2 {
                    return Ok(ScriptResult::Error(
                        "Not enough items for OP_EQUAL".to_string(),
                    ));
                }

                let a = stack.pop().unwrap();
                let b = stack.pop().unwrap();

                if a == b {
                    stack.push(vec![1])?;
                } else {
                    stack.push(vec![0])?;
                }

                Ok(ScriptResult::Success)
            }

            // OP_VERIFY (fail if top item is false, but don't consume it in the middle of execution)
            0x69 => {
                if let Some(top) = stack.top() {
                    if self.is_true(top) {
                        stack.pop(); // Only consume if true
                        Ok(ScriptResult::Success)
                    } else {
                        Ok(ScriptResult::Failure)
                    }
                } else {
                    Ok(ScriptResult::Error("Cannot verify empty stack".to_string()))
                }
            }

            // OP_CHECKSIG (placeholder for signature verification)
            0xac => {
                if stack.size() < 2 {
                    return Ok(ScriptResult::Error(
                        "Not enough items for OP_CHECKSIG".to_string(),
                    ));
                }

                let _pubkey = stack.pop().unwrap();
                let _signature = stack.pop().unwrap();

                // TODO: Implement actual signature verification with ML-DSA
                // For now, always return true (this is a placeholder)
                stack.push(vec![1])?;
                Ok(ScriptResult::Success)
            }

            // OP_CHECKBLOCKHEIGHT (custom opcode for time locks)
            0xf0 => {
                if let Some(required_height_bytes) = stack.pop() {
                    if required_height_bytes.len() != 4 {
                        return Ok(ScriptResult::Error(
                            "Invalid block height format".to_string(),
                        ));
                    }

                    let required_height = u32::from_le_bytes([
                        required_height_bytes[0],
                        required_height_bytes[1],
                        required_height_bytes[2],
                        required_height_bytes[3],
                    ]);

                    if context.block_height >= required_height {
                        stack.push(vec![1])?;
                    } else {
                        stack.push(vec![0])?;
                    }

                    Ok(ScriptResult::Success)
                } else {
                    Ok(ScriptResult::Error(
                        "Cannot check block height on empty stack".to_string(),
                    ))
                }
            }

            // Unknown opcode
            _ => Ok(ScriptResult::Error(format!(
                "Unknown opcode: 0x{:02x}",
                opcode
            ))),
        }
    }

    /// Check if a byte array represents "true"
    fn is_true(&self, data: &[u8]) -> bool {
        !data.is_empty() && data.iter().any(|&b| b != 0)
    }
}

impl Stack {
    fn new(max_size: usize) -> Self {
        Self {
            items: Vec::new(),
            max_size,
        }
    }

    fn push(&mut self, item: Vec<u8>) -> BondResult<()> {
        if self.items.len() >= self.max_size {
            return Err(BondError::ScriptExecutionFailed {
                reason: "Stack overflow".to_string(),
            });
        }
        self.items.push(item);
        Ok(())
    }

    fn pop(&mut self) -> Option<Vec<u8>> {
        self.items.pop()
    }

    fn top(&self) -> Option<&Vec<u8>> {
        self.items.last()
    }

    fn size(&self) -> usize {
        self.items.len()
    }
}

impl Default for ScriptInterpreter {
    fn default() -> Self {
        Self::new()
    }
}

impl ScriptContext {
    /// Create a new script context
    pub fn new(block_height: u32, timestamp: u64, tx_hash: Vec<u8>, input_index: u32) -> Self {
        Self {
            block_height,
            timestamp,
            tx_hash,
            input_index,
        }
    }
}

// Re-export the Script type from utxo module
pub use crate::utxo::Script;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_script_execution() {
        let interpreter = ScriptInterpreter::new();
        let context = ScriptContext::new(100, 1234567890, vec![1, 2, 3], 0);

        // Simple script: push 1 (should succeed by itself)
        let script = vec![0x51]; // OP_1
        let result = interpreter.execute(&script, &context).unwrap();

        assert_eq!(result, ScriptResult::Success);
    }

    #[test]
    fn test_script_failure() {
        let interpreter = ScriptInterpreter::new();
        let context = ScriptContext::new(100, 1234567890, vec![1, 2, 3], 0);

        // Script that pushes 0 and verifies (should fail)
        let script = vec![0x00, 0x69]; // OP_0 OP_VERIFY
        let result = interpreter.execute(&script, &context).unwrap();

        assert_eq!(result, ScriptResult::Failure);
    }

    #[test]
    fn test_block_height_check() {
        let interpreter = ScriptInterpreter::new();
        let context = ScriptContext::new(100, 1234567890, vec![1, 2, 3], 0);

        // Script that checks if block height >= 50
        let mut script = vec![0x04]; // Push 4 bytes
        script.extend_from_slice(&50u32.to_le_bytes()); // Block height 50
        script.push(0xf0); // OP_CHECKBLOCKHEIGHT
                           // Result should be true (1) left on stack

        let result = interpreter.execute(&script, &context).unwrap();
        assert_eq!(result, ScriptResult::Success);
    }

    #[test]
    fn test_stack_operations() {
        let interpreter = ScriptInterpreter::new();
        let context = ScriptContext::new(100, 1234567890, vec![1, 2, 3], 0);

        // Script: push 42, duplicate, check equal
        let script = vec![
            0x01, 42,   // Push byte 42
            0x76, // OP_DUP
            0x87, // OP_EQUAL
        ];

        let result = interpreter.execute(&script, &context).unwrap();
        assert_eq!(result, ScriptResult::Success);
    }

    #[test]
    fn test_operation_limit() {
        let interpreter = ScriptInterpreter::with_limits(5, 100);
        let context = ScriptContext::new(100, 1234567890, vec![1, 2, 3], 0);

        // Script with too many operations
        let script = vec![0x51; 10]; // 10 OP_1 operations
        let result = interpreter.execute(&script, &context).unwrap();

        if let ScriptResult::Error(msg) = result {
            assert!(msg.contains("Operation limit exceeded"));
        } else {
            panic!("Expected operation limit error");
        }
    }
}
