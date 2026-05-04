use std::collections::hash_map::DefaultHasher;
use std::future::Future;
use std::hash::{Hash, Hasher};
use std::path::PathBuf;
use std::pin::Pin;
use std::sync::Arc;
use std::time::SystemTime;

use anyhow::Result;
use serde::Serialize;
use tokio::fs;

use crate::cache::Cache;
use crate::index;

#[derive(Serialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Node {
    File {
        name: String,
        path: String,
        size: u64,
        title: Option<String>,
        headings: Vec<String>,
        hash: String,
    },
    Directory {
        name: String,
        path: String,
        hash: String,
        children: Vec<Node>,
    },
}

pub async fn build(cache: Arc<Cache>, root: PathBuf) -> Result<Node> {
    walk(cache, root, "/".to_string(), String::new()).await
}

fn walk(
    cache: Arc<Cache>,
    fs_path: PathBuf,
    url_path: String,
    name: String,
) -> Pin<Box<dyn Future<Output = Result<Node>> + Send>> {
    Box::pin(async move {
        let mut raw: Vec<(String, bool, u64, SystemTime)> = Vec::new();
        let mut rd = fs::read_dir(&fs_path).await?;
        while let Some(entry) = rd.next_entry().await? {
            let n = match entry.file_name().into_string() {
                Ok(s) => s,
                Err(_) => continue,
            };
            let m = entry.metadata().await?;
            let is_dir = m.is_dir();
            if !is_dir && !n.ends_with(".md") {
                continue;
            }
            raw.push((n, is_dir, m.len(), m.modified()?));
        }
        raw.sort_by(|a, b| match (a.1, b.1) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.0.cmp(&b.0),
        });

        let mut children = Vec::with_capacity(raw.len());
        for (n, is_dir, size, mtime) in raw {
            let child_url = if url_path == "/" {
                format!("/{}", n)
            } else {
                format!("{}/{}", url_path, n)
            };
            let child_fs = fs_path.join(&n);
            if is_dir {
                let node = walk(cache.clone(), child_fs, child_url, n).await?;
                children.push(node);
            } else {
                let meta = index::file_meta(&cache, &child_fs, size, mtime).await?;
                let hash = format!("{:016x}", hash_file(size, mtime));
                children.push(Node::File {
                    name: n,
                    path: child_url,
                    size,
                    title: meta.title,
                    headings: meta.headings,
                    hash,
                });
            }
        }

        let dir_hash = format!("{:016x}", hash_dir(&children));
        Ok(Node::Directory {
            name,
            path: url_path,
            hash: dir_hash,
            children,
        })
    })
}

fn hash_file(size: u64, mtime: SystemTime) -> u64 {
    let mut h = DefaultHasher::new();
    size.hash(&mut h);
    mtime.hash(&mut h);
    h.finish()
}

fn hash_dir(children: &[Node]) -> u64 {
    let mut h = DefaultHasher::new();
    for c in children {
        match c {
            Node::File { name, hash, .. } => {
                b'f'.hash(&mut h);
                name.hash(&mut h);
                hash.hash(&mut h);
            }
            Node::Directory { name, hash, .. } => {
                b'd'.hash(&mut h);
                name.hash(&mut h);
                hash.hash(&mut h);
            }
        }
    }
    h.finish()
}
