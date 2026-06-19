mod commands;
mod crypto;
mod database;
mod models;

use database::Database;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let database = Database::new();

    tauri::Builder::default()
        .manage(database)
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::init_database,
            commands::get_encryption_key,
            commands::change_encryption_key,
            commands::get_parts,
            commands::get_part_by_id,
            commands::create_part,
            commands::update_part,
            commands::delete_part,
            commands::get_part_types,
            commands::create_part_type,
            commands::update_part_type,
            commands::delete_part_type,
            commands::get_part_colors,
            commands::create_part_color,
            commands::update_part_color,
            commands::delete_part_color,
            commands::get_part_sizes,
            commands::create_part_size,
            commands::update_part_size,
            commands::delete_part_size,
            commands::get_locations,
            commands::create_location,
            commands::update_location,
            commands::delete_location,
            commands::get_moc_lists,
            commands::get_moc_list_by_id,
            commands::create_moc_list,
            commands::update_moc_list,
            commands::delete_moc_list,
            commands::compare_moc_inventory,
            commands::get_stats,
            commands::export_parts,
            commands::import_parts,
            commands::save_part_image,
            commands::delete_part_image,
            commands::get_part_image_path,
            commands::change_moc_status,
            commands::get_moc_status_logs,
            commands::save_moc_cover_image,
            commands::delete_moc_cover_image,
            commands::get_operation_logs,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
