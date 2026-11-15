use crate::{AppError, Core, Result};
use std::fs;
use std::path::{Path, PathBuf};

/// The companion's files, relative to the workspace folder (onboarding data 1).
pub(crate) const FILES: [&str; 5] = [
    "SOUL.md",
    "IDENTITY.md",
    "USER.md",
    "MEMORY.md",
    "avatar/manifest.json",
];

impl Core {
    pub(crate) fn workspace_dir(&self) -> PathBuf {
        self.root.join("workspace")
    }

    pub(crate) fn state_dir(&self) -> PathBuf {
        self.root.join("state")
    }

    pub(crate) fn workspace_exists(&self) -> bool {
        FILES.iter().any(|f| self.workspace_dir().join(f).is_file())
    }

    /// onboarding FR-008: starting fresh moves the old companion aside; nothing is deleted.
    pub(crate) fn backup_workspace(&self, now: u64) -> Result<()> {
        let from = self.workspace_dir();
        let to = self.root.join(format!("workspace-backup-{}", stamp(now)));
        fs::rename(&from, &to).map_err(|_| AppError::not_writable(&from))
    }
}

/// Writes a temporary file, then renames it, so a reader never sees half a file.
pub fn write_atomic(path: &Path, bytes: &[u8]) -> Result<()> {
    let tmp = path.with_file_name(format!(
        "{}.tmp",
        path.file_name().unwrap_or_default().to_string_lossy()
    ));
    let written = path
        .parent()
        .map_or(Ok(()), fs::create_dir_all)
        .and_then(|()| fs::write(&tmp, bytes))
        .and_then(|()| fs::rename(&tmp, path));
    if written.is_err() {
        let _ = fs::remove_file(&tmp);
        return Err(AppError::not_writable(path));
    }
    Ok(())
}

/// onboarding FR-035: setup and recreation never write over a file that exists. Returns whether it wrote.
pub(crate) fn create_if_absent(path: &Path, bytes: &[u8]) -> Result<bool> {
    if path.exists() {
        return Ok(false);
    }
    write_atomic(path, bytes).map(|()| true)
}

/// `YYYYMMDD-HHMMSS` in UTC, from Unix seconds (Howard Hinnant's civil-from-days).
fn stamp(secs: u64) -> String {
    let days = (secs / 86_400) as i64;
    let rem = secs % 86_400;
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = yoe + era * 400 + i64::from(month <= 2);
    format!(
        "{year:04}{month:02}{day:02}-{:02}{:02}{:02}",
        rem / 3600,
        rem % 3600 / 60,
        rem % 60
    )
}

#[cfg(test)]
mod tests {
    #[test]
    fn backup_stamp_is_the_utc_date_and_time() {
        assert_eq!(super::stamp(0), "19700101-000000");
        assert_eq!(super::stamp(1_794_571_234), "20261113-120034");
    }
}
