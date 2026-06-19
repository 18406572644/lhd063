use tauri::State;

use crate::database::Database;
use crate::models::*;

#[tauri::command]
pub async fn init_database(
    db: State<'_, Database>,
    app: tauri::AppHandle,
) -> Result<(), String> {
    db.init(&app).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_encryption_key(db: State<'_, Database>) -> Result<String, String> {
    Ok(db.get_encryption_key())
}

#[tauri::command]
pub async fn change_encryption_key(
    db: State<'_, Database>,
    old_key: String,
    new_key: String,
) -> Result<(), String> {
    db.change_encryption_key(&old_key, &new_key)
}

#[tauri::command]
pub async fn get_parts(
    db: State<'_, Database>,
    filter: Option<PartFilter>,
) -> Result<Vec<Part>, String> {
    db.get_parts(filter)
}

#[tauri::command]
pub async fn get_part_by_id(
    db: State<'_, Database>,
    id: String,
) -> Result<Option<Part>, String> {
    db.get_part_by_id(&id)
}

#[tauri::command]
pub async fn create_part(
    db: State<'_, Database>,
    part: Part,
) -> Result<Part, String> {
    db.create_part(&part)
}

#[tauri::command]
pub async fn update_part(
    db: State<'_, Database>,
    part: Part,
) -> Result<Part, String> {
    db.update_part(&part)
}

#[tauri::command]
pub async fn delete_part(
    db: State<'_, Database>,
    id: String,
) -> Result<(), String> {
    db.delete_part(&id)
}

#[tauri::command]
pub async fn get_part_types(db: State<'_, Database>) -> Result<Vec<PartType>, String> {
    db.get_part_types()
}

#[tauri::command]
pub async fn create_part_type(
    db: State<'_, Database>,
    r#type: PartType,
) -> Result<PartType, String> {
    db.create_part_type(&r#type)
}

#[tauri::command]
pub async fn update_part_type(
    db: State<'_, Database>,
    r#type: PartType,
) -> Result<PartType, String> {
    db.update_part_type(&r#type)
}

#[tauri::command]
pub async fn delete_part_type(
    db: State<'_, Database>,
    id: String,
) -> Result<(), String> {
    db.delete_part_type(&id)
}

#[tauri::command]
pub async fn get_part_colors(db: State<'_, Database>) -> Result<Vec<PartColor>, String> {
    db.get_part_colors()
}

#[tauri::command]
pub async fn create_part_color(
    db: State<'_, Database>,
    color: PartColor,
) -> Result<PartColor, String> {
    db.create_part_color(&color)
}

#[tauri::command]
pub async fn update_part_color(
    db: State<'_, Database>,
    color: PartColor,
) -> Result<PartColor, String> {
    db.update_part_color(&color)
}

#[tauri::command]
pub async fn delete_part_color(
    db: State<'_, Database>,
    id: String,
) -> Result<(), String> {
    db.delete_part_color(&id)
}

#[tauri::command]
pub async fn get_part_sizes(db: State<'_, Database>) -> Result<Vec<PartSize>, String> {
    db.get_part_sizes()
}

#[tauri::command]
pub async fn create_part_size(
    db: State<'_, Database>,
    size: PartSize,
) -> Result<PartSize, String> {
    db.create_part_size(&size)
}

#[tauri::command]
pub async fn update_part_size(
    db: State<'_, Database>,
    size: PartSize,
) -> Result<PartSize, String> {
    db.update_part_size(&size)
}

#[tauri::command]
pub async fn delete_part_size(
    db: State<'_, Database>,
    id: String,
) -> Result<(), String> {
    db.delete_part_size(&id)
}

#[tauri::command]
pub async fn get_locations(db: State<'_, Database>) -> Result<Vec<Location>, String> {
    db.get_locations()
}

#[tauri::command]
pub async fn create_location(
    db: State<'_, Database>,
    location: Location,
) -> Result<Location, String> {
    db.create_location(&location)
}

#[tauri::command]
pub async fn update_location(
    db: State<'_, Database>,
    location: Location,
) -> Result<Location, String> {
    db.update_location(&location)
}

#[tauri::command]
pub async fn delete_location(
    db: State<'_, Database>,
    id: String,
) -> Result<(), String> {
    db.delete_location(&id)
}

#[tauri::command]
pub async fn get_moc_lists(db: State<'_, Database>) -> Result<Vec<MocList>, String> {
    db.get_moc_lists()
}

#[tauri::command]
pub async fn get_moc_list_by_id(
    db: State<'_, Database>,
    id: String,
) -> Result<Option<MocList>, String> {
    db.get_moc_list_by_id(&id)
}

#[tauri::command]
pub async fn create_moc_list(
    db: State<'_, Database>,
    moc: MocList,
) -> Result<MocList, String> {
    db.create_moc_list(&moc)
}

#[tauri::command]
pub async fn update_moc_list(
    db: State<'_, Database>,
    moc: MocList,
) -> Result<MocList, String> {
    db.update_moc_list(&moc)
}

#[tauri::command]
pub async fn delete_moc_list(
    db: State<'_, Database>,
    id: String,
) -> Result<(), String> {
    db.delete_moc_list(&id)
}

#[tauri::command]
pub async fn compare_moc_inventory(
    db: State<'_, Database>,
    moc_id: String,
) -> Result<MocList, String> {
    db.compare_moc_inventory(&moc_id)
}

#[tauri::command]
pub async fn get_stats(db: State<'_, Database>) -> Result<StatsData, String> {
    db.get_stats()
}

#[tauri::command]
pub async fn export_parts(
    db: State<'_, Database>,
    format: String,
    part_ids: Option<Vec<String>>,
) -> Result<String, String> {
    db.export_parts(&format, part_ids)
}

#[tauri::command]
pub async fn import_parts(
    db: State<'_, Database>,
    format: String,
    data: String,
) -> Result<ImportResult, String> {
    db.import_parts(&format, &data)
}

#[tauri::command]
pub async fn save_part_image(
    db: State<'_, Database>,
    part_id: String,
    image_data: String,
) -> Result<String, String> {
    db.save_part_image(&part_id, &image_data)
}

#[tauri::command]
pub async fn delete_part_image(
    db: State<'_, Database>,
    part_id: String,
) -> Result<(), String> {
    db.delete_part_image(&part_id)
}

#[tauri::command]
pub async fn get_part_image_path(
    db: State<'_, Database>,
    part_id: String,
) -> Result<Option<String>, String> {
    db.get_part_image_path(&part_id)
}
