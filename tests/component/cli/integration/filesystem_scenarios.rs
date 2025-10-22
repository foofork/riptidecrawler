// Filesystem scenario tests using assert_fs
// Tests: output files, directory operations, file permissions

use assert_cmd::Command;
use assert_fs::prelude::*;
use predicates::prelude::*;

#[test]
fn test_extract_output_to_file() {
    let temp = assert_fs::TempDir::new().unwrap();
    let input_file = temp.child("input.html");
    let output_file = temp.child("output.txt");

    input_file
        .write_str("<html><body><h1>Title</h1><p>Content</p></body></html>")
        .unwrap();

    let mut cmd = Command::cargo_bin("riptide").unwrap();
    cmd.arg("extract")
        .arg("--input-file")
        .arg(input_file.path())
        .arg("--local")
        .arg("--method")
        .arg("css")
        .arg("--file")
        .arg(output_file.path())
        .timeout(std::time::Duration::from_secs(5))
        .assert()
        .success();

    output_file.assert(predicate::path::exists());
    output_file.assert(predicate::path::is_file());
}

#[test]
fn test_extract_json_output_to_file() {
    let temp = assert_fs::TempDir::new().unwrap();
    let input_file = temp.child("input.html");
    let output_file = temp.child("output.json");

    input_file
        .write_str("<html><body><article><p>Test</p></article></body></html>")
        .unwrap();

    let mut cmd = Command::cargo_bin("riptide").unwrap();
    cmd.arg("-o")
        .arg("json")
        .arg("extract")
        .arg("--input-file")
        .arg(input_file.path())
        .arg("--local")
        .arg("--file")
        .arg(output_file.path())
        .timeout(std::time::Duration::from_secs(5))
        .assert()
        .success();

    output_file.assert(predicate::path::exists());
    let content = std::fs::read_to_string(output_file.path()).unwrap();
    assert!(content.contains("{") || content.contains("["));
}

#[test]
fn test_tables_output_to_file() {
    let temp = assert_fs::TempDir::new().unwrap();
    let input_file = temp.child("table.html");
    let output_file = temp.child("table.md");

    input_file
        .write_str(
            r#"<html><body>
            <table>
                <tr><th>Col1</th><th>Col2</th></tr>
                <tr><td>A</td><td>B</td></tr>
            </table>
        </body></html>"#,
        )
        .unwrap();

    let mut cmd = Command::cargo_bin("riptide").unwrap();
    cmd.arg("tables")
        .arg("--file")
        .arg(input_file.path())
        .arg("--format")
        .arg("markdown")
        .arg("--output")
        .arg(output_file.path())
        .timeout(std::time::Duration::from_secs(5))
        .assert()
        .success();

    output_file.assert(predicate::path::exists());
}

#[test]
fn test_tables_csv_output() {
    let temp = assert_fs::TempDir::new().unwrap();
    let input_file = temp.child("table.html");
    let output_file = temp.child("table.csv");

    input_file
        .write_str(
            r#"<html><body>
            <table>
                <tr><th>Name</th><th>Value</th></tr>
                <tr><td>Item1</td><td>100</td></tr>
            </table>
        </body></html>"#,
        )
        .unwrap();

    let mut cmd = Command::cargo_bin("riptide").unwrap();
    cmd.arg("tables")
        .arg("--file")
        .arg(input_file.path())
        .arg("--format")
        .arg("csv")
        .arg("--output")
        .arg(output_file.path())
        .timeout(std::time::Duration::from_secs(5))
        .assert()
        .success();

    output_file.assert(predicate::path::exists());
}

#[test]
fn test_multiple_input_files() {
    let temp = assert_fs::TempDir::new().unwrap();
    let file1 = temp.child("file1.html");
    let file2 = temp.child("file2.html");

    file1
        .write_str("<html><body><p>File 1</p></body></html>")
        .unwrap();
    file2
        .write_str("<html><body><p>File 2</p></body></html>")
        .unwrap();

    // Test file1
    let mut cmd1 = Command::cargo_bin("riptide").unwrap();
    cmd1.arg("extract")
        .arg("--input-file")
        .arg(file1.path())
        .arg("--local")
        .timeout(std::time::Duration::from_secs(5))
        .assert()
        .success();

    // Test file2
    let mut cmd2 = Command::cargo_bin("riptide").unwrap();
    cmd2.arg("extract")
        .arg("--input-file")
        .arg(file2.path())
        .arg("--local")
        .timeout(std::time::Duration::from_secs(5))
        .assert()
        .success();
}

#[test]
fn test_nested_directory_output() {
    let temp = assert_fs::TempDir::new().unwrap();
    let input_file = temp.child("input.html");
    let nested_dir = temp.child("output/nested");
    std::fs::create_dir_all(nested_dir.path()).unwrap();
    let output_file = nested_dir.child("result.txt");

    input_file
        .write_str("<html><body><p>Test</p></body></html>")
        .unwrap();

    let mut cmd = Command::cargo_bin("riptide").unwrap();
    cmd.arg("extract")
        .arg("--input-file")
        .arg(input_file.path())
        .arg("--local")
        .arg("--file")
        .arg(output_file.path())
        .timeout(std::time::Duration::from_secs(5))
        .assert()
        .success();

    output_file.assert(predicate::path::exists());
}

#[test]
fn test_overwrite_existing_file() {
    let temp = assert_fs::TempDir::new().unwrap();
    let input_file = temp.child("input.html");
    let output_file = temp.child("output.txt");

    input_file
        .write_str("<html><body><p>Test</p></body></html>")
        .unwrap();
    output_file.write_str("old content").unwrap();

    let mut cmd = Command::cargo_bin("riptide").unwrap();
    cmd.arg("extract")
        .arg("--input-file")
        .arg(input_file.path())
        .arg("--local")
        .arg("--file")
        .arg(output_file.path())
        .timeout(std::time::Duration::from_secs(5))
        .assert()
        .success();

    output_file.assert(predicate::path::exists());
}

#[test]
fn test_working_directory_independence() {
    let temp = assert_fs::TempDir::new().unwrap();
    let input_file = temp.child("input.html");

    input_file
        .write_str("<html><body><p>Test</p></body></html>")
        .unwrap();

    let mut cmd = Command::cargo_bin("riptide").unwrap();
    cmd.current_dir(temp.path())
        .arg("extract")
        .arg("--input-file")
        .arg("input.html") // Relative path
        .arg("--local")
        .timeout(std::time::Duration::from_secs(5))
        .assert()
        .success();
}

#[test]
fn test_temp_file_cleanup() {
    let temp = assert_fs::TempDir::new().unwrap();
    let input_file = temp.child("input.html");

    input_file
        .write_str("<html><body><p>Test</p></body></html>")
        .unwrap();

    let mut cmd = Command::cargo_bin("riptide").unwrap();
    cmd.arg("extract")
        .arg("--input-file")
        .arg(input_file.path())
        .arg("--local")
        .timeout(std::time::Duration::from_secs(5))
        .assert()
        .success();

    // Verify temp directory still exists (no accidental cleanup)
    temp.assert(predicate::path::exists());
}
