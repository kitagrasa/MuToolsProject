use serde::Serialize;
use std::path::PathBuf;

#[derive(Serialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct MuMuInfo {
    pub install_dir: Option<String>,
    pub mumu_manager_path: Option<String>,
    pub appdata_dir: Option<String>,
    pub registry_key: Option<String>,
    pub full_version: Option<String>,
    pub major_version: Option<String>,
    pub channel: Option<String>,
    pub uninstall_string: Option<String>,
    pub errors: Vec<String>,
}

/// Obtiene la ruta %APPDATA%
fn get_appdata_dir() -> Result<PathBuf, String> {
    std::env::var("APPDATA")
        .map(PathBuf::from)
        .map_err(|e| format!("No se pudo obtener la variable de entorno APPDATA: {}", e))
}

/// Intenta leer install_config.json para obtener la versión y el canal
/// Devuelve (version, channel, major_version, appdata_subdir)
fn read_install_config() -> Result<(String, String, String, String), String> {
    let appdata = get_appdata_dir()?;

    // Ruta V5
    let v5_config = appdata.join("Netease").join("MuMuPlayer").join("install_config.json");
    // Ruta V4
    let v4_config = appdata.join("Netease").join("MuMuPlayer-12.0").join("install_config.json");

    let mut errors = Vec::new();

    for (config_path, label) in &[(&v5_config, "V5"), (&v4_config, "V4")] {
        if config_path.exists() {
            match std::fs::read_to_string(config_path) {
                Ok(content) => {
                    match serde_json::from_str::<serde_json::Value>(&content) {
                        Ok(json) => {
                            let version = json
                                .get("product")
                                .and_then(|p| p.get("version"))
                                .and_then(|v| v.as_str())
                                .map(|s| s.to_string());

                            let channel = json
                                .get("product")
                                .and_then(|p| p.get("fchannel"))
                                .and_then(|c| c.as_str())
                                .map(|s| s.to_string());

                            if let Some(ref ver) = version {
                                let major = parse_major_version(ver);
                                return Ok((ver.clone(), channel.unwrap_or_default(), major, label.to_string()));
                            } else {
                                errors.push(format!("No se encontró el campo version en {} install_config.json", label));
                            }
                        }
                        Err(e) => {
                            errors.push(format!("Error al analizar el JSON de {} install_config.json: {}", label, e));
                        }
                    }
                }
                Err(e) => {
                    errors.push(format!("Error al leer {} install_config.json: {}", label, e));
                }
            }
        }
    }

    Err(format!(
        "No se pudo leer install_config.json. Se probaron V5({}) y V4({}). Errores: {}",
        v5_config.display(),
        v4_config.display(),
        errors.join("; ")
    ))
}

/// Analiza el número de versión principal
fn parse_major_version(version: &str) -> String {
    let first = version.chars().next().unwrap_or('0');
    if first == 'V' || first == 'v' {
        version.chars().nth(1).map(|c| c.to_string()).unwrap_or_else(|| "0".to_string())
    } else {
        first.to_string()
    }
}

/// Elimina las comillas de los extremos de la cadena
fn trim_quotes(s: &str) -> String {
    s.trim_matches('"').to_string()
}

/// Obtiene el directorio raíz de instalación a partir de UninstallString
/// Por ejemplo "D:\Program Files\Netease\MuMu\uninstall.exe" -> "D:\Program Files\Netease\MuMu"
fn parse_install_dir(uninstall_string: &str) -> Option<String> {
    let path = trim_quotes(uninstall_string);
    let path = PathBuf::from(&path);
    path.parent().map(|p| p.to_string_lossy().to_string())
}

/// Lee la información del registro
/// major_version_hint
/// Devuelve (install_dir, uninstall_string, display_version, registry_key_path)
fn read_registry(major_version_hint: &str) -> Result<(Option<String>, Option<String>, Option<String>, String), String> {
    use winreg::enums::*;
    use winreg::RegKey;

    // Ruta del registro para V5
    let v5_key = r"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\MuMuPlayer";
    // Ruta del registro para V4
    let v4_key = r"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\MuMuPlayer-12.0";

    let mut errors = Vec::new();
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);

    // El orden de prioridad depende del número de versión principal
    let try_keys: Vec<(&str, &str)> = if major_version_hint.starts_with('5') || major_version_hint.starts_with('6') {
        vec![("V5", v5_key), ("V4", v4_key)]
    } else if major_version_hint.starts_with('4') {
        vec![("V4", v4_key), ("V5", v5_key)]
    } else {
        // Versión desconocida, se da prioridad a V5
        vec![("V5", v5_key), ("V4", v4_key)]
    };

    for (label, subkey) in &try_keys {
        // Intenta la vista de 64 bits
        match hklm.open_subkey_with_flags(subkey, KEY_READ | KEY_WOW64_64KEY) {
            Ok(key) => {
                return Ok(read_registry_values(&key, subkey));
            }
            Err(e) => {
                errors.push(format!("Error al abrir el registro de {}(64 bits): {}", label, e));
            }
        }
        // Intenta la vista de 32 bits
        match hklm.open_subkey_with_flags(subkey, KEY_READ | KEY_WOW64_32KEY) {
            Ok(key) => {
                return Ok(read_registry_values(&key, subkey));
            }
            Err(e) => {
                errors.push(format!("Error al abrir el registro de {}(32 bits): {}", label, e));
            }
        }
    }

    Err(format!("No se pudo leer ninguna ruta del registro. Errores: {}", errors.join("; ")))
}

fn read_registry_values(key: &winreg::RegKey, key_path: &str) -> (Option<String>, Option<String>, Option<String>, String) {
    let get_value = |name: &str| -> Option<String> {
        key.get_value(name).ok().map(|s: String| trim_quotes(&s))
    };

    let uninstall_string = get_value("UninstallString");
    let display_version = get_value("DisplayVersion");

    let install_dir = uninstall_string.as_ref().and_then(|s| parse_install_dir(s));

    (
        install_dir,
        uninstall_string,
        display_version,
        format!("HKLM\\{}", key_path),
    )
}

/// Obtiene la ruta de MuMuManager.exe
fn get_mumu_manager_path(install_dir: &str, major_version: &str) -> Option<String> {
    let base = PathBuf::from(install_dir);

    // 5.x
    // 4.x
    let candidates: Vec<PathBuf> = if major_version.starts_with('5') || major_version.starts_with('6') {
        vec![base.join("nx_main").join("MuMuManager.exe")]
    } else {
        // 4.x o predeterminado
        vec![base.join("shell").join("MuMuManager.exe")]
    };

    // También se prueba la otra ruta como alternativa
    let mut all_candidates = candidates.clone();
    if major_version.starts_with('5') || major_version.starts_with('6') {
        all_candidates.push(base.join("shell").join("MuMuManager.exe"));
    } else {
        all_candidates.push(base.join("nx_main").join("MuMuManager.exe"));
    }

    for path in &all_candidates {
        if path.exists() {
            return Some(path.to_string_lossy().to_string());
        }
    }

    None
}

#[tauri::command]
pub fn get_mumu_info() -> Result<MuMuInfo, String> {
    let mut info = MuMuInfo::default();
    let mut errors: Vec<String> = Vec::new();

    // Step
    let _appdata_dir = match get_appdata_dir() {
        Ok(dir) => {
            let dir_str = dir.to_string_lossy().to_string();
            info.appdata_dir = Some(dir_str);
            Some(dir)
        }
        Err(e) => {
            errors.push(e);
            info.errors.push(format!("Error al obtener APPDATA: {}", errors.last().unwrap()));
            None
        }
    };

    // Step
    let mut major_version = String::new();
    match read_install_config() {
        Ok((version, channel, major, label)) => {
            info.full_version = Some(version);
            info.channel = Some(channel);
            info.major_version = Some(major.clone());
            major_version = major;
            log::info!("Versión leída de {} install_config.json: {}, canal: {}", label, info.full_version.as_deref().unwrap_or("?"), info.channel.as_deref().unwrap_or("?"));
        }
        Err(e) => {
            errors.push(e.clone());
            info.errors.push(format!("Error al leer install_config.json: {}", e));
            log::warn!("Error al leer install_config.json: {}", e);
        }
    }

    // Step
    match read_registry(&major_version) {
        Ok((install_dir, uninstall_string, display_version, registry_path)) => {
            info.install_dir = install_dir.clone();
            info.uninstall_string = uninstall_string;
            info.registry_key = Some(registry_path.clone());

            // Si no se obtuvo la versión de install_config.json, se usa DisplayVersion del registro
            if info.full_version.is_none() {
                info.full_version = display_version.clone();
            }
            if display_version.is_some() && info.full_version.is_none() {
                info.full_version = display_version;
            }
            if let Some(ref ver) = info.full_version {
                if info.major_version.is_none() {
                    info.major_version = Some(parse_major_version(ver));
                    major_version = info.major_version.clone().unwrap_or_default();
                }
            }

            log::info!("Leído de la ruta del registro ({}): directorio de instalación={:?}", registry_path, info.install_dir);

            // Step
            if let Some(ref install_dir) = info.install_dir {
                let mumu_path = get_mumu_manager_path(install_dir, &major_version);
                if mumu_path.is_some() {
                    info.mumu_manager_path = mumu_path;
                } else {
                    let msg = "Comprueba que la versión sea 4.0.0.3179 o superior".to_string();
                    info.errors.push(msg.clone());
                    log::warn!("{}", msg);
                }
            }
        }
        Err(e) => {
            errors.push(e.clone());
            info.errors.push(format!("Error al leer el registro: {}", e));
            log::warn!("Error al leer el registro: {}", e);
        }
    }

    // Si el registro no devolvió el directorio de instalación, pero hay número de versión principal, se prueba la ruta predeterminada
    if info.install_dir.is_none() && !major_version.is_empty() {
        let default_dir = if major_version.starts_with('5') || major_version.starts_with('6') {
            r"D:\Program Files\Netease\MuMu"
        } else {
            r"D:\Program Files\Netease\MuMu"
        };
        let default_path = PathBuf::from(default_dir);
        if default_path.exists() {
            info.install_dir = Some(default_dir.to_string());
            let mumu_path = get_mumu_manager_path(default_dir, &major_version);
            info.mumu_manager_path = mumu_path;
            log::info!("Se usa el directorio de instalación predeterminado: {}", default_dir);
        }
    }

    Ok(info)
}