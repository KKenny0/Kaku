use anyhow::{anyhow, bail, Context};
use clap::Parser;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Parser, Clone, Default)]
pub struct ConfigCommand {}

impl ConfigCommand {
    pub fn run(&self) -> anyhow::Result<()> {
        let config_path = resolve_user_config_path();
        ensure_config_exists(&config_path)?;
        open_config(&config_path)?;
        println!("Opened config: {}", config_path.display());
        Ok(())
    }
}

fn resolve_user_config_path() -> PathBuf {
    config::CONFIG_DIRS
        .first()
        .cloned()
        .unwrap_or_else(|| config::HOME_DIR.join(".config").join("kaku"))
        .join("kaku.lua")
}

fn ensure_config_exists(config_path: &Path) -> anyhow::Result<()> {
    if config_path.exists() {
        return Ok(());
    }

    let parent = config_path
        .parent()
        .ok_or_else(|| anyhow!("invalid config path: {}", config_path.display()))?;
    config::create_user_owned_dirs(parent).context("create config directory")?;

    if let Some(template) = find_template_config() {
        std::fs::copy(&template, config_path)
            .with_context(|| format!("copy template config from {}", template.display()))?;
        return Ok(());
    }

    let fallback = "local wezterm = require 'wezterm'\n\nlocal config = wezterm.config_builder()\n\nreturn config\n";
    std::fs::write(config_path, fallback).context("write default config file")?;
    Ok(())
}

fn find_template_config() -> Option<PathBuf> {
    let mut candidates = Vec::new();

    if let Ok(exe) = std::env::current_exe() {
        if let Some(contents_dir) = exe.parent().and_then(|p| p.parent()) {
            candidates.push(contents_dir.join("Resources").join("kaku.lua"));
        }
    }

    candidates.push(PathBuf::from("/Applications/Kaku.app/Contents/Resources/kaku.lua"));
    candidates.push(
        config::HOME_DIR
            .join("Applications")
            .join("Kaku.app")
            .join("Contents")
            .join("Resources")
            .join("kaku.lua"),
    );

    if let Ok(cwd) = std::env::current_dir() {
        candidates.push(
            cwd.join("assets")
                .join("macos")
                .join("Kaku.app")
                .join("Contents")
                .join("Resources")
                .join("kaku.lua"),
        );
    }

    candidates.into_iter().find(|p| p.exists())
}

fn open_config(config_path: &Path) -> anyhow::Result<()> {
    if open_with_editor(config_path)? {
        return Ok(());
    }

    let status = Command::new("/usr/bin/open")
        .arg(config_path)
        .status()
        .context("open config file with default app")?;
    if status.success() {
        return Ok(());
    }
    bail!("failed to open config file: {}", config_path.display());
}

fn open_with_editor(config_path: &Path) -> anyhow::Result<bool> {
    let Some(editor) = std::env::var_os("EDITOR") else {
        return Ok(false);
    };

    let editor = editor.to_string_lossy().trim().to_string();
    if editor.is_empty() {
        return Ok(false);
    }

    let parts = shell_words::split(&editor)
        .with_context(|| format!("failed to parse EDITOR value `{}`", editor))?;
    if parts.is_empty() {
        return Ok(false);
    }

    let status = Command::new(&parts[0])
        .args(parts.iter().skip(1))
        .arg(config_path)
        .status()
        .with_context(|| format!("launch editor `{}`", parts[0]))?;

    Ok(status.success())
}
