use std::path::Path;

use tokio::fs::File;
use tokio::io::AsyncReadExt;

const PREVIEW_BYTES: usize = 4096;

pub struct Preview {
    pub title: Option<String>,
    pub headings: Vec<String>,
}

pub async fn extract(path: &Path) -> std::io::Result<Preview> {
    let mut file = File::open(path).await?;
    let mut buf = vec![0u8; PREVIEW_BYTES];
    let n = file.read(&mut buf).await?;
    buf.truncate(n);
    let text = String::from_utf8_lossy(&buf);
    Ok(parse(&text))
}

fn parse(text: &str) -> Preview {
    let mut title = None;
    let mut headings = Vec::new();
    for raw in text.lines() {
        let line = raw.trim();
        if title.is_none() {
            if let Some(rest) = strip_heading(line, 1) {
                title = Some(rest.to_string());
                continue;
            }
        }
        if let Some(rest) = strip_heading(line, 2) {
            headings.push(rest.to_string());
        }
    }
    Preview { title, headings }
}

fn strip_heading(line: &str, level: usize) -> Option<&str> {
    let bytes = line.as_bytes();
    if bytes.len() <= level {
        return None;
    }
    if !bytes[..level].iter().all(|&b| b == b'#') {
        return None;
    }
    if bytes[level] != b' ' {
        return None;
    }
    Some(line[level + 1..].trim())
}
