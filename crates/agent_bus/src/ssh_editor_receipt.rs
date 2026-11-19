//! Client-side receipts for ambiguous remote editor commits.
use crate::{companion::identity, setup::atomic_write};
use sha2::{Digest, Sha256};
use std::{
    fs::{self, File},
    io::{self, Read},
    path::{Path, PathBuf},
};

const RECEIPT_LEN: u64 = 129;

#[derive(Debug, PartialEq, Eq)]
pub struct EditorSaveReceipt {
    pub baseline: String,
    pub intended: String,
}

fn valid_hash(hash: &str) -> bool {
    hash.len() == 64 && hash.bytes().all(|byte| matches!(byte, b'0'..=b'9' | b'a'..=b'f'))
}

#[derive(Debug)]
pub struct EditorSaveJournal {
    directory: PathBuf,
    document: String,
}

impl EditorSaveJournal {
    pub(crate) fn new(directory: PathBuf, identity: &[&str]) -> Self {
        let mut digest = Sha256::new();
        for value in identity {
            digest.update((value.len() as u64).to_be_bytes());
            digest.update(value.as_bytes());
        }
        Self { directory, document: format!("{:x}", digest.finalize()) }
    }

    fn path(&self) -> PathBuf {
        self.directory.join(&self.document)
    }

    /// Atomic replacement allows readers to observe a complete receipt without taking a lock.
    pub fn read(&self) -> io::Result<Option<EditorSaveReceipt>> {
        match fs::symlink_metadata(&self.directory) {
            Ok(_) => identity::private_directory(&self.directory)?,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
            Err(error) => return Err(error),
        }
        read_receipt(&self.path())
    }

    pub fn lock(&self) -> io::Result<LockedEditorSaveJournal<'_>> {
        identity::private_directory(&self.directory)?;
        // Stripe locks bound the permanent lock files. Only saves sharing a stripe contend.
        let stripe = u8::from_str_radix(&self.document[..2], 16).expect("SHA-256 hex") % 64;
        let lock = identity::private_file(&self.directory.join(format!("{stripe:02x}.lock")))?;
        lock.try_lock().map_err(|error| match error {
            fs::TryLockError::WouldBlock => io::Error::from(io::ErrorKind::WouldBlock),
            fs::TryLockError::Error(error) => error,
        })?;
        Ok(LockedEditorSaveJournal { journal: self, _lock: lock })
    }
}

fn read_receipt(path: &Path) -> io::Result<Option<EditorSaveReceipt>> {
    let metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error),
    };
    if !metadata.is_file() || metadata.len() != RECEIPT_LEN {
        return Err(io::ErrorKind::InvalidData.into());
    }
    let mut receipt = String::new();
    let file = match File::open(path) {
        Ok(file) => file,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error),
    };
    file.take(RECEIPT_LEN + 1).read_to_string(&mut receipt)?;
    let (baseline, intended) = receipt.split_once('\n').ok_or(io::ErrorKind::InvalidData)?;
    if !valid_hash(baseline) || !valid_hash(intended) {
        return Err(io::ErrorKind::InvalidData.into());
    }
    Ok(Some(EditorSaveReceipt { baseline: baseline.into(), intended: intended.into() }))
}

pub struct LockedEditorSaveJournal<'a> {
    journal: &'a EditorSaveJournal,
    _lock: File,
}

impl LockedEditorSaveJournal<'_> {
    pub fn read(&self) -> io::Result<Option<EditorSaveReceipt>> {
        read_receipt(&self.journal.path())
    }

    pub fn write(&self, baseline: &str, intended: &str) -> io::Result<()> {
        if !valid_hash(baseline) || !valid_hash(intended) {
            return Err(io::ErrorKind::InvalidInput.into());
        }
        atomic_write(&self.journal.path(), format!("{baseline}\n{intended}").as_bytes())
            .map_err(io::Error::other)
    }

    pub fn clear(&self) -> io::Result<()> {
        match fs::remove_file(self.journal.path()) {
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
            result => result,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn receipt_survives_reopen_but_isolated_documents_and_owners_cannot_read_it() {
        let root = tempfile::tempdir().unwrap();
        let journal = |identity: &[&str]| EditorSaveJournal::new(root.path().join("receipts"), identity);
        let original = journal(&["account", "service", "project", "src/文.txt"]);
        assert_eq!(original.read().unwrap(), None);
        let original_lock = original.lock().unwrap();
        assert!(matches!(original.lock(), Err(error) if error.kind() == io::ErrorKind::WouldBlock));
        let baseline = "a".repeat(64);
        let intended = "b".repeat(64);
        original_lock.write(&baseline, &intended).unwrap();
        drop(original_lock);
        let reopened = journal(&["account", "service", "project", "src/文.txt"]);
        assert_eq!(reopened.read().unwrap(), Some(EditorSaveReceipt { baseline, intended }));
        assert_eq!(journal(&["other", "service", "project", "src/文.txt"]).read().unwrap(), None);
        assert_eq!(journal(&["account", "other", "project", "src/文.txt"]).read().unwrap(), None);
        assert_eq!(journal(&["account", "service", "project", "src/other.txt"]).read().unwrap(), None);
        reopened.lock().unwrap().clear().unwrap();
        assert_eq!(original.read().unwrap(), None);
    }
}
