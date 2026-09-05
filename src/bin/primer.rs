use primer::{sieve_primes, SEGMENT_BYTES};
use std::env;
use std::mem::size_of;
use std::process::ExitCode;
use std::time::Instant;

const DEFAULT_LIMIT: u64 = 500_000;

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("error: {message}");
            eprintln!();
            print_usage();
            ExitCode::from(2)
        }
    }
}

fn run() -> Result<(), String> {
    let mut arguments = env::args().skip(1);

    let limit = match arguments.next() {
        None => DEFAULT_LIMIT,
        Some(argument) if argument == "-h" || argument == "--help" => {
            print_usage();
            return Ok(());
        }
        Some(argument) if argument == "-V" || argument == "--version" => {
            println!("primer {}", env!("CARGO_PKG_VERSION"));
            return Ok(());
        }
        Some(argument) => parse_limit(&argument)?,
    };

    if let Some(extra) = arguments.next() {
        return Err(format!("unexpected extra argument: {extra}"));
    }

    println!("Primer {}", env!("CARGO_PKG_VERSION"));
    println!("Cache-aware, bit-packed segmented prime sieve");
    println!();

    let started = Instant::now();
    let primes = sieve_primes(limit);
    let elapsed = started.elapsed();

    let preview_length = primes.len().min(10);
    let result_capacity_bytes = primes.capacity() * size_of::<u64>();

    println!("Limit:                 {}", format_integer(limit));
    println!(
        "Primes generated:      {}",
        format_integer(primes.len() as u64)
    );
    println!("Elapsed:               {elapsed:?}");
    println!("Reusable segment:      {} KiB", SEGMENT_BYTES / 1024);
    println!(
        "Result vector capacity: {} bytes",
        format_integer(result_capacity_bytes as u64)
    );
    println!();
    println!("First {preview_length}: {:?}", &primes[..preview_length]);
    println!(
        "Last {preview_length}:  {:?}",
        &primes[primes.len().saturating_sub(preview_length)..]
    );
    println!();
    println!("Note: the 32 KiB figure is the reusable segment buffer; bootstrap primes,");
    println!("the returned vector, allocator metadata, and process overhead are additional.");

    Ok(())
}

fn parse_limit(value: &str) -> Result<u64, String> {
    let normalized: String = value
        .chars()
        .filter(|character| *character != '_' && *character != ',')
        .collect();

    if normalized.is_empty() {
        return Err("the limit is empty".to_owned());
    }

    normalized
        .parse::<u64>()
        .map_err(|error| format!("invalid limit {value:?}: {error}"))
}

fn format_integer(value: u64) -> String {
    let digits = value.to_string();
    let mut output = String::with_capacity(digits.len() + digits.len() / 3);

    for (index, character) in digits.chars().enumerate() {
        if index != 0 && (digits.len() - index) % 3 == 0 {
            output.push(',');
        }
        output.push(character);
    }

    output
}

fn print_usage() {
    println!("Usage: primer [LIMIT]");
    println!();
    println!("Generate every prime up to and including LIMIT.");
    println!("LIMIT may contain commas or underscores.");
    println!();
    println!("Examples:");
    println!("  primer");
    println!("  primer 1_000_000");
    println!("  primer 50,000,000");
    println!();
    println!("Options:");
    println!("  -h, --help       Show this help");
    println!("  -V, --version    Show the version");
}
