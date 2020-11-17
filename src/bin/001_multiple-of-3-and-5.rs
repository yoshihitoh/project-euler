use std::error::Error as StdError;

#[derive(Debug, Copy, Clone)]
struct Input {
    lhs: i32,
    rhs: i32,
    max: i32,
}

trait SumOfMultiples {
    fn solve(&self, input: Input) -> i32;
}

struct Brute;

impl SumOfMultiples for Brute {
    fn solve(&self, input: Input) -> i32 {
        (1..input.max)
            .filter(|n| n % input.lhs == 0 || n % input.rhs == 0)
            .sum()
    }
}

struct Formula;

impl SumOfMultiples for Formula {
    fn solve(&self, input: Input) -> i32 {
        let multiples_sum = |divisor| {
            let n = (input.max - 1) / divisor;
            (divisor * n * (1 + n)) / 2
        };

        multiples_sum(input.lhs) + multiples_sum(input.rhs) - multiples_sum(input.lhs * input.rhs)
    }
}

fn solve<T: SumOfMultiples>(solution: &T, input: Input) -> i32 {
    solution.solve(input)
}

fn main() -> Result<(), Box<dyn StdError>> {
    let input = Input {
        lhs: 3,
        rhs: 5,
        max: 1000,
    };
    // let answer = solve(&Brute, input);
    let answer = solve(&Formula, input);
    dbg!(answer);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_with<T: SumOfMultiples>(solution: &T) {
        let input = |lhs, rhs, max| Input { max, lhs, rhs };

        assert_eq!(23, solve(solution, input(3, 5, 10)));
        assert_eq!(233_168, solve(solution, input(3, 5, 1000)));

        assert_eq!(32, solve(solution, input(2, 3, 10)));
    }

    #[test]
    fn brute() {
        test_with(&Brute);
    }

    #[test]
    fn formula() {
        test_with(&Formula);
    }
}
