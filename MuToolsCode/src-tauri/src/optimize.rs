use encoding_rs::GBK;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{Read, Write};
use std::os::windows::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::config::AppState;
use crate::mumu_info::{get_mumu_info, MuMuInfo};
use log;

const CREATE_NO_WINDOW: u32 = 0x08000000;

/// Decodifica la salida de la línea de comandos de Windows
fn decode_windows_output(bytes: &[u8]) -> String {
    match std::str::from_utf8(bytes) {
        Ok(s) => s.to_string(),
        Err(_) => {
            let (cow, _encoding, had_errors) = GBK.decode(bytes);
            if had_errors {
                String::from_utf8_lossy(bytes).to_string()
            } else {
                cow.into_owned()
            }
        }
    }
}

fn get_hosts_path() -> PathBuf {
    PathBuf::from(r"C:\Windows\System32\drivers\etc\hosts")
}

fn ensure_host_entry(hosts_path: &Path, line: &str) -> Result<(), String> {
    let content = fs::read_to_string(hosts_path)
        .map_err(|e| format!("No se pudo leer el archivo hosts: {}", e))?;
    if !content.contains(line.trim()) {
        let mut file = fs::OpenOptions::new()
            .append(true)
            .open(hosts_path)
            .map_err(|e| format!("No se pudo abrir el archivo hosts: {}", e))?;
        // Si el archivo no termina en salto de línea, añade primero un \n para que la nueva entrada no se pegue a la anterior
        if !content.ends_with('\n') {
            write!(file, "\n").map_err(|e| format!("No se pudo escribir en el archivo hosts: {}", e))?;
        }
        writeln!(file, "{}", line).map_err(|e| format!("No se pudo escribir en el archivo hosts: {}", e))?;
    }
    Ok(())
}

fn remove_host_entry(hosts_path: &Path, line: &str) -> Result<(), String> {
    let content = fs::read_to_string(hosts_path)
        .map_err(|e| format!("No se pudo leer el archivo hosts: {}", e))?;
    let trimmed = line.trim();
    let mut new_content = content
        .lines()
        .filter(|l| l.trim() != trimmed)
        .collect::<Vec<_>>()
        .join("\n");
    // Garantiza el salto de línea final para que no se pegue al añadir después desde ensure_host_entry
    if !new_content.ends_with('\n') {
        new_content.push('\n');
    }
    fs::write(hosts_path, new_content)
        .map_err(|e| format!("No se pudo escribir en el archivo hosts: {}", e))?;
    Ok(())
}

fn flush_dns() -> Result<(), String> {
    let output = Command::new("ipconfig")
        .args(["/flushdns"])
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .map_err(|e| format!("No se pudo ejecutar ipconfig /flushdns: {}", e))?;
    if !output.status.success() {
        return Err(format!("No se pudo actualizar el DNS: {}", decode_windows_output(&output.stderr)));
    }
    Ok(())
}

fn check_hosts_blocked(domain: &str) -> Result<bool, String> {
    let output = Command::new("ping")
        .args([domain, "-n", "1"])
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .map_err(|e| format!("No se pudo ejecutar ping: {}", e))?;
    let stdout = decode_windows_output(&output.stdout);
    Ok(stdout.contains("127.0.0.1"))
}

fn run_netsh(args: &[&str]) -> Result<String, String> {
    let output = Command::new("netsh")
        .args(args)
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .map_err(|e| format!("No se pudo ejecutar netsh: {}", e))?;
    let stdout = decode_windows_output(&output.stdout);
    if !output.status.success() {
        let stderr = decode_windows_output(&output.stderr);
        return Err(format!("Error al ejecutar netsh: {}\nSalida: {}", stderr, stdout));
    }
    Ok(stdout)
}


fn get_install_dir_from_info(info: &MuMuInfo) -> Result<String, String> {
    info.install_dir
        .clone()
        .ok_or_else(|| "No se pudo obtener la carpeta de instalación de MuMu".to_string())
}

fn get_appdata_dir_from_info(info: &MuMuInfo) -> Result<String, String> {
    info.appdata_dir
        .clone()
        .ok_or_else(|| "No se pudo obtener la carpeta APPDATA".to_string())
}

fn ensure_parent_dir(path: &Path) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("No se pudo crear la carpeta: {}", e))?;
    }
    Ok(())
}

fn major_version(info: &MuMuInfo) -> String {
    info.major_version.clone().unwrap_or_default()
}

// Bloquear los dominios de actualización

#[tauri::command]
pub async fn block_update_domains() -> Result<String, String> {
    let hosts = get_hosts_path();
    let entries = [
        "127.0.0.1 mumu.nie.netease.com",
        "127.0.0.1 g.fp.ps.netease.com",
    ];
    for entry in &entries {
        ensure_host_entry(&hosts, entry)?;
    }
    flush_dns()?;
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    if !check_hosts_blocked("g.fp.ps.netease.com")? {
        return Err("No se pudo modificar hosts: el dominio todavía se resuelve con normalidad".to_string());
    }
    Ok("Dominios de actualización bloqueados".to_string())
}

#[tauri::command]
pub async fn unblock_update_domains() -> Result<String, String> {
    let hosts = get_hosts_path();
    let entries = [
        "127.0.0.1 mumu.nie.netease.com",
        "127.0.0.1 g.fp.ps.netease.com",
    ];
    for entry in &entries {
        remove_host_entry(&hosts, entry)?;
    }
    flush_dns()?;
    if check_hosts_blocked("g.fp.ps.netease.com").unwrap_or(false) {
        Err("La resolución aún no se ha restaurado; comprueba la red o la configuración de hosts".to_string())
    } else {
        Ok("Dominios de actualización restaurados".to_string())
    }
}

// Reglas de firewall

#[tauri::command]
pub fn add_firewall_rule_1() -> Result<String, String> {
    let info = get_mumu_info()?;
    let install_dir = get_install_dir_from_info(&info)?;
    let program = format!(r"{}\shell\MuMuPlayer.exe", install_dir);
    let output = run_netsh(&[
        "advfirewall", "firewall", "add", "rule",
        "name=MuMuADBlock1",
        r#"description=Regla de firewall 1 creada por Mutools para bloquear los anuncios del centro de mensajes"#,
        "dir=out",
        "action=block",
        &format!("program={}", program),
        "remoteip=42.186.241.52,42.186.110.59",
        "enable=yes",
    ])?;
    Ok(format!("Regla de firewall 1 añadida\n{}", output))
}

#[tauri::command]
pub fn remove_firewall_rule_1() -> Result<String, String> {
    let output = run_netsh(&[
        "advfirewall", "firewall", "delete", "rule", "name=MuMuADBlock1",
    ])?;
    Ok(format!("Regla de firewall 1 eliminada\n{}", output))
}

// Desactivar la imagen de inicio

#[tauri::command]
pub fn disable_startup_image() -> Result<String, String> {
    let info = get_mumu_info()?;
    let appdata_dir = get_appdata_dir_from_info(&info)?;
    let netease_appdata = PathBuf::from(&appdata_dir).join("Netease");

    // Compatibilidad con las distintas ubicaciones de la imagen de inicio según la versión
    let candidates: Vec<PathBuf> = vec![
        netease_appdata.join("MuMuPlayer").join("startupImage"),
        netease_appdata.join("MuMuPlayer-12.0").join("startupImage"),
        netease_appdata.join("MuMuPlayer").join("data").join("startupImage"),
        netease_appdata.join("MuMuPlayer-12.0").join("data").join("startupImage"),
    ];

    let mut handled = 0usize;
    for candidate in &candidates {
        if candidate.is_dir() {
            fs::remove_dir_all(candidate).map_err(|e| format!("No se pudo eliminar la carpeta startupImage ({}): {}", candidate.display(), e))?;
        } else if candidate.is_file() {
            // Ya es un archivo
            handled += 1;
            continue;
        }
        // La carpeta no existe o ya se ha eliminado: se crea un archivo vacío como marcador
        ensure_parent_dir(candidate)?;
        fs::write(candidate, b"").map_err(|e| format!("No se pudo crear el archivo startupImage ({}): {}", candidate.display(), e))?;
        handled += 1;
    }

    if handled == 0 {
        return Err("No se encontró ninguna carpeta startupImage".to_string());
    }

    Ok("Imagen de inicio desactivada".to_string())
}

#[tauri::command]
pub fn restore_startup_image() -> Result<String, String> {
    let info = get_mumu_info()?;
    let appdata_dir = get_appdata_dir_from_info(&info)?;
    let netease_appdata = PathBuf::from(&appdata_dir).join("Netease");

    let candidates: Vec<PathBuf> = vec![
        netease_appdata.join("MuMuPlayer").join("startupImage"),
        netease_appdata.join("MuMuPlayer-12.0").join("startupImage"),
        netease_appdata.join("MuMuPlayer").join("data").join("startupImage"),
        netease_appdata.join("MuMuPlayer-12.0").join("data").join("startupImage"),
    ];

    let mut restored = 0usize;
    for candidate in &candidates {
        if candidate.exists() && candidate.is_file() {
            fs::remove_file(candidate).map_err(|e| format!("No se pudo eliminar el archivo startupImage ({}): {}", candidate.display(), e))?;
            fs::create_dir_all(candidate).map_err(|e| format!("No se pudo crear la carpeta startupImage ({}): {}", candidate.display(), e))?;
            restored += 1;
        }
    }

    if restored == 0 {
        Err("No se encontró ningún archivo startupImage; no hace falta restaurar".to_string())
    } else {
        Ok(format!("Imagen de inicio restaurada; se han procesado {} ubicaciones", restored))
    }
}

// Importar el paquete de optimización de la partición Data para varias instancias

/// Tipos de archivo del paquete de recursos que se usan para la optimización
fn find_optimize_files(pkg_dir: &Path, extensions: &[&str]) -> Vec<PathBuf> {
    let mut result = Vec::new();
    if let Ok(entries) = fs::read_dir(pkg_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                    if extensions.iter().any(|&e| e.eq_ignore_ascii_case(ext)) {
                        result.push(path);
                    }
                }
            }
        }
    }
    result
}

/// Lista los paquetes de recursos instalados que sirven para la optimización de Data
#[tauri::command]
pub fn list_data_optimize_packages(state: tauri::State<AppState>) -> Result<Vec<serde_json::Value>, String> {
    let rd = &state.resource_dir;
    if !rd.exists() {
        return Ok(Vec::new());
    }
    let mut result = Vec::new();
    if let Ok(entries) = fs::read_dir(rd) {
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() { continue; }
            let files = find_optimize_files(&path, &["mumudata"]);
            let name = path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
            let manifest = crate::packages::read_manifest(&path);
            let display_name = if !manifest.name.is_empty() { manifest.name.clone() } else { name.clone() };
            let descs = crate::packages::read_desc_files(&path);
            let version = descs.first().map(|d| d.display_version.clone()).unwrap_or_default();
            for file_path in files {
                let file_name = file_path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
                result.push(serde_json::json!({
                    "packageName": name,
                    "displayName": display_name,
                    "description": manifest.description,
                    "version": version,
                    "fileName": file_name,
                    "filePath": file_path.to_string_lossy().to_string(),
                }));
            }
        }
    }
    result.sort_by(|a, b| a["displayName"].as_str().unwrap_or("").to_lowercase().cmp(&b["displayName"].as_str().unwrap_or("").to_lowercase()));
    Ok(result)
}

#[tauri::command]
pub fn import_data_package(path: String) -> Result<String, String> {
    let info = get_mumu_info()?;
    let major = major_version(&info);
    let manager = info.mumu_manager_path.ok_or_else(|| "No se encontró MuMuManager".to_string())?;

    if !major.starts_with('5') && !major.starts_with('6') && !major.starts_with('4') {
        return Err("Versión no compatible".to_string());
    }

    let output = Command::new(&manager)
        .args(["import", "-p", &path, "-n", "1"])
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .map_err(|e| format!("No se pudo ejecutar MuMuManager import: {}", e))?;

    let stdout = decode_windows_output(&output.stdout);
    if !output.status.success() {
        return Err(format!("Error al importar: {}", decode_windows_output(&output.stderr)));
    }

    Ok(format!("Paquete de optimización de Data importado\n{}", stdout))
}

#[tauri::command]
pub fn undo_import_data_package() -> Result<String, String> {
    Ok("No se pueden identificar automáticamente las instancias que usan el paquete de optimización de Data; elimina las instancias correspondientes a mano".to_string())
}

// Reglas de firewall

#[tauri::command]
pub fn add_firewall_rule_2() -> Result<String, String> {
    let info = get_mumu_info()?;
    let install_dir = get_install_dir_from_info(&info)?;

    let remote_ip = "42.186.25.77,115.236.122.147,117.147.201.42,101.71.7.42";

    // Regla 1
    let program1 = r"%ProgramFiles%\MuMuVMMVbox\Hypervisor\MuMuVMMHeadless.exe".to_string();
    let program1_expanded = expand_environment_path(&program1)?;
    let output1 = run_netsh(&[
        "advfirewall", "firewall", "add", "rule",
        "name=MuMuADBlock2",
        r#"description=Mutools create from mumu.nie.netease.com"#,
        "dir=out",
        "action=block",
        &format!("program={}", program1_expanded),
        &format!("remoteip={}", remote_ip),
        "enable=yes",
    ])?;

    // Regla 2
    let program2 = format!(r"{}\nx_device\12.0\hypervisor\MuMuVMMHeadless.exe", install_dir);
    let output2 = run_netsh(&[
        "advfirewall", "firewall", "add", "rule",
        "name=MuMuADBlock3",
        r#"description=Mutools create from mumu.nie.netease.com"#,
        "dir=out",
        "action=block",
        &format!("program={}", program2),
        &format!("remoteip={}", remote_ip),
        "enable=yes",
    ])?;

    Ok(format!("Regla de firewall 2 añadida\n[Regla 1]\n{}\n[Regla 2]\n{}", output1, output2))
}

#[tauri::command]
pub fn remove_firewall_rule_2() -> Result<String, String> {
    let output1 = run_netsh(&[
        "advfirewall", "firewall", "delete", "rule", "name=MuMuADBlock2",
    ])?;
    let output2 = run_netsh(&[
        "advfirewall", "firewall", "delete", "rule", "name=MuMuADBlock3",
    ])?;
    Ok(format!("Regla de firewall 2 eliminada\n[Regla 1]\n{}\n[Regla 2]\n{}", output1, output2))
}

fn expand_environment_path(path: &str) -> Result<String, String> {
    // Best-effort expansion of %ProgramFiles%
    let program_files = std::env::var("ProgramFiles")
        .map_err(|e| format!("No se pudo obtener la variable de entorno ProgramFiles: {}", e))?;
    Ok(path.replace("%ProgramFiles%", &program_files))
}

// Desactivar/restaurar MuMuPlayerUpdater.exe

#[tauri::command]
pub fn disable_updater() -> Result<String, String> {
    let info = get_mumu_info()?;
    let install_dir = get_install_dir_from_info(&info)?;
    let major = major_version(&info);

    let updater_path = if major.starts_with('5') || major.starts_with('6') {
        PathBuf::from(&install_dir).join("nx_main").join("MuMuPlayerUpdater.exe")
    } else {
        PathBuf::from(&install_dir).join("shell").join("MuMuPlayerUpdater.exe")
    };

    let bak_path = updater_path.with_extension("exe.bak");

    if !updater_path.exists() && bak_path.exists() {
        // Already disabled
        return Ok("El actualizador ya está desactivado".to_string());
    }

    if updater_path.exists() {
        if bak_path.exists() {
            fs::remove_file(&updater_path).map_err(|e| format!("No se pudo eliminar el actualizador original: {}", e))?;
        } else {
            fs::rename(&updater_path, &bak_path).map_err(|e| format!("No se pudo renombrar el actualizador: {}", e))?;
        }
    }

    fs::write(&updater_path, b"").map_err(|e| format!("No se pudo crear el actualizador vacío: {}", e))?;
    Ok("Actualizador desactivado".to_string())
}

#[tauri::command]
pub fn restore_updater() -> Result<String, String> {
    let info = get_mumu_info()?;
    let install_dir = get_install_dir_from_info(&info)?;
    let major = major_version(&info);

    let updater_path = if major.starts_with('5') || major.starts_with('6') {
        PathBuf::from(&install_dir).join("nx_main").join("MuMuPlayerUpdater.exe")
    } else {
        PathBuf::from(&install_dir).join("shell").join("MuMuPlayerUpdater.exe")
    };

    let bak_path = updater_path.with_extension("exe.bak");

    if bak_path.exists() {
        if updater_path.exists() {
            fs::remove_file(&updater_path).map_err(|e| format!("No se pudo eliminar el actualizador vacío: {}", e))?;
        }
        fs::rename(&bak_path, &updater_path).map_err(|e| format!("No se pudo restaurar el actualizador: {}", e))?;
        Ok("Actualizador restaurado".to_string())
    } else {
        Err("No se encontró el archivo de copia de seguridad .bak".to_string())
    }
}

// [4.x] Optimización del modo imitación Premium (sustitución de overlay)

/// Busca el 7z.exe instalado, primero en AppState.resource_dir y, si no, en la carpeta resources junto al exe
fn find_seven_zip(resource_dir: &Path) -> Result<PathBuf, String> {
    let primary = resource_dir.join("7zip").join("7z.exe");
    if primary.exists() {
        return Ok(primary);
    }
    let fallback = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        .map(|d| d.join("resources").join("7zip").join("7z.exe"))
        .unwrap_or_else(|| PathBuf::from("7z.exe"));
    if fallback.exists() {
        Ok(fallback)
    } else {
        Err("No se encontró 7z.exe; instala antes el paquete de recursos 7zip".to_string())
    }
}

/// Lista los paquetes de recursos instalados que sirven para la optimización del modo imitación Premium (overlay) de 4.x
#[tauri::command]
pub fn list_overlay_optimize_packages(state: tauri::State<AppState>) -> Result<Vec<serde_json::Value>, String> {
    let rd = &state.resource_dir;
    if !rd.exists() {
        return Ok(Vec::new());
    }
    let mut result = Vec::new();
    if let Ok(entries) = fs::read_dir(rd) {
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() { continue; }
            let files = find_optimize_files(&path, &["7z", "zip"]);
            let name = path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
            let manifest = crate::packages::read_manifest(&path);
            let descs = crate::packages::read_desc_files(&path);
            let display_name = if !manifest.name.is_empty() { manifest.name.clone() } else { name.clone() };
            let version = descs.first().map(|d| d.display_version.clone()).unwrap_or_default();
            for file_path in files {
                let file_name = file_path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
                result.push(serde_json::json!({
                    "packageName": name,
                    "displayName": display_name,
                    "description": manifest.description,
                    "version": version,
                    "fileName": file_name,
                    "filePath": file_path.to_string_lossy().to_string(),
                }));
            }
        }
    }
    result.sort_by(|a, b| a["displayName"].as_str().unwrap_or("").to_lowercase().cmp(&b["displayName"].as_str().unwrap_or("").to_lowercase()));
    Ok(result)
}

#[tauri::command]
pub fn apply_overlay_package(state: tauri::State<AppState>, path: String) -> Result<String, String> {
    let info = get_mumu_info()?;
    let major = major_version(&info);
    if !major.starts_with('4') {
        return Err("Usa otra opción".to_string());
    }
    let install_dir = get_install_dir_from_info(&info)?;
    let overlay_dir = PathBuf::from(&install_dir).join("overlay");
    let backup_dir = PathBuf::from(&install_dir).join("overlay_backup");

    if overlay_dir.exists() && !backup_dir.exists() {
        fs::rename(&overlay_dir, &backup_dir).map_err(|e| format!("No se pudo hacer la copia de seguridad de overlay: {}", e))?;
        fs::create_dir_all(&overlay_dir).map_err(|e| format!("No se pudo crear la carpeta overlay: {}", e))?;
    } else if overlay_dir.exists() {
        fs::remove_dir_all(&overlay_dir).map_err(|e| format!("No se pudo eliminar el overlay antiguo: {}", e))?;
    }

    let seven_zip = find_seven_zip(&state.resource_dir)?;

    let output = Command::new(&seven_zip)
        .args(["x", "-y", &format!("-o{}", install_dir)])
        .arg(&path)
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .map_err(|e| format!("No se pudo extraer el paquete overlay: {}", e))?;

    if !output.status.success() {
        return Err(format!("No se pudo extraer el paquete overlay: {}", decode_windows_output(&output.stderr)));
    }

    Ok("Overlay del modo imitación Premium aplicado".to_string())
}

#[tauri::command]
pub fn undo_overlay_package() -> Result<String, String> {
    let info = get_mumu_info()?;
    let major = major_version(&info);
    if !major.starts_with('4') {
        return Err("Usa otra opción".to_string());
    }
    let install_dir = get_install_dir_from_info(&info)?;
    let overlay_dir = PathBuf::from(&install_dir).join("overlay");
    let backup_dir = PathBuf::from(&install_dir).join("overlay_backup");

    if overlay_dir.exists() {
        fs::remove_dir_all(&overlay_dir).map_err(|e| format!("No se pudo eliminar overlay: {}", e))?;
    }

    if backup_dir.exists() {
        fs::rename(&backup_dir, &overlay_dir).map_err(|e| format!("No se pudo restaurar overlay: {}", e))?;
    } else {
        fs::create_dir_all(&overlay_dir).map_err(|e| format!("No se pudo crear un overlay vacío: {}", e))?;
    }

    Ok("Overlay del modo imitación Premium restaurado".to_string())
}

// Modificar el identificador de canal fchannel

#[tauri::command]
pub fn apply_fchannel() -> Result<String, String> {
    let info = get_mumu_info()?;
    let major = major_version(&info);
    if !major.starts_with('5') && !major.starts_with('6') {
        return Err("Esta función solo es compatible con las versiones 5.x o 6.x".to_string());
    }
    let appdata_dir = get_appdata_dir_from_info(&info)?;
    let config_path = PathBuf::from(&appdata_dir)
        .join("Netease")
        .join("MuMuPlayer")
        .join("install_config.json");

    let content = fs::read_to_string(&config_path)
        .map_err(|e| format!("No se pudo leer install_config.json: {}", e))?;
    let mut json: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| format!("No se pudo analizar install_config.json: {}", e))?;

    let product = json
        .get_mut("product")
        .ok_or_else(|| "Falta el nodo product en install_config.json".to_string())?;

    let current = product.get("fchannel").and_then(|v| v.as_str()).unwrap_or("");
    if current == "yx-yys" {
        return Ok("fchannel ya es yx-yys".to_string());
    }

    product["fchannel"] = serde_json::Value::String("yx-yys".to_string());
    fs::write(&config_path, serde_json::to_string_pretty(&json).map_err(|e| format!("Error de serialización: {}", e))?)
        .map_err(|e| format!("No se pudo escribir install_config.json: {}", e))?;

    Ok("fchannel cambiado a yx-yys".to_string())
}

#[tauri::command]
pub fn undo_fchannel() -> Result<String, String> {
    let info = get_mumu_info()?;
    let major = major_version(&info);
    if !major.starts_with('5') && !major.starts_with('6') {
        return Err("Esta función solo es compatible con las versiones 5.x o 6.x".to_string());
    }
    let appdata_dir = get_appdata_dir_from_info(&info)?;
    let config_path = PathBuf::from(&appdata_dir)
        .join("Netease")
        .join("MuMuPlayer")
        .join("install_config.json");

    let content = fs::read_to_string(&config_path)
        .map_err(|e| format!("No se pudo leer install_config.json: {}", e))?;
    let mut json: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| format!("No se pudo analizar install_config.json: {}", e))?;

    let product = json
        .get_mut("product")
        .ok_or_else(|| "Falta el nodo product en install_config.json".to_string())?;

    product["fchannel"] = serde_json::Value::String("nochannel-mumu12".to_string());
    fs::write(&config_path, serde_json::to_string_pretty(&json).map_err(|e| format!("Error de serialización: {}", e))?)
        .map_err(|e| format!("No se pudo escribir install_config.json: {}", e))?;

    Ok("fchannel restaurado a nochannel-mumu12".to_string())
}

// Modificar system.vdi para añadir la marca de versión internacional

/// Devuelve la lista de rutas de system.vdi según la versión principal
fn get_system_vdi_paths(install_dir: &str, major: &str) -> Result<Vec<PathBuf>, String> {
    let mut paths = Vec::new();
    if major.starts_with('5') {
        paths.push(PathBuf::from(install_dir)
            .join("nx_device")
            .join("12.0")
            .join("vms")
            .join("MuMuPlayer-12.0-base")
            .join("system.vdi"));
    } else if major.starts_with('6') {
        // La v6 gestiona a la vez las versiones 12.0 y 15.0
        paths.push(PathBuf::from(install_dir)
            .join("nx_device")
            .join("12.0")
            .join("vms")
            .join("MuMuPlayer-12.0-base")
            .join("system.vdi"));
        paths.push(PathBuf::from(install_dir)
            .join("nx_device")
            .join("15.0")
            .join("vms")
            .join("MuMuPlayer-15.0-base")
            .join("system.vdi"));
    } else {
        return Err("Esta función solo es compatible con las versiones 5.x o 6.x".to_string());
    }
    Ok(paths)
}

const OVERSEAS_ORIGINAL: &str = "####################################\n# from generate-common-build-props\n# These properties identify this partition image.\n####################################";
const OVERSEAS_REPLACEMENT: &str = "####################################\n# from generate-common-build-props\n# These properties identify this partition image.\n#####\nro.build.version.overseas=true";

#[derive(Serialize, Deserialize)]
struct SystemVdiPatchRecord {
    start_pos: usize,
    #[serde(alias = "original_base64")]
    original_data: String,
    #[serde(alias = "md5")]
    md5_hash: String,
    timestamp: u64,
}

fn md5_file(path: &Path) -> Result<String, String> {
    let mut file = fs::File::open(path).map_err(|e| format!("No se pudo abrir el archivo: {}", e))?;
    let mut context = md5::Context::new();
    let mut buf = [0u8; 8192];
    loop {
        let n = file.read(&mut buf).map_err(|e| format!("No se pudo leer el archivo: {}", e))?;
        if n == 0 { break; }
        context.consume(&buf[..n]);
    }
    Ok(format!("{:x}", context.compute()))
}

#[tauri::command]
pub async fn patch_system_vdi_overseas() -> Result<String, String> {
    let info = get_mumu_info()?;
    let major = major_version(&info);
    let install_dir = get_install_dir_from_info(&info)?;
    let vdi_paths = get_system_vdi_paths(&install_dir, &major)?;

    log::info!("=== Inicio de la marca de versión internacional, versión: {}, archivos: {} ===", major, vdi_paths.len());

    let mut results = Vec::new();
    for vdi_path in &vdi_paths {
        let vdi_path = vdi_path.clone();
        let record_path = vdi_path.with_extension("vdi.mutools_patch");

        log::info!("Procesando: {}", vdi_path.display());

        if !vdi_path.exists() {
            log::info!("Se omite, el archivo no existe: {}", vdi_path.display());
            results.push(format!("Se omite {} (el archivo no existe)", vdi_path.display()));
            continue;
        }

        // Si ya existe un registro de parche, se omite
        if record_path.exists() {
            log::info!("Se omite, ya hay un registro de parche: {}", record_path.display());
            results.push(format!("Se omite {} (ya modificado)", vdi_path.display()));
            continue;
        }

        log::info!("Iniciando la modificación: {}", vdi_path.display());

        // La E/S de archivos grandes se mueve a spawn_blocking para no bloquear el hilo principal
        let vdi_path_for_block = vdi_path.clone();
        let (md5_result, record) = tokio::task::spawn_blocking(move || -> Result<(String, SystemVdiPatchRecord), String> {
            let content = fs::read(&vdi_path_for_block).map_err(|e| format!("No se pudo leer system.vdi: {}", e))?;
            log::info!("Tamaño del archivo: {} bytes", content.len());

            let original_bytes = OVERSEAS_ORIGINAL.as_bytes();
            let start_pos = content.windows(original_bytes.len())
                .position(|window| window == original_bytes)
                .ok_or_else(|| "No se encontró el contenido coincidente".to_string())?;

            log::info!("Posición coincidente encontrada: {}", start_pos);

            let end_pos = start_pos + original_bytes.len();
            let original_segment = content[start_pos..end_pos].to_vec();
            let replacement_bytes = OVERSEAS_REPLACEMENT.as_bytes();

            // Sustitución mediante concatenación de bytes (permite que cambie el tamaño del archivo)
            let new_content = [&content[..start_pos], replacement_bytes, &content[end_pos..]].concat();
            fs::write(&vdi_path_for_block, &new_content).map_err(|e| format!("No se pudo escribir system.vdi: {}", e))?;

            let md5 = md5_file(&vdi_path_for_block)?;
            log::info!("Nuevo MD5: {}", md5);

            let record = SystemVdiPatchRecord {
                start_pos,
                original_data: base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &original_segment),
                md5_hash: md5.clone(),
                timestamp: fs::metadata(&vdi_path_for_block)
                    .map_err(|e| format!("No se pudieron obtener los metadatos del archivo: {}", e))?
                    .modified()
                    .map_err(|e| format!("No se pudo obtener la fecha de modificación: {}", e))?
                    .duration_since(std::time::UNIX_EPOCH)
                    .map_err(|e| format!("Error al convertir la fecha: {}", e))?
                    .as_secs(),
            };
            Ok((md5, record))
        })
        .await
        .map_err(|e| format!("Error en la tarea de modificación del VDI: {}", e))??;

        fs::write(&record_path, serde_json::to_string_pretty(&record).map_err(|e| format!("No se pudo serializar el registro: {}", e))?)
            .map_err(|e| format!("No se pudo escribir el archivo de registro: {}", e))?;

        log::info!("Completado: {} (MD5: {})", vdi_path.display(), md5_result);
        results.push(format!("Marca de versión internacional añadida en {}, MD5: {}", vdi_path.display(), md5_result));
    }

    log::info!("=== Marca de versión internacional añadida ===");
    Ok(results.join("\n"))
}

#[tauri::command]
pub async fn undo_patch_system_vdi_overseas() -> Result<String, String> {
    let info = get_mumu_info()?;
    let major = major_version(&info);
    let install_dir = get_install_dir_from_info(&info)?;
    let vdi_paths = get_system_vdi_paths(&install_dir, &major)?;

    log::info!("=== Inicio de la restauración de la marca de versión internacional, versión: {}, archivos: {} ===", major, vdi_paths.len());

    let mut results = Vec::new();
    for vdi_path in &vdi_paths {
        let vdi_path = vdi_path.clone();
        let record_path = vdi_path.with_extension("vdi.mutools_patch");

        log::info!("Procesando: {}", vdi_path.display());

        if !vdi_path.exists() {
            log::info!("Se omite, el archivo no existe: {}", vdi_path.display());
            results.push(format!("Se omite {} (el archivo no existe)", vdi_path.display()));
            continue;
        }

        if !record_path.exists() {
            log::info!("Se omite, no se encontró el registro de parche: {}", record_path.display());
            results.push(format!("Se omite {} (no se encontró el registro de parche)", vdi_path.display()));
            continue;
        }

        log::info!("Iniciando la restauración: {}", vdi_path.display());

        let record_content = fs::read_to_string(&record_path)
            .map_err(|e| format!("No se pudo leer el archivo de registro: {}", e))?;
        let record: SystemVdiPatchRecord = serde_json::from_str(&record_content)
            .map_err(|e| format!("No se pudo analizar el archivo de registro: {}", e))?;

        log::info!("Posición registrada: {}, MD5 original: {}", record.start_pos, record.md5_hash);

        // La E/S de archivos grandes se mueve a spawn_blocking para no bloquear el hilo principal
        let original_bytes = base64::Engine::decode(&base64::engine::general_purpose::STANDARD, &record.original_data)
            .map_err(|e| format!("No se pudieron decodificar los bytes originales: {}", e))?;
        let start_pos = record.start_pos;
        let expected_md5 = record.md5_hash.clone();
        let vdi_path_for_block = vdi_path.clone();
        let record_path_for_block = record_path.clone();
        tokio::task::spawn_blocking(move || -> Result<(), String> {
            let current_md5 = md5_file(&vdi_path_for_block)?;
            log::info!("MD5 actual: {}", current_md5);
            if current_md5 != expected_md5 {
                return Err("Otro programa ha modificado el archivo; no se puede restaurar".to_string());
            }

            let replacement_bytes = OVERSEAS_REPLACEMENT.as_bytes();

            let content = fs::read(&vdi_path_for_block).map_err(|e| format!("No se pudo leer system.vdi: {}", e))?;
            let end_pos = start_pos + replacement_bytes.len();

            // Comprobar si la posición es válida
            if start_pos >= content.len() || end_pos > content.len() {
                return Err(format!("Posición registrada no válida: posición {} | tamaño del archivo {}", start_pos, content.len()));
            }

            // Restauración del contenido original mediante concatenación de bytes
            let new_content = [&content[..start_pos], &original_bytes, &content[end_pos..]].concat();
            fs::write(&vdi_path_for_block, &new_content).map_err(|e| format!("No se pudo escribir system.vdi: {}", e))?;
            fs::remove_file(&record_path_for_block).map_err(|e| format!("No se pudo eliminar el archivo de registro: {}", e))?;

            log::info!("Restauración completada: {}", vdi_path_for_block.display());
            Ok(())
        })
        .await
        .map_err(|e| format!("Error en la tarea de restauración del VDI: {}", e))??;

        results.push(format!("Marca de versión internacional restaurada en {}", vdi_path.display()));
    }

    log::info!("=== Marca de versión internacional restaurada ===");
    Ok(results.join("\n"))
}

// Bloquear los dominios de envío de datos

#[tauri::command]
pub async fn block_report_domains() -> Result<String, String> {
    let hosts = get_hosts_path();
    let entries = [
        "127.0.0.1 sentry.netease.com",
        "127.0.0.1 report.mumu.nie.netease.com",
    ];
    for entry in &entries {
        ensure_host_entry(&hosts, entry)?;
    }
    flush_dns()?;
    if !check_hosts_blocked("report.mumu.nie.netease.com")? {
        return Err("No se pudo modificar hosts: el dominio todavía se resuelve con normalidad".to_string());
    }
    Ok("Dominios de envío de datos bloqueados".to_string())
}

#[tauri::command]
pub async fn unblock_report_domains() -> Result<String, String> {
    let hosts = get_hosts_path();
    let entries = [
        "127.0.0.1 sentry.netease.com",
        "127.0.0.1 report.mumu.nie.netease.com",
    ];
    for entry in &entries {
        remove_host_entry(&hosts, entry)?;
    }
    flush_dns()?;
    if check_hosts_blocked("report.mumu.nie.netease.com").unwrap_or(false) {
        Err("La resolución aún no se ha restaurado; comprueba la red o la configuración de hosts".to_string())
    } else {
        Ok("Dominios de envío de datos restaurados".to_string())
    }
}
