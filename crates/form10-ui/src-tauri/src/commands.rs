use std::path::PathBuf;

use form10_core::ConverterService;
use tauri::State;

use crate::dto::{ExportPreviewDto, SettingsInputDto, SourceInfoDto, SummaryDto, display_path};
use crate::state::AppState;

fn source(state: &AppState) -> Result<form10_core::ImportedSource, String> {
    state
        .source
        .lock()
        .map_err(|_| "The converter state is unavailable.".to_owned())?
        .clone()
        .ok_or_else(|| "Choose an Excel file first.".to_owned())
}

#[tauri::command]
pub async fn import_source(
    path: String,
    state: State<'_, AppState>,
) -> Result<SourceInfoDto, String> {
    let source = tauri::async_runtime::spawn_blocking(move || {
        ConverterService::new()
            .import(PathBuf::from(path))
            .map_err(|error| error.to_string())
    })
    .await
    .map_err(|error| format!("Could not import the workbook: {error}"))??;

    let info = SourceInfoDto::from(&source);
    *state
        .source
        .lock()
        .map_err(|_| "The converter state is unavailable.".to_owned())? = Some(source);
    Ok(info)
}

#[tauri::command]
pub fn remove_source(state: State<'_, AppState>) -> Result<(), String> {
    *state
        .source
        .lock()
        .map_err(|_| "The converter state is unavailable.".to_owned())? = None;
    Ok(())
}

#[tauri::command]
pub fn summarize(
    settings: SettingsInputDto,
    state: State<'_, AppState>,
) -> Result<SummaryDto, String> {
    let source = source(&state)?;
    ConverterService::new()
        .summarize(&source.data, settings.into())
        .map(SummaryDto::from)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn preview_export(
    settings: SettingsInputDto,
    destination: String,
    state: State<'_, AppState>,
) -> Result<ExportPreviewDto, String> {
    let source = source(&state)?;
    ConverterService::new()
        .preview(&source, settings.into(), PathBuf::from(destination))
        .map(|preview| ExportPreviewDto {
            destination_path: display_path(&preview.destination_path),
            destination_exists: preview.destination_exists,
            suggested_file_name: preview.suggested_file_name,
        })
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn export_workbook(
    settings: SettingsInputDto,
    destination: String,
    overwrite_existing: bool,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let source = source(&state)?;
    tauri::async_runtime::spawn_blocking(move || {
        let service = ConverterService::new();
        let preview = service
            .preview(&source, settings.into(), PathBuf::from(destination))
            .map_err(|error| error.to_string())?;
        service
            .export(&source.data, &preview, overwrite_existing)
            .map(|path| display_path(&path))
            .map_err(|error| error.to_string())
    })
    .await
    .map_err(|error| format!("Could not create the workbook: {error}"))?
}

#[tauri::command]
pub fn open_output(path: String) -> Result<(), String> {
    opener::open(PathBuf::from(path)).map_err(|error| format!("Could not open the file: {error}"))
}

#[tauri::command]
pub fn reveal_output(path: String) -> Result<(), String> {
    opener::reveal(PathBuf::from(path))
        .map_err(|error| format!("Could not show the file in its folder: {error}"))
}
