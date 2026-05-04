use std::collections::hash_map::DefaultHasher;
use std::fmt::Write as _;
use std::hash::{Hash, Hasher};
use std::path::Path;
use std::sync::Arc;
use std::time::SystemTime;

use anyhow::Result;
use tokio::fs;

use crate::cache::{Cache, DirIndex, FileMeta};
use crate::markdown;

struct Entry {
    name: String,
    is_dir: bool,
    size: u64,
    mtime: SystemTime,
}

pub async fn render(cache: &Cache, dir_path: &Path, url_path: &str) -> Result<Arc<String>> {
    let mut entries = Vec::new();
    let mut rd = fs::read_dir(dir_path).await?;
    while let Some(e) = rd.next_entry().await? {
        let name = match e.file_name().into_string() {
            Ok(s) => s,
            Err(_) => continue,
        };
        let meta = e.metadata().await?;
        let is_dir = meta.is_dir();
        if !is_dir && !name.ends_with(".md") {
            continue;
        }
        entries.push(Entry {
            name,
            is_dir,
            size: meta.len(),
            mtime: meta.modified()?,
        });
    }
    entries.sort_by(|a, b| match (a.is_dir, b.is_dir) {
        (true, false) => std::cmp::Ordering::Less,
        (false, true) => std::cmp::Ordering::Greater,
        _ => a.name.cmp(&b.name),
    });

    let fingerprint = fingerprint(&entries);
    if let Some(cached) = cache.dirs.get(dir_path) {
        if cached.fingerprint == fingerprint {
            return Ok(cached.rendered.clone());
        }
    }

    let mut out = String::with_capacity(512);
    let _ = writeln!(out, "## {}\n", url_path);
    let _ = writeln!(out, "| Name | Title | Sections |");
    let _ = writeln!(out, "| --- | --- | --- |");
    for e in &entries {
        let display = if e.is_dir {
            format!("{}/", e.name)
        } else {
            e.name.clone()
        };
        let link = format!(
            "[{}]({})",
            escape_link_text(&display),
            build_url(url_path, &e.name)
        );
        if e.is_dir {
            let _ = writeln!(out, "| {} |  |  |", link);
        } else {
            let entry_path = dir_path.join(&e.name);
            let meta = file_meta(cache, &entry_path, e.size, e.mtime).await?;
            let title = meta.title.as_deref().map(escape_cell).unwrap_or_default();
            let sections = if meta.headings.is_empty() {
                String::new()
            } else {
                meta.headings
                    .iter()
                    .map(|h| escape_cell(h))
                    .collect::<Vec<_>>()
                    .join(", ")
            };
            let _ = writeln!(out, "| {} | {} | {} |", link, title, sections);
        }
    }

    let rendered = Arc::new(out);
    cache.dirs.insert(
        dir_path.to_path_buf(),
        DirIndex {
            fingerprint,
            rendered: rendered.clone(),
        },
    );
    Ok(rendered)
}

fn fingerprint(entries: &[Entry]) -> u64 {
    let mut hasher = DefaultHasher::new();
    for e in entries {
        e.name.hash(&mut hasher);
        e.is_dir.hash(&mut hasher);
        e.size.hash(&mut hasher);
        e.mtime.hash(&mut hasher);
    }
    hasher.finish()
}

pub(crate) async fn file_meta(
    cache: &Cache,
    path: &Path,
    size: u64,
    mtime: SystemTime,
) -> Result<FileMeta> {
    if let Some(cached) = cache.files.get(path) {
        if cached.size == size && cached.mtime == mtime {
            return Ok(cached.clone());
        }
    }
    let preview = markdown::extract(path).await?;
    let meta = FileMeta {
        size,
        mtime,
        title: preview.title,
        headings: preview.headings,
    };
    cache.files.insert(path.to_path_buf(), meta.clone());
    Ok(meta)
}

fn escape_cell(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '|' => out.push_str("\\|"),
            '\n' | '\r' => out.push(' '),
            _ => out.push(ch),
        }
    }
    out
}

fn escape_link_text(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '|' => out.push_str("\\|"),
            '[' => out.push_str("\\["),
            ']' => out.push_str("\\]"),
            '\n' | '\r' => out.push(' '),
            _ => out.push(ch),
        }
    }
    out
}

fn build_url(parent: &str, name: &str) -> String {
    let mut segments: Vec<String> = parent
        .split('/')
        .filter(|s| !s.is_empty())
        .map(encode_segment)
        .collect();
    segments.push(encode_segment(name));
    format!("/{}", segments.join("/"))
}

fn encode_segment(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for byte in s.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(byte as char);
            }
            _ => {
                let _ = write!(out, "%{:02X}", byte);
            }
        }
    }
    out
}
