use std::fs;
use std::io::{Read, Write};
use std::os::windows::process::CommandExt;
use std::path::{Path, PathBuf};
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use tauri::Manager;

use crate::config::AppState;

const CREATE_NO_WINDOW: u32 = 0x08000000;

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ManifestInfo {
    pub name: String,
    pub description: String,
    pub webpage: String,
}

impl Default for ManifestInfo {
    fn default() -> Self {
        Self {
            name: String::new(),
            description: String::new(),
            webpage: String::new(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FileListEntry {
    pub filename: String,
    pub sha256: String,
    pub size: i64,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase", default)]
pub struct DescInfo {
    pub name: String,
    pub builder: String,
    pub description: String,
    pub time: f64,
    // El archivo desc puede usar display_version o file_version; se admite mediante alias
    #[serde(alias = "display_version", alias = "file_version")]
    pub display_version: String,
    #[serde(alias = "build_version")]
    pub build_version: String,
    #[serde(alias = "file_list")]
    pub file_list: Vec<FileListEntry>,
    /// Nombre del archivo .desc correspondiente, usado para localizar cada subpaquete
    #[serde(skip_deserializing)]
    pub desc_file: String,
}

impl Default for DescInfo {
    fn default() -> Self {
        Self {
            name: String::new(),
            builder: String::new(),
            description: String::new(),
            time: 0.0,
            display_version: String::new(),
            build_version: String::new(),
            file_list: Vec::new(),
            desc_file: String::new(),
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PackageDetail {
    pub name: String,
    pub path: String,
    pub kind: String,
    pub size: u64,
    pub manifest: ManifestInfo,
    pub desc_infos: Vec<DescInfo>,
}

pub fn safe_name(name: &str) -> String {
    let cleaned: String = name
        .chars()
        .map(|c| if c.is_alphanumeric() || c == '_' || c == '-' || c == '.' { c } else { '_' })
        .collect();
    if cleaned.trim().is_empty() {
        "package".to_string()
    } else {
        cleaned
    }
}

fn sanitize_entry_path(root: &Path, entry_path: &Path) -> Result<PathBuf, String> {
    let mut clean = PathBuf::new();
    for component in entry_path.components() {
        match component {
            std::path::Component::Normal(seg) => clean.push(seg),
            _ => continue,
        }
    }
    if clean.as_os_str().is_empty() {
        return Err("Ruta de archivo comprimido no válida".to_string());
    }
    Ok(root.join(clean))
}

pub fn extract_7z(archive_path: &Path, dest_dir: &Path, resource_dir: &Path) -> Result<(), String> {
    let seven_zip = resource_dir.join("7zip").join("7z.exe");
    if !seven_zip.exists() {
        return Err("No se encontró 7z.exe; instale primero el paquete de recursos 7zip".to_string());
    }
    let output = std::process::Command::new(&seven_zip)
        .args(["x", "-y"])
        .arg(format!("-o{}", dest_dir.to_string_lossy()))
        .arg(archive_path.to_string_lossy().to_string())
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .map_err(|e| format!("Error al ejecutar la descompresión con 7z: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Error al descomprimir con 7z: {}", stderr));
    }
    Ok(())
}

pub fn handle_single_top_dir(dir: &Path) {
    let entries: Vec<_> = match fs::read_dir(dir) {
        Ok(entries) => entries.flatten().collect(),
        Err(_) => return,
    };
    if entries.len() == 1 {
        if let Some(entry) = entries.first() {
            let path = entry.path();
            if path.is_dir() {
                if let Ok(sub_entries) = fs::read_dir(&path) {
                    for sub_entry in sub_entries.flatten() {
                        let name = sub_entry.file_name();
                        let src = sub_entry.path();
                        let dst = dir.join(&name);
                        if src.is_dir() {
                            let _ = copy_dir(&src, &dst);
                            let _ = fs::remove_dir_all(&src);
                        } else {
                            let _ = fs::copy(&src, &dst);
                            let _ = fs::remove_file(&src);
                        }
                    }
                }
                let _ = fs::remove_dir_all(&path);
            }
        }
    }
}

pub fn extract_zip(zip_path: &Path, dest_dir: &Path) -> Result<(), String> {
    let file = fs::File::open(zip_path).map_err(|e| format!("No se pudo abrir el archivo comprimido: {}", e))?;
    let mut archive = zip::ZipArchive::new(file).map_err(|e| format!("No se pudo analizar el archivo comprimido: {}", e))?;
    for i in 0..archive.len() {
        let mut entry = archive.by_index(i).map_err(|e| format!("Error al leer la entrada del archivo comprimido: {}", e))?;
        let entry_path = PathBuf::from(entry.name());
        let out_path = sanitize_entry_path(dest_dir, &entry_path)?;
        if !out_path.starts_with(dest_dir) {
            return Err("La ruta del archivo comprimido está fuera del destino".to_string());
        }
        if entry.is_dir() {
            fs::create_dir_all(&out_path).map_err(|e| format!("Error al crear la carpeta: {}", e))?;
        } else {
            if let Some(parent) = out_path.parent() {
                fs::create_dir_all(parent).map_err(|e| format!("Error al crear la carpeta principal: {}", e))?;
            }
            let mut buffer = Vec::new();
            entry.read_to_end(&mut buffer).map_err(|e| format!("Error al leer el contenido comprimido: {}", e))?;
            fs::write(&out_path, buffer).map_err(|e| format!("Error al escribir el archivo: {}", e))?;
        }
    }
    Ok(())
}

fn dir_size(path: &Path) -> u64 {
    let mut total = 0u64;
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_dir() {
                total += dir_size(&p);
            } else if let Ok(meta) = fs::metadata(&p) {
                total += meta.len();
            }
        }
    }
    total
}

pub fn read_manifest(dir: &Path) -> ManifestInfo {
    let manifest_path = dir.join("manifest.json");
    if manifest_path.exists() {
        if let Ok(content) = fs::read_to_string(&manifest_path) {
            if let Ok(m) = serde_json::from_str::<ManifestInfo>(&content) {
                return m;
            }
        }
    }
    ManifestInfo::default()
}

pub fn read_desc_files(dir: &Path) -> Vec<DescInfo> {
    let mut list = Vec::new();
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map(|e| e == "desc").unwrap_or(false) {
                if let Ok(content) = fs::read_to_string(&path) {
                    if let Ok(mut d) = serde_json::from_str::<DescInfo>(&content) {
                        d.desc_file = path.file_name()
                            .map(|n| n.to_string_lossy().to_string())
                            .unwrap_or_default();
                        list.push(d);
                    }
                }
            }
        }
    }
    // Se permiten varias entradas con el mismo nombre, sin filtrar duplicados
    list
}

pub fn write_manifest(dir: &Path, name: &str) -> ManifestInfo {
    let manifest_path = dir.join("manifest.json");
    let m = ManifestInfo {
        name: name.to_string(),
        ..Default::default()
    };
    if let Ok(content) = serde_json::to_string_pretty(&m) {
        let _ = fs::write(&manifest_path, content);
    }
    m
}

pub fn make_package_detail(dir: &Path) -> PackageDetail {
    let name = dir.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
    let meta = fs::metadata(dir).ok();
    let size = meta.map(|_| dir_size(dir)).unwrap_or(0);
    let kind = if dir.join("manifest.json").exists() { "resource".to_string() } else { "directory".to_string() };
    let manifest = read_manifest(dir);
    let desc_infos = read_desc_files(dir);
    PackageDetail { name, path: dir.to_string_lossy().to_string(), kind, size, manifest, desc_infos }
}

#[tauri::command]
pub fn list_resource_packages(state: tauri::State<AppState>) -> Result<Vec<PackageDetail>, String> {
    let dir = &state.resource_dir;
    fs::create_dir_all(dir).map_err(|e| format!("No se pudo crear la carpeta de recursos: {}", e))?;
    let mut list = Vec::new();
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                list.push(make_package_detail(&path));
            }
        }
    }
    list.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    Ok(list)
}

#[tauri::command]
pub fn install_resource_from_path(state: tauri::State<AppState>, source_path: String) -> Result<PackageDetail, String> {
    let src = PathBuf::from(&source_path);
    if !src.exists() {
        return Err(format!("La ruta no existe: {}", source_path));
    }
    let rd = &state.resource_dir;
    fs::create_dir_all(rd).map_err(|e| format!("No se pudo crear la carpeta de recursos: {}", e))?;

    if src.is_dir() {
        return Err("No se admite instalar paquetes de recursos desde una carpeta; seleccione un archivo comprimido".to_string());
    }

    let ext = src.extension().and_then(|e| e.to_str().map(|s| s.to_lowercase())).unwrap_or_default();
    let stem = src.file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_else(|| "package".to_string());
    let folder_name = safe_name(&stem);
    let dest = rd.join(&folder_name);

    match ext.as_str() {
        "zip" => {
            if dest.exists() { fs::remove_dir_all(&dest).ok(); }
            fs::create_dir_all(&dest).map_err(|e| format!("Error al crear la carpeta: {}", e))?;
            extract_zip(&src, &dest).map_err(|e| format!("Error al descomprimir: {}", e))?;
            handle_single_top_dir(&dest);
            if !dest.join("manifest.json").exists() {
                write_manifest(&dest, &folder_name);
            }
            Ok(make_package_detail(&dest))
        }
        "7z" => {
            if dest.exists() { fs::remove_dir_all(&dest).ok(); }
            fs::create_dir_all(&dest).map_err(|e| format!("Error al crear la carpeta: {}", e))?;
            extract_7z(&src, &dest, rd).map_err(|e| format!("Error al descomprimir: {}", e))?;
            handle_single_top_dir(&dest);
            // Comprueba si es un paquete de recursos 7zip o aria2 y genera la información correspondiente
            let is_7zip_related = folder_name.to_lowercase().contains("7zip") || folder_name.to_lowercase().contains("7z");
            let is_aria2_related = folder_name.to_lowercase().contains("aria2");
            if is_7zip_related || is_aria2_related {
                let m = if is_7zip_related {
                    ManifestInfo {
                        name: "7zip".to_string(),
                        description: "Herramienta de línea de comandos 7-Zip".to_string(),
                        webpage: String::new(),
                    }
                } else {
                    ManifestInfo {
                        name: "aria2".to_string(),
                        description: "Herramienta de descarga aria2".to_string(),
                        webpage: String::new(),
                    }
                };
                let manifest_path = dest.join("manifest.json");
                if let Ok(content) = serde_json::to_string_pretty(&m) {
                    let _ = fs::write(&manifest_path, content);
                }
            } else if !dest.join("manifest.json").exists() {
                // No se genera el manifest automáticamente
            }
            Ok(make_package_detail(&dest))
        }
        "exe" => {
            // Paquete exe autoextraíble
            let dest = rd.clone();
            log::info!("exe autoextraíble: \"{}\" -y -o{}", src.display(), dest.display());
            let output = std::process::Command::new(&src)
                .arg("-y")
                .arg(format!("-o{}", dest.to_string_lossy()))
                .creation_flags(CREATE_NO_WINDOW)
                .output()
                .map_err(|e| format!("Error al ejecutar la autoextracción: {}", e))?;
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                log::error!("Error en la autoextracción: {}", stderr);
                return Err(format!("Error en la autoextracción: {}", stderr));
            }
            // Se descomprime en la raíz de resources sin llamar a handle_single_top_dir (la raíz tiene varias subcarpetas)
            // Tampoco se escribe manifest.json para no contaminar la raíz de resources
            Ok(make_package_detail(&dest))
        }
        _ => {
            // Otros archivos
            if dest.exists() { fs::remove_dir_all(&dest).ok(); }
            fs::create_dir_all(&dest).map_err(|e| format!("Error al crear la carpeta: {}", e))?;
            let dest_file = dest.join(src.file_name().unwrap_or_default());
            fs::copy(&src, &dest_file).map_err(|e| format!("Error al copiar el archivo: {}", e))?;
            if !dest.join("manifest.json").exists() {
                write_manifest(&dest, &folder_name);
            }
            Ok(make_package_detail(&dest))
        }
    }
}

fn copy_dir(src: &Path, dst: &Path) -> std::io::Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        if ty.is_dir() {
            copy_dir(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)?;
        }
    }
    Ok(())
}

#[tauri::command]
pub async fn install_resource_from_url(state: tauri::State<'_, AppState>, url: String) -> Result<PackageDetail, String> {
    let rd = &state.resource_dir;
    fs::create_dir_all(rd).map_err(|e| format!("No se pudo crear la carpeta de recursos: {}", e))?;

    let filename = url.split('?').next().and_then(|value| value.rsplit('/').next())
        .filter(|name| !name.is_empty())
        .unwrap_or("download.zip").to_string();

    let safe_filename = safe_name(&filename);
    let stem = Path::new(&safe_filename).file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_else(|| "package".to_string());

    let tmp_file = rd.join("__download_tmp");
    let mut aria2_candidates = vec![rd.join("aria2")];
    if let Ok(entries) = fs::read_dir(rd) {
        aria2_candidates.extend(entries.flatten()
            .map(|entry| entry.path())
            .filter(|path| path.is_dir()
                && path.file_name().map(|name| name.to_string_lossy().to_lowercase().contains("aria2")) == Some(true)));
    }
    let aria2 = aria2_candidates.into_iter()
        .flat_map(|dir| [dir.join("aria2c.exe"), dir.join("aria2c"), dir.join("aria2.exe")])
        .find(|path| path.is_file());
    let used_aria2 = if let Some(aria2_path) = aria2 {
        let aria2_path = aria2_path.clone();
        let rd_clone = rd.clone();
        let url_clone = url.clone();
        let result = tauri::async_runtime::spawn_blocking(move || {
            std::process::Command::new(&aria2_path)
                .args(["--allow-overwrite=true", "--dir", &rd_clone.to_string_lossy(), "--out", "__download_tmp", &url_clone])
                .creation_flags(CREATE_NO_WINDOW)
                .output()
        }).await;
        match result {
            Ok(Ok(ref result)) if result.status.success() && tmp_file.exists() => true,
            _ => false,
        }
    } else {
        false
    };

    if !used_aria2 {
        let client = reqwest::Client::new();
        let resp = client.get(&url).send().await.map_err(|e| format!("Error en la solicitud de descarga: {}", e))?;
        if !resp.status().is_success() {
            return Err(format!("Error de descarga: HTTP {}", resp.status()));
        }
        let mut out = fs::File::create(&tmp_file).map_err(|e| format!("Error al crear el archivo temporal: {}", e))?;
        let mut stream = resp.bytes_stream();
        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|e| format!("Error al descargar los datos: {}", e))?;
            out.write_all(&chunk).map_err(|e| format!("Error al escribir el archivo: {}", e))?;
        }
    }

    let folder_name = safe_name(&stem);
    let dest = rd.join(&folder_name);
    if dest.exists() { fs::remove_dir_all(&dest).ok(); }

    if safe_filename.ends_with(".zip") {
        fs::create_dir_all(&dest).map_err(|e| format!("Error al crear la carpeta: {}", e))?;
        extract_zip(&tmp_file, &dest).map_err(|e| format!("Error al descomprimir: {}", e))?;
        handle_single_top_dir(&dest);
        fs::remove_file(&tmp_file).ok();
        if !dest.join("manifest.json").exists() {
            write_manifest(&dest, &folder_name);
        }
        Ok(make_package_detail(&dest))
    } else if safe_filename.ends_with(".7z") {
        fs::create_dir_all(&dest).map_err(|e| format!("Error al crear la carpeta: {}", e))?;
        extract_7z(&tmp_file, &dest, rd).map_err(|e| format!("Error al descomprimir: {}", e))?;
        handle_single_top_dir(&dest);
        fs::remove_file(&tmp_file).ok();
        let is_7zip_related = folder_name.to_lowercase().contains("7zip") || folder_name.to_lowercase().contains("7z");
        let is_aria2_related = folder_name.to_lowercase().contains("aria2");
        if is_7zip_related || is_aria2_related {
            let m = if is_7zip_related {
                ManifestInfo {
                    name: "7zip".to_string(),
                    description: "Herramienta de línea de comandos 7-Zip".to_string(),
                    webpage: String::new(),
                }
            } else {
                ManifestInfo {
                    name: "aria2".to_string(),
                    description: "Herramienta de descarga aria2".to_string(),
                    webpage: String::new(),
                }
            };
            let manifest_path = dest.join("manifest.json");
            if let Ok(content) = serde_json::to_string_pretty(&m) {
                let _ = fs::write(&manifest_path, content);
            }
        }
        Ok(make_package_detail(&dest))
    } else if safe_filename.ends_with(".exe") {
        // Paquete exe autoextraíble
        let dest = rd.clone();
        log::info!("exe autoextraíble: \"{}\" -y -o{}", tmp_file.display(), dest.display());
        let output = std::process::Command::new(&tmp_file)
            .arg("-y")
            .arg(format!("-o{}", dest.to_string_lossy()))
            .creation_flags(CREATE_NO_WINDOW)
            .output()
            .map_err(|e| format!("Error al ejecutar la autoextracción: {}", e))?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            log::error!("Error en la autoextracción: {}", stderr);
            fs::remove_file(&tmp_file).ok();
            return Err(format!("Error en la autoextracción: {}", stderr));
        }
        fs::remove_file(&tmp_file).ok();
        // Se descomprime en la raíz de resources sin llamar a handle_single_top_dir (la raíz tiene varias subcarpetas)
        // Tampoco se escribe manifest.json para no contaminar la raíz de resources
        Ok(make_package_detail(&dest))
    } else {
        fs::create_dir_all(&dest).map_err(|e| format!("Error al crear la carpeta: {}", e))?;
        let dest_file = dest.join(&safe_filename);
        fs::copy(&tmp_file, &dest_file).map_err(|e| format!("Error al copiar el archivo: {}", e))?;
        fs::remove_file(&tmp_file).ok();
        if !dest.join("manifest.json").exists() {
            write_manifest(&dest, &folder_name);
        }
        Ok(make_package_detail(&dest))
    }
}

#[tauri::command]
pub fn delete_resource_package(state: tauri::State<AppState>, name: String) -> Result<(), String> {
    let target = state.resource_dir.join(&name);
    if !target.exists() {
        return Err(format!("El paquete de recursos no existe: {}", name));
    }
    if target.is_dir() {
        fs::remove_dir_all(&target).map_err(|e| format!("Error al eliminar la carpeta: {}", e))?;
    } else {
        fs::remove_file(&target).map_err(|e| format!("Error al eliminar el archivo: {}", e))?;
    }
    Ok(())
}

#[tauri::command]
pub fn delete_sub_package(state: tauri::State<AppState>, package_name: String, desc_file: String) -> Result<(), String> {
    let pkg_dir = state.resource_dir.join(&package_name);
    if !pkg_dir.is_dir() {
        return Err(format!("El paquete de recursos no existe: {}", package_name));
    }
    let desc_path = pkg_dir.join(&desc_file);
    if !desc_path.exists() {
        return Err(format!("El archivo de descripción no existe: {}", desc_file));
    }
    // Lee el archivo desc y localiza los archivos de datos asociados
    let data_files: Vec<String> = if let Ok(content) = fs::read_to_string(&desc_path) {
        if let Ok(di) = serde_json::from_str::<DescInfo>(&content) {
            di.file_list.iter().map(|f| f.filename.clone()).collect()
        } else {
            Vec::new()
        }
    } else {
        Vec::new()
    };
    // Elimina los archivos de datos
    for df in &data_files {
        let df_path = pkg_dir.join(df);
        if df_path.exists() {
            if df_path.is_dir() {
                fs::remove_dir_all(&df_path).map_err(|e| format!("Error al eliminar el archivo de datos: {}: {}", df, e))?;
            } else {
                fs::remove_file(&df_path).map_err(|e| format!("Error al eliminar el archivo de datos: {}: {}", df, e))?;
            }
        }
    }
    // Elimina el propio archivo desc
    fs::remove_file(&desc_path).map_err(|e| format!("Error al eliminar el archivo de descripción: {}", e))?;
    Ok(())
}

#[tauri::command]
pub fn open_resource_dir(app: tauri::AppHandle, name: String) -> Result<(), String> {
    use tauri_plugin_opener::OpenerExt;
    let rd = app.state::<AppState>().resource_dir.clone();
    let path = if name.is_empty() { rd } else { rd.join(&name) };
    if !path.exists() {
        return Err(format!("La ruta no existe: {}", path.display()));
    }
    app.opener().open_path(path.to_string_lossy().to_string(), None::<&str>)
        .map_err(|e| format!("No se pudo abrir: {}", e))?;
    Ok(())
}

#[tauri::command]
pub fn open_url(app: tauri::AppHandle, url: String) -> Result<(), String> {
    use tauri_plugin_opener::OpenerExt;
    app.opener().open_url(url, None::<&str>)
        .map_err(|e| format!("No se pudo abrir el enlace: {}", e))?;
    Ok(())
}
