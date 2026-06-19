use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Part {
    pub id: String,
    pub name: String,
    pub part_number: String,
    pub r#type: String,
    pub color: String,
    pub size: String,
    pub quantity: i32,
    pub location: String,
    pub description: Option<String>,
    pub image_path: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl Part {
    pub fn new(
        name: String,
        part_number: String,
        r#type: String,
        color: String,
        size: String,
        quantity: i32,
        location: String,
        description: Option<String>,
        image_path: Option<String>,
    ) -> Self {
        let now: DateTime<Utc> = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            part_number,
            r#type,
            color,
            size,
            quantity,
            location,
            description,
            image_path,
            created_at: now.to_rfc3339(),
            updated_at: now.to_rfc3339(),
        }
    }

    pub fn update_timestamp(&mut self) {
        let now: DateTime<Utc> = Utc::now();
        self.updated_at = now.to_rfc3339();
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartType {
    pub id: String,
    pub name: String,
    pub code: String,
    pub description: Option<String>,
}

impl PartType {
    pub fn new(name: String, code: String, description: Option<String>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            code,
            description,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartColor {
    pub id: String,
    pub name: String,
    pub hex: String,
    pub lego_code: Option<String>,
}

impl PartColor {
    pub fn new(name: String, hex: String, lego_code: Option<String>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            hex,
            lego_code,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartSize {
    pub id: String,
    pub name: String,
    pub width: f64,
    pub height: f64,
    pub unit: String,
}

impl PartSize {
    pub fn new(name: String, width: f64, height: f64, unit: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            width,
            height,
            unit,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Location {
    pub id: String,
    pub name: String,
    pub code: String,
    pub description: Option<String>,
    pub parent_id: Option<String>,
}

impl Location {
    pub fn new(
        name: String,
        code: String,
        description: Option<String>,
        parent_id: Option<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            code,
            description,
            parent_id,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MocList {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub parts: Vec<MocPart>,
    pub created_at: String,
    pub updated_at: String,
}

impl MocList {
    pub fn new(
        name: String,
        description: Option<String>,
        parts: Vec<MocPart>,
    ) -> Self {
        let now: DateTime<Utc> = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            description,
            parts,
            created_at: now.to_rfc3339(),
            updated_at: now.to_rfc3339(),
        }
    }

    pub fn update_timestamp(&mut self) {
        let now: DateTime<Utc> = Utc::now();
        self.updated_at = now.to_rfc3339();
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MocPart {
    pub part_id: String,
    pub part_number: String,
    pub part_name: String,
    pub color: String,
    pub quantity: i32,
    pub in_stock: i32,
    pub is_missing: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartFilter {
    pub r#type: Option<String>,
    pub color: Option<String>,
    pub size: Option<String>,
    pub location: Option<String>,
    pub keyword: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatsData {
    pub total_parts: i64,
    pub total_quantity: i64,
    pub total_types: i64,
    pub total_colors: i64,
    pub total_locations: i64,
    pub total_mocs: i64,
    pub low_stock_parts: i64,
    pub missing_parts_in_mocs: i64,
    pub parts_by_type: Vec<TypeCount>,
    pub parts_by_color: Vec<ColorCount>,
    pub parts_by_location: Vec<LocationCount>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeCount {
    pub name: String,
    pub count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorCount {
    pub name: String,
    pub count: i64,
    pub hex: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocationCount {
    pub name: String,
    pub count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportExportPart {
    pub name: String,
    pub part_number: String,
    pub r#type: String,
    pub color: String,
    pub size: String,
    pub quantity: i32,
    pub location: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportResult {
    pub imported: i32,
    pub errors: Vec<String>,
}
