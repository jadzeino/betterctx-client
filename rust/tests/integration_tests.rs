use std::process::Command;

fn better_ctx_bin() -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_better-ctx"));
    cmd.current_dir(env!("CARGO_MANIFEST_DIR"));
    cmd.env("BETTER_CTX_ACTIVE", "1");
    cmd
}

#[test]
fn binary_prints_version() {
    let output = better_ctx_bin()
        .arg("--version")
        .output()
        .expect("failed to run better-ctx");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("better-ctx"),
        "version output should contain 'better-ctx', got: {stdout}"
    );
}

#[test]
fn binary_prints_help() {
    let output = better_ctx_bin()
        .arg("--help")
        .output()
        .expect("failed to run better-ctx");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Intelligence Layer"),
        "help should contain tagline"
    );
    assert!(stdout.contains("better-ctx"), "help should mention better-ctx");
}

#[test]
fn binary_read_file() {
    let output = better_ctx_bin()
        .args(["read", "Cargo.toml", "-m", "signatures"])
        .output()
        .expect("failed to run better-ctx");
    assert!(output.status.success(), "read should succeed");
}

#[test]
fn binary_config_shows_defaults() {
    let output = better_ctx_bin()
        .arg("config")
        .output()
        .expect("failed to run better-ctx");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("checkpoint_interval"),
        "config should show checkpoint_interval"
    );
}

#[test]
fn shell_hook_compresses_echo() {
    let output = better_ctx_bin()
        .args(["-c", "echo", "hello", "world"])
        .output()
        .expect("failed to run better-ctx -c");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("hello"),
        "shell hook should pass through echo output"
    );
}

#[test]
fn disabled_env_bypasses_compression() {
    let output = Command::new(env!("CARGO_BIN_EXE_better-ctx"))
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .env("BETTER_CTX_DISABLED", "1")
        .env("BETTER_CTX_COMPRESS", "1")
        .args(["-c", "echo", "passthrough test"])
        .output()
        .expect("failed to run better-ctx with BETTER_CTX_DISABLED");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("passthrough"),
        "BETTER_CTX_DISABLED should pass output through unmodified"
    );
    assert!(
        !stdout.contains("[better-ctx:"),
        "BETTER_CTX_DISABLED should not add compression markers"
    );
}

#[test]
fn help_shows_environment_section() {
    let output = better_ctx_bin()
        .arg("--help")
        .output()
        .expect("failed to run better-ctx");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("BETTER_CTX_DISABLED"),
        "help should document BETTER_CTX_DISABLED"
    );
    assert!(
        stdout.contains("BETTER_CTX_RAW"),
        "help should document BETTER_CTX_RAW"
    );
}
