//! Per-user data location. The app used to be called AirCard; its data is carried over.

use std::fs;
use std::path::{Path, PathBuf};

pub const APP_DIR: &str = "NexusCard";
const LEGACY_DIR: &str = "AirCard";

pub fn local_app_data() -> PathBuf {
    PathBuf::from(
        std::env::var("LOCALAPPDATA").unwrap_or_else(|_| r"C:\Users\Default\AppData\Local".to_string()),
    )
}

/// Moves (or merges) the old `%LOCALAPPDATA%\AirCard` folder into `%LOCALAPPDATA%\NexusCard`
/// so saved cards, settings and original-card backups survive the rename.
pub fn migrate_legacy_data() {
    migrate(&local_app_data().join(LEGACY_DIR), &local_app_data().join(APP_DIR));
}

fn migrate(old: &Path, new: &Path) {
    if !old.is_dir() {
        return;
    }
    if !new.exists() && fs::rename(old, new).is_ok() {
        return;
    }
    // Both exist (or the rename failed): copy whatever the new folder does not have yet.
    copy_missing(old, new);
}

fn copy_missing(from: &Path, to: &Path) {
    let _ = fs::create_dir_all(to);
    let Ok(entries) = fs::read_dir(from) else {
        return;
    };
    for entry in entries.flatten() {
        let source = entry.path();
        let target = to.join(entry.file_name());
        if source.is_dir() {
            copy_missing(&source, &target);
        } else if !target.exists() {
            let _ = fs::copy(&source, &target);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("nexuscard-paths-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn moves_the_old_folder_when_the_new_one_is_missing() {
        let root = temp("move");
        let (old, new) = (root.join("AirCard"), root.join("NexusCard"));
        fs::create_dir_all(old.join("wallet-backups")).unwrap();
        fs::write(old.join("cards.json"), "[]").unwrap();
        fs::write(old.join("wallet-backups/a.png"), "x").unwrap();

        migrate(&old, &new);
        assert!(!old.exists());
        assert_eq!(fs::read_to_string(new.join("cards.json")).unwrap(), "[]");
        assert!(new.join("wallet-backups/a.png").is_file());
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn merges_without_overwriting_when_both_exist() {
        let root = temp("merge");
        let (old, new) = (root.join("AirCard"), root.join("NexusCard"));
        fs::create_dir_all(&old).unwrap();
        fs::create_dir_all(&new).unwrap();
        fs::write(old.join("cards.json"), "old").unwrap();
        fs::write(old.join("settings.json"), "old-settings").unwrap();
        fs::write(new.join("settings.json"), "new-settings").unwrap();

        migrate(&old, &new);
        assert_eq!(fs::read_to_string(new.join("cards.json")).unwrap(), "old");
        assert_eq!(fs::read_to_string(new.join("settings.json")).unwrap(), "new-settings");
        let _ = fs::remove_dir_all(root);
    }
}
