//! NUL-delimited status records carry literal path bytes.

use std::io::Write;
use std::process::Stdio;

mod common;

#[test]
fn status_paths_with_hint_text_newlines_and_non_utf8_are_byte_exact_under_z() {
    let repo = common::temp_git_repo();
    let git = || {
        let mut cmd = common::native_command("git");
        cmd.current_dir(repo.path())
            .args(["-c", "core.protectNTFS=false"]);
        cmd
    };

    // Put this name only in the index so the same fixture works on Windows.
    let blob = git()
        .args(["hash-object", "-w", "--stdin"])
        .output()
        .expect("hash the empty blob");
    assert!(blob.status.success());
    let oid = std::str::from_utf8(&blob.stdout).expect("object id").trim();
    let path_record = b"(use \"git add x)\nnon-utf8-\xff\0";
    let mut record = format!("100644 {oid}\t").into_bytes();
    record.extend_from_slice(path_record);
    let mut index = git()
        .args(["update-index", "-z", "--index-info"])
        .stdin(Stdio::piped())
        .spawn()
        .expect("run git update-index");
    index
        .stdin
        .take()
        .expect("index stdin")
        .write_all(&record)
        .expect("write index entry");
    assert!(index.wait().expect("wait for git update-index").success());
    std::fs::write(repo.path().join("zz"), b"").expect("write a second path");

    let native = git().args(["status", "-z"]).output().expect("git status");
    assert!(native.status.success());
    assert!(
        native
            .stdout
            .windows(path_record.len())
            .any(|s| s == path_record)
    );
    let rtk = common::rtk_command()
        .current_dir(repo.path())
        .args(["git", "-c", "core.protectNTFS=false", "status", "-z"])
        .output()
        .expect("rtk git status");
    assert_eq!(rtk.status.code(), native.status.code());
    assert_eq!(rtk.stdout, native.stdout);
}
