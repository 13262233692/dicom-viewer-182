mod dicom_parser;
mod volume;
mod sr_parser;

use dicom_parser::{DicomParser, MultiframeData, MultiframeInfo};
use sr_parser::SRParser;
use serde::{Deserialize, Serialize};
use tauri::State;
use std::sync::Mutex;

struct AppState {
    parser: Mutex<DicomParser>,
    sr_parser: Mutex<SRParser>,
}

#[derive(Debug, Serialize, Deserialize)]
struct LoadDicomRequest {
    path: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct LoadDicomDirectoryRequest {
    directory: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct MprRequest {
    plane: String,
    slice: usize,
}

#[derive(Debug, Serialize, Deserialize)]
struct FrameRequest {
    frame_index: usize,
}

#[tauri::command]
fn load_dicom_file(
    request: LoadDicomRequest,
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let mut parser = state.parser.lock().map_err(|e| e.to_string())?;
    parser.load_file(&request.path)
        .map_err(|e| e.to_string())?;
    let image = parser.get_current_image();
    Ok(serde_json::to_value(image).map_err(|e| e.to_string())?)
}

#[tauri::command]
fn load_dicom_directory(
    request: LoadDicomDirectoryRequest,
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let mut parser = state.parser.lock().map_err(|e| e.to_string())?;
    parser.load_directory(&request.directory)
        .map_err(|e| e.to_string())?;
    let series_info = parser.get_series_info();
    Ok(serde_json::to_value(series_info).map_err(|e| e.to_string())?)
}

#[tauri::command]
fn get_volume_info(state: State<'_, AppState>) -> Result<serde_json::Value, String> {
    let parser = state.parser.lock().map_err(|e| e.to_string())?;
    let info = parser.get_volume_info();
    Ok(serde_json::to_value(info).map_err(|e| e.to_string())?)
}

#[tauri::command]
fn get_mpr_slice(
    request: MprRequest,
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let parser = state.parser.lock().map_err(|e| e.to_string())?;
    let slice = parser.get_mpr_slice(&request.plane, request.slice)
        .map_err(|e| e.to_string())?;
    Ok(serde_json::to_value(slice).map_err(|e| e.to_string())?)
}

#[tauri::command]
fn get_volume_data(state: State<'_, AppState>) -> Result<serde_json::Value, String> {
    let parser = state.parser.lock().map_err(|e| e.to_string())?;
    let volume = parser.get_volume_data()
        .map_err(|e| e.to_string())?;
    Ok(serde_json::to_value(volume).map_err(|e| e.to_string())?)
}

#[tauri::command]
fn get_multiframe_info(state: State<'_, AppState>) -> Result<serde_json::Value, String> {
    let parser = state.parser.lock().map_err(|e| e.to_string())?;
    let info = parser.get_multiframe_info();
    Ok(serde_json::to_value(info).map_err(|e| e.to_string())?)
}

#[tauri::command]
fn get_multiframe_data(state: State<'_, AppState>) -> Result<serde_json::Value, String> {
    let parser = state.parser.lock().map_err(|e| e.to_string())?;
    let data = parser.get_multiframe_data();
    Ok(serde_json::to_value(data).map_err(|e| e.to_string())?)
}

#[tauri::command]
fn get_frame(
    request: FrameRequest,
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let parser = state.parser.lock().map_err(|e| e.to_string())?;
    let frame = parser.get_frame(request.frame_index);
    Ok(serde_json::to_value(frame).map_err(|e| e.to_string())?)
}

#[tauri::command]
fn get_frame_time(
    request: FrameRequest,
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let parser = state.parser.lock().map_err(|e| e.to_string())?;
    let time = parser.get_frame_time(request.frame_index);
    Ok(serde_json::to_value(time).map_err(|e| e.to_string())?)
}

#[tauri::command]
fn load_sr_file(
    request: LoadDicomRequest,
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let mut sr_parser = state.sr_parser.lock().map_err(|e| e.to_string())?;
    sr_parser.load_sr_file(&request.path)
        .map_err(|e| e.to_string())?;
    let doc = sr_parser.get_sr_document();
    Ok(serde_json::to_value(doc).map_err(|e| e.to_string())?)
}

#[tauri::command]
fn get_sr_document(state: State<'_, AppState>) -> Result<serde_json::Value, String> {
    let sr_parser = state.sr_parser.lock().map_err(|e| e.to_string())?;
    let doc = sr_parser.get_sr_document();
    Ok(serde_json::to_value(doc).map_err(|e| e.to_string())?)
}

#[tauri::command]
fn get_sr_measurements(state: State<'_, AppState>) -> Result<serde_json::Value, String> {
    let sr_parser = state.sr_parser.lock().map_err(|e| e.to_string())?;
    let measurements = sr_parser.get_sr_measurements();
    Ok(serde_json::to_value(measurements).map_err(|e| e.to_string())?)
}

#[tauri::command]
fn get_sr_conclusions(state: State<'_, AppState>) -> Result<serde_json::Value, String> {
    let sr_parser = state.sr_parser.lock().map_err(|e| e.to_string())?;
    let conclusions = sr_parser.get_sr_conclusions();
    Ok(serde_json::to_value(conclusions).map_err(|e| e.to_string())?)
}

#[tauri::command]
fn get_sr_image_references(state: State<'_, AppState>) -> Result<serde_json::Value, String> {
    let sr_parser = state.sr_parser.lock().map_err(|e| e.to_string())?;
    let refs = sr_parser.get_sr_image_references();
    Ok(serde_json::to_value(refs).map_err(|e| e.to_string())?)
}

#[derive(Debug, Serialize, Deserialize)]
struct MeasurementsForImageRequest {
    sop_instance_uid: String,
}

#[tauri::command]
fn get_measurements_for_image(
    request: MeasurementsForImageRequest,
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let sr_parser = state.sr_parser.lock().map_err(|e| e.to_string())?;
    let measurements = sr_parser.get_measurements_for_image(&request.sop_instance_uid);
    Ok(serde_json::to_value(measurements).map_err(|e| e.to_string())?)
}

fn main() {
    env_logger::init();
    
    tauri::Builder::default()
        .manage(AppState {
            parser: Mutex::new(DicomParser::new()),
            sr_parser: Mutex::new(SRParser::new()),
        })
        .invoke_handler(tauri::generate_handler![
            load_dicom_file,
            load_dicom_directory,
            get_volume_info,
            get_mpr_slice,
            get_volume_data,
            get_multiframe_info,
            get_multiframe_data,
            get_frame,
            get_frame_time,
            load_sr_file,
            get_sr_document,
            get_sr_measurements,
            get_sr_conclusions,
            get_sr_image_references,
            get_measurements_for_image
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
