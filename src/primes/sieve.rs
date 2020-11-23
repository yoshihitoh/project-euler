pub struct SievePrimes {
    sieve: Vec<bool>,
    current: usize,
}

impl SievePrimes {
    pub fn new(max: u32) -> SievePrimes {
        let mut sieve = vec![true; max as usize + 1];
        sieve[0] = false;
        sieve[1] = false;

        SievePrimes { sieve, current: 1 }
    }
}

impl Iterator for SievePrimes {
    type Item = u32;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            self.current += 1;
            if self.current >= self.sieve.len() || self.sieve[self.current] {
                break;
            }
        }

        if self.current < self.sieve.len() {
            if self.current * self.current <= self.sieve.len() {
                for i in (self.current..self.sieve.len())
                    .step_by(self.current)
                    .skip(1)
                {
                    self.sieve[i] = false;
                }
            }

            Some(self.current as u32)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::primes::sieve::SievePrimes;

    #[test]
    fn sieve_primes() {
        let mut primes = SievePrimes::new(17);
        assert_eq!(Some(2), primes.next());
        assert_eq!(Some(3), primes.next());
        assert_eq!(Some(5), primes.next());
        assert_eq!(Some(7), primes.next());
        assert_eq!(Some(11), primes.next());
        assert_eq!(Some(13), primes.next());
        assert_eq!(Some(17), primes.next());
        assert_eq!(None, primes.next());
    }
}
