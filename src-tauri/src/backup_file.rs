//! Encrypted owner-selected backup files. Never writes plaintext or overwrites a
//! pre-existing file. Possession of a valid file does not authorize any cloud write.
use crate::{
    error::{Error, Result},
    release::hash,
};
use rand::{rngs::OsRng, RngCore};
use ring::{aead, pbkdf2};
use std::{
    fs::File,
    io::{Read, Seek, SeekFrom, Write},
    num::NonZeroU32,
    path::{Path, PathBuf},
};
use zeroize::Zeroizing;

const MAGIC: &[u8; 8] = b"VILLOWBK";
const VERSION: u32 = 1;
const ITERATIONS: u32 = 600_000;
const HEADER: usize = 52;
const TAG: usize = 16;
pub const MAX_PAYLOAD_BYTES: usize = 256 * 1024 * 1024;

#[derive(Clone)]
pub struct SavedBackup {
    pub path: String,
    pub sha256: String,
    pub bytes: u64,
}

pub fn validate_password(password: &str) -> Result<()> {
    if password.trim().chars().count() < 12 || password.len() > 1024 || password.contains('\0') {
        return Err(Error::BackupPassword);
    }
    Ok(())
}

fn key(password: &str, salt: &[u8]) -> Result<aead::LessSafeKey> {
    let mut bytes = Zeroizing::new([0u8; 32]);
    pbkdf2::derive(
        pbkdf2::PBKDF2_HMAC_SHA256,
        NonZeroU32::new(ITERATIONS).unwrap(),
        salt,
        password.as_bytes(),
        bytes.as_mut(),
    );
    let key = aead::UnboundKey::new(&aead::AES_256_GCM, bytes.as_ref())
        .map_err(|_| Error::BackupInvalid)?;
    Ok(aead::LessSafeKey::new(key))
}

pub fn seal(mut payload: Zeroizing<Vec<u8>>, password: &str) -> Result<Vec<u8>> {
    validate_password(password)?;
    if payload.is_empty() || payload.len() > MAX_PAYLOAD_BYTES {
        return Err(Error::BackupTooLarge);
    }
    let mut header = [0u8; HEADER];
    header[..8].copy_from_slice(MAGIC);
    header[8..12].copy_from_slice(&VERSION.to_le_bytes());
    header[12..16].copy_from_slice(&ITERATIONS.to_le_bytes());
    OsRng.fill_bytes(&mut header[16..44]);
    header[44..52].copy_from_slice(&(payload.len() as u64).to_le_bytes());
    let nonce = aead::Nonce::assume_unique_for_key(header[32..44].try_into().unwrap());
    key(password, &header[16..32])?
        .seal_in_place_append_tag(nonce, aead::Aad::from(&header), &mut *payload)
        .map_err(|_| Error::BackupInvalid)?;
    let mut encrypted = Vec::with_capacity(HEADER + payload.len());
    encrypted.extend_from_slice(&header);
    encrypted.extend_from_slice(&payload);
    Ok(encrypted)
}

pub fn open(bytes: &[u8], password: &str) -> Result<Zeroizing<Vec<u8>>> {
    if bytes.len() < HEADER + TAG
        || bytes.len() > HEADER + MAX_PAYLOAD_BYTES + TAG
        || password.len() > 1024
    {
        return Err(Error::BackupInvalid);
    }
    let header = &bytes[..HEADER];
    if &header[..8] != MAGIC
        || u32::from_le_bytes(header[8..12].try_into().unwrap()) != VERSION
        || u32::from_le_bytes(header[12..16].try_into().unwrap()) != ITERATIONS
    {
        return Err(Error::BackupInvalid);
    }
    let declared = u64::from_le_bytes(header[44..52].try_into().unwrap());
    if declared == 0
        || declared > MAX_PAYLOAD_BYTES as u64
        || bytes.len() as u64 != HEADER as u64 + TAG as u64 + declared
    {
        return Err(Error::BackupInvalid);
    }
    let nonce = aead::Nonce::assume_unique_for_key(header[32..44].try_into().unwrap());
    let mut plaintext = Zeroizing::new(bytes[HEADER..].to_vec());
    let length = key(password, &header[16..32])?
        .open_in_place(nonce, aead::Aad::from(header), &mut plaintext)
        .map_err(|_| Error::BackupInvalid)?
        .len();
    plaintext.truncate(length);
    Ok(plaintext)
}

fn read_bounded(file: &mut File) -> Result<Vec<u8>> {
    let limit = (HEADER + MAX_PAYLOAD_BYTES + TAG) as u64;
    if file.metadata().map_err(|_| Error::BackupStorage)?.len() > limit {
        return Err(Error::BackupTooLarge);
    }
    let mut bytes = Vec::new();
    file.seek(SeekFrom::Start(0))
        .map_err(|_| Error::BackupStorage)?;
    file.take(limit + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| Error::BackupStorage)?;
    if bytes.len() as u64 > limit {
        return Err(Error::BackupTooLarge);
    }
    Ok(bytes)
}

pub fn read(path: &Path, password: &str) -> Result<Zeroizing<Vec<u8>>> {
    let mut file = File::open(path).map_err(|_| Error::BackupStorage)?;
    open(&read_bounded(&mut file)?, password)
}

pub fn write_verified(
    path: &Path,
    payload: Zeroizing<Vec<u8>>,
    password: &str,
) -> Result<SavedBackup> {
    write_with(path, payload, password, false, |file, bytes| {
        file.write_all(bytes)
    })
}

pub fn staging_path(path: &Path) -> Result<PathBuf> {
    let mut filename = path.file_name().ok_or(Error::BackupStorage)?.to_os_string();
    filename.push(".staging");
    Ok(path.with_file_name(filename))
}

pub fn write_verified_managed(
    path: &Path,
    payload: Zeroizing<Vec<u8>>,
    password: &str,
) -> Result<SavedBackup> {
    write_with(path, payload, password, true, |file, bytes| {
        file.write_all(bytes)
    })
}

fn write_with(
    path: &Path,
    payload: Zeroizing<Vec<u8>>,
    password: &str,
    managed: bool,
    write: impl FnOnce(&mut File, &[u8]) -> std::io::Result<()>,
) -> Result<SavedBackup> {
    if !path.is_absolute() || path.file_name().is_none() || path.exists() {
        return Err(Error::BackupStorage);
    }
    let path_string = path.to_str().ok_or(Error::BackupStorage)?.to_owned();
    let parent = path.parent().ok_or(Error::BackupStorage)?;
    let expected = hash(&payload);
    let encrypted = seal(payload, password)?;
    let mut temp = if managed {
        let staging = staging_path(path)?;
        // A hard process kill leaves this exact discoverable ciphertext name.
        // create-new and persist_noclobber still refuse all overwrites.
        tempfile::Builder::new()
            .prefix(staging.file_name().ok_or(Error::BackupStorage)?)
            .rand_bytes(0)
            .tempfile_in(parent)
    } else {
        tempfile::NamedTempFile::new_in(parent)
    }
    .map_err(|_| Error::BackupStorage)?;
    write(temp.as_file_mut(), &encrypted).map_err(|_| Error::BackupStorage)?;
    temp.as_file()
        .sync_all()
        .map_err(|_| Error::BackupStorage)?;
    let disk = read_bounded(temp.as_file_mut())?;
    if disk != encrypted || hash(&open(&disk, password)?) != expected {
        return Err(Error::BackupInvalid);
    }
    let saved = temp
        .persist_noclobber(path)
        .map_err(|_| Error::BackupStorage)?;
    saved.sync_all().map_err(|_| Error::BackupStorage)?;
    drop(saved);
    let final_bytes = read_bounded(&mut File::open(path).map_err(|_| Error::BackupStorage)?)?;
    if final_bytes != encrypted || hash(&open(&final_bytes, password)?) != expected {
        return Err(Error::BackupInvalid);
    }
    Ok(SavedBackup {
        path: path_string,
        sha256: hash(&final_bytes),
        bytes: final_bytes.len() as u64,
    })
}

/// Recheck exact encrypted bytes after successful read-back. This cannot establish
/// restore eligibility or the authenticity of an imported receipt.
pub fn matches_saved(path: &Path, expected: &str, expected_size: u64) -> Result<()> {
    let mut file = File::open(path).map_err(|_| Error::BackupStorage)?;
    let bytes = read_bounded(&mut file)?;
    if bytes.len() as u64 != expected_size || hash(&bytes) != expected {
        return Err(Error::BackupInvalid);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    const PASSWORD: &str = "SENTINEL backup password only";
    fn payload() -> Zeroizing<Vec<u8>> {
        Zeroizing::new(b"SENTINEL private app rows and original instance key".to_vec())
    }

    #[test]
    fn managed_hard_exit_preserves_discoverable_staging() {
        const CHILD_DIR: &str = "VILLOW_SYNTHETIC_BACKUP_CRASH_DIR";
        if let Some(root) = std::env::var_os(CHILD_DIR) {
            let path = Path::new(&root).join("synthetic.villowbackup");
            let _ = write_with(&path, payload(), PASSWORD, true, |file, bytes| {
                file.write_all(&bytes[..bytes.len() / 2])?;
                file.sync_all()?;
                std::process::exit(73); // No Drop: models process termination.
            });
            panic!("child did not terminate while staging");
        }
        let directory = tempfile::tempdir().unwrap();
        let mut child = std::process::Command::new(std::env::current_exe().unwrap());
        child
            .args([
                "--exact",
                "backup_file::tests::managed_hard_exit_preserves_discoverable_staging",
            ])
            .env(CHILD_DIR, directory.path())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null());
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            child.creation_flags(0x08000000);
        }
        assert_eq!(child.status().unwrap().code(), Some(73));
        let path = directory.path().join("synthetic.villowbackup");
        assert!(!path.exists());
        assert!(staging_path(&path).unwrap().exists());
        assert_eq!(std::fs::read_dir(directory.path()).unwrap().count(), 1);
        assert!(read(&staging_path(&path).unwrap(), PASSWORD).is_err());
    }

    #[test]
    fn encrypted_backup_round_trip_is_randomized_and_rejects_wrong_password_or_tampering() {
        let first = seal(payload(), PASSWORD).unwrap();
        let second = seal(payload(), PASSWORD).unwrap();
        assert_ne!(first, second);
        assert!(!first.windows(8).any(|w| w == b"SENTINEL"));
        assert_eq!(*open(&first, PASSWORD).unwrap(), *payload());
        assert!(open(&first, "another unrelated password").is_err());
        for index in [0, 8, 12, 16, 32, 44, HEADER, first.len() - 1] {
            let mut damaged = first.clone();
            damaged[index] ^= 1;
            assert!(open(&damaged, PASSWORD).is_err());
        }
        assert!(open(&first[..first.len() - 1], PASSWORD).is_err());
        let mut extra = first;
        extra.push(0);
        assert!(open(&extra, PASSWORD).is_err());
    }

    #[test]
    fn backup_file_is_read_back_verified_and_never_overwrites_an_existing_file() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("saved.villowbackup");
        let saved = write_verified(&path, payload(), PASSWORD).unwrap();
        assert_eq!(*read(&path, PASSWORD).unwrap(), *payload());
        matches_saved(&path, &saved.sha256, saved.bytes).unwrap();
        let original = std::fs::read(&path).unwrap();
        assert!(write_verified(&path, payload(), PASSWORD).is_err());
        assert_eq!(std::fs::read(&path).unwrap(), original);
        std::fs::write(&path, b"damaged").unwrap();
        assert!(matches_saved(&path, &saved.sha256, saved.bytes).is_err());
    }

    #[test]
    fn incomplete_invalid_or_unwritable_backup_never_produces_a_receipt() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("saved.villowbackup");
        assert!(write_verified(&path, payload(), "short").is_err());
        assert!(!path.exists());
        assert!(
            write_with(&path, payload(), PASSWORD, false, |file, bytes| {
                file.write_all(&bytes[..bytes.len() / 2])?;
                Err(std::io::Error::from(std::io::ErrorKind::StorageFull))
            })
            .is_err()
        );
        assert!(!path.exists());
        assert_eq!(std::fs::read_dir(directory.path()).unwrap().count(), 0);
        assert!(
            write_verified(&directory.path().join("absent/backup"), payload(), PASSWORD).is_err()
        );
        assert!(write_verified(Path::new("relative.villowbackup"), payload(), PASSWORD).is_err());
        assert!(open(&[], PASSWORD).is_err());
        assert!(seal(Zeroizing::new(Vec::new()), PASSWORD).is_err());
    }
}
