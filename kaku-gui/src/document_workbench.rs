use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::process::Command;
use termwiz::input::{KeyCode, Modifiers as KeyModifiers};

pub(crate) const EVENT_NAME: &str = "kaku-document-workbench";
const MAX_RECENT_DOCUMENTS: usize = 100;
const MAX_DISCOVERED_DOCUMENTS: usize = 80;
const DEFAULT_WIDTH_PX: usize = 680;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkbenchView {
    Source,
    Preview,
    Split,
}

impl WorkbenchView {
    pub(crate) fn next(self) -> Self {
        match self {
            Self::Source => Self::Preview,
            Self::Preview => Self::Split,
            Self::Split => Self::Source,
        }
    }

    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Source => "Source",
            Self::Preview => "Preview",
            Self::Split => "Split",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DocumentKind {
    Markdown,
    Html,
    Text,
}

impl DocumentKind {
    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Markdown => "MD",
            Self::Html => "HTML",
            Self::Text => "TXT",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DocumentEntry {
    pub(crate) path: PathBuf,
    pub(crate) display_path: String,
    pub(crate) kind: DocumentKind,
    pub(crate) source: DocumentSource,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DocumentSource {
    Recent,
    Git,
    Cwd,
}

impl DocumentSource {
    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Recent => "recent",
            Self::Git => "dirty",
            Self::Cwd => "cwd",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorkbenchHit {
    Panel,
    ResizeHandle,
    Close,
    CopyPath,
    OpenExternal,
    Save,
    Discard,
    Reload,
    View(WorkbenchView),
    SelectDocument(usize),
    Editor,
}

#[derive(Debug)]
pub(crate) struct DocumentWorkbenchState {
    pub(crate) visible: bool,
    pub(crate) focused: bool,
    pub(crate) width_px: usize,
    pub(crate) selected: usize,
    pub(crate) view: WorkbenchView,
    pub(crate) documents: Vec<DocumentEntry>,
    pub(crate) buffer: String,
    original: String,
    pub(crate) cursor: usize,
    pub(crate) scroll_line: usize,
    pub(crate) status: String,
    cwd: Option<PathBuf>,
}

impl Default for DocumentWorkbenchState {
    fn default() -> Self {
        Self {
            visible: false,
            focused: false,
            width_px: load_saved_width().unwrap_or(DEFAULT_WIDTH_PX),
            selected: 0,
            view: WorkbenchView::Split,
            documents: Vec::new(),
            buffer: String::new(),
            original: String::new(),
            cursor: 0,
            scroll_line: 0,
            status: "ready".to_string(),
            cwd: None,
        }
    }
}

impl DocumentWorkbenchState {
    pub(crate) fn toggle(&mut self, cwd: &Path) {
        if self.visible {
            self.visible = false;
            self.focused = false;
            return;
        }
        self.visible = true;
        self.focused = false;
        self.refresh(cwd);
    }

    pub(crate) fn show_unavailable(&mut self, status: impl Into<String>) {
        self.visible = true;
        self.focused = false;
        self.documents.clear();
        self.buffer.clear();
        self.original.clear();
        self.cursor = 0;
        self.scroll_line = 0;
        self.cwd = None;
        self.status = status.into();
    }

    pub(crate) fn refresh(&mut self, cwd: &Path) {
        self.cwd = Some(cwd.to_path_buf());
        let selected_path = self.current_path().map(PathBuf::from);
        match discover_documents(cwd) {
            Ok(docs) => {
                self.documents = docs;
                self.selected = selected_path
                    .and_then(|path| self.documents.iter().position(|doc| doc.path == path))
                    .unwrap_or(0)
                    .min(self.documents.len().saturating_sub(1));
                self.load_selected(cwd);
            }
            Err(err) => {
                self.status = format!("refresh failed: {err:#}");
            }
        }
    }

    pub(crate) fn is_dirty(&self) -> bool {
        self.buffer != self.original
    }

    pub(crate) fn current(&self) -> Option<&DocumentEntry> {
        self.documents.get(self.selected)
    }

    pub(crate) fn current_path(&self) -> Option<&Path> {
        self.current().map(|doc| doc.path.as_path())
    }

    pub(crate) fn select(&mut self, idx: usize, cwd: &Path) {
        if idx < self.documents.len() {
            self.selected = idx;
            self.load_selected(cwd);
        }
    }

    pub(crate) fn load_selected(&mut self, cwd: &Path) {
        let Some(path) = self.current_path().map(PathBuf::from) else {
            self.buffer.clear();
            self.original.clear();
            self.cursor = 0;
            self.scroll_line = 0;
            self.status = "no documents found".to_string();
            return;
        };
        if let Err(err) = validate_document_path(&path, cwd) {
            self.status = err.to_string();
            return;
        }
        match std::fs::read_to_string(&path) {
            Ok(content) => {
                self.original = content.clone();
                self.buffer = content;
                self.cursor = 0;
                self.scroll_line = 0;
                self.status = format!("loaded {}", path.display());
            }
            Err(err) => {
                self.buffer.clear();
                self.original.clear();
                self.cursor = 0;
                self.status = format!("read failed: {err}");
            }
        }
    }

    pub(crate) fn save(&mut self) {
        let Some(path) = self.current_path().map(PathBuf::from) else {
            self.status = "nothing to save".to_string();
            return;
        };
        let Some(cwd) = self.cwd.as_deref() else {
            self.status = "local file pane required".to_string();
            return;
        };
        if let Err(err) = validate_document_path(&path, cwd) {
            self.status = err.to_string();
            return;
        }
        match std::fs::write(&path, &self.buffer) {
            Ok(()) => {
                self.original = self.buffer.clone();
                self.status = format!("saved {}", path.display());
                let _ = record_document_candidate(&path);
            }
            Err(err) => {
                self.status = format!("save failed: {err}");
            }
        }
    }

    pub(crate) fn discard(&mut self) {
        self.buffer = self.original.clone();
        self.cursor = self.cursor.min(self.buffer.len());
        self.status = "discarded local edits".to_string();
    }

    pub(crate) fn copy_path(&mut self) {
        let Some(path) = self.current_path() else {
            self.status = "no path selected".to_string();
            return;
        };
        self.status = format!("copied {}", path.display());
    }

    pub(crate) fn open_external(&mut self) {
        let Some(path) = self.current_path().map(PathBuf::from) else {
            self.status = "no path selected".to_string();
            return;
        };
        let Some(cwd) = self.cwd.as_deref() else {
            self.status = "local file pane required".to_string();
            return;
        };
        if let Err(err) = validate_document_path(&path, cwd) {
            self.status = err.to_string();
            return;
        }
        match url::Url::from_file_path(&path) {
            Ok(url) => {
                wezterm_open_url::open_url(url.as_str());
                self.status = format!("opened {}", path.display());
            }
            Err(()) => {
                self.status = format!("cannot open {}", path.display());
            }
        }
    }

    pub(crate) fn handle_key(&mut self, key: KeyCode, mods: KeyModifiers) -> bool {
        if !self.visible || !self.focused {
            return false;
        }

        let command = mods.contains(KeyModifiers::SUPER) || mods.contains(KeyModifiers::CTRL);
        match (key, mods) {
            (KeyCode::Escape, KeyModifiers::NONE) => {
                self.focused = false;
                self.status = "terminal focus restored".to_string();
                true
            }
            (KeyCode::Tab, KeyModifiers::NONE) => {
                self.view = self.view.next();
                true
            }
            (KeyCode::UpArrow, KeyModifiers::NONE) => {
                self.move_cursor_vertical(-1);
                true
            }
            (KeyCode::DownArrow, KeyModifiers::NONE) => {
                self.move_cursor_vertical(1);
                true
            }
            (KeyCode::LeftArrow, KeyModifiers::NONE) => {
                self.cursor = previous_boundary(&self.buffer, self.cursor);
                true
            }
            (KeyCode::RightArrow, KeyModifiers::NONE) => {
                self.cursor = next_boundary(&self.buffer, self.cursor);
                true
            }
            (KeyCode::Home, KeyModifiers::NONE) => {
                self.cursor = line_start(&self.buffer, self.cursor);
                true
            }
            (KeyCode::End, KeyModifiers::NONE) => {
                self.cursor = line_end(&self.buffer, self.cursor);
                true
            }
            (KeyCode::PageUp, KeyModifiers::NONE) => {
                self.scroll_line = self.scroll_line.saturating_sub(10);
                true
            }
            (KeyCode::PageDown, KeyModifiers::NONE) => {
                self.scroll_line += 10;
                true
            }
            (KeyCode::Backspace, KeyModifiers::NONE) => {
                let prev = previous_boundary(&self.buffer, self.cursor);
                if prev < self.cursor {
                    self.buffer.replace_range(prev..self.cursor, "");
                    self.cursor = prev;
                }
                true
            }
            (KeyCode::Delete, KeyModifiers::NONE) => {
                let next = next_boundary(&self.buffer, self.cursor);
                if next > self.cursor {
                    self.buffer.replace_range(self.cursor..next, "");
                }
                true
            }
            (KeyCode::Enter, KeyModifiers::NONE) => {
                self.insert_text("\n");
                true
            }
            (KeyCode::Char('s'), _) if command => {
                self.save();
                true
            }
            (KeyCode::Char('r'), _) if command => {
                if let Some(cwd) = self.cwd.clone() {
                    self.status = "reloaded from disk".to_string();
                    self.load_selected(&cwd);
                } else {
                    self.status = "local file pane required".to_string();
                }
                true
            }
            (KeyCode::Char('a'), _) if mods.contains(KeyModifiers::CTRL) => {
                self.cursor = line_start(&self.buffer, self.cursor);
                true
            }
            (KeyCode::Char('e'), _) if mods.contains(KeyModifiers::CTRL) => {
                self.cursor = line_end(&self.buffer, self.cursor);
                true
            }
            (KeyCode::Char(c), _) if !command && !mods.contains(KeyModifiers::ALT) => {
                self.insert_text(&c.to_string());
                true
            }
            _ => true,
        }
    }

    pub(crate) fn insert_text(&mut self, text: &str) {
        self.cursor = self.cursor.min(self.buffer.len());
        while !self.buffer.is_char_boundary(self.cursor) {
            self.cursor = self.cursor.saturating_sub(1);
        }
        self.buffer.insert_str(self.cursor, text);
        self.cursor += text.len();
    }

    fn move_cursor_vertical(&mut self, delta: isize) {
        let current_col = self
            .cursor
            .saturating_sub(line_start(&self.buffer, self.cursor));
        let mut line_starts = vec![0usize];
        for (idx, ch) in self.buffer.char_indices() {
            if ch == '\n' {
                line_starts.push(idx + 1);
            }
        }
        let line_idx = line_starts
            .iter()
            .enumerate()
            .rev()
            .find(|(_, start)| **start <= self.cursor)
            .map(|(idx, _)| idx)
            .unwrap_or(0);
        let target_idx = if delta < 0 {
            line_idx.saturating_sub(delta.unsigned_abs())
        } else {
            (line_idx + delta as usize).min(line_starts.len().saturating_sub(1))
        };
        let start = line_starts[target_idx];
        let end = line_starts
            .get(target_idx + 1)
            .copied()
            .unwrap_or_else(|| self.buffer.len());
        self.cursor = (start + current_col).min(end.saturating_sub(1).max(start));
        while !self.buffer.is_char_boundary(self.cursor) {
            self.cursor = self.cursor.saturating_sub(1);
        }
    }

    pub(crate) fn source_lines(&self, width: usize, max_lines: usize) -> Vec<String> {
        let mut lines = Vec::new();
        let cursor_line = self.buffer[..self.cursor.min(self.buffer.len())]
            .bytes()
            .filter(|b| *b == b'\n')
            .count();
        let cursor_col = self
            .cursor
            .saturating_sub(line_start(&self.buffer, self.cursor));
        for (idx, line) in self.buffer.lines().enumerate().skip(self.scroll_line) {
            if lines.len() >= max_lines {
                break;
            }
            let mut line = line.to_string();
            if self.focused && idx == cursor_line {
                let col = cursor_col.min(line.len());
                if line.is_char_boundary(col) {
                    line.insert(col, '|');
                }
            }
            lines.push(format!(
                "{:>4} {}",
                idx + 1,
                truncate(&line, width.saturating_sub(6))
            ));
        }
        if lines.is_empty() {
            lines.push("(empty)".to_string());
        }
        lines
    }

    pub(crate) fn preview_lines(&self, width: usize, max_lines: usize) -> Vec<String> {
        let text = match self.current().map(|doc| doc.kind) {
            Some(DocumentKind::Markdown) => markdown_preview(&self.buffer),
            Some(DocumentKind::Html) => html_preview(&self.buffer),
            Some(DocumentKind::Text) | None => self.buffer.clone(),
        };
        let mut out = Vec::new();
        for line in text.lines() {
            for wrapped in wrap_line(line, width) {
                if out.len() >= max_lines {
                    return out;
                }
                out.push(wrapped);
            }
        }
        if out.is_empty() {
            out.push("(empty preview)".to_string());
        }
        out
    }
}

#[derive(Serialize, Deserialize)]
struct RecentDocumentFile {
    paths: Vec<PathBuf>,
}

#[derive(Serialize, Deserialize)]
struct WorkbenchSettings {
    width_px: usize,
}

fn record_document_candidate(path: &Path) -> Result<()> {
    if !is_supported_document(path) {
        return Ok(());
    }
    let mut paths = load_recent_paths().unwrap_or_default();
    let path = path.to_path_buf();
    paths.retain(|p| p != &path);
    paths.insert(0, path);
    paths.truncate(MAX_RECENT_DOCUMENTS);
    let state = RecentDocumentFile { paths };
    let dir = document_workbench_dir();
    std::fs::create_dir_all(&dir).with_context(|| format!("create {}", dir.display()))?;
    std::fs::write(
        recent_documents_file(),
        serde_json::to_vec_pretty(&state).context("encode document workbench recents")?,
    )
    .context("write document workbench recents")?;
    Ok(())
}

fn document_workbench_dir() -> PathBuf {
    config::DATA_DIR.join("document-workbench")
}

fn recent_documents_file() -> PathBuf {
    document_workbench_dir().join("recent-documents.json")
}

fn settings_file() -> PathBuf {
    document_workbench_dir().join("state.json")
}

pub(crate) fn persist_width(width_px: usize) -> Result<()> {
    let dir = document_workbench_dir();
    std::fs::create_dir_all(&dir).with_context(|| format!("create {}", dir.display()))?;
    let settings = WorkbenchSettings { width_px };
    std::fs::write(
        settings_file(),
        serde_json::to_vec_pretty(&settings).context("encode document workbench state")?,
    )
    .context("write document workbench state")?;
    Ok(())
}

fn load_saved_width() -> Option<usize> {
    let data = std::fs::read(settings_file()).ok()?;
    let settings: WorkbenchSettings = serde_json::from_slice(&data).ok()?;
    Some(settings.width_px)
}

fn load_recent_paths() -> Result<Vec<PathBuf>> {
    let path = recent_documents_file();
    let data = std::fs::read(&path).with_context(|| format!("read {}", path.display()))?;
    let file: RecentDocumentFile = serde_json::from_slice(&data).context("parse recents")?;
    Ok(file.paths)
}

fn discover_documents(cwd: &Path) -> Result<Vec<DocumentEntry>> {
    let cwd_path = std::fs::canonicalize(cwd)
        .with_context(|| format!("resolve working directory {}", cwd.display()))?;
    let mut seen = BTreeSet::new();
    let mut paths = Vec::new();
    for path in load_recent_paths().unwrap_or_default() {
        if validate_document_path(&path, &cwd_path).is_ok() {
            if seen.insert(path.clone()) {
                paths.push((0u8, path));
            }
        }
    }
    for path in git_dirty_documents(&cwd_path) {
        if seen.insert(path.clone()) {
            paths.push((1u8, path));
        }
    }
    for path in cwd_documents(&cwd_path) {
        if seen.insert(path.clone()) {
            paths.push((2u8, path));
        }
    }

    let mut docs = Vec::new();
    for (source, path) in paths.into_iter().take(MAX_DISCOVERED_DOCUMENTS) {
        let source = match source {
            0 => DocumentSource::Recent,
            1 => DocumentSource::Git,
            _ => DocumentSource::Cwd,
        };
        if validate_document_path(&path, &cwd_path).is_ok() {
            let Some(kind) = document_kind(&path) else {
                continue;
            };
            docs.push(DocumentEntry {
                display_path: display_path(&path, &cwd_path),
                path,
                kind,
                source,
            });
        }
    }
    Ok(docs)
}

fn git_dirty_documents(cwd: &Path) -> Vec<PathBuf> {
    let output = Command::new("git")
        .arg("-C")
        .arg(cwd)
        .arg("status")
        .arg("--short")
        .arg("--untracked-files=all")
        .output();
    let Ok(output) = output else {
        return Vec::new();
    };
    if !output.status.success() {
        return Vec::new();
    }
    let text = String::from_utf8_lossy(&output.stdout);
    text.lines()
        .filter_map(|line| {
            let path = line.get(3..)?.trim();
            let path = path.split(" -> ").last().unwrap_or(path);
            let full = cwd.join(path);
            if validate_document_path(&full, cwd).is_ok() {
                Some(full)
            } else {
                None
            }
        })
        .collect()
}

fn cwd_documents(cwd: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    collect_cwd_documents(cwd, 0, &mut out);
    let docs = cwd.join("docs");
    if std::fs::symlink_metadata(&docs)
        .map(|meta| meta.is_dir() && !meta.file_type().is_symlink())
        .unwrap_or(false)
    {
        collect_cwd_documents(&docs, 0, &mut out);
    }
    out.sort();
    out.dedup();
    out.truncate(MAX_DISCOVERED_DOCUMENTS);
    out
}

fn collect_cwd_documents(dir: &Path, depth: usize, out: &mut Vec<PathBuf>) {
    if depth > 2 || out.len() >= MAX_DISCOVERED_DOCUMENTS {
        return;
    }
    let Ok(read_dir) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in read_dir.flatten() {
        let path = entry.path();
        let name = entry.file_name();
        if name.to_string_lossy().starts_with('.') {
            continue;
        }
        let Ok(file_type) = entry.file_type() else {
            continue;
        };
        if file_type.is_symlink() {
            continue;
        }
        if file_type.is_dir() {
            collect_cwd_documents(&path, depth + 1, out);
        } else if file_type.is_file() && is_supported_document(&path) {
            out.push(path);
        }
        if out.len() >= MAX_DISCOVERED_DOCUMENTS {
            break;
        }
    }
}

fn display_path(path: &Path, cwd: &Path) -> String {
    path.strip_prefix(cwd)
        .unwrap_or(path)
        .to_string_lossy()
        .into_owned()
}

fn document_kind(path: &Path) -> Option<DocumentKind> {
    let ext = path.extension()?.to_string_lossy().to_ascii_lowercase();
    match ext.as_str() {
        "md" | "markdown" => Some(DocumentKind::Markdown),
        "html" | "htm" => Some(DocumentKind::Html),
        "txt" | "text" | "log" => Some(DocumentKind::Text),
        _ => None,
    }
}

fn is_supported_document(path: &Path) -> bool {
    document_kind(path).is_some()
}

fn validate_document_path(path: &Path, cwd: &Path) -> Result<()> {
    if !is_supported_document(path) {
        bail!("unsupported document type: {}", path.display());
    }
    let meta =
        std::fs::symlink_metadata(path).with_context(|| format!("inspect {}", path.display()))?;
    if meta.file_type().is_symlink() {
        bail!("refusing symlinked document {}", path.display());
    }
    if !meta.is_file() {
        bail!("not a file: {}", path.display());
    }
    crate::ai_tools::paths::reject_if_sensitive(path)?;
    let canonical_cwd = std::fs::canonicalize(cwd)
        .with_context(|| format!("resolve working directory {}", cwd.display()))?;
    let canonical_path =
        std::fs::canonicalize(path).with_context(|| format!("resolve {}", path.display()))?;
    if !canonical_path.starts_with(&canonical_cwd) {
        bail!("refusing document outside cwd: {}", path.display());
    }
    Ok(())
}

fn markdown_preview(input: &str) -> String {
    let mut out = String::new();
    let mut in_fence = false;
    for line in input.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
            in_fence = !in_fence;
            continue;
        }
        if in_fence {
            out.push_str("    ");
            out.push_str(line);
            out.push('\n');
            continue;
        }
        let line = trimmed
            .trim_start_matches("# ")
            .trim_start_matches("## ")
            .trim_start_matches("### ")
            .trim_start_matches("#### ")
            .trim_start_matches("> ");
        out.push_str(line);
        out.push('\n');
    }
    out
}

fn html_preview(input: &str) -> String {
    let mut out = String::new();
    let mut in_tag = false;
    for ch in input.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => {
                in_tag = false;
                out.push(' ');
            }
            c if !in_tag => out.push(c),
            _ => {}
        }
    }
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

pub(crate) fn truncate(text: &str, width: usize) -> String {
    if width == 0 {
        return String::new();
    }
    let mut out = String::new();
    for ch in text.chars() {
        if out.chars().count() + 1 >= width {
            out.push_str("...");
            return out;
        }
        out.push(ch);
    }
    out
}

fn wrap_line(line: &str, width: usize) -> Vec<String> {
    let width = width.max(8);
    if line.is_empty() {
        return vec![String::new()];
    }
    let mut out = Vec::new();
    let mut current = String::new();
    for ch in line.chars() {
        if current.chars().count() >= width {
            out.push(current);
            current = String::new();
        }
        current.push(ch);
    }
    if !current.is_empty() {
        out.push(current);
    }
    out
}

fn previous_boundary(s: &str, cursor: usize) -> usize {
    let mut prev = 0;
    for (idx, _) in s.char_indices() {
        if idx >= cursor {
            break;
        }
        prev = idx;
    }
    prev
}

fn next_boundary(s: &str, cursor: usize) -> usize {
    for (idx, _) in s.char_indices() {
        if idx > cursor {
            return idx;
        }
    }
    s.len()
}

fn line_start(s: &str, cursor: usize) -> usize {
    s[..cursor.min(s.len())]
        .rfind('\n')
        .map(|idx| idx + 1)
        .unwrap_or(0)
}

fn line_end(s: &str, cursor: usize) -> usize {
    s[cursor.min(s.len())..]
        .find('\n')
        .map(|idx| cursor + idx)
        .unwrap_or_else(|| s.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_supported_documents() {
        assert_eq!(
            document_kind(Path::new("docs/a.md")),
            Some(DocumentKind::Markdown)
        );
        assert_eq!(
            document_kind(Path::new("docs/a.html")),
            Some(DocumentKind::Html)
        );
        assert_eq!(document_kind(Path::new("docs/a.rs")), None);
    }

    #[test]
    fn markdown_preview_strips_heading_marks() {
        let preview = markdown_preview("# Title\n\n- item\n```rs\nfn main() {}\n```");
        assert!(preview.contains("Title"));
        assert!(preview.contains("- item"));
        assert!(preview.contains("fn main()"));
    }

    #[test]
    fn editor_insert_and_backspace_keep_cursor_valid() {
        let mut state = DocumentWorkbenchState::default();
        state.visible = true;
        state.focused = true;
        state.insert_text("hello");
        assert_eq!(state.buffer, "hello");
        assert!(state.handle_key(KeyCode::Backspace, KeyModifiers::NONE));
        assert_eq!(state.buffer, "hell");
    }

    #[test]
    fn validation_rejects_symlinked_documents() {
        let dir = tempfile::tempdir().expect("tempdir");
        let target = dir.path().join("outside.md");
        let link = dir.path().join("note.md");
        std::fs::write(&target, "secret").expect("write target");
        std::os::unix::fs::symlink(&target, &link).expect("symlink");

        let err = validate_document_path(&link, dir.path()).expect_err("symlink must be rejected");
        assert!(err.to_string().contains("symlinked document"));
    }

    #[test]
    fn validation_rejects_documents_outside_cwd() {
        let cwd = tempfile::tempdir().expect("cwd");
        let outside = tempfile::tempdir().expect("outside");
        let doc = outside.path().join("note.md");
        std::fs::write(&doc, "outside").expect("write doc");

        let err = validate_document_path(&doc, cwd.path()).expect_err("outside cwd must reject");
        assert!(err.to_string().contains("outside cwd"));
    }
}
