pub struct Fibonacci {
    prev: u32,
    current: u32,
}

impl Fibonacci {
    pub fn new() -> Fibonacci {
        Fibonacci {
            prev: 0,
            current: 1,
        }
    }
}

impl Iterator for Fibonacci {
    type Item = u32;

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.prev + self.current;
        self.prev = self.current;
        self.current = next;

        Some(next)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_10() {
        let mut fib = Fibonacci::new();
        assert_eq!(Some(1), fib.next());
        assert_eq!(Some(2), fib.next());
        assert_eq!(Some(3), fib.next());
        assert_eq!(Some(5), fib.next());
        assert_eq!(Some(8), fib.next());
        assert_eq!(Some(13), fib.next());
        assert_eq!(Some(21), fib.next());
        assert_eq!(Some(34), fib.next());
        assert_eq!(Some(55), fib.next());
        assert_eq!(Some(89), fib.next());
    }
}
