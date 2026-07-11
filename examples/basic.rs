use primer::sieve_primes;

fn main() {
    let primes = sieve_primes(1_000);
    println!("Found {} primes up to 1,000.", primes.len());
    println!(
        "The last one is {}.",
        primes.last().expect("1,000 has primes")
    );
}
