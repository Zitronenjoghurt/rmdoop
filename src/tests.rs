use crate::cli::Cli;
use std::fs;
use tempfile::tempdir;

fn create_file(dir: &std::path::Path, name: &str, content: &[u8]) -> std::path::PathBuf {
    let path = dir.join(name);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(&path, content).unwrap();
    path
}

#[test]
fn test_autonomous_with_distinct_source_and_target() -> anyhow::Result<()> {
    let dir = tempdir()?;

    let source_dir = dir.path().join("source");
    let target_dir = dir.path().join("target");

    let orig = create_file(&source_dir, "orig.txt", b"cookies");
    let dup1 = create_file(&target_dir, "dup1.txt", b"cookies");
    let dup2 = create_file(&target_dir, "sub/dup2.txt", b"cookies");
    let unique = create_file(&target_dir, "unique.txt", b"cheesecake");

    let cli = Cli {
        targets: vec![target_dir.clone()],
        sources: vec![source_dir.clone()],
        verbose: true,
        quiet: true,
        autonomous: true,
        list: false,
        promote_first_duplicate: false,
    };

    let files_deleted = cli.execute()?;

    assert_eq!(files_deleted, 2);
    assert!(orig.exists(), "Source must never be deleted");
    assert!(!dup1.exists(), "Target duplicate should be deleted");
    assert!(!dup2.exists(), "Nested target duplicate should be deleted");
    assert!(unique.exists(), "Unique target file should be kept");

    Ok(())
}

#[test]
fn test_autonomous_no_source_no_promote() -> anyhow::Result<()> {
    let dir = tempdir()?;
    let file1 = create_file(dir.path(), "1.txt", b"data");
    let file2 = create_file(dir.path(), "2.txt", b"data");

    let cli = Cli {
        targets: vec![dir.path().to_path_buf()],
        sources: vec![],
        verbose: true,
        quiet: false,
        autonomous: true,
        list: false,
        promote_first_duplicate: false,
    };

    let files_deleted = cli.execute()?;

    assert_eq!(files_deleted, 0);
    assert!(file1.exists());
    assert!(file2.exists());

    Ok(())
}

#[test]
fn test_autonomous_with_promotion() -> anyhow::Result<()> {
    let dir = tempdir()?;
    let dup1 = create_file(dir.path(), "dup1.txt", b"data");
    let dup2 = create_file(dir.path(), "dup2.txt", b"data");
    let dup3 = create_file(dir.path(), "dup3.txt", b"data");

    let cli = Cli {
        targets: vec![dir.path().to_path_buf()],
        sources: vec![],
        verbose: true,
        quiet: false,
        autonomous: true,
        list: false,
        promote_first_duplicate: true,
    };

    let files_deleted = cli.execute()?;
    assert_eq!(files_deleted, 2);

    assert!(!dup1.exists(), "dup1.txt should have been deleted");
    assert!(!dup2.exists(), "dup2.txt should have been deleted");
    assert!(
        dup3.exists(),
        "dup3.txt should have been promoted to source and kept"
    );

    Ok(())
}
