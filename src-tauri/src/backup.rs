use std::fs;
use std::io::{Read as IoRead, Write as IoWrite};
use std::path::{Path, PathBuf};

use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Key, Nonce,
};
use chrono::Utc;
use crc32fast::Hasher;
use rand::RngCore;
use serde_json;
use tauri::Manager;
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipArchive, ZipWriter};

use crate::models::*;
use crate::Database;

const BACKUP_VERSION: &str = "1.0";
const APP_VERSION: &str = "0.1.0";

pub struct BackupService;

impl BackupService {
    fn get_backup_dir(app: &tauri::AppHandle) -> Result<PathBuf, String> {
        let app_dir = app
            .path()
            .app_data_dir()
            .map_err(|e| format!("Failed to get app data dir: {}", e))?;
        let backup_dir = app_dir.join("backups");
        fs::create_dir_all(&backup_dir)
            .map_err(|e| format!("Failed to create backup dir: {}", e))?;
        Ok(backup_dir)
    }

    fn get_app_data_dir(app: &tauri::AppHandle) -> Result<PathBuf, String> {
        app.path()
            .app_data_dir()
            .map_err(|e| format!("Failed to get app data dir: {}", e))
    }

    pub fn create_backup(
        _db: &Database,
        app: &tauri::AppHandle,
        password: Option<String>,
    ) -> Result<BackupInfo, String> {
        let app_dir = Self::get_app_data_dir(app)?;
        let backup_dir = Self::get_backup_dir(app)?;
        let config = Self::get_backup_config(app)?;

        let now = Utc::now();
        let date_str = now.format("%Y%m%d").to_string();
        let filename = format!("lego-backup-{}.lpk", date_str);
        let backup_path = backup_dir.join(&filename);

        let temp_path = backup_path.with_extension("tmp");

        let db_path = app_dir.join("lego.db");
        let key_path = app_dir.join(".key");
        let images_dir = app_dir.join("images");

        let db_data = fs::read(&db_path)
            .map_err(|e| format!("Failed to read database: {}", e))?;
        let key_data = if key_path.exists() {
            Some(fs::read_to_string(&key_path)
                .map_err(|e| format!("Failed to read key file: {}", e))?)
        } else {
            None
        };

        let mut image_count = 0i32;
        let mut image_entries: Vec<(String, Vec<u8>)> = Vec::new();
        if images_dir.exists() {
            Self::collect_images(&images_dir, &images_dir, &mut image_entries, &mut image_count)?;
        }

        let db_size = db_data.len() as i64;
        let is_encrypted = password.is_some();

        let manifest = BackupManifest {
            version: BACKUP_VERSION.to_string(),
            created_at: now.to_rfc3339(),
            encrypted: is_encrypted,
            db_size,
            image_count,
            app_version: APP_VERSION.to_string(),
        };

        {
            let file = fs::File::create(&temp_path)
                .map_err(|e| format!("Failed to create backup file: {}", e))?;
            let mut zip = ZipWriter::new(file);
            let options = SimpleFileOptions::default()
                .compression_method(CompressionMethod::Deflated);

            let manifest_json = serde_json::to_string_pretty(&manifest)
                .map_err(|e| format!("Failed to serialize manifest: {}", e))?;
            zip.start_file("manifest.json", options)
                .map_err(|e| format!("Failed to add manifest: {}", e))?;
            zip.write_all(manifest_json.as_bytes())
                .map_err(|e| format!("Failed to write manifest: {}", e))?;

            zip.start_file("lego.db", options)
                .map_err(|e| format!("Failed to add database: {}", e))?;
            zip.write_all(&db_data)
                .map_err(|e| format!("Failed to write database: {}", e))?;

            if let Some(ref key_str) = key_data {
                zip.start_file(".key", options)
                    .map_err(|e| format!("Failed to add key file: {}", e))?;
                zip.write_all(key_str.as_bytes())
                    .map_err(|e| format!("Failed to write key file: {}", e))?;
            }

            for (rel_path, img_data) in &image_entries {
                zip.start_file(format!("images/{}", rel_path), options)
                    .map_err(|e| format!("Failed to add image {}: {}", rel_path, e))?;
                zip.write_all(img_data)
                    .map_err(|e| format!("Failed to write image {}: {}", rel_path, e))?;
            }

            zip.finish()
                .map_err(|e| format!("Failed to finalize zip: {}", e))?;
        }

        let final_path = if let Some(ref pwd) = password {
            let raw_data = fs::read(&temp_path)
                .map_err(|e| format!("Failed to read temp backup: {}", e))?;
            let encrypted_data = Self::encrypt_backup_data(&raw_data, pwd)?;
            fs::write(&backup_path, &encrypted_data)
                .map_err(|e| format!("Failed to write encrypted backup: {}", e))?;
            fs::remove_file(&temp_path).ok();
            backup_path
        } else {
            fs::rename(&temp_path, &backup_path)
                .map_err(|e| format!("Failed to rename backup file: {}", e))?;
            backup_path
        };

        Self::cleanup_old_backups(&backup_dir, config.keep_count)?;

        let file_size = fs::metadata(&final_path)
            .map(|m| m.len() as i64)
            .unwrap_or(0);

        Ok(BackupInfo {
            filename,
            file_size,
            created_at: now.to_rfc3339(),
            encrypted: is_encrypted,
            version: BACKUP_VERSION.to_string(),
        })
    }

    fn collect_images(
        base_dir: &Path,
        current_dir: &Path,
        entries: &mut Vec<(String, Vec<u8>)>,
        count: &mut i32,
    ) -> Result<(), String> {
        let dir_entries = fs::read_dir(current_dir)
            .map_err(|e| format!("Failed to read images dir: {}", e))?;

        for entry in dir_entries {
            let entry = entry.map_err(|e| format!("Failed to read dir entry: {}", e))?;
            let path = entry.path();

            if path.is_dir() {
                Self::collect_images(base_dir, &path, entries, count)?;
            } else {
                if let Some(ext) = path.extension() {
                    let ext_str = ext.to_string_lossy().to_lowercase();
                    if ext_str == "jpg" || ext_str == "jpeg" || ext_str == "png" {
                        let rel_path = path.strip_prefix(base_dir)
                            .map_err(|e| format!("Path strip error: {}", e))?
                            .to_string_lossy()
                            .to_string();
                        let data = fs::read(&path)
                            .map_err(|e| format!("Failed to read image {}: {}", rel_path, e))?;
                        entries.push((rel_path, data));
                        *count += 1;
                    }
                }
            }
        }
        Ok(())
    }

    pub fn list_backups(app: &tauri::AppHandle) -> Result<Vec<BackupInfo>, String> {
        let backup_dir = Self::get_backup_dir(app)?;
        let mut backups = Vec::new();

        let entries = fs::read_dir(&backup_dir)
            .map_err(|e| format!("Failed to read backup dir: {}", e))?;

        for entry in entries {
            let entry = entry.map_err(|e| format!("Failed to read dir entry: {}", e))?;
            let path = entry.path();

            if path.extension().map(|e| e == "lpk").unwrap_or(false) {
                let filename = path.file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();

                let file_size = fs::metadata(&path)
                    .map(|m| m.len() as i64)
                    .unwrap_or(0);

                let manifest = Self::read_manifest(&path).ok();

                let (created_at, encrypted, version) = match manifest {
                    Some(m) => (m.created_at, m.encrypted, m.version),
                    None => {
                        let created = fs::metadata(&path)
                            .ok()
                            .and_then(|m| m.created().ok())
                            .map(|t| {
                                let dt: chrono::DateTime<Utc> = t.into();
                                dt.to_rfc3339()
                            })
                            .unwrap_or_default();
                        (created, false, "unknown".to_string())
                    }
                };

                backups.push(BackupInfo {
                    filename,
                    file_size,
                    created_at,
                    encrypted,
                    version,
                });
            }
        }

        backups.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        Ok(backups)
    }

    fn read_manifest(backup_path: &Path) -> Result<BackupManifest, String> {
        let file = fs::File::open(backup_path)
            .map_err(|e| format!("Failed to open backup: {}", e))?;

        let mut archive = ZipArchive::new(file)
            .map_err(|e| format!("Failed to open zip archive: {}", e))?;

        let mut manifest_file = archive.by_name("manifest.json")
            .map_err(|e| format!("Failed to find manifest: {}", e))?;

        let mut manifest_json = String::new();
        manifest_file.read_to_string(&mut manifest_json)
            .map_err(|e| format!("Failed to read manifest: {}", e))?;

        serde_json::from_str(&manifest_json)
            .map_err(|e| format!("Failed to parse manifest: {}", e))
    }

    pub fn restore_backup(
        db: &Database,
        app: &tauri::AppHandle,
        filename: String,
        password: Option<String>,
        mode: String,
    ) -> Result<RestoreResult, String> {
        let app_dir = Self::get_app_data_dir(app)?;
        let backup_dir = Self::get_backup_dir(app)?;
        let backup_path = backup_dir.join(&filename);

        if !backup_path.exists() {
            return Err("Backup file not found".to_string());
        }

        let zip_data = if password.is_some() {
            let encrypted_data = fs::read(&backup_path)
                .map_err(|e| format!("Failed to read backup: {}", e))?;
            Self::decrypt_backup_data(&encrypted_data, password.as_ref().unwrap())?
        } else {
            fs::read(&backup_path)
                .map_err(|e| format!("Failed to read backup: {}", e))?
        };

        let mut archive = ZipArchive::new(std::io::Cursor::new(zip_data))
            .map_err(|e| format!("Failed to open backup archive: {}", e))?;

        let _manifest = {
            let mut manifest_file = archive.by_name("manifest.json")
                .map_err(|e| format!("Failed to find manifest: {}", e))?;
            let mut manifest_json = String::new();
            manifest_file.read_to_string(&mut manifest_json)
                .map_err(|e| format!("Failed to read manifest: {}", e))?;
            serde_json::from_str::<BackupManifest>(&manifest_json)
                .map_err(|e| format!("Failed to parse manifest: {}", e))?
        };

        if mode == "full" {
            let conn_guard = db.get_conn()?;
            drop(conn_guard);

            let db_data = {
                let mut db_file = archive.by_name("lego.db")
                    .map_err(|e| format!("Failed to find database in backup: {}", e))?;
                let mut data = Vec::new();
                db_file.read_to_end(&mut data)
                    .map_err(|e| format!("Failed to read database: {}", e))?;
                data
            };

            let db_path = app_dir.join("lego.db");
            fs::write(&db_path, &db_data)
                .map_err(|e| format!("Failed to write database: {}", e))?;

            if let Ok(mut key_file) = archive.by_name(".key") {
                let mut key_str = String::new();
                key_file.read_to_string(&mut key_str)
                    .map_err(|e| format!("Failed to read key: {}", e))?;
                let key_path = app_dir.join(".key");
                fs::write(&key_path, &key_str)
                    .map_err(|e| format!("Failed to write key file: {}", e))?;
            }

            let images_dir = app_dir.join("images");
            if images_dir.exists() {
                let _ = fs::remove_dir_all(&images_dir);
            }
            fs::create_dir_all(&images_dir)
                .map_err(|e| format!("Failed to create images dir: {}", e))?;

            let mut images_restored = 0i32;
            for i in 0..archive.len() {
                let mut file = archive.by_index(i).map_err(|e| format!("Failed to read zip entry: {}", e))?;
                let name = file.name().to_string();
                if name.starts_with("images/") && !name.ends_with('/') {
                    let rel_path = &name["images/".len()..];
                    let dest_path = images_dir.join(rel_path);
                    if let Some(parent) = dest_path.parent() {
                        fs::create_dir_all(parent)
                            .map_err(|e| format!("Failed to create dir: {}", e))?;
                    }
                    let mut data = Vec::new();
                    file.read_to_end(&mut data)
                        .map_err(|e| format!("Failed to read image: {}", e))?;
                    fs::write(&dest_path, &data)
                        .map_err(|e| format!("Failed to write image: {}", e))?;
                    images_restored += 1;
                }
            }

            Ok(RestoreResult {
                success: true,
                mode: "full".to_string(),
                db_restored: true,
                images_restored,
                key_restored: true,
                message: format!("全量恢复成功，已恢复 {} 张图片", images_restored),
            })
        } else {
            let mut images_restored = 0i32;
            let images_dir = app_dir.join("images");
            fs::create_dir_all(&images_dir)
                .map_err(|e| format!("Failed to create images dir: {}", e))?;

            for i in 0..archive.len() {
                let mut file = archive.by_index(i).map_err(|e| format!("Failed to read zip entry: {}", e))?;
                let name = file.name().to_string();
                if name.starts_with("images/") && !name.ends_with('/') {
                    let rel_path = &name["images/".len()..];
                    let dest_path = images_dir.join(rel_path);
                    if !dest_path.exists() {
                        if let Some(parent) = dest_path.parent() {
                            fs::create_dir_all(parent)
                                .map_err(|e| format!("Failed to create dir: {}", e))?;
                        }
                        let mut data = Vec::new();
                        file.read_to_end(&mut data)
                            .map_err(|e| format!("Failed to read image: {}", e))?;
                        fs::write(&dest_path, &data)
                            .map_err(|e| format!("Failed to write image: {}", e))?;
                        images_restored += 1;
                    }
                }
            }

            Ok(RestoreResult {
                success: true,
                mode: "merge".to_string(),
                db_restored: false,
                images_restored,
                key_restored: false,
                message: format!("增量合并成功，新增 {} 张图片，数据库保持不变", images_restored),
            })
        }
    }

    pub fn delete_backup(app: &tauri::AppHandle, filename: String) -> Result<(), String> {
        let backup_dir = Self::get_backup_dir(app)?;
        let path = backup_dir.join(&filename);
        if path.exists() {
            fs::remove_file(&path)
                .map_err(|e| format!("Failed to delete backup: {}", e))?;
        }
        Ok(())
    }

    pub fn get_backup_config(app: &tauri::AppHandle) -> Result<BackupConfig, String> {
        let app_dir = Self::get_app_data_dir(app)?;
        let config_path = app_dir.join("backup_config.json");

        if config_path.exists() {
            let data = fs::read_to_string(&config_path)
                .map_err(|e| format!("Failed to read backup config: {}", e))?;
            serde_json::from_str(&data)
                .map_err(|e| format!("Failed to parse backup config: {}", e))
        } else {
            Ok(BackupConfig::default())
        }
    }

    pub fn update_backup_config(
        app: &tauri::AppHandle,
        config: BackupConfig,
    ) -> Result<(), String> {
        let app_dir = Self::get_app_data_dir(app)?;
        let config_path = app_dir.join("backup_config.json");

        let json = serde_json::to_string_pretty(&config)
            .map_err(|e| format!("Failed to serialize backup config: {}", e))?;
        fs::write(&config_path, json)
            .map_err(|e| format!("Failed to write backup config: {}", e))?;
        Ok(())
    }

    pub fn check_database_integrity(
        db: &Database,
        app: &tauri::AppHandle,
    ) -> Result<IntegrityCheckResult, String> {
        let mut errors = Vec::new();

        let conn_guard = db.get_conn()?;
        let conn = conn_guard.as_ref().unwrap();

        let integrity_result: String = conn
            .query_row("PRAGMA integrity_check", [], |row| row.get(0))
            .map_err(|e| format!("Failed to run integrity check: {}", e))?;

        if integrity_result != "ok" {
            errors.push(format!("数据库完整性检查失败: {}", integrity_result));
        }

        let tables = ["parts", "part_types", "part_colors", "part_sizes", "locations", "moc_lists", "moc_parts", "operation_logs"];
        for table in &tables {
            let count_result: Result<i64, _> = conn
                .query_row(
                    &format!("SELECT COUNT(*) FROM {}", table),
                    [],
                    |row| row.get(0),
                );
            if let Err(e) = count_result {
                errors.push(format!("表 {} 无法访问: {}", table, e));
            }
        }

        let ok = errors.is_empty();
        let latest_backup = Self::list_backups(app)
            .ok()
            .and_then(|b| b.first().cloned());

        Ok(IntegrityCheckResult {
            ok,
            errors,
            can_auto_recover: latest_backup.is_some(),
            latest_backup,
        })
    }

    pub fn export_backup_to_path(
        _db: &Database,
        app: &tauri::AppHandle,
        filename: String,
        dest_dir: String,
    ) -> Result<String, String> {
        let backup_dir = Self::get_backup_dir(app)?;
        let src_path = backup_dir.join(&filename);

        if !src_path.exists() {
            return Err("Backup file not found".to_string());
        }

        let dest_path = Path::new(&dest_dir).join(&filename);
        fs::copy(&src_path, &dest_path)
            .map_err(|e| format!("Failed to copy backup: {}", e))?;

        Ok(dest_path.to_string_lossy().to_string())
    }

    pub fn import_backup_from_path(
        app: &tauri::AppHandle,
        src_path: String,
    ) -> Result<BackupInfo, String> {
        let backup_dir = Self::get_backup_dir(app)?;
        let src = Path::new(&src_path);

        if !src.exists() {
            return Err("Source file not found".to_string());
        }

        let filename = src.file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        let dest_path = backup_dir.join(&filename);
        fs::copy(src, &dest_path)
            .map_err(|e| format!("Failed to import backup: {}", e))?;

        let file_size = fs::metadata(&dest_path)
            .map(|m| m.len() as i64)
            .unwrap_or(0);

        let manifest = Self::read_manifest(&dest_path).ok();

        let (created_at, encrypted, version) = match manifest {
            Some(m) => (m.created_at, m.encrypted, m.version),
            None => (Utc::now().to_rfc3339(), false, "unknown".to_string()),
        };

        Ok(BackupInfo {
            filename,
            file_size,
            created_at,
            encrypted,
            version,
        })
    }

    fn cleanup_old_backups(backup_dir: &Path, keep_count: i32) -> Result<(), String> {
        let mut backups: Vec<(String, String)> = Vec::new();

        let entries = fs::read_dir(backup_dir)
            .map_err(|e| format!("Failed to read backup dir: {}", e))?;

        for entry in entries {
            let entry = entry.map_err(|e| format!("Failed to read dir entry: {}", e))?;
            let path = entry.path();

            if path.extension().map(|e| e == "lpk").unwrap_or(false) {
                let filename = path.file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();

                let manifest = Self::read_manifest(&path).ok();
                let created_at = manifest
                    .map(|m| m.created_at)
                    .unwrap_or_default();

                backups.push((filename, created_at));
            }
        }

        backups.sort_by(|a, b| b.1.cmp(&a.1));

        let keep = keep_count as usize;
        if backups.len() > keep {
            for (filename, _) in backups.iter().skip(keep) {
                let path = backup_dir.join(filename);
                let _ = fs::remove_file(&path);
            }
        }

        Ok(())
    }

    fn encrypt_backup_data(data: &[u8], password: &str) -> Result<Vec<u8>, String> {
        let mut key_bytes = vec![0u8; 32];
        let password_bytes = password.as_bytes();
        for (i, byte) in key_bytes.iter_mut().enumerate() {
            *byte = password_bytes[i % password_bytes.len()];
        }

        for (i, byte) in key_bytes.iter_mut().enumerate() {
            *byte = *byte ^ (i as u8).wrapping_add(0x5A);
        }

        let mut hasher = Hasher::new();
        hasher.update(&key_bytes);
        let checksum = crc32fast::hash(password.as_bytes());
        let key_val = checksum.to_le_bytes();
        for (i, byte) in key_bytes[i32::BITS as usize / 8..].iter_mut().enumerate() {
            if i < key_val.len() {
                *byte = *byte ^ key_val[i];
            }
        }

        let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&key_bytes));

        let mut nonce_bytes = [0u8; 12];
        rand::thread_rng().fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = cipher
            .encrypt(nonce, data)
            .map_err(|e| format!("Encryption failed: {}", e))?;

        let mut result = Vec::with_capacity(8 + nonce_bytes.len() + ciphertext.len());
        result.extend_from_slice(&checksum.to_le_bytes());
        result.extend_from_slice(&nonce_bytes);
        result.extend_from_slice(&ciphertext);

        Ok(result)
    }

    fn decrypt_backup_data(data: &[u8], password: &str) -> Result<Vec<u8>, String> {
        if data.len() < 20 {
            return Err("Invalid encrypted backup data".to_string());
        }

        let stored_checksum = u32::from_le_bytes(
            data[0..4].try_into().map_err(|e: std::array::TryFromSliceError| format!("Checksum parse error: {}", e))?
        );
        let actual_checksum = crc32fast::hash(password.as_bytes());
        if stored_checksum != actual_checksum {
            return Err("密码错误，无法解密备份".to_string());
        }

        let mut key_bytes = vec![0u8; 32];
        let password_bytes = password.as_bytes();
        for (i, byte) in key_bytes.iter_mut().enumerate() {
            *byte = password_bytes[i % password_bytes.len()];
        }

        for (i, byte) in key_bytes.iter_mut().enumerate() {
            *byte = *byte ^ (i as u8).wrapping_add(0x5A);
        }

        let key_val = actual_checksum.to_le_bytes();
        for (i, byte) in key_bytes[i32::BITS as usize / 8..].iter_mut().enumerate() {
            if i < key_val.len() {
                *byte = *byte ^ key_val[i];
            }
        }

        let nonce_bytes = &data[4..16];
        let ciphertext = &data[16..];

        let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&key_bytes));
        let nonce = Nonce::from_slice(nonce_bytes);

        cipher
            .decrypt(nonce, ciphertext)
            .map_err(|_| "密码错误或备份数据已损坏".to_string())
    }

    pub fn should_auto_backup(app: &tauri::AppHandle) -> Result<bool, String> {
        let config = Self::get_backup_config(app)?;
        if !config.enabled {
            return Ok(false);
        }

        let backups = Self::list_backups(app)?;
        if backups.is_empty() {
            return Ok(true);
        }

        let now = Utc::now();
        if let Some(latest) = backups.first() {
            if let Ok(latest_time) = latest.created_at.parse::<chrono::DateTime<Utc>>() {
                let duration = now - latest_time;
                match config.frequency.as_str() {
                    "daily" => return Ok(duration.num_hours() >= 24),
                    "weekly" => return Ok(duration.num_days() >= 7),
                    _ => return Ok(duration.num_hours() >= 24),
                }
            }
        }

        Ok(true)
    }
}
