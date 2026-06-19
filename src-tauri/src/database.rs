use std::path::PathBuf;
use std::sync::Mutex;

use base64::Engine;
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, OptionalExtension};
use tauri::Manager;
use uuid::Uuid;

use crate::crypto::CryptoService;
use crate::models::*;

pub struct Database {
    conn: Mutex<Option<Connection>>,
    crypto: CryptoService,
    app_data_dir: Mutex<Option<PathBuf>>,
}

impl Database {
    pub fn new() -> Self {
        Self {
            conn: Mutex::new(None),
            crypto: CryptoService::new(),
            app_data_dir: Mutex::new(None),
        }
    }

    pub fn init(&self, app: &tauri::AppHandle) -> Result<(), String> {
        let app_dir = app
            .path()
            .app_data_dir()
            .map_err(|e| format!("Failed to get app data dir: {}", e))?;

        std::fs::create_dir_all(&app_dir)
            .map_err(|e| format!("Failed to create app data dir: {}", e))?;

        let images_dir = app_dir.join("images");
        std::fs::create_dir_all(&images_dir)
            .map_err(|e| format!("Failed to create images dir: {}", e))?;

        let moc_images_dir = images_dir.join("moc");
        std::fs::create_dir_all(&moc_images_dir)
            .map_err(|e| format!("Failed to create moc images dir: {}", e))?;

        *self.app_data_dir.lock().unwrap() = Some(app_dir.clone());

        let key_path = app_dir.join(".key");
        let key = if key_path.exists() {
            let key_b64 = std::fs::read_to_string(&key_path)
                .map_err(|e| format!("Failed to read key file: {}", e))?;
            base64::engine::general_purpose::STANDARD
                .decode(key_b64.trim())
                .map_err(|e| format!("Failed to decode key: {}", e))?
        } else {
            let new_key = CryptoService::generate_key();
            let key_b64 = base64::engine::general_purpose::STANDARD.encode(&new_key);
            std::fs::write(&key_path, key_b64)
                .map_err(|e| format!("Failed to write key file: {}", e))?;
            new_key
        };

        self.crypto.init_with_key(&key)?;

        let db_path = app_dir.join("lego.db");
        let conn = Connection::open(&db_path)
            .map_err(|e| format!("Failed to open database: {}", e))?;

        self.init_tables(&conn)?;
        self.init_default_data(&conn)?;

        *self.conn.lock().unwrap() = Some(conn);

        Ok(())
    }

    fn init_tables(&self, conn: &Connection) -> Result<(), String> {
        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS part_types (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                code TEXT NOT NULL UNIQUE,
                description TEXT
            );

            CREATE TABLE IF NOT EXISTS part_colors (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                hex TEXT NOT NULL,
                lego_code TEXT
            );

            CREATE TABLE IF NOT EXISTS part_sizes (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                width REAL NOT NULL,
                height REAL NOT NULL,
                unit TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS locations (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                code TEXT NOT NULL UNIQUE,
                description TEXT,
                parent_id TEXT,
                FOREIGN KEY (parent_id) REFERENCES locations(id)
            );

            CREATE TABLE IF NOT EXISTS parts (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                part_number TEXT NOT NULL,
                type TEXT NOT NULL,
                color TEXT NOT NULL,
                size TEXT NOT NULL,
                quantity INTEGER NOT NULL DEFAULT 0,
                location TEXT NOT NULL,
                description TEXT,
                image_path TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_parts_type ON parts(type);
            CREATE INDEX IF NOT EXISTS idx_parts_color ON parts(color);
            CREATE INDEX IF NOT EXISTS idx_parts_size ON parts(size);
            CREATE INDEX IF NOT EXISTS idx_parts_location ON parts(location);
            CREATE INDEX IF NOT EXISTS idx_parts_part_number ON parts(part_number);

            CREATE TABLE IF NOT EXISTS moc_lists (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                description TEXT,
                cover_image_path TEXT,
                status TEXT NOT NULL DEFAULT 'planning',
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS moc_parts (
                id TEXT PRIMARY KEY,
                moc_id TEXT NOT NULL,
                part_id TEXT NOT NULL,
                part_number TEXT NOT NULL,
                part_name TEXT NOT NULL,
                color TEXT NOT NULL,
                quantity INTEGER NOT NULL,
                FOREIGN KEY (moc_id) REFERENCES moc_lists(id) ON DELETE CASCADE
            );

            CREATE INDEX IF NOT EXISTS idx_moc_parts_moc_id ON moc_parts(moc_id);

            CREATE TABLE IF NOT EXISTS moc_status_logs (
                id TEXT PRIMARY KEY,
                moc_id TEXT NOT NULL,
                old_status TEXT,
                new_status TEXT NOT NULL,
                changed_at TEXT NOT NULL,
                remark TEXT,
                FOREIGN KEY (moc_id) REFERENCES moc_lists(id) ON DELETE CASCADE
            );

            CREATE INDEX IF NOT EXISTS idx_moc_status_logs_moc_id ON moc_status_logs(moc_id);

            CREATE TABLE IF NOT EXISTS operation_logs (
                id TEXT PRIMARY KEY,
                operation_type TEXT NOT NULL,
                object_type TEXT NOT NULL,
                object_id TEXT NOT NULL,
                object_name TEXT,
                before_snapshot TEXT,
                after_snapshot TEXT,
                changed_at TEXT NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_operation_logs_object ON operation_logs(object_type, object_id);
            CREATE INDEX IF NOT EXISTS idx_operation_logs_type ON operation_logs(operation_type);
            CREATE INDEX IF NOT EXISTS idx_operation_logs_changed_at ON operation_logs(changed_at DESC);
            ",
        )
        .map_err(|e| format!("Failed to create tables: {}", e))?;

        self.run_migrations(conn)?;

        Ok(())
    }

    fn run_migrations(&self, conn: &Connection) -> Result<(), String> {
        let columns: Vec<String> = conn
            .prepare("PRAGMA table_info(moc_lists)")
            .map_err(|e| format!("Failed to prepare pragma: {}", e))?
            .query_map([], |row| row.get::<_, String>(1))
            .map_err(|e| format!("Failed to query columns: {}", e))?
            .filter_map(|r| r.ok())
            .collect();

        if !columns.iter().any(|c| c == "cover_image_path") {
            conn.execute_batch(
                "ALTER TABLE moc_lists ADD COLUMN cover_image_path TEXT;",
            )
            .map_err(|e| format!("Failed to add cover_image_path column: {}", e))?;
        }

        if !columns.iter().any(|c| c == "status") {
            conn.execute_batch(
                "ALTER TABLE moc_lists ADD COLUMN status TEXT NOT NULL DEFAULT 'planning';",
            )
            .map_err(|e| format!("Failed to add status column: {}", e))?;
        }

        Ok(())
    }

    fn init_default_data(&self, conn: &Connection) -> Result<(), String> {
        let type_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM part_types", [], |row| row.get(0))
            .map_err(|e| format!("Failed to count part types: {}", e))?;

        if type_count == 0 {
            let default_types = vec![
                ("砖", "BRICK", "标准乐高砖"),
                ("板", "PLATE", "薄板"),
                ("圆砖", "ROUND_BRICK", "圆形砖"),
                ("斜坡", "SLOPE", "斜坡砖"),
                ("科技件", "TECHNIC", "科技系列零件"),
                ("小人仔", "MINIFIG", "小人仔及配件"),
                ("轮子", "WHEEL", "轮子及轮胎"),
                ("窗户", "WINDOW", "窗户和门"),
                ("特殊件", "SPECIAL", "特殊形状零件"),
            ];

            for (name, code, desc) in default_types {
                let id = Uuid::new_v4().to_string();
                conn.execute(
                    "INSERT INTO part_types (id, name, code, description) VALUES (?1, ?2, ?3, ?4)",
                    params![id, name, code, desc],
                )
                .map_err(|e| format!("Failed to insert part type: {}", e))?;
            }
        }

        let color_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM part_colors", [], |row| row.get(0))
            .map_err(|e| format!("Failed to count part colors: {}", e))?;

        if color_count == 0 {
            let default_colors = vec![
                ("白色", "#FFFFFF", Some("1")),
                ("黑色", "#000000", Some("26")),
                ("红色", "#C91A09", Some("21")),
                ("蓝色", "#0055BF", Some("23")),
                ("黄色", "#F2CD37", Some("24")),
                ("绿色", "#237841", Some("28")),
                ("橙色", "#FE8A18", Some("192")),
                ("紫色", "#81007B", Some("22")),
                ("粉色", "#FC97AC", Some("231")),
                ("浅灰", "#9BA19D", Some("199")),
                ("深灰", "#6B6A67", Some("199")),
                ("棕色", "#583927", Some("192")),
                ("透明", "#FCFCFC", Some("40")),
            ];

            for (name, hex, lego_code) in default_colors {
                let id = Uuid::new_v4().to_string();
                conn.execute(
                    "INSERT INTO part_colors (id, name, hex, lego_code) VALUES (?1, ?2, ?3, ?4)",
                    params![id, name, hex, lego_code],
                )
                .map_err(|e| format!("Failed to insert part color: {}", e))?;
            }
        }

        let size_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM part_sizes", [], |row| row.get(0))
            .map_err(|e| format!("Failed to count part sizes: {}", e))?;

        if size_count == 0 {
            let default_sizes = vec![
                ("1x1", 1.0, 1.0, "stud"),
                ("1x2", 1.0, 2.0, "stud"),
                ("2x2", 2.0, 2.0, "stud"),
                ("2x3", 2.0, 3.0, "stud"),
                ("2x4", 2.0, 4.0, "stud"),
                ("2x6", 2.0, 6.0, "stud"),
                ("4x4", 4.0, 4.0, "stud"),
                ("4x6", 4.0, 6.0, "stud"),
                ("4x10", 4.0, 10.0, "stud"),
                ("圆形1x1", 1.0, 1.0, "stud"),
                ("圆形2x2", 2.0, 2.0, "stud"),
            ];

            for (name, width, height, unit) in default_sizes {
                let id = Uuid::new_v4().to_string();
                conn.execute(
                    "INSERT INTO part_sizes (id, name, width, height, unit) VALUES (?1, ?2, ?3, ?4, ?5)",
                    params![id, name, width, height, unit],
                )
                .map_err(|e| format!("Failed to insert part size: {}", e))?;
            }
        }

        let location_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM locations", [], |row| row.get(0))
            .map_err(|e| format!("Failed to count locations: {}", e))?;

        if location_count == 0 {
            let default_locations: Vec<(&str, &str, &str, Option<&str>)> = vec![
                ("收纳盒 A", "BOX_A", "主收纳盒 - 常用零件", None),
                ("收纳盒 B", "BOX_B", "主收纳盒 - 备用零件", None),
                ("抽屉 1", "DRAWER_1", "工作台抽屉1", None),
                ("抽屉 2", "DRAWER_2", "工作台抽屉2", None),
                ("展示架", "DISPLAY", "展示架上的模型", None),
            ];

            for (name, code, desc, parent_id) in default_locations {
                let id = Uuid::new_v4().to_string();
                conn.execute(
                    "INSERT INTO locations (id, name, code, description, parent_id) VALUES (?1, ?2, ?3, ?4, ?5)",
                    params![id, name, code, desc, parent_id],
                )
                .map_err(|e| format!("Failed to insert location: {}", e))?;
            }
        }

        Ok(())
    }

    fn get_conn(&self) -> Result<std::sync::MutexGuard<'_, Option<Connection>>, String> {
        let guard = self.conn.lock().unwrap();
        if guard.is_none() {
            return Err("Database not initialized".to_string());
        }
        Ok(guard)
    }

    pub fn get_encryption_key(&self) -> String {
        self.crypto.get_key_base64()
    }

    pub fn change_encryption_key(&self, old_key_b64: &str, new_key_b64: &str) -> Result<(), String> {
        use base64::engine::general_purpose::STANDARD as BASE64;

        let old_key = BASE64
            .decode(old_key_b64)
            .map_err(|e| format!("Invalid old key: {}", e))?;
        let new_key = BASE64
            .decode(new_key_b64)
            .map_err(|e| format!("Invalid new key: {}", e))?;

        if old_key.len() != 32 || new_key.len() != 32 {
            return Err("Key must be 32 bytes".to_string());
        }

        let conn_guard = self.get_conn()?;
        let conn = conn_guard.as_ref().unwrap();

        let mut stmt = conn
            .prepare("SELECT id, name, description FROM parts")
            .map_err(|e| format!("Failed to prepare query: {}", e))?;

        let rows = stmt
            .query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?, row.get::<_, Option<String>>(2)?))
            })
            .map_err(|e| format!("Failed to query parts: {}", e))?;

        for row in rows {
            let (id, name, description) = row.map_err(|e| format!("Failed to read row: {}", e))?;

            let reencrypted_name = self
                .crypto
                .reencrypt_data(&old_key, &new_key, &name)
                .map_err(|e| format!("Failed to reencrypt name: {}", e))?;

            let reencrypted_desc = if let Some(desc) = description {
                Some(
                    self.crypto
                        .reencrypt_data(&old_key, &new_key, &desc)
                        .map_err(|e| format!("Failed to reencrypt description: {}", e))?,
                )
            } else {
                None
            };

            conn.execute(
                "UPDATE parts SET name = ?1, description = ?2 WHERE id = ?3",
                params![reencrypted_name, reencrypted_desc, id],
            )
            .map_err(|e| format!("Failed to update part: {}", e))?;
        }

        self.crypto.init_with_key(&new_key)?;

        if let Some(app_dir) = self.app_data_dir.lock().unwrap().as_ref() {
            let key_path = app_dir.join(".key");
            std::fs::write(&key_path, new_key_b64)
                .map_err(|e| format!("Failed to write new key: {}", e))?;
        }

        Ok(())
    }

    pub fn get_parts(&self, filter: Option<PartFilter>) -> Result<Vec<Part>, String> {
        let conn_guard = self.get_conn()?;
        let conn = conn_guard.as_ref().unwrap();

        let mut sql = "SELECT id, name, part_number, type, color, size, quantity, location, description, image_path, created_at, updated_at FROM parts WHERE 1=1".to_string();
        let mut params_list: Vec<String> = Vec::new();

        if let Some(f) = filter {
            if let Some(t) = f.r#type {
                sql.push_str(" AND type = ?");
                params_list.push(t);
            }
            if let Some(c) = f.color {
                sql.push_str(" AND color = ?");
                params_list.push(c);
            }
            if let Some(s) = f.size {
                sql.push_str(" AND size = ?");
                params_list.push(s);
            }
            if let Some(l) = f.location {
                sql.push_str(" AND location = ?");
                params_list.push(l);
            }
            if let Some(k) = f.keyword {
                sql.push_str(" AND (part_number LIKE ? OR name LIKE ?)");
                params_list.push(format!("%{}%", k));
                params_list.push(format!("%{}%", k));
            }
        }

        sql.push_str(" ORDER BY updated_at DESC");

        let mut stmt = conn
            .prepare(&sql)
            .map_err(|e| format!("Failed to prepare query: {}", e))?;

        let params_refs: Vec<&dyn rusqlite::ToSql> = params_list
            .iter()
            .map(|p| p as &dyn rusqlite::ToSql)
            .collect();

        let rows = stmt
            .query_map(params_refs.as_slice(), |row| {
                Ok(Part {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    part_number: row.get(2)?,
                    r#type: row.get(3)?,
                    color: row.get(4)?,
                    size: row.get(5)?,
                    quantity: row.get(6)?,
                    location: row.get(7)?,
                    description: row.get(8)?,
                    image_path: row.get(9)?,
                    created_at: row.get(10)?,
                    updated_at: row.get(11)?,
                })
            })
            .map_err(|e| format!("Failed to query parts: {}", e))?;

        let mut parts = Vec::new();
        for row in rows {
            let mut part = row.map_err(|e| format!("Failed to read part: {}", e))?;
            part.name = self.crypto.decrypt_string(&part.name).unwrap_or(part.name);
            if let Some(desc) = part.description {
                part.description = Some(self.crypto.decrypt_string(&desc).unwrap_or(desc));
            }
            parts.push(part);
        }

        Ok(parts)
    }

    pub fn get_part_by_id(&self, id: &str) -> Result<Option<Part>, String> {
        let conn_guard = self.get_conn()?;
        let conn = conn_guard.as_ref().unwrap();

        let part = conn
            .query_row(
                "SELECT id, name, part_number, type, color, size, quantity, location, description, image_path, created_at, updated_at FROM parts WHERE id = ?1",
                params![id],
                |row| {
                    Ok(Part {
                        id: row.get(0)?,
                        name: row.get(1)?,
                        part_number: row.get(2)?,
                        r#type: row.get(3)?,
                        color: row.get(4)?,
                        size: row.get(5)?,
                        quantity: row.get(6)?,
                        location: row.get(7)?,
                        description: row.get(8)?,
                        image_path: row.get(9)?,
                        created_at: row.get(10)?,
                        updated_at: row.get(11)?,
                    })
                },
            )
            .optional()
            .map_err(|e| format!("Failed to query part: {}", e))?;

        Ok(part.map(|mut p| {
            p.name = self.crypto.decrypt_string(&p.name).unwrap_or(p.name);
            if let Some(desc) = p.description {
                p.description = Some(self.crypto.decrypt_string(&desc).unwrap_or(desc));
            }
            p
        }))
    }

    pub fn create_part(&self, input: PartForCreate) -> Result<Part, String> {
        let conn_guard = self.get_conn()?;
        let conn = conn_guard.as_ref().unwrap();

        let part = input.into_part();

        let encrypted_name = self
            .crypto
            .encrypt_string(&part.name)
            .map_err(|e| format!("Failed to encrypt name: {}", e))?;

        let encrypted_desc = part
            .description
            .as_ref()
            .map(|d| {
                self.crypto
                    .encrypt_string(d)
                    .map_err(|e| format!("Failed to encrypt description: {}", e))
            })
            .transpose()?;

        conn.execute(
            "INSERT INTO parts (id, name, part_number, type, color, size, quantity, location, description, image_path, created_at, updated_at) 
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            params![
                part.id,
                encrypted_name,
                part.part_number,
                part.r#type,
                part.color,
                part.size,
                part.quantity,
                part.location,
                encrypted_desc,
                part.image_path,
                part.created_at,
                part.updated_at
            ],
        )
        .map_err(|e| format!("Failed to insert part: {}", e))?;

        let after_snapshot = serde_json::to_string(&part)
            .map_err(|e| format!("Failed to serialize after snapshot: {}", e))?;
        self.insert_operation_log(
            conn,
            OperationType::Create,
            ObjectType::Part,
            &part.id,
            Some(&part.name),
            None,
            Some(&after_snapshot),
        )?;

        Ok(part)
    }

    pub fn update_part(&self, part: &Part) -> Result<Part, String> {
        let conn_guard = self.get_conn()?;
        let conn = conn_guard.as_ref().unwrap();

        let old_part = self.get_part_by_id(&part.id)?;

        let mut part = part.clone();
        part.update_timestamp();

        let encrypted_name = self
            .crypto
            .encrypt_string(&part.name)
            .map_err(|e| format!("Failed to encrypt name: {}", e))?;

        let encrypted_desc = part
            .description
            .as_ref()
            .map(|d| {
                self.crypto
                    .encrypt_string(d)
                    .map_err(|e| format!("Failed to encrypt description: {}", e))
            })
            .transpose()?;

        conn.execute(
            "UPDATE parts SET name = ?1, part_number = ?2, type = ?3, color = ?4, size = ?5, quantity = ?6, location = ?7, description = ?8, image_path = ?9, updated_at = ?10 WHERE id = ?11",
            params![
                encrypted_name,
                part.part_number,
                part.r#type,
                part.color,
                part.size,
                part.quantity,
                part.location,
                encrypted_desc,
                part.image_path,
                part.updated_at,
                part.id
            ],
        )
        .map_err(|e| format!("Failed to update part: {}", e))?;

        if let Some(old) = old_part {
            let before_snapshot = serde_json::to_string(&old)
                .map_err(|e| format!("Failed to serialize before snapshot: {}", e))?;
            let after_snapshot = serde_json::to_string(&part)
                .map_err(|e| format!("Failed to serialize after snapshot: {}", e))?;
            self.insert_operation_log(
                conn,
                OperationType::Update,
                ObjectType::Part,
                &part.id,
                Some(&part.name),
                Some(&before_snapshot),
                Some(&after_snapshot),
            )?;
        }

        Ok(part)
    }

    pub fn delete_part(&self, id: &str) -> Result<(), String> {
        let conn_guard = self.get_conn()?;
        let conn = conn_guard.as_ref().unwrap();

        let old_part = self.get_part_by_id(id)?;

        if let Some(app_dir) = self.app_data_dir.lock().unwrap().as_ref() {
            let image_path = app_dir.join("images").join(format!("{}.jpg", id));
            if image_path.exists() {
                let _ = std::fs::remove_file(&image_path);
            }
        }

        conn.execute("DELETE FROM parts WHERE id = ?1", params![id])
            .map_err(|e| format!("Failed to delete part: {}", e))?;

        if let Some(old) = old_part {
            let before_snapshot = serde_json::to_string(&old)
                .map_err(|e| format!("Failed to serialize before snapshot: {}", e))?;
            self.insert_operation_log(
                conn,
                OperationType::Delete,
                ObjectType::Part,
                id,
                Some(&old.name),
                Some(&before_snapshot),
                None,
            )?;
        }

        Ok(())
    }

    pub fn get_part_types(&self) -> Result<Vec<PartType>, String> {
        let conn_guard = self.get_conn()?;
        let conn = conn_guard.as_ref().unwrap();

        let mut stmt = conn
            .prepare("SELECT id, name, code, description FROM part_types ORDER BY name")
            .map_err(|e| format!("Failed to prepare query: {}", e))?;

        let rows = stmt
            .query_map([], |row| {
                Ok(PartType {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    code: row.get(2)?,
                    description: row.get(3)?,
                })
            })
            .map_err(|e| format!("Failed to query part types: {}", e))?;

        let mut types = Vec::new();
        for row in rows {
            types.push(row.map_err(|e| format!("Failed to read part type: {}", e))?);
        }

        Ok(types)
    }

    pub fn create_part_type(&self, input: PartTypeForCreate) -> Result<PartType, String> {
        let conn_guard = self.get_conn()?;
        let conn = conn_guard.as_ref().unwrap();

        let part_type = input.into_type();

        conn.execute(
            "INSERT INTO part_types (id, name, code, description) VALUES (?1, ?2, ?3, ?4)",
            params![part_type.id, part_type.name, part_type.code, part_type.description],
        )
        .map_err(|e| format!("Failed to insert part type: {}", e))?;

        let after_snapshot = serde_json::to_string(&part_type)
            .map_err(|e| format!("Failed to serialize after snapshot: {}", e))?;
        self.insert_operation_log(
            conn,
            OperationType::Create,
            ObjectType::PartType,
            &part_type.id,
            Some(&part_type.name),
            None,
            Some(&after_snapshot),
        )?;

        Ok(part_type)
    }

    pub fn update_part_type(&self, part_type: &PartType) -> Result<PartType, String> {
        let conn_guard = self.get_conn()?;
        let conn = conn_guard.as_ref().unwrap();

        let old = conn
            .query_row(
                "SELECT id, name, code, description FROM part_types WHERE id = ?1",
                params![part_type.id],
                |row| {
                    Ok(PartType {
                        id: row.get(0)?,
                        name: row.get(1)?,
                        code: row.get(2)?,
                        description: row.get(3)?,
                    })
                },
            )
            .optional()
            .map_err(|e| format!("Failed to query old part type: {}", e))?;

        conn.execute(
            "UPDATE part_types SET name = ?1, code = ?2, description = ?3 WHERE id = ?4",
            params![part_type.name, part_type.code, part_type.description, part_type.id],
        )
        .map_err(|e| format!("Failed to update part type: {}", e))?;

        if let Some(old_val) = old {
            let before_snapshot = serde_json::to_string(&old_val)
                .map_err(|e| format!("Failed to serialize before snapshot: {}", e))?;
            let after_snapshot = serde_json::to_string(&part_type)
                .map_err(|e| format!("Failed to serialize after snapshot: {}", e))?;
            self.insert_operation_log(
                conn,
                OperationType::Update,
                ObjectType::PartType,
                &part_type.id,
                Some(&part_type.name),
                Some(&before_snapshot),
                Some(&after_snapshot),
            )?;
        }

        Ok(part_type.clone())
    }

    pub fn delete_part_type(&self, id: &str) -> Result<(), String> {
        let conn_guard = self.get_conn()?;
        let conn = conn_guard.as_ref().unwrap();

        let old = conn
            .query_row(
                "SELECT id, name, code, description FROM part_types WHERE id = ?1",
                params![id],
                |row| {
                    Ok(PartType {
                        id: row.get(0)?,
                        name: row.get(1)?,
                        code: row.get(2)?,
                        description: row.get(3)?,
                    })
                },
            )
            .optional()
            .map_err(|e| format!("Failed to query old part type: {}", e))?;

        conn.execute("DELETE FROM part_types WHERE id = ?1", params![id])
            .map_err(|e| format!("Failed to delete part type: {}", e))?;

        if let Some(old_val) = old {
            let before_snapshot = serde_json::to_string(&old_val)
                .map_err(|e| format!("Failed to serialize before snapshot: {}", e))?;
            self.insert_operation_log(
                conn,
                OperationType::Delete,
                ObjectType::PartType,
                id,
                Some(&old_val.name),
                Some(&before_snapshot),
                None,
            )?;
        }

        Ok(())
    }

    pub fn get_part_colors(&self) -> Result<Vec<PartColor>, String> {
        let conn_guard = self.get_conn()?;
        let conn = conn_guard.as_ref().unwrap();

        let mut stmt = conn
            .prepare("SELECT id, name, hex, lego_code FROM part_colors ORDER BY name")
            .map_err(|e| format!("Failed to prepare query: {}", e))?;

        let rows = stmt
            .query_map([], |row| {
                Ok(PartColor {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    hex: row.get(2)?,
                    lego_code: row.get(3)?,
                })
            })
            .map_err(|e| format!("Failed to query part colors: {}", e))?;

        let mut colors = Vec::new();
        for row in rows {
            colors.push(row.map_err(|e| format!("Failed to read part color: {}", e))?);
        }

        Ok(colors)
    }

    pub fn create_part_color(&self, input: PartColorForCreate) -> Result<PartColor, String> {
        let conn_guard = self.get_conn()?;
        let conn = conn_guard.as_ref().unwrap();

        let color = input.into_color();

        conn.execute(
            "INSERT INTO part_colors (id, name, hex, lego_code) VALUES (?1, ?2, ?3, ?4)",
            params![color.id, color.name, color.hex, color.lego_code],
        )
        .map_err(|e| format!("Failed to insert part color: {}", e))?;

        let after_snapshot = serde_json::to_string(&color)
            .map_err(|e| format!("Failed to serialize after snapshot: {}", e))?;
        self.insert_operation_log(
            conn,
            OperationType::Create,
            ObjectType::PartColor,
            &color.id,
            Some(&color.name),
            None,
            Some(&after_snapshot),
        )?;

        Ok(color)
    }

    pub fn update_part_color(&self, color: &PartColor) -> Result<PartColor, String> {
        let conn_guard = self.get_conn()?;
        let conn = conn_guard.as_ref().unwrap();

        let old = conn
            .query_row(
                "SELECT id, name, hex, lego_code FROM part_colors WHERE id = ?1",
                params![color.id],
                |row| {
                    Ok(PartColor {
                        id: row.get(0)?,
                        name: row.get(1)?,
                        hex: row.get(2)?,
                        lego_code: row.get(3)?,
                    })
                },
            )
            .optional()
            .map_err(|e| format!("Failed to query old part color: {}", e))?;

        conn.execute(
            "UPDATE part_colors SET name = ?1, hex = ?2, lego_code = ?3 WHERE id = ?4",
            params![color.name, color.hex, color.lego_code, color.id],
        )
        .map_err(|e| format!("Failed to update part color: {}", e))?;

        if let Some(old_val) = old {
            let before_snapshot = serde_json::to_string(&old_val)
                .map_err(|e| format!("Failed to serialize before snapshot: {}", e))?;
            let after_snapshot = serde_json::to_string(&color)
                .map_err(|e| format!("Failed to serialize after snapshot: {}", e))?;
            self.insert_operation_log(
                conn,
                OperationType::Update,
                ObjectType::PartColor,
                &color.id,
                Some(&color.name),
                Some(&before_snapshot),
                Some(&after_snapshot),
            )?;
        }

        Ok(color.clone())
    }

    pub fn delete_part_color(&self, id: &str) -> Result<(), String> {
        let conn_guard = self.get_conn()?;
        let conn = conn_guard.as_ref().unwrap();

        let old = conn
            .query_row(
                "SELECT id, name, hex, lego_code FROM part_colors WHERE id = ?1",
                params![id],
                |row| {
                    Ok(PartColor {
                        id: row.get(0)?,
                        name: row.get(1)?,
                        hex: row.get(2)?,
                        lego_code: row.get(3)?,
                    })
                },
            )
            .optional()
            .map_err(|e| format!("Failed to query old part color: {}", e))?;

        conn.execute("DELETE FROM part_colors WHERE id = ?1", params![id])
            .map_err(|e| format!("Failed to delete part color: {}", e))?;

        if let Some(old_val) = old {
            let before_snapshot = serde_json::to_string(&old_val)
                .map_err(|e| format!("Failed to serialize before snapshot: {}", e))?;
            self.insert_operation_log(
                conn,
                OperationType::Delete,
                ObjectType::PartColor,
                id,
                Some(&old_val.name),
                Some(&before_snapshot),
                None,
            )?;
        }

        Ok(())
    }

    pub fn get_part_sizes(&self) -> Result<Vec<PartSize>, String> {
        let conn_guard = self.get_conn()?;
        let conn = conn_guard.as_ref().unwrap();

        let mut stmt = conn
            .prepare("SELECT id, name, width, height, unit FROM part_sizes ORDER BY width, height")
            .map_err(|e| format!("Failed to prepare query: {}", e))?;

        let rows = stmt
            .query_map([], |row| {
                Ok(PartSize {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    width: row.get(2)?,
                    height: row.get(3)?,
                    unit: row.get(4)?,
                })
            })
            .map_err(|e| format!("Failed to query part sizes: {}", e))?;

        let mut sizes = Vec::new();
        for row in rows {
            sizes.push(row.map_err(|e| format!("Failed to read part size: {}", e))?);
        }

        Ok(sizes)
    }

    pub fn create_part_size(&self, input: PartSizeForCreate) -> Result<PartSize, String> {
        let conn_guard = self.get_conn()?;
        let conn = conn_guard.as_ref().unwrap();

        let size = input.into_size();

        conn.execute(
            "INSERT INTO part_sizes (id, name, width, height, unit) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![size.id, size.name, size.width, size.height, size.unit],
        )
        .map_err(|e| format!("Failed to insert part size: {}", e))?;

        let after_snapshot = serde_json::to_string(&size)
            .map_err(|e| format!("Failed to serialize after snapshot: {}", e))?;
        self.insert_operation_log(
            conn,
            OperationType::Create,
            ObjectType::PartSize,
            &size.id,
            Some(&size.name),
            None,
            Some(&after_snapshot),
        )?;

        Ok(size)
    }

    pub fn update_part_size(&self, size: &PartSize) -> Result<PartSize, String> {
        let conn_guard = self.get_conn()?;
        let conn = conn_guard.as_ref().unwrap();

        let old = conn
            .query_row(
                "SELECT id, name, width, height, unit FROM part_sizes WHERE id = ?1",
                params![size.id],
                |row| {
                    Ok(PartSize {
                        id: row.get(0)?,
                        name: row.get(1)?,
                        width: row.get(2)?,
                        height: row.get(3)?,
                        unit: row.get(4)?,
                    })
                },
            )
            .optional()
            .map_err(|e| format!("Failed to query old part size: {}", e))?;

        conn.execute(
            "UPDATE part_sizes SET name = ?1, width = ?2, height = ?3, unit = ?4 WHERE id = ?5",
            params![size.name, size.width, size.height, size.unit, size.id],
        )
        .map_err(|e| format!("Failed to update part size: {}", e))?;

        if let Some(old_val) = old {
            let before_snapshot = serde_json::to_string(&old_val)
                .map_err(|e| format!("Failed to serialize before snapshot: {}", e))?;
            let after_snapshot = serde_json::to_string(&size)
                .map_err(|e| format!("Failed to serialize after snapshot: {}", e))?;
            self.insert_operation_log(
                conn,
                OperationType::Update,
                ObjectType::PartSize,
                &size.id,
                Some(&size.name),
                Some(&before_snapshot),
                Some(&after_snapshot),
            )?;
        }

        Ok(size.clone())
    }

    pub fn delete_part_size(&self, id: &str) -> Result<(), String> {
        let conn_guard = self.get_conn()?;
        let conn = conn_guard.as_ref().unwrap();

        let old = conn
            .query_row(
                "SELECT id, name, width, height, unit FROM part_sizes WHERE id = ?1",
                params![id],
                |row| {
                    Ok(PartSize {
                        id: row.get(0)?,
                        name: row.get(1)?,
                        width: row.get(2)?,
                        height: row.get(3)?,
                        unit: row.get(4)?,
                    })
                },
            )
            .optional()
            .map_err(|e| format!("Failed to query old part size: {}", e))?;

        conn.execute("DELETE FROM part_sizes WHERE id = ?1", params![id])
            .map_err(|e| format!("Failed to delete part size: {}", e))?;

        if let Some(old_val) = old {
            let before_snapshot = serde_json::to_string(&old_val)
                .map_err(|e| format!("Failed to serialize before snapshot: {}", e))?;
            self.insert_operation_log(
                conn,
                OperationType::Delete,
                ObjectType::PartSize,
                id,
                Some(&old_val.name),
                Some(&before_snapshot),
                None,
            )?;
        }

        Ok(())
    }

    pub fn get_locations(&self) -> Result<Vec<Location>, String> {
        let conn_guard = self.get_conn()?;
        let conn = conn_guard.as_ref().unwrap();

        let mut stmt = conn
            .prepare("SELECT id, name, code, description, parent_id FROM locations ORDER BY name")
            .map_err(|e| format!("Failed to prepare query: {}", e))?;

        let rows = stmt
            .query_map([], |row| {
                Ok(Location {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    code: row.get(2)?,
                    description: row.get(3)?,
                    parent_id: row.get(4)?,
                })
            })
            .map_err(|e| format!("Failed to query locations: {}", e))?;

        let mut locations = Vec::new();
        for row in rows {
            locations.push(row.map_err(|e| format!("Failed to read location: {}", e))?);
        }

        Ok(locations)
    }

    pub fn create_location(&self, input: LocationForCreate) -> Result<Location, String> {
        let conn_guard = self.get_conn()?;
        let conn = conn_guard.as_ref().unwrap();

        let location = input.into_location();

        conn.execute(
            "INSERT INTO locations (id, name, code, description, parent_id) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![location.id, location.name, location.code, location.description, location.parent_id],
        )
        .map_err(|e| format!("Failed to insert location: {}", e))?;

        let after_snapshot = serde_json::to_string(&location)
            .map_err(|e| format!("Failed to serialize after snapshot: {}", e))?;
        self.insert_operation_log(
            conn,
            OperationType::Create,
            ObjectType::Location,
            &location.id,
            Some(&location.name),
            None,
            Some(&after_snapshot),
        )?;

        Ok(location)
    }

    pub fn update_location(&self, location: &Location) -> Result<Location, String> {
        let conn_guard = self.get_conn()?;
        let conn = conn_guard.as_ref().unwrap();

        let old = conn
            .query_row(
                "SELECT id, name, code, description, parent_id FROM locations WHERE id = ?1",
                params![location.id],
                |row| {
                    Ok(Location {
                        id: row.get(0)?,
                        name: row.get(1)?,
                        code: row.get(2)?,
                        description: row.get(3)?,
                        parent_id: row.get(4)?,
                    })
                },
            )
            .optional()
            .map_err(|e| format!("Failed to query old location: {}", e))?;

        conn.execute(
            "UPDATE locations SET name = ?1, code = ?2, description = ?3, parent_id = ?4 WHERE id = ?5",
            params![location.name, location.code, location.description, location.parent_id, location.id],
        )
        .map_err(|e| format!("Failed to update location: {}", e))?;

        if let Some(old_val) = old {
            let before_snapshot = serde_json::to_string(&old_val)
                .map_err(|e| format!("Failed to serialize before snapshot: {}", e))?;
            let after_snapshot = serde_json::to_string(&location)
                .map_err(|e| format!("Failed to serialize after snapshot: {}", e))?;
            self.insert_operation_log(
                conn,
                OperationType::Update,
                ObjectType::Location,
                &location.id,
                Some(&location.name),
                Some(&before_snapshot),
                Some(&after_snapshot),
            )?;
        }

        Ok(location.clone())
    }

    pub fn delete_location(&self, id: &str) -> Result<(), String> {
        let conn_guard = self.get_conn()?;
        let conn = conn_guard.as_ref().unwrap();

        let old = conn
            .query_row(
                "SELECT id, name, code, description, parent_id FROM locations WHERE id = ?1",
                params![id],
                |row| {
                    Ok(Location {
                        id: row.get(0)?,
                        name: row.get(1)?,
                        code: row.get(2)?,
                        description: row.get(3)?,
                        parent_id: row.get(4)?,
                    })
                },
            )
            .optional()
            .map_err(|e| format!("Failed to query old location: {}", e))?;

        conn.execute("DELETE FROM locations WHERE id = ?1", params![id])
            .map_err(|e| format!("Failed to delete location: {}", e))?;

        if let Some(old_val) = old {
            let before_snapshot = serde_json::to_string(&old_val)
                .map_err(|e| format!("Failed to serialize before snapshot: {}", e))?;
            self.insert_operation_log(
                conn,
                OperationType::Delete,
                ObjectType::Location,
                id,
                Some(&old_val.name),
                Some(&before_snapshot),
                None,
            )?;
        }

        Ok(())
    }

    pub fn get_moc_lists(&self) -> Result<Vec<MocList>, String> {
        let conn_guard = self.get_conn()?;
        let conn = conn_guard.as_ref().unwrap();

        let mut stmt = conn
            .prepare("SELECT id, name, description, cover_image_path, status, created_at, updated_at FROM moc_lists ORDER BY updated_at DESC")
            .map_err(|e| format!("Failed to prepare query: {}", e))?;

        let rows = stmt
            .query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, Option<String>>(2)?,
                    row.get::<_, Option<String>>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, String>(5)?,
                    row.get::<_, String>(6)?,
                ))
            })
            .map_err(|e| format!("Failed to query moc lists: {}", e))?;

        let mut moc_lists = Vec::new();
        for row in rows {
            let (id, name, description, cover_image_path, status, created_at, updated_at) =
                row.map_err(|e| format!("Failed to read moc list: {}", e))?;

            let parts = self.get_moc_parts(conn, &id)?;

            moc_lists.push(MocList {
                id,
                name,
                description,
                cover_image_path,
                status: MocStatus::from_str(&status),
                parts,
                created_at,
                updated_at,
            });
        }

        Ok(moc_lists)
    }

    fn get_moc_parts(&self, conn: &Connection, moc_id: &str) -> Result<Vec<MocPart>, String> {
        let mut stmt = conn
            .prepare(
                "SELECT part_id, part_number, part_name, color, quantity FROM moc_parts WHERE moc_id = ?1",
            )
            .map_err(|e| format!("Failed to prepare moc parts query: {}", e))?;

        let rows = stmt
            .query_map(params![moc_id], |row| {
                Ok(MocPart {
                    part_id: row.get(0)?,
                    part_number: row.get(1)?,
                    part_name: row.get(2)?,
                    color: row.get(3)?,
                    quantity: row.get(4)?,
                    in_stock: 0,
                    is_missing: false,
                })
            })
            .map_err(|e| format!("Failed to query moc parts: {}", e))?;

        let mut parts = Vec::new();
        for row in rows {
            parts.push(row.map_err(|e| format!("Failed to read moc part: {}", e))?);
        }

        Ok(parts)
    }

    fn get_moc_list_by_id_with_conn(&self, conn: &Connection, id: &str) -> Result<Option<MocList>, String> {
        let moc_info = conn
            .query_row(
                "SELECT id, name, description, cover_image_path, status, created_at, updated_at FROM moc_lists WHERE id = ?1",
                params![id],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, Option<String>>(2)?,
                        row.get::<_, Option<String>>(3)?,
                        row.get::<_, String>(4)?,
                        row.get::<_, String>(5)?,
                        row.get::<_, String>(6)?,
                    ))
                },
            )
            .optional()
            .map_err(|e| format!("Failed to query moc list: {}", e))?;

        Ok(moc_info.map(|(id, name, description, cover_image_path, status, created_at, updated_at)| {
            let parts = self.get_moc_parts(conn, &id).unwrap_or_default();
            MocList {
                id,
                name,
                description,
                cover_image_path,
                status: MocStatus::from_str(&status),
                parts,
                created_at,
                updated_at,
            }
        }))
    }

    pub fn get_moc_list_by_id(&self, id: &str) -> Result<Option<MocList>, String> {
        let conn_guard = self.get_conn()?;
        let conn = conn_guard.as_ref().unwrap();
        self.get_moc_list_by_id_with_conn(conn, id)
    }

    pub fn create_moc_list(&self, input: MocListForCreate) -> Result<MocList, String> {
        let mut conn_guard = self.get_conn()?;
        let conn = conn_guard.as_mut().unwrap();

        let moc = input.into_moc_list();

        let tx = conn
            .transaction()
            .map_err(|e| format!("Failed to start transaction: {}", e))?;

        tx.execute(
            "INSERT INTO moc_lists (id, name, description, cover_image_path, status, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![moc.id, moc.name, moc.description, moc.cover_image_path, moc.status.as_str(), moc.created_at, moc.updated_at],
        )
        .map_err(|e| format!("Failed to insert moc list: {}", e))?;

        self.insert_status_log(
            &tx,
            &moc.id,
            None,
            moc.status.as_str(),
            Some("创建 MOC 清单".to_string()),
        )?;

        for part in &moc.parts {
            let part_id = Uuid::new_v4().to_string();
            tx.execute(
                "INSERT INTO moc_parts (id, moc_id, part_id, part_number, part_name, color, quantity) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![
                    part_id,
                    moc.id,
                    part.part_id,
                    part.part_number,
                    part.part_name,
                    part.color,
                    part.quantity
                ],
            )
            .map_err(|e| format!("Failed to insert moc part: {}", e))?;
        }

        let after_snapshot = serde_json::to_string(&moc)
            .map_err(|e| format!("Failed to serialize after snapshot: {}", e))?;
        self.insert_operation_log(
            &tx,
            OperationType::Create,
            ObjectType::MocList,
            &moc.id,
            Some(&moc.name),
            None,
            Some(&after_snapshot),
        )?;

        tx.commit()
            .map_err(|e| format!("Failed to commit transaction: {}", e))?;

        Ok(moc)
    }

    pub fn update_moc_list(&self, moc: &MocList) -> Result<MocList, String> {
        let mut conn_guard = self.get_conn()?;
        let conn = conn_guard.as_mut().unwrap();

        let old_moc = self.get_moc_list_by_id_with_conn(conn, &moc.id)?;

        let mut moc = moc.clone();
        moc.update_timestamp();

        let tx = conn
            .transaction()
            .map_err(|e| format!("Failed to start transaction: {}", e))?;

        tx.execute(
            "UPDATE moc_lists SET name = ?1, description = ?2, cover_image_path = ?3, status = ?4, updated_at = ?5 WHERE id = ?6",
            params![moc.name, moc.description, moc.cover_image_path, moc.status.as_str(), moc.updated_at, moc.id],
        )
        .map_err(|e| format!("Failed to update moc list: {}", e))?;

        tx.execute("DELETE FROM moc_parts WHERE moc_id = ?1", params![moc.id])
            .map_err(|e| format!("Failed to delete old moc parts: {}", e))?;

        for part in &moc.parts {
            let part_id = Uuid::new_v4().to_string();
            tx.execute(
                "INSERT INTO moc_parts (id, moc_id, part_id, part_number, part_name, color, quantity) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![
                    part_id,
                    moc.id,
                    part.part_id,
                    part.part_number,
                    part.part_name,
                    part.color,
                    part.quantity
                ],
            )
            .map_err(|e| format!("Failed to insert moc part: {}", e))?;
        }

        if let Some(old) = old_moc {
            let before_snapshot = serde_json::to_string(&old)
                .map_err(|e| format!("Failed to serialize before snapshot: {}", e))?;
            let after_snapshot = serde_json::to_string(&moc)
                .map_err(|e| format!("Failed to serialize after snapshot: {}", e))?;
            self.insert_operation_log(
                &tx,
                OperationType::Update,
                ObjectType::MocList,
                &moc.id,
                Some(&moc.name),
                Some(&before_snapshot),
                Some(&after_snapshot),
            )?;
        }

        tx.commit()
            .map_err(|e| format!("Failed to commit transaction: {}", e))?;

        Ok(moc)
    }

    pub fn delete_moc_list(&self, id: &str) -> Result<(), String> {
        let conn_guard = self.get_conn()?;
        let conn = conn_guard.as_ref().unwrap();

        let old_moc = self.get_moc_list_by_id(id)?;

        if let Some(app_dir) = self.app_data_dir.lock().unwrap().as_ref() {
            let image_path = app_dir.join("images").join("moc").join(format!("{}.jpg", id));
            if image_path.exists() {
                let _ = std::fs::remove_file(&image_path);
            }
        }

        conn.execute("DELETE FROM moc_lists WHERE id = ?1", params![id])
            .map_err(|e| format!("Failed to delete moc list: {}", e))?;

        if let Some(old) = old_moc {
            let before_snapshot = serde_json::to_string(&old)
                .map_err(|e| format!("Failed to serialize before snapshot: {}", e))?;
            self.insert_operation_log(
                conn,
                OperationType::Delete,
                ObjectType::MocList,
                id,
                Some(&old.name),
                Some(&before_snapshot),
                None,
            )?;
        }

        Ok(())
    }

    fn compare_moc_inventory_with_conn(&self, conn: &Connection, moc_id: &str) -> Result<MocList, String> {
        let moc = self
            .get_moc_list_by_id_with_conn(conn, moc_id)?
            .ok_or_else(|| "MOC list not found".to_string())?;

        let mut updated_parts = Vec::new();
        for part in &moc.parts {
            let stock: Option<i32> = conn
                .query_row(
                    "SELECT quantity FROM parts WHERE part_number = ?1 AND color = ?2",
                    params![part.part_number, part.color],
                    |row| row.get(0),
                )
                .optional()
                .map_err(|e| format!("Failed to query stock: {}", e))?;

            let in_stock = stock.unwrap_or(0);
            let is_missing = in_stock < part.quantity;

            updated_parts.push(MocPart {
                part_id: part.part_id.clone(),
                part_number: part.part_number.clone(),
                part_name: part.part_name.clone(),
                color: part.color.clone(),
                quantity: part.quantity,
                in_stock,
                is_missing,
            });
        }

        Ok(MocList {
            parts: updated_parts,
            ..moc
        })
    }

    pub fn compare_moc_inventory(&self, moc_id: &str) -> Result<MocList, String> {
        let conn_guard = self.get_conn()?;
        let conn = conn_guard.as_ref().unwrap();
        self.compare_moc_inventory_with_conn(conn, moc_id)
    }

    fn insert_status_log(
        &self,
        conn: &Connection,
        moc_id: &str,
        old_status: Option<&str>,
        new_status: &str,
        remark: Option<String>,
    ) -> Result<(), String> {
        use chrono::Utc;
        let log_id = Uuid::new_v4().to_string();
        let now: DateTime<Utc> = Utc::now();

        conn.execute(
            "INSERT INTO moc_status_logs (id, moc_id, old_status, new_status, changed_at, remark) 
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                log_id,
                moc_id,
                old_status,
                new_status,
                now.to_rfc3339(),
                remark
            ],
        )
        .map_err(|e| format!("Failed to insert status log: {}", e))?;

        Ok(())
    }

    pub fn change_moc_status(
        &self,
        change: MocStatusChange,
    ) -> Result<MocList, String> {
        let mut conn_guard = self.get_conn()?;
        let conn = conn_guard.as_mut().unwrap();

        let old_moc = self
            .get_moc_list_by_id_with_conn(conn, &change.moc_id)?
            .ok_or_else(|| "MOC list not found".to_string())?;

        let old_status = old_moc.status.as_str().to_string();
        let new_status = change.new_status.as_str().to_string();

        if old_status == new_status {
            return Ok(old_moc);
        }

        let tx = conn
            .transaction()
            .map_err(|e| format!("Failed to start transaction: {}", e))?;

        use chrono::Utc;
        let now: DateTime<Utc> = Utc::now();
        let updated_at = now.to_rfc3339();

        tx.execute(
            "UPDATE moc_lists SET status = ?1, updated_at = ?2 WHERE id = ?3",
            params![new_status, updated_at, change.moc_id],
        )
        .map_err(|e| format!("Failed to update moc status: {}", e))?;

        self.insert_status_log(
            &tx,
            &change.moc_id,
            Some(&old_status),
            &new_status,
            change.remark,
        )?;

        let updated_moc = self.get_moc_list_by_id_with_conn(&tx, &change.moc_id)?
            .ok_or_else(|| "MOC list not found after update".to_string())?;

        let before_snapshot = serde_json::to_string(&old_moc)
            .map_err(|e| format!("Failed to serialize before snapshot: {}", e))?;
        let after_snapshot = serde_json::to_string(&updated_moc)
            .map_err(|e| format!("Failed to serialize after snapshot: {}", e))?;
        self.insert_operation_log(
            &tx,
            OperationType::Update,
            ObjectType::MocList,
            &change.moc_id,
            Some(&updated_moc.name),
            Some(&before_snapshot),
            Some(&after_snapshot),
        )?;

        tx.commit()
            .map_err(|e| format!("Failed to commit transaction: {}", e))?;

        Ok(updated_moc)
    }

    pub fn get_moc_status_logs(&self, moc_id: &str) -> Result<Vec<MocStatusLog>, String> {
        let conn_guard = self.get_conn()?;
        let conn = conn_guard.as_ref().unwrap();

        let mut stmt = conn
            .prepare(
                "SELECT id, moc_id, old_status, new_status, changed_at, remark 
                 FROM moc_status_logs WHERE moc_id = ?1 ORDER BY changed_at DESC",
            )
            .map_err(|e| format!("Failed to prepare query: {}", e))?;

        let rows = stmt
            .query_map(params![moc_id], |row| {
                Ok(MocStatusLog {
                    id: row.get(0)?,
                    moc_id: row.get(1)?,
                    old_status: row.get(2)?,
                    new_status: row.get(3)?,
                    changed_at: row.get(4)?,
                    remark: row.get(5)?,
                })
            })
            .map_err(|e| format!("Failed to query status logs: {}", e))?;

        let mut logs = Vec::new();
        for row in rows {
            logs.push(row.map_err(|e| format!("Failed to read status log: {}", e))?);
        }

        Ok(logs)
    }

    pub fn save_moc_cover_image(&self, moc_id: &str, image_data_base64: &str) -> Result<String, String> {
        let app_dir = self
            .app_data_dir
            .lock()
            .unwrap()
            .clone()
            .ok_or_else(|| "App data dir not initialized".to_string())?;

        let moc_images_dir = app_dir.join("images").join("moc");
        std::fs::create_dir_all(&moc_images_dir)
            .map_err(|e| format!("Failed to create moc images dir: {}", e))?;

        let image_path = moc_images_dir.join(format!("{}.jpg", moc_id));

        let image_bytes = base64::engine::general_purpose::STANDARD
            .decode(image_data_base64)
            .map_err(|e| format!("Failed to decode base64 image: {}", e))?;

        std::fs::write(&image_path, &image_bytes)
            .map_err(|e| format!("Failed to write image file: {}", e))?;

        let path_str = image_path
            .to_str()
            .ok_or_else(|| "Failed to convert path to string".to_string())?;

        let conn_guard = self.get_conn()?;
        let conn = conn_guard.as_ref().unwrap();

        let old_moc = self.get_moc_list_by_id(moc_id)?;

        use chrono::Utc;
        let now: DateTime<Utc> = Utc::now();

        conn.execute(
            "UPDATE moc_lists SET cover_image_path = ?1, updated_at = ?2 WHERE id = ?3",
            params![path_str, now.to_rfc3339(), moc_id],
        )
        .map_err(|e| format!("Failed to update moc cover image path: {}", e))?;

        if let Some(mut old) = old_moc.clone() {
            let before_snapshot = serde_json::to_string(&old)
                .map_err(|e| format!("Failed to serialize before snapshot: {}", e))?;
            old.cover_image_path = Some(path_str.to_string());
            old.update_timestamp();
            let after_snapshot = serde_json::to_string(&old)
                .map_err(|e| format!("Failed to serialize after snapshot: {}", e))?;
            self.insert_operation_log(
                conn,
                OperationType::Update,
                ObjectType::MocList,
                moc_id,
                old_moc.as_ref().map(|m| m.name.as_str()),
                Some(&before_snapshot),
                Some(&after_snapshot),
            )?;
        }

        Ok(path_str.to_string())
    }

    pub fn delete_moc_cover_image(&self, moc_id: &str) -> Result<(), String> {
        let app_dir = self
            .app_data_dir
            .lock()
            .unwrap()
            .clone()
            .ok_or_else(|| "App data dir not initialized".to_string())?;

        let image_path = app_dir.join("images").join("moc").join(format!("{}.jpg", moc_id));
        if image_path.exists() {
            std::fs::remove_file(&image_path)
                .map_err(|e| format!("Failed to delete image file: {}", e))?;
        }

        let conn_guard = self.get_conn()?;
        let conn = conn_guard.as_ref().unwrap();

        let old_moc = self.get_moc_list_by_id(moc_id)?;

        use chrono::Utc;
        let now: DateTime<Utc> = Utc::now();

        conn.execute(
            "UPDATE moc_lists SET cover_image_path = NULL, updated_at = ?1 WHERE id = ?2",
            params![now.to_rfc3339(), moc_id],
        )
        .map_err(|e| format!("Failed to update moc cover image path: {}", e))?;

        if let Some(mut old) = old_moc.clone() {
            let before_snapshot = serde_json::to_string(&old)
                .map_err(|e| format!("Failed to serialize before snapshot: {}", e))?;
            old.cover_image_path = None;
            old.update_timestamp();
            let after_snapshot = serde_json::to_string(&old)
                .map_err(|e| format!("Failed to serialize after snapshot: {}", e))?;
            self.insert_operation_log(
                conn,
                OperationType::Update,
                ObjectType::MocList,
                moc_id,
                old_moc.as_ref().map(|m| m.name.as_str()),
                Some(&before_snapshot),
                Some(&after_snapshot),
            )?;
        }

        Ok(())
    }

    pub fn get_stats(&self) -> Result<StatsData, String> {
        let conn_guard = self.get_conn()?;
        let conn = conn_guard.as_ref().unwrap();

        let total_parts: i64 = conn
            .query_row("SELECT COUNT(*) FROM parts", [], |row| row.get(0))
            .map_err(|e| format!("Failed to count parts: {}", e))?;

        let total_quantity: i64 = conn
            .query_row("SELECT COALESCE(SUM(quantity), 0) FROM parts", [], |row| row.get(0))
            .map_err(|e| format!("Failed to sum quantity: {}", e))?;

        let total_types: i64 = conn
            .query_row("SELECT COUNT(*) FROM part_types", [], |row| row.get(0))
            .map_err(|e| format!("Failed to count types: {}", e))?;

        let total_colors: i64 = conn
            .query_row("SELECT COUNT(*) FROM part_colors", [], |row| row.get(0))
            .map_err(|e| format!("Failed to count colors: {}", e))?;

        let total_locations: i64 = conn
            .query_row("SELECT COUNT(*) FROM locations", [], |row| row.get(0))
            .map_err(|e| format!("Failed to count locations: {}", e))?;

        let total_mocs: i64 = conn
            .query_row("SELECT COUNT(*) FROM moc_lists", [], |row| row.get(0))
            .map_err(|e| format!("Failed to count mocs: {}", e))?;

        let low_stock_parts: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM parts WHERE quantity <= 5",
                [],
                |row| row.get(0),
            )
            .map_err(|e| format!("Failed to count low stock parts: {}", e))?;

        let mut parts_by_type_stmt = conn
            .prepare(
                "SELECT t.name, COUNT(p.id) as cnt 
                 FROM part_types t 
                 LEFT JOIN parts p ON p.type = t.code 
                 GROUP BY t.id, t.name 
                 ORDER BY cnt DESC",
            )
            .map_err(|e| format!("Failed to prepare parts by type query: {}", e))?;

        let parts_by_type_rows = parts_by_type_stmt
            .query_map([], |row| {
                Ok(TypeCount {
                    name: row.get(0)?,
                    count: row.get(1)?,
                })
            })
            .map_err(|e| format!("Failed to query parts by type: {}", e))?;

        let mut parts_by_type = Vec::new();
        for row in parts_by_type_rows {
            parts_by_type
                .push(row.map_err(|e| format!("Failed to read type count: {}", e))?);
        }

        let mut parts_by_color_stmt = conn
            .prepare(
                "SELECT c.name, c.hex, COUNT(p.id) as cnt 
                 FROM part_colors c 
                 LEFT JOIN parts p ON p.color = c.name 
                 GROUP BY c.id, c.name, c.hex 
                 ORDER BY cnt DESC",
            )
            .map_err(|e| format!("Failed to prepare parts by color query: {}", e))?;

        let parts_by_color_rows = parts_by_color_stmt
            .query_map([], |row| {
                Ok(ColorCount {
                    name: row.get(0)?,
                    hex: row.get(1)?,
                    count: row.get(2)?,
                })
            })
            .map_err(|e| format!("Failed to query parts by color: {}", e))?;

        let mut parts_by_color = Vec::new();
        for row in parts_by_color_rows {
            parts_by_color
                .push(row.map_err(|e| format!("Failed to read color count: {}", e))?);
        }

        let mut parts_by_location_stmt = conn
            .prepare(
                "SELECT l.name, COUNT(p.id) as cnt 
                 FROM locations l 
                 LEFT JOIN parts p ON p.location = l.code 
                 GROUP BY l.id, l.name 
                 ORDER BY cnt DESC",
            )
            .map_err(|e| format!("Failed to prepare parts by location query: {}", e))?;

        let parts_by_location_rows = parts_by_location_stmt
            .query_map([], |row| {
                Ok(LocationCount {
                    name: row.get(0)?,
                    count: row.get(1)?,
                })
            })
            .map_err(|e| format!("Failed to query parts by location: {}", e))?;

        let mut parts_by_location = Vec::new();
        for row in parts_by_location_rows {
            parts_by_location
                .push(row.map_err(|e| format!("Failed to read location count: {}", e))?);
        }

        let mut missing_count = 0i64;
        let mut moc_stmt = conn
            .prepare("SELECT id FROM moc_lists")
            .map_err(|e| format!("Failed to prepare moc query: {}", e))?;

        let moc_rows = moc_stmt
            .query_map([], |row| row.get::<_, String>(0))
            .map_err(|e| format!("Failed to query mocs: {}", e))?;

        for moc_row in moc_rows {
            let moc_id = moc_row.map_err(|e| format!("Failed to read moc id: {}", e))?;
            let moc = self.compare_moc_inventory_with_conn(conn, &moc_id)?;
            missing_count += moc.parts.iter().filter(|p| p.is_missing).count() as i64;
        }

        let mut mocs_by_status_stmt = conn
            .prepare(
                "SELECT status, COUNT(id) as cnt FROM moc_lists GROUP BY status ORDER BY cnt DESC",
            )
            .map_err(|e| format!("Failed to prepare mocs by status query: {}", e))?;

        let mocs_by_status_rows = mocs_by_status_stmt
            .query_map([], |row| {
                Ok(MocStatusCount {
                    status: row.get(0)?,
                    count: row.get(1)?,
                })
            })
            .map_err(|e| format!("Failed to query mocs by status: {}", e))?;

        let mut mocs_by_status = Vec::new();
        for row in mocs_by_status_rows {
            mocs_by_status
                .push(row.map_err(|e| format!("Failed to read status count: {}", e))?);
        }

        Ok(StatsData {
            total_parts,
            total_quantity,
            total_types,
            total_colors,
            total_locations,
            total_mocs,
            low_stock_parts,
            missing_parts_in_mocs: missing_count,
            parts_by_type,
            parts_by_color,
            parts_by_location,
            mocs_by_status,
        })
    }

    pub fn export_parts(
        &self,
        format: &str,
        part_ids: Option<Vec<String>>,
    ) -> Result<String, String> {
        let conn_guard = self.get_conn()?;
        let conn = conn_guard.as_ref().unwrap();

        let mut sql = "SELECT name, part_number, type, color, size, quantity, location, description FROM parts".to_string();
        let mut params: Vec<String> = Vec::new();

        if let Some(ids) = part_ids {
            let placeholders: Vec<String> = ids.iter().map(|_| "?".to_string()).collect();
            sql.push_str(&format!(" WHERE id IN ({})", placeholders.join(",")));
            params.extend(ids);
        }

        sql.push_str(" ORDER BY name");

        let mut stmt = conn
            .prepare(&sql)
            .map_err(|e| format!("Failed to prepare export query: {}", e))?;

        let params_refs: Vec<&dyn rusqlite::ToSql> = params
            .iter()
            .map(|p| p as &dyn rusqlite::ToSql)
            .collect();

        let rows = stmt
            .query_map(params_refs.as_slice(), |row| {
                Ok(ImportExportPart {
                    name: row.get(0)?,
                    part_number: row.get(1)?,
                    r#type: row.get(2)?,
                    color: row.get(3)?,
                    size: row.get(4)?,
                    quantity: row.get(5)?,
                    location: row.get(6)?,
                    description: row.get(7)?,
                })
            })
            .map_err(|e| format!("Failed to query parts for export: {}", e))?;

        let mut parts = Vec::new();
        for row in rows {
            let mut part = row.map_err(|e| format!("Failed to read part for export: {}", e))?;
            part.name = self.crypto.decrypt_string(&part.name).unwrap_or(part.name);
            if let Some(desc) = part.description {
                part.description = Some(self.crypto.decrypt_string(&desc).unwrap_or(desc));
            }
            parts.push(part);
        }

        match format {
            "json" => serde_json::to_string_pretty(&parts)
                .map_err(|e| format!("Failed to serialize to JSON: {}", e)),
            "csv" => {
                let mut wtr = csv::Writer::from_writer(vec![]);
                for part in &parts {
                    wtr.serialize(part)
                        .map_err(|e| format!("Failed to serialize to CSV: {}", e))?;
                }
                let data = String::from_utf8(
                    wtr.into_inner()
                        .map_err(|e| format!("Failed to get CSV data: {}", e))?,
                )
                .map_err(|e| format!("Failed to convert CSV to string: {}", e))?;
                Ok(data)
            }
            _ => Err(format!("Unsupported format: {}", format)),
        }
    }

    pub fn import_parts(
        &self,
        format: &str,
        data: &str,
    ) -> Result<ImportResult, String> {
        let parts: Vec<ImportExportPart> = match format {
            "json" => serde_json::from_str(data)
                .map_err(|e| format!("Failed to parse JSON: {}", e))?,
            "csv" => {
                let mut rdr = csv::Reader::from_reader(data.as_bytes());
                let mut result = Vec::new();
                for record in rdr.deserialize() {
                    let part: ImportExportPart = record
                        .map_err(|e| format!("Failed to parse CSV record: {}", e))?;
                    result.push(part);
                }
                result
            }
            _ => return Err(format!("Unsupported format: {}", format)),
        };

        let mut imported = 0;
        let mut errors = Vec::new();

        for (index, import_part) in parts.iter().enumerate() {
            if import_part.name.is_empty() {
                errors.push(format!("Row {}: name is empty", index + 1));
                continue;
            }
            if import_part.part_number.is_empty() {
                errors.push(format!("Row {}: part_number is empty", index + 1));
                continue;
            }
            if import_part.quantity < 0 {
                errors.push(format!("Row {}: quantity cannot be negative", index + 1));
                continue;
            }

            let part_for_create = PartForCreate {
                name: import_part.name.clone(),
                part_number: import_part.part_number.clone(),
                r#type: import_part.r#type.clone(),
                color: import_part.color.clone(),
                size: import_part.size.clone(),
                quantity: import_part.quantity,
                location: import_part.location.clone(),
                description: import_part.description.clone(),
                image_path: None,
            };

            match self.create_part(part_for_create) {
                Ok(_) => imported += 1,
                Err(e) => errors.push(format!("Row {}: {}", index + 1, e)),
            }
        }

        Ok(ImportResult { imported, errors })
    }

    pub fn save_part_image(&self, part_id: &str, image_data_base64: &str) -> Result<String, String> {
        let app_dir = self
            .app_data_dir
            .lock()
            .unwrap()
            .clone()
            .ok_or_else(|| "App data dir not initialized".to_string())?;

        let images_dir = app_dir.join("images");
        std::fs::create_dir_all(&images_dir)
            .map_err(|e| format!("Failed to create images dir: {}", e))?;

        let image_path = images_dir.join(format!("{}.jpg", part_id));

        let image_bytes = base64::engine::general_purpose::STANDARD
            .decode(image_data_base64)
            .map_err(|e| format!("Failed to decode base64 image: {}", e))?;

        std::fs::write(&image_path, &image_bytes)
            .map_err(|e| format!("Failed to write image file: {}", e))?;

        let path_str = image_path
            .to_str()
            .ok_or_else(|| "Failed to convert path to string".to_string())?;

        let conn_guard = self.get_conn()?;
        let conn = conn_guard.as_ref().unwrap();

        let old_part = self.get_part_by_id(part_id)?;

        conn.execute(
            "UPDATE parts SET image_path = ?1 WHERE id = ?2",
            params![path_str, part_id],
        )
        .map_err(|e| format!("Failed to update part image path: {}", e))?;

        if let Some(mut old) = old_part.clone() {
            let before_snapshot = serde_json::to_string(&old)
                .map_err(|e| format!("Failed to serialize before snapshot: {}", e))?;
            old.image_path = Some(path_str.to_string());
            old.update_timestamp();
            let after_snapshot = serde_json::to_string(&old)
                .map_err(|e| format!("Failed to serialize after snapshot: {}", e))?;
            self.insert_operation_log(
                conn,
                OperationType::Update,
                ObjectType::Part,
                part_id,
                old_part.as_ref().map(|p| p.name.as_str()),
                Some(&before_snapshot),
                Some(&after_snapshot),
            )?;
        }

        Ok(path_str.to_string())
    }

    pub fn delete_part_image(&self, part_id: &str) -> Result<(), String> {
        let app_dir = self
            .app_data_dir
            .lock()
            .unwrap()
            .clone()
            .ok_or_else(|| "App data dir not initialized".to_string())?;

        let image_path = app_dir.join("images").join(format!("{}.jpg", part_id));
        if image_path.exists() {
            std::fs::remove_file(&image_path)
                .map_err(|e| format!("Failed to delete image file: {}", e))?;
        }

        let conn_guard = self.get_conn()?;
        let conn = conn_guard.as_ref().unwrap();

        let old_part = self.get_part_by_id(part_id)?;

        conn.execute(
            "UPDATE parts SET image_path = NULL WHERE id = ?1",
            params![part_id],
        )
        .map_err(|e| format!("Failed to update part image path: {}", e))?;

        if let Some(mut old) = old_part.clone() {
            let before_snapshot = serde_json::to_string(&old)
                .map_err(|e| format!("Failed to serialize before snapshot: {}", e))?;
            old.image_path = None;
            old.update_timestamp();
            let after_snapshot = serde_json::to_string(&old)
                .map_err(|e| format!("Failed to serialize after snapshot: {}", e))?;
            self.insert_operation_log(
                conn,
                OperationType::Update,
                ObjectType::Part,
                part_id,
                old_part.as_ref().map(|p| p.name.as_str()),
                Some(&before_snapshot),
                Some(&after_snapshot),
            )?;
        }

        Ok(())
    }

    pub fn get_part_image_path(&self, part_id: &str) -> Result<Option<String>, String> {
        let conn_guard = self.get_conn()?;
        let conn = conn_guard.as_ref().unwrap();

        let path: Option<String> = conn
            .query_row(
                "SELECT image_path FROM parts WHERE id = ?1",
                params![part_id],
                |row| row.get(0),
            )
            .optional()
            .map_err(|e| format!("Failed to query image path: {}", e))?;

        Ok(path)
    }

    fn insert_operation_log(
        &self,
        conn: &Connection,
        operation_type: OperationType,
        object_type: ObjectType,
        object_id: &str,
        object_name: Option<&str>,
        before_snapshot: Option<&str>,
        after_snapshot: Option<&str>,
    ) -> Result<(), String> {
        use chrono::Utc;
        let log_id = Uuid::new_v4().to_string();
        let now: DateTime<Utc> = Utc::now();

        conn.execute(
            "INSERT INTO operation_logs (id, operation_type, object_type, object_id, object_name, before_snapshot, after_snapshot, changed_at) 
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                log_id,
                operation_type.as_str(),
                object_type.as_str(),
                object_id,
                object_name,
                before_snapshot,
                after_snapshot,
                now.to_rfc3339()
            ],
        )
        .map_err(|e| format!("Failed to insert operation log: {}", e))?;

        Ok(())
    }

    pub fn get_operation_logs(&self, filter: Option<OperationLogFilter>) -> Result<Vec<OperationLog>, String> {
        let conn_guard = self.get_conn()?;
        let conn = conn_guard.as_ref().unwrap();

        let mut sql = "SELECT id, operation_type, object_type, object_id, object_name, before_snapshot, after_snapshot, changed_at FROM operation_logs WHERE 1=1".to_string();
        let mut params_list: Vec<String> = Vec::new();

        if let Some(f) = filter {
            if let Some(op_type) = f.operation_type {
                sql.push_str(" AND operation_type = ?");
                params_list.push(op_type);
            }
            if let Some(obj_type) = f.object_type {
                sql.push_str(" AND object_type = ?");
                params_list.push(obj_type);
            }
            if let Some(obj_id) = f.object_id {
                sql.push_str(" AND object_id = ?");
                params_list.push(obj_id);
            }
        }

        sql.push_str(" ORDER BY changed_at DESC LIMIT 500");

        let mut stmt = conn
            .prepare(&sql)
            .map_err(|e| format!("Failed to prepare query: {}", e))?;

        let params_refs: Vec<&dyn rusqlite::ToSql> = params_list
            .iter()
            .map(|p| p as &dyn rusqlite::ToSql)
            .collect();

        let rows = stmt
            .query_map(params_refs.as_slice(), |row| {
                Ok(OperationLog {
                    id: row.get(0)?,
                    operation_type: OperationType::from_str(&row.get::<_, String>(1)?),
                    object_type: ObjectType::from_str(&row.get::<_, String>(2)?),
                    object_id: row.get(3)?,
                    object_name: row.get(4)?,
                    before_snapshot: row.get(5)?,
                    after_snapshot: row.get(6)?,
                    changed_at: row.get(7)?,
                })
            })
            .map_err(|e| format!("Failed to query operation logs: {}", e))?;

        let mut logs = Vec::new();
        for row in rows {
            logs.push(row.map_err(|e| format!("Failed to read operation log: {}", e))?);
        }

        Ok(logs)
    }
}

impl Default for Database {
    fn default() -> Self {
        Self::new()
    }
}
