use core::str;
use std::path::Path;

use assert_cmd::{Command, cargo::cargo_bin_cmd};
use rstest::{fixture, rstest};
use tempfile::TempDir;

#[fixture]
fn tmpdir() -> TempDir {
    TempDir::new().expect("failed to create temp dir")
}

fn validate_prefix_micromamba(target: &TempDir) {
    let micromamba_output = Command::new("micromamba")
        .arg("-p")
        .arg(target.path())
        .arg("run")
        .arg("which")
        .arg("python")
        .output()
        .expect("failed to execute micromamba on prefix");

    let micromamba_stdout = str::from_utf8(&micromamba_output.stdout).unwrap();
    assert!(micromamba_stdout.trim_end().ends_with("python"));
}

fn validate_prefix_conda(target: &TempDir) {
    let conda_output = Command::new("conda")
        .arg("run")
        .arg("-p")
        .arg(target.path())
        .arg("which")
        .arg("python")
        .output()
        .expect("failed to execute conda on prefix");

    let conda_stdout = str::from_utf8(&conda_output.stdout).unwrap();
    assert!(conda_stdout.trim_end().ends_with("python"));
}

#[rstest]
#[case::by_name("jetson", "openssl-3.3.1-h68df207_0.json")]
#[case::by_subdir_fallback("linux-64", "openssl-3.3.1-h4ab18f5_0.json")]
#[cfg_attr(windows, ignore = "installing unix packages requires a unix host")]
fn test_named_platform_install(
    #[case] platform: &str,
    #[case] expected_openssl_record: &str,
    tmpdir: TempDir,
) {
    let test_dir = Path::new("examples/named-platforms");
    let target = tmpdir;

    let output = cargo_bin_cmd!("pixi-install-to-prefix")
        .current_dir(test_dir)
        .arg(target.as_ref())
        .arg("--platform")
        .arg(platform)
        .output()
        .expect("failed to execute pixi-install-to-prefix");
    assert!(
        output.status.success(),
        "stderr: {}",
        str::from_utf8(&output.stderr).unwrap()
    );

    // The prefix contains the packages of the conda subdir the named platform
    // points to.
    assert!(
        target
            .path()
            .join("conda-meta")
            .join(expected_openssl_record)
            .exists()
    );
}

#[rstest]
#[case::ambiguous_subdir(
    "linux-aarch64",
    "platform linux-aarch64 is ambiguous, use one of the platform names from the lockfile instead: jetson, jetson-cuda12\nValid values for --platform: jetson (linux-aarch64), jetson-cuda12 (linux-aarch64), osx-arm64, workstation (linux-64)"
)]
#[case::unknown_name(
    "does-not-exist",
    "platform not found in lockfile: does-not-exist\nValid values for --platform: jetson (linux-aarch64), jetson-cuda12 (linux-aarch64), osx-arm64, workstation (linux-64)"
)]
fn test_named_platform_install_failure(
    #[case] platform: &str,
    #[case] expected_error: &str,
    tmpdir: TempDir,
) {
    let test_dir = Path::new("examples/named-platforms");
    let target = tmpdir;

    let output = cargo_bin_cmd!("pixi-install-to-prefix")
        .current_dir(test_dir)
        .arg(target.as_ref())
        .arg("--platform")
        .arg(platform)
        .output()
        .expect("failed to execute pixi-install-to-prefix");
    assert!(!output.status.success());

    let stderr = str::from_utf8(&output.stderr).unwrap();
    assert!(stderr.contains(expected_error), "stderr: {}", stderr);
}

#[rstest]
fn test_install(tmpdir: TempDir) {
    let test_dir = Path::new("tests/test-env");
    let target = tmpdir;

    cargo_bin_cmd!("pixi-install-to-prefix")
        .current_dir(test_dir)
        .arg(target.as_ref())
        .output()
        .expect("failed to execute pixi-install-to-prefix");

    let history_file = target.path().join("conda-meta/history");
    assert!(history_file.exists());

    validate_prefix_micromamba(&target);
    validate_prefix_conda(&target);
}
