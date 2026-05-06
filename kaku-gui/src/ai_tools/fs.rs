//! Filesystem read/write/patch/delete tools.

use anyhow::{Context, Result};
use std::io::{BufRead, Read};
use std::path::PathBuf;

use super::paths::{reject_if_sensitive, reject_relative_cwd_escape, resolve};

pub(super) fn exec_fs_read(args: &serde_json::Value, cwd: &str, cap: usize) -> Result<String> {
    let raw_path = args["path"].as_str().context("missing path")?;
    let path = resolve(raw_path, cwd)?;
    reject_if_sensitive(&path)?;
    reject_relative_cwd_escape(raw_path, &path, cwd)?;
    let file = std::fs::File::open(&path).with_context(|| format!("read {}", path.display()))?;

    let start_line = args["start_line"].as_u64().map(|n| n as usize);
    let end_line = args["end_line"].as_u64().map(|n| n as usize);

    if start_line.is_some() || end_line.is_some() {
        let reader = std::io::BufReader::new(file);
        let start = start_line.unwrap_or(1);
        let end = end_line.unwrap_or(usize::MAX);
        let mut out = String::new();
        let mut line_num = 1usize;
        for line_result in reader.lines() {
            let line = line_result.with_context(|| format!("read {}", path.display()))?;
            if line_num < start {
                line_num += 1;
                continue;
            }
            if line_num > end {
                break;
            }
            out.push_str(&line);
            out.push('\n');
            if out.len() >= cap {
                out.push_str(&format!(
                    "[truncated: output exceeded {} bytes at line {}]",
                    cap, line_num
                ));
                break;
            }
            line_num += 1;
        }
        if out.is_empty() {
            Ok(format!(
                "(no content in lines {}..={})",
                start,
                end_line
                    .map(|n| n.to_string())
                    .unwrap_or_else(|| "EOF".into())
            ))
        } else {
            Ok(out)
        }
    } else {
        let mut buf = Vec::with_capacity(cap + 512);
        file.take((cap + 512) as u64)
            .read_to_end(&mut buf)
            .with_context(|| format!("read {}", path.display()))?;
        Ok(String::from_utf8_lossy(&buf).into_owned())
    }
}

pub(super) fn exec_fs_list(args: &serde_json::Value, cwd: &str) -> Result<String> {
    let raw_path = args["path"].as_str().context("missing path")?;
    let path = resolve(raw_path, cwd)?;
    reject_if_sensitive(&path)?;
    reject_relative_cwd_escape(raw_path, &path, cwd)?;
    let mut entries: Vec<String> = std::fs::read_dir(&path)
        .with_context(|| format!("list {}", path.display()))?
        .filter_map(|e| e.ok())
        .map(|e| {
            let name = e.file_name().to_string_lossy().into_owned();
            if e.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                format!("{}/", name)
            } else {
                name
            }
        })
        .collect();
    entries.sort();
    Ok(entries.join("\n"))
}

pub(super) fn exec_fs_write(args: &serde_json::Value, cwd: &str) -> Result<String> {
    let raw_path = args["path"].as_str().context("missing path")?;
    let path = resolve(raw_path, cwd)?;
    reject_if_sensitive(&path)?;
    reject_relative_cwd_escape(raw_path, &path, cwd)?;
    let content = args["content"].as_str().context("missing content")?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&path, content).with_context(|| format!("write {}", path.display()))?;
    Ok(format!(
        "Written {} bytes to {}",
        content.len(),
        path.display()
    ))
}

pub(super) fn exec_fs_patch(args: &serde_json::Value, cwd: &str) -> Result<String> {
    let raw_path = args["path"].as_str().context("missing path")?;
    let path = resolve(raw_path, cwd)?;
    reject_if_sensitive(&path)?;
    reject_relative_cwd_escape(raw_path, &path, cwd)?;
    let old_text = args["old_text"].as_str().context("missing old_text")?;
    let new_text = args["new_text"].as_str().context("missing new_text")?;
    let original =
        std::fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    if !original.contains(old_text) {
        anyhow::bail!("old_text not found in {}", path.display());
    }
    let patched = original.replacen(old_text, new_text, 1);
    std::fs::write(&path, &patched).with_context(|| format!("write {}", path.display()))?;
    Ok(format!(
        "Patched {} (replaced 1 occurrence)",
        path.display()
    ))
}

pub(super) fn exec_fs_mkdir(args: &serde_json::Value, cwd: &str) -> Result<String> {
    let raw_path = args["path"].as_str().context("missing path")?;
    let path = resolve(raw_path, cwd)?;
    reject_if_sensitive(&path)?;
    reject_relative_cwd_escape(raw_path, &path, cwd)?;
    std::fs::create_dir_all(&path).with_context(|| format!("mkdir {}", path.display()))?;
    Ok(format!("Created {}", path.display()))
}

pub(super) fn exec_fs_delete(args: &serde_json::Value, cwd: &str) -> Result<String> {
    let raw_path = args["path"].as_str().context("missing path")?;
    let path: PathBuf = resolve(raw_path, cwd)?;
    reject_if_sensitive(&path)?;
    reject_relative_cwd_escape(raw_path, &path, cwd)?;
    if path.is_dir() {
        std::fs::remove_dir_all(&path).with_context(|| format!("rmdir {}", path.display()))?;
    } else {
        std::fs::remove_file(&path).with_context(|| format!("rm {}", path.display()))?;
    }
    Ok(format!("Deleted {}", path.display()))
}
