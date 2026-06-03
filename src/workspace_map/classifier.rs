use std::path::Path;

/// Classifies a path into one of the legend categories:
/// - directory
/// - rust
/// - config
/// - document
/// - image
/// - pdf
/// - data
/// - binary
/// - unknown
pub fn classify_entry(path: &Path, is_dir: bool) -> (String, Option<String>) {
    if is_dir {
        return ("directory".to_string(), Some("inode/directory".to_string()));
    }

    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    let filename = path
        .file_name()
        .and_then(|f| f.to_str())
        .unwrap_or("")
        .to_lowercase();

    // 1. Rust files
    if ext == "rs" {
        return ("rust".to_string(), Some("text/rust".to_string()));
    }

    // 2. Configuration files
    if ext == "toml"
        || ext == "yaml"
        || ext == "yml"
        || filename == "cargo.lock"
        || filename == "package-lock.json"
        || filename.ends_with(".config.json")
        || filename == "package.json"
        || filename == "tsconfig.json"
    {
        let mime = mime_guess::from_path(path)
            .first_raw()
            .map(|s| s.to_string());
        return ("config".to_string(), mime);
    }

    // 3. Document files
    if ext == "md" || ext == "txt" {
        return ("document".to_string(), Some("text/markdown".to_string()));
    }

    // 4. PDF files
    if ext == "pdf" {
        return ("pdf".to_string(), Some("application/pdf".to_string()));
    }

    // 5. Data files
    if ext == "csv" || ext == "json" {
        return ("data".to_string(), Some("application/json".to_string()));
    }

    // 6. Guess MIME type using mime_guess
    let guessed_mime = mime_guess::from_path(path).first();
    if let Some(ref mime) = guessed_mime {
        let mime_str = mime.to_string();
        if mime_str.starts_with("image/") {
            return ("image".to_string(), Some(mime_str));
        }
        if mime_str.starts_with("text/") {
            return ("document".to_string(), Some(mime_str));
        }
    }

    // 7. Check file signatures using infer (great for checking binary header details)
    if let Ok(file_bytes) = std::fs::read(path) {
        if let Some(kind) = infer::get(&file_bytes) {
            let mime_str = kind.mime_type();
            if mime_str.starts_with("image/") {
                return ("image".to_string(), Some(mime_str.to_string()));
            }
            if mime_str.starts_with("application/pdf") {
                return ("pdf".to_string(), Some(mime_str.to_string()));
            }
            // If it matches binary executable signatures, label as binary
            if mime_str.starts_with("application/x-executable")
                || mime_str.starts_with("application/vnd.microsoft.portable-executable")
            {
                return ("binary".to_string(), Some(mime_str.to_string()));
            }
        }

        // Verify if it contains null bytes, indicating binary files
        let is_binary = file_bytes.iter().take(4096).any(|&b| b == 0);
        if is_binary {
            return (
                "binary".to_string(),
                Some("application/octet-stream".to_string()),
            );
        }
    }

    // 8. Fallback
    let mime_str = guessed_mime.map(|m| m.to_string());
    ("unknown".to_string(), mime_str)
}
