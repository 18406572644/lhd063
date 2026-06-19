use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub enum MocStatus {
    Planning,
    Purchasing,
    PartsReady,
    Building,
    Completed,
    Archived,
}

impl MocStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            MocStatus::Planning => "planning",
            MocStatus::Purchasing => "purchasing",
            MocStatus::PartsReady => "parts_ready",
            MocStatus::Building => "building",
            MocStatus::Completed => "completed",
            MocStatus::Archived => "archived",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "purchasing" => MocStatus::Purchasing,
            "parts_ready" => MocStatus::PartsReady,
            "building" => MocStatus::Building,
            "completed" => MocStatus::Completed,
            "archived" => MocStatus::Archived,
            _ => MocStatus::Planning,
        }
    }

    pub fn order(&self) -> i32 {
        match self {
            MocStatus::Planning => 0,
            MocStatus::Purchasing => 1,
            MocStatus::PartsReady => 2,
            MocStatus::Building => 3,
            MocStatus::Completed => 4,
            MocStatus::Archived => 5,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
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
#[serde(rename_all = "camelCase")]
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
#[serde(rename_all = "camelCase")]
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
#[serde(rename_all = "camelCase")]
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
#[serde(rename_all = "camelCase")]
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
#[serde(rename_all = "camelCase")]
pub struct MocList {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub cover_image_path: Option<String>,
    pub status: MocStatus,
    pub parts: Vec<MocPart>,
    pub created_at: String,
    pub updated_at: String,
}

impl MocList {
    pub fn new(
        name: String,
        description: Option<String>,
        cover_image_path: Option<String>,
        parts: Vec<MocPart>,
    ) -> Self {
        let now: DateTime<Utc> = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            description,
            cover_image_path,
            status: MocStatus::Planning,
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
#[serde(rename_all = "camelCase")]
pub struct MocStatusLog {
    pub id: String,
    pub moc_id: String,
    pub old_status: Option<String>,
    pub new_status: String,
    pub changed_at: String,
    pub remark: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
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
#[serde(rename_all = "camelCase")]
pub struct PartFilter {
    pub r#type: Option<String>,
    pub color: Option<String>,
    pub size: Option<String>,
    pub location: Option<String>,
    pub keyword: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
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
    pub mocs_by_status: Vec<MocStatusCount>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MocStatusCount {
    pub status: String,
    pub count: i64,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PartForCreate {
    pub name: String,
    pub part_number: String,
    pub r#type: String,
    pub color: String,
    pub size: String,
    pub quantity: i32,
    pub location: String,
    pub description: Option<String>,
    pub image_path: Option<String>,
}

impl PartForCreate {
    pub fn into_part(self) -> Part {
        let now: DateTime<Utc> = Utc::now();
        Part {
            id: Uuid::new_v4().to_string(),
            name: self.name,
            part_number: self.part_number,
            r#type: self.r#type,
            color: self.color,
            size: self.size,
            quantity: self.quantity,
            location: self.location,
            description: self.description,
            image_path: self.image_path,
            created_at: now.to_rfc3339(),
            updated_at: now.to_rfc3339(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PartTypeForCreate {
    pub name: String,
    pub code: String,
    pub description: Option<String>,
}

impl PartTypeForCreate {
    pub fn into_type(self) -> PartType {
        PartType {
            id: Uuid::new_v4().to_string(),
            name: self.name,
            code: self.code,
            description: self.description,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PartColorForCreate {
    pub name: String,
    pub hex: String,
    pub lego_code: Option<String>,
}

impl PartColorForCreate {
    pub fn into_color(self) -> PartColor {
        PartColor {
            id: Uuid::new_v4().to_string(),
            name: self.name,
            hex: self.hex,
            lego_code: self.lego_code,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PartSizeForCreate {
    pub name: String,
    pub width: f64,
    pub height: f64,
    pub unit: String,
}

impl PartSizeForCreate {
    pub fn into_size(self) -> PartSize {
        PartSize {
            id: Uuid::new_v4().to_string(),
            name: self.name,
            width: self.width,
            height: self.height,
            unit: self.unit,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LocationForCreate {
    pub name: String,
    pub code: String,
    pub description: Option<String>,
    pub parent_id: Option<String>,
}

impl LocationForCreate {
    pub fn into_location(self) -> Location {
        Location {
            id: Uuid::new_v4().to_string(),
            name: self.name,
            code: self.code,
            description: self.description,
            parent_id: self.parent_id,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MocListForCreate {
    pub name: String,
    pub description: Option<String>,
    pub cover_image_path: Option<String>,
    pub parts: Vec<MocPart>,
}

impl MocListForCreate {
    pub fn into_moc_list(self) -> MocList {
        MocList::new(
            self.name,
            self.description,
            self.cover_image_path,
            self.parts,
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MocStatusChange {
    pub moc_id: String,
    pub new_status: MocStatus,
    pub remark: Option<String>,
}
