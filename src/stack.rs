pub struct Stack {
    items: [u16; 16], // Array of 16 16-bit values
    top: usize,       // Index of the top element
}

impl Stack {
    pub fn default() -> Self {
        Stack {
            items: [0; 16], // Initialize array with zeros
            top: 0,         // Start with an empty stack
        }
    }

    pub fn push(&mut self, value: u16) -> Result<(), &'static str> {
        if self.top < 16 {
            self.items[self.top] = value; // Push value onto the stack
            self.top += 1; // Increment the top index
            Ok(())
        } else {
            Err("Stack overflow") // Handle overflow case
        }
    }

    pub fn pop(&mut self) -> Result<u16, &'static str> {
        if self.top > 0 {
            self.top -= 1; // Decrement the top index
            Ok(self.items[self.top]) // Return the popped value
        } else {
            Err("Stack underflow") // Handle underflow case
        }
    }

    pub fn is_empty(&self) -> bool {
        self.top == 0 // Check if the stack is empty
    }

    pub fn size(&self) -> usize {
        self.top // Return the current size of the stack
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stack_default() {
        let stack = Stack::default();
        assert!(stack.is_empty());
        assert_eq!(stack.size(), 0);
    }

    #[test]
    fn test_push() {
        let mut stack = Stack::default();
        assert_eq!(stack.push(10), Ok(()));
        assert_eq!(stack.size(), 1);
        assert!(!stack.is_empty());
        assert_eq!(stack.items[0], 10);

        // Fill the stack
        for i in 1..16 {
            assert_eq!(stack.push(i as u16), Ok(()));
        }

        // Test stack overflow
        assert_eq!(stack.push(17), Err("Stack overflow"));
    }

    #[test]
    fn test_pop() {
        let mut stack = Stack::default();

        // Test underflow
        assert_eq!(stack.pop(), Err("Stack underflow"));

        // Push some elements
        for i in 0..5 {
            stack.push(i as u16).unwrap();
        }

        // Pop elements and check values
        for i in (0..5).rev() {
            assert_eq!(stack.pop(), Ok(i as u16));
        }

        // Check if stack is empty after popping all elements
        assert!(stack.is_empty());
    }

    #[test]
    fn test_size_and_empty() {
        let mut stack = Stack::default();

        assert!(stack.is_empty());
        assert_eq!(stack.size(), 0);

        stack.push(1).unwrap();

        assert!(!stack.is_empty());
        assert_eq!(stack.size(), 1);

        stack.pop().unwrap();

        assert!(stack.is_empty());
        assert_eq!(stack.size(), 0);
    }

    #[test]
    fn test_overflow_and_underflow() {
        let mut stack = Stack::default();

        // Fill the stack to capacity
        for i in 0..16 {
            stack.push(i as u16).unwrap();
        }

        // Check overflow
        assert_eq!(stack.push(17), Err("Stack overflow"));

        // Empty the stack
        for _ in 0..16 {
            stack.pop().unwrap();
        }

        // Check underflow
        assert_eq!(stack.pop(), Err("Stack underflow"));
    }
}
