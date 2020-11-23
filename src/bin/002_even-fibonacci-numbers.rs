use project_euler::fibonacci::Fibonacci;
use std::error::Error as StdError;

struct Input {
    upper_bound: u32,
}

impl Input {
    fn new(upper_bound: u32) -> Input {
        Input { upper_bound }
    }
}

trait EvenFibonacciNumbers {
    fn sum_even_fibonacci_numbers(&self, input: Input) -> u32;
}

struct Brute;

impl EvenFibonacciNumbers for Brute {
    fn sum_even_fibonacci_numbers(&self, input: Input) -> u32 {
        let mut items = Fibonacci::new()
            .take_while(|&n| n <= input.upper_bound)
            .filter(|&n| n % 2 == 0)
            .collect::<Vec<_>>();
        dbg!(items);
        Fibonacci::new()
            .take_while(|&n| n <= input.upper_bound)
            .filter(|&n| n % 2 == 0)
            .sum()
    }
}

fn main() -> Result<(), Box<dyn StdError>> {
    let input = Input {
        upper_bound: 4_000_000,
    };
    let answer = Brute.sum_even_fibonacci_numbers(input);
    dbg!(answer);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn basics<T: EvenFibonacciNumbers>(solution: &T) {
        assert_eq!(2, solution.sum_even_fibonacci_numbers(Input::new(2))); // 2
        assert_eq!(10, solution.sum_even_fibonacci_numbers(Input::new(8))); // 2, 8
        assert_eq!(44, solution.sum_even_fibonacci_numbers(Input::new(34))); // 2, 8, 34
    }

    #[test]
    fn brute() {
        basics(&Brute);
    }
}
