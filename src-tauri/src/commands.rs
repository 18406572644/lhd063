use tauri::State;

use crate::database::Database;
use crate::models::*;
use crate::backup::BackupService;

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
    part: PartForCreate,
) -> Result<Part, String> {
    db.create_part(part)
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
    r#type: PartTypeForCreate,
) -> Result<PartType, String> {
    db.create_part_type(r#type)
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
    color: PartColorForCreate,
) -> Result<PartColor, String> {
    db.create_part_color(color)
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
    size: PartSizeForCreate,
) -> Result<PartSize, String> {
    db.create_part_size(size)
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
    location: LocationForCreate,
) -> Result<Location, String> {
    db.create_location(location)
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
    moc: MocListForCreate,
) -> Result<MocList, String> {
    db.create_moc_list(moc)
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

#[tauri::command]
pub async fn change_moc_status(
    db: State<'_, Database>,
    change: MocStatusChange,
) -> Result<MocList, String> {
    db.change_moc_status(change)
}

#[tauri::command]
pub async fn get_moc_status_logs(
    db: State<'_, Database>,
    moc_id: String,
) -> Result<Vec<MocStatusLog>, String> {
    db.get_moc_status_logs(&moc_id)
}

#[tauri::command]
pub async fn save_moc_cover_image(
    db: State<'_, Database>,
    moc_id: String,
    image_data: String,
) -> Result<String, String> {
    db.save_moc_cover_image(&moc_id, &image_data)
}

#[tauri::command]
pub async fn delete_moc_cover_image(
    db: State<'_, Database>,
    moc_id: String,
) -> Result<(), String> {
    db.delete_moc_cover_image(&moc_id)
}

#[tauri::command]
pub async fn get_operation_logs(
    db: State<'_, Database>,
    filter: Option<OperationLogFilter>,
) -> Result<Vec<OperationLog>, String> {
    db.get_operation_logs(filter)
}

#[tauri::command]
pub async fn create_backup(
    db: State<'_, Database>,
    app: tauri::AppHandle,
    password: Option<String>,
) -> Result<BackupInfo, String> {
    BackupService::create_backup(&db, &app, password)
}

#[tauri::command]
pub async fn list_backups(
    app: tauri::AppHandle,
) -> Result<Vec<BackupInfo>, String> {
    BackupService::list_backups(&app)
}

#[tauri::command]
pub async fn restore_backup(
    db: State<'_, Database>,
    app: tauri::AppHandle,
    filename: String,
    password: Option<String>,
    mode: String,
) -> Result<RestoreResult, String> {
    BackupService::restore_backup(&db, &app, filename, password, mode)
}

#[tauri::command]
pub async fn delete_backup(
    app: tauri::AppHandle,
    filename: String,
) -> Result<(), String> {
    BackupService::delete_backup(&app, filename)
}

#[tauri::command]
pub async fn get_backup_config(
    app: tauri::AppHandle,
) -> Result<BackupConfig, String> {
    BackupService::get_backup_config(&app)
}

#[tauri::command]
pub async fn update_backup_config(
    app: tauri::AppHandle,
    config: BackupConfig,
) -> Result<(), String> {
    BackupService::update_backup_config(&app, config)
}

#[tauri::command]
pub async fn check_database_integrity(
    db: State<'_, Database>,
    app: tauri::AppHandle,
) -> Result<IntegrityCheckResult, String> {
    BackupService::check_database_integrity(&db, &app)
}

#[tauri::command]
pub async fn export_backup_to_path(
    db: State<'_, Database>,
    app: tauri::AppHandle,
    filename: String,
    dest_dir: String,
) -> Result<String, String> {
    BackupService::export_backup_to_path(&db, &app, filename, dest_dir)
}

#[tauri::command]
pub async fn import_backup_from_path(
    app: tauri::AppHandle,
    src_path: String,
) -> Result<BackupInfo, String> {
    BackupService::import_backup_from_path(&app, src_path)
}

#[tauri::command]
pub async fn should_auto_backup(
    app: tauri::AppHandle,
) -> Result<bool, String> {
    BackupService::should_auto_backup(&app)
}
