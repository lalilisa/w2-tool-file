#[cfg(test)]
mod tests {
    use crate::command::ReplaceArgs;
    use crate::FileHandler::{format_size, run_replace};
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_format_size() {
        assert_eq!(format_size(1023, true), "1023.00 B");
        assert_eq!(format_size(1024, true), "1.00 KB");
        assert_eq!(format_size(1_500_000, true), "1.43 MB");
        assert_eq!(format_size(3_000_000_000, true), "2.79 GB");
    }

    #[test]
    fn test_search_replace() -> anyhow::Result<()> {
        let dir = tempdir()?;
        let file_path = dir.path().join("test.txt");
        fs::write(&file_path, "hello world\nhello rust")?;

        let args = ReplaceArgs {
            path: dir.path().to_path_buf(),
            old_string: "hello".to_string(),
            new_string: "hi".to_string(),
            dry_run: false,
            backup: true,
        };

        run_replace(&args)?;

        let content = fs::read_to_string(&file_path)?;
        assert_eq!(content, "hi world\nhi rust");

        Ok(())
    }
}