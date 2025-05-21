use core::str;
use std::path::Path;

use assert_cmd::Command;
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
fn test_install(tmpdir: TempDir) {
    let test_dir = Path::new("tests/test-env");
    let target = tmpdir;

    Command::cargo_bin("pixi-install-to-prefix")
        .unwrap()
        .current_dir(test_dir)
        .arg(target.as_ref())
        .output()
        .expect("failed to execute pixi-install-to-prefix");

    let history_file = target.path().join("conda-meta/history");
    assert!(history_file.exists());

    validate_prefix_micromamba(&target);
    validate_prefix_conda(&target);
}
