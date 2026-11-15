//! Semantic search: the one-time model download. Free space first, each file resumed from its partial copy with an HTTP
//! range, cancellable, and verified against the SHA-256 shipped in the manifest before it is put in place (search R7,
//! search interfaces).

use super::semantic::{ModelFile, SemanticProblem};
use sha2::{Digest, Sha256};
use std::io::{Read, Write};
use std::path::Path;
use tokio::sync::Notify;

/// The files must fit with this much room to spare (search R7).
const SPACE_MARGIN: f64 = 1.1;

/// How a download ended, when it didn't complete.
#[derive(Debug, PartialEq)]
pub(crate) enum Ended {
    Cancelled,
    Failed(SemanticProblem),
}

/// Downloads the files from `source` into `dir` and verifies them; `progress` hears the bytes held so far.
pub(crate) async fn fetch(
    source: &str,
    files: &[ModelFile],
    dir: &Path,
    progress: &(dyn Fn(u64) + Sync),
    stop: &Notify,
) -> Result<(), Ended> {
    let failed = |p| Ended::Failed(p);
    std::fs::create_dir_all(dir).map_err(|_| failed(SemanticProblem::NoSpace))?;
    let total: u64 = files.iter().map(|f| f.bytes).sum();
    let held = |f: &ModelFile| {
        std::fs::metadata(dir.join(format!("{}.part", f.name))).map_or(0, |m| m.len())
    };
    let needed = files
        .iter()
        .map(|f| f.bytes.saturating_sub(held(f)))
        .sum::<u64>();
    let free = fs4::available_space(dir).unwrap_or(u64::MAX);
    if (needed as f64 * SPACE_MARGIN) as u64 > free {
        return Err(failed(SemanticProblem::NoSpace));
    }
    let client = reqwest::Client::builder()
        .user_agent(concat!("MyPec/", env!("CARGO_PKG_VERSION")))
        .build()
        .map_err(|_| failed(SemanticProblem::Unreachable))?;
    let mut before = 0;
    for file in files {
        let part = dir.join(format!("{}.part", file.name));
        let mut have = held(file);
        if have < file.bytes {
            let mut request = client.get(format!("{source}{}", file.path));
            if have > 0 {
                request = request.header("Range", format!("bytes={have}-"));
            }
            let response = tokio::select! {
                r = request.send() => r.map_err(|_| failed(SemanticProblem::Unreachable))?,
                () = stop.notified() => return Err(Ended::Cancelled),
            };
            let status = response.status().as_u16();
            let mut options = std::fs::OpenOptions::new();
            match status {
                206 => options.create(true).append(true),
                // The whole file, also when the server ignored the range: start it over.
                200 => {
                    have = 0;
                    options.create(true).write(true).truncate(true)
                }
                416 => &mut options,
                _ => return Err(failed(SemanticProblem::Unreachable)),
            };
            let mut response = response;
            if status != 416 {
                let mut out = options
                    .open(&part)
                    .map_err(|_| failed(SemanticProblem::NoSpace))?;
                loop {
                    let chunk = tokio::select! {
                        c = response.chunk() => c.map_err(|_| failed(SemanticProblem::Unreachable))?,
                        () = stop.notified() => return Err(Ended::Cancelled),
                    };
                    let Some(chunk) = chunk else { break };
                    out.write_all(&chunk)
                        .map_err(|_| failed(SemanticProblem::NoSpace))?;
                    have += chunk.len() as u64;
                    progress((before + have).min(total));
                }
            }
        }
        // search FR-012: the exact file, or none at all.
        if sha256(&part).as_deref() != Some(file.sha256.as_str()) {
            let _ = std::fs::remove_file(&part);
            return Err(failed(SemanticProblem::Damaged));
        }
        before += file.bytes;
        progress(before.min(total));
    }
    for file in files {
        std::fs::rename(
            dir.join(format!("{}.part", file.name)),
            dir.join(&file.name),
        )
        .map_err(|_| failed(SemanticProblem::NoSpace))?;
    }
    Ok(())
}

/// A file's SHA-256 in lowercase hex.
pub(crate) fn sha256(path: &Path) -> Option<String> {
    let mut file = std::fs::File::open(path).ok()?;
    let mut hasher = Sha256::new();
    let mut buffer = vec![0u8; 1 << 16];
    loop {
        let read = file.read(&mut buffer).ok()?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Some(
        hasher
            .finalize()
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect(),
    )
}
