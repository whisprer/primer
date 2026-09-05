use std::process::Command;

fn primer() -> Command {
    Command::new(env!("CARGO_BIN_EXE_primer"))
}

#[test]
fn version_reports_package_version() {
    let output = primer()
        .arg("--version")
        .output()
        .expect("primer executable should run");

    assert!(output.status.success());
    assert_eq!(
        String::from_utf8(output.stdout).expect("stdout should be UTF-8"),
        format!("primer {}\n", env!("CARGO_PKG_VERSION"))
    );
    assert!(output.stderr.is_empty());
}

#[test]
fn accepts_formatted_limit_and_reports_result() {
    let output = primer()
        .arg("1_000")
        .output()
        .expect("primer executable should run");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout should be UTF-8");
    assert!(stdout.contains("Limit:                 1,000"));
    assert!(stdout.contains("Primes generated:      168"));
    assert!(stdout.contains("Last 10:"));
    assert!(stdout.contains("997"));
    assert!(output.stderr.is_empty());
}

#[test]
fn zero_limit_is_handled_without_slice_errors() {
    let output = primer()
        .arg("0")
        .output()
        .expect("primer executable should run");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout should be UTF-8");
    assert!(stdout.contains("Primes generated:      0"));
    assert!(stdout.contains("First 0: []"));
    assert!(stdout.contains("Last 0:  []"));
}

#[test]
fn invalid_limit_returns_usage_error() {
    let output = primer()
        .arg("not-a-number")
        .output()
        .expect("primer executable should run");

    assert_eq!(output.status.code(), Some(2));
    let stdout = String::from_utf8(output.stdout).expect("stdout should be UTF-8");
    let stderr = String::from_utf8(output.stderr).expect("stderr should be UTF-8");
    assert!(stdout.contains("Usage: primer [LIMIT]"));
    assert!(stderr.contains("error: invalid limit"));
}

#[test]
fn extra_argument_returns_usage_error() {
    let output = primer()
        .args(["100", "200"])
        .output()
        .expect("primer executable should run");

    assert_eq!(output.status.code(), Some(2));
    let stderr = String::from_utf8(output.stderr).expect("stderr should be UTF-8");
    assert!(stderr.contains("error: unexpected extra argument: 200"));
}
