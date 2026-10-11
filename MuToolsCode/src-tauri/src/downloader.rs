use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{Seek, Write};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use tauri::Emitter;

use crate::config::AppState;

#[derive(Deserialize)]
pub struct DownloadResponse {
    pub errcode: i32,
    pub errmsg: String,
    pub data: Option<DownloadData>,
    #[allow(dead_code)]
    pub sign: Option<String>,
}

#[derive(Deserialize)]
pub struct DownloadData {
    #[serde(rename = "mumu")]
    pub mumu: Option<DownloadLink>,
    #[allow(dead_code)]
    pub engine: Option<String>,
    #[allow(dead_code)]
    pub version: Option<String>,
    #[allow(dead_code)]
    pub architecture: Option<String>,
}

#[derive(Deserialize)]
#[allow(dead_code)]
pub struct DownloadLink {
    pub link: String,
    pub checksum: String,
}

// Estructura de respuesta de la API V6

#[derive(Deserialize, Serialize, Clone)]
pub struct V6DownloadResponse {
    pub errcode: i32,
    pub errmsg: String,
    pub data: Option<V6DownloadData>,
    #[allow(dead_code)]
    pub sign: Option<String>,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct V6DownloadData {
    #[allow(dead_code)]
    pub engine: Option<String>,
    pub version: Option<String>,
    #[allow(dead_code)]
    pub architecture: Option<String>,
    pub components: Option<Vec<V6Component>>,
    pub default_download_engine: Option<String>,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct V6Component {
    pub name: String,
    pub version: String,
    pub checksum: String,
    pub link: String,
    pub size: i64,
}

// Indicadores globales de control de descarga

static DOWNLOAD_CANCELLED: AtomicBool = AtomicBool::new(false);
static DOWNLOAD_PAUSED: AtomicBool = AtomicBool::new(false);
static ARIA2_PID: Mutex<Option<u32>> = Mutex::new(None);
static PARTIAL_FILES: Mutex<Vec<PathBuf>> = Mutex::new(Vec::new());

#[tauri::command]
pub fn cancel_download() -> Result<(), String> {
    DOWNLOAD_CANCELLED.store(true, Ordering::SeqCst);
    DOWNLOAD_PAUSED.store(false, Ordering::SeqCst);
    if let Some(pid) = *ARIA2_PID.lock().unwrap() {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        let _ = std::process::Command::new("taskkill")
            .args(["/F", "/PID", &pid.to_string()])
            .creation_flags(CREATE_NO_WINDOW)
            .output();
    }
    // Esperar a que los procesos aria2/reqwest terminen por completo y liberen los descriptores de archivo
    std::thread::sleep(std::time::Duration::from_millis(200));
    // Eliminar todos los archivos parciales ya descargados
    let files = PARTIAL_FILES.lock().unwrap().clone();
    for f in &files {
        if f.exists() {
            // Reintentar
            let mut removed = false;
            for _attempt in 0..3 {
                match fs::remove_file(f) {
                    Ok(()) => {
                        log::info!("Se elimino el archivo de la descarga cancelada: {}", f.display());
                        removed = true;
                        break;
                    }
                    Err(e) => {
                        log::warn!("No se pudo eliminar el archivo (se reintentara): {} - {}", f.display(), e);
                        std::thread::sleep(std::time::Duration::from_millis(200));
                    }
                }
            }
            if !removed {
                // Último intento
                log::error!("No se puede eliminar el archivo: {}", f.display());
            }
        }
    }
    PARTIAL_FILES.lock().unwrap().clear();
    log::info!("Descarga cancelada");
    Ok(())
}

#[tauri::command]
pub fn pause_download() -> Result<(), String> {
    DOWNLOAD_PAUSED.store(true, Ordering::SeqCst);
    if let Some(pid) = *ARIA2_PID.lock().unwrap() {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        let _ = std::process::Command::new("taskkill")
            .args(["/F", "/PID", &pid.to_string()])
            .creation_flags(CREATE_NO_WINDOW)
            .output();
    }
    log::info!("Descarga pausada");
    Ok(())
}

#[tauri::command]
pub fn resume_download() -> Result<(), String> {
    DOWNLOAD_PAUSED.store(false, Ordering::SeqCst);
    log::info!("Descarga reanudada");
    Ok(())
}

// Obtencion de enlaces de descarga por version

#[tauri::command]
pub async fn fetch_download_url(version: String) -> Result<String, String> {
    let link = match version.as_str() {
        "MuMu12-V4" => fetch_download_url_v4_inner().await?,
        "MuMu12-V5" => fetch_download_url_v5_inner().await?,
        "MuMu12-V6" => fetch_download_url_v6_inner().await?,
        _ => return Err("Solo se admite la obtencion de enlaces para las versiones MuMu-V4V5V6".to_string()),
    };
    log::info!("Enlace de descarga de {} obtenido correctamente: {}", version, link);
    Ok(link)
}

#[tauri::command]
pub async fn browse_directory(app: tauri::AppHandle) -> Result<Option<String>, String> {
    use std::sync::mpsc;
    use tauri_plugin_dialog::DialogExt;
    let (tx, rx) = mpsc::channel();
    app.dialog().file().pick_folder(move |d| {
        let _ = tx.send(d.and_then(|d| d.as_path().map(|p| p.to_string_lossy().to_string())));
    });
    let result = rx.recv().map_err(|e| format!("Error del cuadro de dialogo: {}", e))?;
    Ok(result)
}

#[tauri::command]
pub async fn browse_file(app: tauri::AppHandle) -> Result<Option<String>, String> {
    use std::sync::mpsc;
    use tauri_plugin_dialog::DialogExt;
    let (tx, rx) = mpsc::channel();
    app.dialog()
        .file()
        .add_filter("Paquete de instalación", &["exe", "7z", "zip", "rar", "mumudata"])
        .pick_file(move |d| {
            let _ = tx.send(d.and_then(|d| d.as_path().map(|p| p.to_string_lossy().to_string())));
        });
    let result = rx.recv().map_err(|e| format!("Error del cuadro de dialogo: {}", e))?;
    Ok(result)
}

#[tauri::command]
pub fn start_install(
    version: String,
    method: String,
    path: String,
    install_dir: String,
) -> Result<String, String> {
    let method_str = if method == "local" {
        "Instalación local"
    } else {
        "Instalación en línea"
    };
    let msg = format!(
        "La tarea de instalación de {} {} ha comenzado\nRuta del paquete de instalación: {}\nCarpeta de instalación: {}",
        version, method_str, path, install_dir
    );
    log::info!("{}", msg);
    Ok(msg)
}

/// Busca la ruta del ejecutable aria2c
fn find_aria2(resource_dir: &PathBuf) -> Option<PathBuf> {
    let mut candidates = vec![resource_dir.join("aria2")];

    if let Ok(entries) = fs::read_dir(resource_dir) {
        candidates.extend(entries.flatten().map(|entry| entry.path()).filter(|path| {
            path.is_dir()
                && path
                    .file_name()
                    .map(|name| name.to_string_lossy().to_lowercase().contains("aria2"))
                    == Some(true)
        }));
    }

    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            candidates.push(exe_dir.join("aria2"));
            candidates.push(exe_dir.join("testdata").join("aria2"));
        }
    }

    candidates.push(PathBuf::from("testdata").join("aria2"));
    candidates.push(PathBuf::from("aria2"));

    candidates
        .into_iter()
        .flat_map(|dir| {
            [
                dir.join("aria2c.exe"),
                dir.join("aria2c"),
                dir.join("aria2.exe"),
            ]
        })
        .find(|path| path.is_file())
}

/// Envia un evento de progreso de descarga
fn emit_progress(
    app: &tauri::AppHandle,
    status: &str,
    action: &str,
    percent: u32,
    downloaded: u64,
    total: u64,
) {
    let _ = app.emit(
        "download-progress",
        serde_json::json!({
            "status": status,
            "action": action,
            "percent": percent,
            "downloaded": downloaded,
            "total": total
        }),
    );
}

/// Devuelve la ruta completa del archivo descargado
async fn download_file(
    app: &tauri::AppHandle,
    state: &AppState,
    url: &str,
    dir: &PathBuf,
    filename: &str,
    label: Option<&str>,
    known_size: u64,
    progress_offset: f64,
    progress_weight: f64,
    max_connections: u32,
    split: u32,
) -> Result<PathBuf, String> {
    // Comprobar si ya se ha cancelado
    if DOWNLOAD_CANCELLED.load(Ordering::SeqCst) {
        return Err("Descarga cancelada".to_string());
    }
    // Comprobar si esta en pausa
    if DOWNLOAD_PAUSED.load(Ordering::SeqCst) {
        return Err("Descarga pausada".to_string());
    }

    let installer_path = dir.join(filename);
    let display_label = label.unwrap_or(filename);

    // Comprobar el tamaño del archivo existente (para conservar el progreso al reanudar)
    let existing_size = if installer_path.exists() {
        fs::metadata(&installer_path).map(|m| m.len()).unwrap_or(0)
    } else {
        0
    };

    // Obtener el tamaño total del archivo
    let total_size = if known_size > 0 {
        known_size
    } else {
        let client = reqwest::Client::new();
        // Se usa una petición Range en lugar de HEAD porque algunas CDN no admiten HEAD o
        // HEAD no devuelve Content-Length; en la respuesta 206 de una petición Range
        // la cabecera Content-Range incluye el tamaño total del archivo
        match client.get(url).header("Range", "bytes=0-0").send().await {
            Ok(resp) => {
                if resp.status() == reqwest::StatusCode::PARTIAL_CONTENT {
                    resp.headers()
                        .get("content-range")
                        .and_then(|v| v.to_str().ok())
                        .and_then(|v| v.split('/').last())
                        .and_then(|v| v.parse::<u64>().ok())
                        .unwrap_or_else(|| resp.content_length().unwrap_or(0))
                } else {
                    // El servidor no admite Range, se recurre a Content-Length
                    resp.content_length().unwrap_or(0)
                }
            }
            Err(_) => 0,
        }
    };

    log::info!(
        "Iniciando descarga: {} (url={}, total_size={} bytes, existing={} bytes)",
        display_label,
        url,
        total_size,
        existing_size
    );

    // Al reanudar la descarga se calcula el progreso inicial a partir del tamaño del archivo existente para evitar que la barra vuelva a cero
    let initial_pct = if total_size > 0 && existing_size > 0 {
        ((progress_offset + progress_weight * (existing_size as f64 / total_size as f64)) * 100.0)
            .min(100.0) as u32
    } else {
        0u32
    };
    emit_progress(
        app,
        "downloading",
        &format!("Descargando {}...", display_label),
        initial_pct,
        existing_size,
        total_size,
    );

    // Se da prioridad a la descarga con aria2
    let aria2 = find_aria2(&state.resource_dir);
    let download_result: Result<(), String> = if let Some(aria2_path) = aria2 {
        log::info!(
            "Descargando con aria2: {} (max_connections={}, split={})",
            aria2_path.display(),
            max_connections,
            split
        );
        let ok = download_via_aria2(
            app,
            &aria2_path,
            url,
            dir,
            filename,
            &installer_path,
            total_size,
            display_label,
            progress_offset,
            progress_weight,
            max_connections,
            split,
        )
        .await;
        if ok {
            Ok(())
        } else if DOWNLOAD_CANCELLED.load(Ordering::SeqCst) {
            // Cancelada
            log::info!("Descarga cancelada, no se recurre al descargador integrado: {}", display_label);
            Err("Descarga cancelada".to_string())
        } else if DOWNLOAD_PAUSED.load(Ordering::SeqCst) {
            // Pausada
            log::info!("Descarga pausada, se conserva la parte ya descargada: {}", display_label);
            Err("Descarga pausada".to_string())
        } else {
            // Fallo propio de aria2 (ni pausa ni cancelación)
            log::info!("aria2 ha fallado, se recurre al descargador integrado: {}", display_label);
            download_via_reqwest(
                app,
                url,
                &installer_path,
                total_size,
                display_label,
                progress_offset,
                progress_weight,
            )
            .await
        }
    } else {
        log::info!("No se ha encontrado aria2, se usa el descargador integrado");
        download_via_reqwest(
            app,
            url,
            &installer_path,
            total_size,
            display_label,
            progress_offset,
            progress_weight,
        )
        .await
    };

    download_result?;

    let final_size = fs::metadata(&installer_path).map(|m| m.len()).unwrap_or(0);
    emit_progress(
        app,
        "downloading",
        &format!("{} descarga completada", display_label),
        100,
        final_size,
        final_size,
    );
    log::info!(
        "Descarga completada: {} ({} bytes)",
        installer_path.display(),
        final_size
    );

    Ok(installer_path)
}

/// Descarga un archivo con aria2 y devuelve si la operacion ha tenido exito
async fn download_via_aria2(
    app: &tauri::AppHandle,
    aria2_path: &PathBuf,
    url: &str,
    dir: &PathBuf,
    filename: &str,
    installer_path: &PathBuf,
    total_size: u64,
    label: &str,
    progress_offset: f64,
    progress_weight: f64,
    max_connections: u32,
    split: u32,
) -> bool {
    // Iniciar el proceso aria2
    // --file-allocation=none
    // --continue=true
    // --max-connection-per-server=N
    // --split=N
    // --console-log-level=error
    let mut child = {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        std::process::Command::new(aria2_path)
            .args([
                "--allow-overwrite=true",
                "--file-allocation=none",
                "--continue=true",
                "--max-connection-per-server",
                &max_connections.to_string(),
                "--split",
                &split.to_string(),
                "--console-log-level=error",
                "--dir",
                &dir.to_string_lossy(),
                "--out",
                filename,
                url,
            ])
            .creation_flags(CREATE_NO_WINDOW)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
    };

    // Guardar el PID del proceso aria2 para poder matarlo al pausar/cancelar
    if let Ok(ref child) = child {
        *ARIA2_PID.lock().unwrap() = Some(child.id());
    }

    let mut success = false;
    if let Ok(ref mut child) = child {
        // Iniciar el hilo de supervision del progreso
        let app_clone = app.clone();
        let monitor_path = installer_path.clone();
        let label_owned = label.to_string();
        let running = Arc::new(AtomicBool::new(true));
        let running_clone = running.clone();
        let monitor_handle = std::thread::spawn(move || {
            let mut last_size = 0u64;
            while running_clone.load(Ordering::Relaxed) {
                std::thread::sleep(std::time::Duration::from_millis(300));

                if DOWNLOAD_CANCELLED.load(Ordering::Relaxed)
                    || DOWNLOAD_PAUSED.load(Ordering::Relaxed)
                {
                    break;
                }

                if let Ok(metadata) = fs::metadata(&monitor_path) {
                    let current_size = metadata.len();
                    if current_size != last_size {
                        // Calcular el porcentaje de progreso dentro del componente
                        let inner_pct = if total_size > 0 {
                            ((current_size as f64 / total_size as f64) * 100.0).min(100.0)
                        } else {
                            0.0
                        };
                        // Calcular el progreso total
                        let total_pct = ((progress_offset + progress_weight * inner_pct / 100.0)
                            * 100.0)
                            .min(100.0) as u32;
                        emit_progress(
                            &app_clone,
                            "downloading",
                            &format!(
                                "Descargando {} ({:.2} MB / {:.2} MB)",
                                label_owned,
                                current_size as f64 / 1024.0 / 1024.0,
                                total_size as f64 / 1024.0 / 1024.0
                            ),
                            total_pct,
                            current_size,
                            total_size,
                        );
                        last_size = current_size;
                    }
                }
            }
        });

        // Esperar a que aria2 termine
        let exit_status = child.wait();
        running.store(false, Ordering::Relaxed);
        let _ = monitor_handle.join();
        *ARIA2_PID.lock().unwrap() = None;

        if DOWNLOAD_CANCELLED.load(Ordering::SeqCst) {
            return false;
        }
        if DOWNLOAD_PAUSED.load(Ordering::SeqCst) {
            return false;
        }

        match exit_status {
            Ok(status) if status.success() && installer_path.exists() => {
                success = true;
            }
            Ok(status) => {
                log::warn!(
                    "La descarga con aria2 de {} ha fallado (estado: {}), se recurre a la descarga integrada",
                    label,
                    status
                );
            }
            Err(e) => {
                log::warn!("Fallo al esperar a aria2 para {}: {}, se recurre a la descarga integrada", label, e);
            }
        }
    }

    success
}

/// Descarga un archivo con el reqwest integrado
async fn download_via_reqwest(
    app: &tauri::AppHandle,
    url: &str,
    installer_path: &PathBuf,
    total_size: u64,
    label: &str,
    progress_offset: f64,
    progress_weight: f64,
) -> Result<(), String> {
    let client = reqwest::Client::new();

    // Comprobar si ya existe un archivo parcial descargado para admitir la reanudación
    let existing_size = if installer_path.exists() {
        fs::metadata(installer_path).map(|m| m.len()).unwrap_or(0)
    } else {
        0
    };

    let total_size = if total_size > 0 { total_size } else { 0 };

    // Si el archivo ya existe y su tamaño coincide con total_size, se considera descargado por completo
    if existing_size > 0 && total_size > 0 && existing_size >= total_size {
        log::info!(
            "El archivo ya existe y su tamaño coincide, se omite la descarga: {} ({} bytes)",
            label,
            existing_size
        );
        return Ok(());
    }

    let mut request = client.get(url);
    let mut downloaded: u64 = 0;

    if existing_size > 0 {
        // Reanudación desde el punto de interrupcion
        log::info!(
            "Reanudación: {} ya tiene {} bytes, se continua la descarga desde esa posición",
            label,
            existing_size
        );
        request = request.header("Range", format!("bytes={}-", existing_size));
        downloaded = existing_size;
    }

    let resp = request
        .send()
        .await
        .map_err(|e| format!("Error en la petición de descarga: {}", e))?;

    // Comprobar si el servidor admite la reanudación
    let is_partial = resp.status() == reqwest::StatusCode::PARTIAL_CONTENT;
    if existing_size > 0 && !is_partial {
        // El servidor no admite Range, se descarga desde el principio
        log::warn!("El servidor no admite la reanudación, se descargara desde el principio: {}", label);
        let resp = client
            .get(url)
            .send()
            .await
            .map_err(|e| format!("Error en la petición de descarga: {}", e))?;
        if !resp.status().is_success() {
            return Err(format!("Fallo en la descarga: HTTP {}", resp.status()));
        }
        let final_total = if total_size > 0 {
            total_size
        } else {
            resp.content_length().unwrap_or(0)
        };
        return download_via_reqwest_stream(
            app,
            resp,
            installer_path,
            final_total,
            label,
            progress_offset,
            progress_weight,
            0,
        )
        .await;
    }

    if !resp.status().is_success() && !is_partial {
        return Err(format!("Fallo en la descarga: HTTP {}", resp.status()));
    }

    let effective_total = if total_size > 0 {
        total_size
    } else if is_partial {
        // Obtener el tamaño total a partir de la cabecera Content-Range
        resp.headers()
            .get("content-range")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.split('/').last())
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(resp.content_length().unwrap_or(0))
    } else {
        resp.content_length().unwrap_or(0)
    };

    download_via_reqwest_stream(
        app,
        resp,
        installer_path,
        effective_total,
        label,
        progress_offset,
        progress_weight,
        downloaded,
    )
    .await
}

/// Cuerpo de ejecución de la descarga en flujo con reqwest
async fn download_via_reqwest_stream(
    app: &tauri::AppHandle,
    resp: reqwest::Response,
    installer_path: &PathBuf,
    total_size: u64,
    label: &str,
    progress_offset: f64,
    progress_weight: f64,
    initial_downloaded: u64,
) -> Result<(), String> {
    // Calcular el progreso inicial a partir del tamaño ya descargado para evitar que la barra vuelva a cero
    let init_pct = if total_size > 0 {
        ((progress_offset + progress_weight * (initial_downloaded as f64 / total_size as f64))
            * 100.0)
            .min(100.0) as u32
    } else {
        0u32
    };
    emit_progress(
        app,
        "downloading",
        &format!("Descargando {}...", label),
        init_pct,
        initial_downloaded,
        total_size,
    );

    // Abrir el archivo en modo de adicion (admite la reanudación)
    // Usar el modo de escritura con truncate=false
    // Evitar recrear un archivo ya eliminado al volver a descargar tras cancelar (si initial_downloaded=0 después de cancelar se crea un archivo vacio nuevo)
    let mut out = {
        let mut opts = fs::OpenOptions::new();
        opts.create(true).write(true);
        if initial_downloaded > 0 {
            opts.append(true);
        } else {
            opts.truncate(true);
        }
        opts.open(installer_path)
            .map_err(|e| format!("No se pudo crear el archivo: {}", e))?
    };

    // Asegurar que el puntero del archivo este al final
    if initial_downloaded > 0 {
        out.seek(std::io::SeekFrom::End(0))
            .map_err(|e| format!("No se pudo posicionar el archivo: {}", e))?;
    }

    let mut downloaded = initial_downloaded;
    let mut stream = resp.bytes_stream();
    let mut last_emit = std::time::Instant::now();

    while let Some(chunk) = stream.next().await {
        // Comprobar si se ha cancelado o pausado
        if DOWNLOAD_CANCELLED.load(Ordering::Relaxed) {
            return Err("Descarga cancelada".to_string());
        }
        if DOWNLOAD_PAUSED.load(Ordering::Relaxed) {
            return Err("Descarga pausada".to_string());
        }

        let chunk = chunk.map_err(|e| format!("Fallo al descargar los datos: {}", e))?;
        out.write_all(&chunk)
            .map_err(|e| format!("Fallo al escribir en el archivo: {}", e))?;
        downloaded += chunk.len() as u64;

        // Limitar la frecuencia de los eventos de progreso
        if last_emit.elapsed() >= std::time::Duration::from_millis(200) {
            let inner_pct = if total_size > 0 {
                ((downloaded as f64 / total_size as f64) * 100.0).min(100.0)
            } else {
                0.0
            };
            let total_pct =
                ((progress_offset + progress_weight * inner_pct / 100.0) * 100.0).min(100.0) as u32;
            emit_progress(
                app,
                "downloading",
                &format!(
                    "Descargando {} ({:.2} MB / {:.2} MB)",
                    label,
                    downloaded as f64 / 1024.0 / 1024.0,
                    total_size as f64 / 1024.0 / 1024.0
                ),
                total_pct,
                downloaded,
                total_size,
            );
            last_emit = std::time::Instant::now();
        }
    }

    Ok(())
}

/// Descarga el paquete de instalación en la carpeta indicada
#[tauri::command]
pub async fn download_v4_installer(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
    save_dir: String,
    url: String,
    version: String,
    resume: bool,
) -> Result<String, String> {
    // Restablecer los indicadores de control de descarga
    DOWNLOAD_CANCELLED.store(false, Ordering::SeqCst);
    if !resume {
        DOWNLOAD_PAUSED.store(false, Ordering::SeqCst);
    }

    let url = if url.is_empty() {
        return Err("El enlace de descarga del paquete de instalación no puede estar vacio".to_string());
    } else {
        url
    };
    log::info!("Enlace de descarga: {}", url);

    let dir = PathBuf::from(&save_dir);
    fs::create_dir_all(&dir).map_err(|e| format!("No se puede crear la carpeta de descarga: {}", e))?;

    let config = crate::config::read_config();

    // Usar un nombre de archivo especifico por version para evitar que V4 y V5 se sobrescriban entre si y la reanudación se evalue de forma erronea
    let filename = match version.as_str() {
        "MuMu12-V5" => "MuMu_Setup_V5.exe",
        _ => "MuMu_Setup_V4.exe",
    };

    let save_path = dir.join(filename);

    // Si no es una reanudación, eliminar el archivo en conflicto para garantizar que la descarga empiece de cero
    if !resume && save_path.exists() {
        let _ = fs::remove_file(&save_path);
        log::info!("Archivo en conflicto eliminado: {}", save_path.display());
    }

    // Registrar el archivo parcial para poder eliminarlo al cancelar
    PARTIAL_FILES.lock().unwrap().push(save_path.clone());

    let installer_path = download_file(
        &app,
        &state,
        &url,
        &dir,
        filename,
        None,
        0,
        0.0,
        1.0,
        config.aria2_max_connections,
        config.aria2_split,
    )
    .await;

    // Quitar el archivo de la lista de seguimiento una vez completada la descarga
    PARTIAL_FILES.lock().unwrap().retain(|f| f != &save_path);

    match installer_path {
        Ok(path) => Ok(path.to_string_lossy().to_string()),
        Err(e) => {
            // Al cancelar la descarga, eliminar el archivo incompleto ya descargado
            if DOWNLOAD_CANCELLED.load(Ordering::SeqCst) && save_path.exists() {
                match fs::remove_file(&save_path) {
                    Ok(()) => log::info!("Descarga cancelada, archivo incompleto eliminado: {}", save_path.display()),
                    Err(err) => {
                        log::warn!("Descarga cancelada, no se pudo eliminar el archivo: {} - {}", save_path.display(), err)
                    }
                }
            }
            Err(e)
        }
    }
}

/// V6: descarga todos los paquetes de instalación de los componentes en la carpeta indicada
#[tauri::command]
pub async fn download_v6_installer(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
    save_dir: String,
    components_json: String,
    resume: bool,
) -> Result<String, String> {
    // Restablecer los indicadores de control de descarga
    DOWNLOAD_CANCELLED.store(false, Ordering::SeqCst);
    if !resume {
        DOWNLOAD_PAUSED.store(false, Ordering::SeqCst);
    }

    let components: Vec<V6Component> =
        serde_json::from_str(&components_json).map_err(|e| format!("No se pudo analizar la lista de componentes: {}", e))?;

    let dir = PathBuf::from(&save_dir);
    fs::create_dir_all(&dir).map_err(|e| format!("No se puede crear la carpeta de descarga: {}", e))?;

    let config = crate::config::read_config();

    let total_components = components.len();
    let mut downloaded_files = Vec::new();

    for (i, comp) in components.iter().enumerate() {
        // Comprobar si se ha cancelado
        if DOWNLOAD_CANCELLED.load(Ordering::SeqCst) {
            return Err("Descarga cancelada".to_string());
        }

        let url = &comp.link;
        let filename = url
            .split('?')
            .next()
            .and_then(|v| v.rsplit('/').next())
            .unwrap_or(&comp.name)
            .to_string();
        let installer_path = dir.join(&filename);

        log::info!(
            "Descargando componente V6 [{}/{}]: {} -> {}",
            i + 1,
            total_components,
            comp.name,
            filename
        );

        // Si no es una reanudación, también hay que eliminar los archivos que ya existan
        if !resume && installer_path.exists() {
            let _ = fs::remove_file(&installer_path);
            log::info!("Archivo en conflicto eliminado: {}", installer_path.display());
        }

        // Al reanudar, se omite la descarga de los archivos que ya existan
        if resume && installer_path.exists() {
            let existing_size = fs::metadata(&installer_path).map(|m| m.len()).unwrap_or(0);
            if existing_size > 0 && comp.size > 0 && existing_size >= comp.size as u64 {
                log::info!(
                    "El componente {} ya existe y su tamaño coincide, se omite la descarga: {}",
                    comp.name,
                    installer_path.display()
                );
                downloaded_files.push(installer_path.to_string_lossy().to_string());
                continue;
            }
            // El tamaño no coincide, se continua la descarga (reanudación)
            log::info!(
                "El componente {} esta descargado en parte, se reanuda la descarga: {} ({} bytes existentes)",
                comp.name,
                installer_path.display(),
                existing_size
            );
        }

        // Registrar el archivo parcial para poder eliminarlo al cancelar
        PARTIAL_FILES.lock().unwrap().push(installer_path.clone());

        // Cada componente representa 1/total_components del progreso total
        let weight = 1.0 / total_components as f64;
        let offset = i as f64 / total_components as f64;

        let path = download_file(
            &app,
            &state,
            url,
            &dir,
            &filename,
            Some(&comp.name),
            comp.size as u64,
            offset,
            weight,
            config.aria2_max_connections,
            config.aria2_split,
        )
        .await;

        match path {
            Ok(p) => {
                // Quitar el archivo de la lista de seguimiento una vez completada la descarga
                PARTIAL_FILES
                    .lock()
                    .unwrap()
                    .retain(|f| f != &installer_path);
                downloaded_files.push(p.to_string_lossy().to_string());
                log::info!("Descarga del componente {} completada: {}", comp.name, p.display());
            }
            Err(e) => {
                // Al cancelar la descarga, eliminar todos los archivos incompletos ya descargados
                if DOWNLOAD_CANCELLED.load(Ordering::SeqCst) {
                    let files = PARTIAL_FILES.lock().unwrap().clone();
                    for f in &files {
                        if f.exists() {
                            match fs::remove_file(f) {
                                Ok(()) => log::info!("Descarga cancelada, archivo incompleto eliminado: {}", f.display()),
                                Err(err) => {
                                    log::warn!("Descarga cancelada, no se pudo eliminar el archivo: {} - {}", f.display(), err)
                                }
                            }
                        }
                    }
                    PARTIAL_FILES.lock().unwrap().clear();
                }
                return Err(e);
            }
        }
    }

    emit_progress(&app, "downloading", "Descarga de todos los componentes completada", 100, 0, 0);

    Ok(serde_json::to_string(&downloaded_files).unwrap_or_default())
}

/// V6: instalación silenciosa
#[tauri::command]
pub async fn install_v6_setup(
    app: tauri::AppHandle,
    components_json: String,
    downloaded_files_json: String,
    install_dir: String,
    product_version: String,
) -> Result<String, String> {
    let components: Vec<V6Component> =
        serde_json::from_str(&components_json).map_err(|e| format!("No se pudo analizar la lista de componentes: {}", e))?;
    let downloaded_files: Vec<String> = serde_json::from_str(&downloaded_files_json)
        .map_err(|e| format!("No se pudo analizar la lista de archivos: {}", e))?;

    fs::create_dir_all(&install_dir).map_err(|e| format!("No se puede crear la carpeta de instalación: {}", e))?;

    // Generar un txn_id unico
    let txn_id = uuid::Uuid::new_v4().to_string();

    log::info!(
        "Inicio de la instalación V6: txn_id={}, product_version={}, install_dir={}",
        txn_id,
        product_version,
        install_dir
    );
    log::info!(
        "V6 número de componentes: {}, número de archivos: {}",
        components.len(),
        downloaded_files.len()
    );

    // Asociar el nombre de cada componente con la ruta del archivo descargado
    let mut component_file_map: Vec<(&V6Component, &String)> = Vec::new();
    for comp in &components {
        let comp_name_lower = comp.name.to_lowercase();
        let matched = downloaded_files.iter().find(|f| {
            let f_lower = f.to_lowercase();
            // Buscar coincidencias con el nombre del componente (en minusculas) dentro del nombre del archivo
            f_lower.contains(&comp_name_lower) || f_lower.contains(&format!("-{}", comp_name_lower))
        });
        if let Some(file_path) = matched {
            component_file_map.push((comp, file_path));
        } else {
            log::warn!("No se ha encontrado el archivo de instalación del componente {}, se omite", comp.name);
        }
    }

    if component_file_map.is_empty() {
        return Err("No se ha encontrado ningun archivo de instalación de componente disponible".to_string());
    }

    let total_steps = component_file_map.len() * 2; // install_prepare + commit, una vez cada uno
    let mut current_step = 0u32;

    // install_prepare
    for (comp, file_path) in &component_file_map {
        let install_dir_arg = install_dir.clone();
        let product_version_arg = product_version.clone();
        let txn_id_arg = txn_id.clone();

        current_step += 1;
        let action_msg = format!(
            "Instalando componente {}/{}: {} (install_prepare)",
            current_step, total_steps, comp.name
        );
        log::info!("{}", action_msg);
        log::info!("Línea de comandos: \"{}\" /from_orchestrator=1 /action=install_prepare /txn_id={} /silent=1 /product_version={} /auto_start=false /fchannel=nochannel-mumu12 /auto_run=false /D={}", file_path, txn_id_arg, product_version_arg, install_dir_arg);

        let _ = app.emit(
            "download-progress",
            serde_json::json!({
                "status": "installing",
                "action": action_msg,
                "percent": ((current_step as f64 / total_steps as f64) * 100.0) as u32,
                "downloaded": 0,
                "total": 0
            }),
        );

        let output = {
            use std::os::windows::process::CommandExt;
            const CREATE_NO_WINDOW: u32 = 0x08000000;
            std::process::Command::new(file_path)
                .arg("/from_orchestrator=1")
                .arg("/action=install_prepare")
                .arg(format!("/txn_id={}", txn_id_arg))
                .arg("/silent=1")
                .arg(format!("/product_version={}", product_version_arg))
                .arg("/auto_start=false")
                .arg("/fchannel=nochannel-mumu12")
                .arg("/auto_run=false")
                .raw_arg(format!("/D={}", install_dir_arg))
                .creation_flags(CREATE_NO_WINDOW)
                .output()
                .map_err(|e| format!("No se pudo ejecutar el programa de instalación: {}", e))?
        };

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        if !output.status.success() {
            let exit_code = output.status.code().unwrap_or(-1);
            log::error!(
                "Fallo de install_prepare del componente {} (exit: {}): stdout={}, stderr={}",
                comp.name,
                exit_code,
                stdout,
                stderr
            );
            return Err(format!(
                "Fallo en la instalación del componente {} (exit: {})\n{}\n{}",
                comp.name, exit_code, stdout, stderr
            ));
        }
        log::info!("install_prepare del componente {} correcto", comp.name);
    }

    // commit
    for (comp, file_path) in component_file_map.iter().rev() {
        let install_dir_arg = install_dir.clone();
        let product_version_arg = product_version.clone();
        let txn_id_arg = txn_id.clone();

        current_step += 1;
        let action_msg = format!(
            "Instalando componente {}/{}: {} (commit)",
            current_step, total_steps, comp.name
        );
        log::info!("{}", action_msg);
        log::info!("Línea de comandos: \"{}\" /from_orchestrator=1 /action=commit /txn_id={} /silent=1 /product_version={} /auto_start=false /fchannel=nochannel-mumu12 /auto_run=false /D={}", file_path, txn_id_arg, product_version_arg, install_dir_arg);

        let _ = app.emit(
            "download-progress",
            serde_json::json!({
                "status": "installing",
                "action": action_msg,
                "percent": ((current_step as f64 / total_steps as f64) * 100.0) as u32,
                "downloaded": 0,
                "total": 0
            }),
        );

        let output = {
            use std::os::windows::process::CommandExt;
            const CREATE_NO_WINDOW: u32 = 0x08000000;
            std::process::Command::new(file_path)
                .arg("/from_orchestrator=1")
                .arg("/action=commit")
                .arg(format!("/txn_id={}", txn_id_arg))
                .arg("/silent=1")
                .arg(format!("/product_version={}", product_version_arg))
                .arg("/auto_start=false")
                .arg("/fchannel=nochannel-mumu12")
                .arg("/auto_run=false")
                .raw_arg(format!("/D={}", install_dir_arg))
                .creation_flags(CREATE_NO_WINDOW)
                .output()
                .map_err(|e| format!("No se pudo ejecutar el programa de instalación: {}", e))?
        };

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        if !output.status.success() {
            let exit_code = output.status.code().unwrap_or(-1);
            log::error!(
                "Fallo de commit del componente {} (exit: {}): stdout={}, stderr={}",
                comp.name,
                exit_code,
                stdout,
                stderr
            );
            return Err(format!(
                "Fallo de commit del componente {} (exit: {})\n{}\n{}",
                comp.name, exit_code, stdout, stderr
            ));
        }
        log::info!("commit del componente {} correcto", comp.name);
    }

    let _ = app.emit(
        "download-progress",
        serde_json::json!({
            "status": "completed",
            "action": "Instalación completada",
            "percent": 100,
            "downloaded": 0,
            "total": 0
        }),
    );

    log::info!("Instalación V6 completada: {}", install_dir);

    // Eliminar el paquete de instalación
    let config = crate::config::read_config();
    if config.auto_delete_installer {
        for file_path in &downloaded_files {
            let p = PathBuf::from(file_path);
            if p.exists() {
                match fs::remove_file(&p) {
                    Ok(()) => log::info!("Paquete de instalación eliminado: {}", file_path),
                    Err(e) => log::warn!("No se pudo eliminar el paquete de instalación: {} - {}", file_path, e),
                }
            }
        }
    }

    Ok(format!("Instalación V6 correcta\nCarpeta de instalación: {}", install_dir))
}

/// Instalación silenciosa Setup.exe /S /D=install_path
#[tauri::command]
pub async fn install_v4_setup(
    app: tauri::AppHandle,
    installer_path: String,
    install_dir: String,
) -> Result<String, String> {
    let installer = PathBuf::from(&installer_path);
    if !installer.exists() {
        return Err(format!("El paquete de instalación no existe: {}", installer_path));
    }

    fs::create_dir_all(&install_dir).map_err(|e| format!("No se puede crear la carpeta de instalación: {}", e))?;

    log::info!("Iniciando la instalación silenciosa: {} -> {}", installer_path, install_dir);
    log::info!("Línea de comandos de instalación: \"{}\" /S /D={}", installer_path, install_dir);
    let _ = app.emit(
        "download-progress",
        serde_json::json!({
            "status": "installing",
            "action": "Instalando en modo silencioso, espere por favor...",
            "percent": 0,
            "downloaded": 0,
            "total": 0
        }),
    );

    // El instalador NSIS exige que /D= sea el último parametro y que la ruta no lleve comillas.
    // std::process::Command::arg() anade comillas automaticamente a los parametros con espacios, lo que rompe el análisis de parametros de NSIS.
    // Se usa raw_arg() para pasar la cadena de parametros original directamente y evitar problemas con las comillas.
    let output = {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        std::process::Command::new(&installer)
            .arg("/S")
            .raw_arg(format!("/D={}", install_dir))
            .creation_flags(CREATE_NO_WINDOW)
            .output()
            .map_err(|e| format!("No se pudo ejecutar el programa de instalación: {}", e))?
    };

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    if output.status.success() {
        let _ = app.emit(
            "download-progress",
            serde_json::json!({
                "status": "completed",
                "action": "Instalación completada",
                "percent": 100,
                "downloaded": 0,
                "total": 0
            }),
        );
        log::info!("Instalación correcta: {}", install_dir);
        if !stdout.is_empty() {
            log::info!("stdout de la instalación: {}", stdout);
        }
        // Tras una instalación correcta, la configuración decide si se elimina el paquete de instalación
        let config = crate::config::read_config();
        if config.auto_delete_installer && installer.exists() {
            match fs::remove_file(&installer) {
                Ok(()) => log::info!("Paquete de instalación eliminado: {}", installer_path),
                Err(e) => log::warn!("No se pudo eliminar el paquete de instalación: {} - {}", installer_path, e),
            }
        }
        Ok(format!("Instalación correcta\nCarpeta de instalación: {}\n{}", install_dir, stdout))
    } else {
        let exit_code = output.status.code().unwrap_or(-1);
        log::error!(
            "Fallo en la instalación (exit: {}): stdout={}, stderr={}",
            exit_code,
            stdout,
            stderr
        );
        let _ = app.emit(
            "download-progress",
            serde_json::json!({
                "status": "error",
                "action": format!("Fallo en la instalación (exit: {})", exit_code),
                "percent": 0,
                "downloaded": 0,
                "total": 0
            }),
        );
        Err(format!(
            "Fallo en la instalación (exit: {})\n{}\n{}",
            exit_code, stdout, stderr
        ))
    }
}

/// Obtiene el enlace de descarga de V4
async fn fetch_download_url_v4_inner() -> Result<String, String> {
    let uuid = uuid::Uuid::new_v4();
    let params = format!(
        "architecture=x86_64&channel=V4.1.31.3724&detectinfo=&downloader_version=3.1.14.0&has_installed=0&language=zh-Hans&machine=%7B%0A%20%20%22base_board%22%3A%20%22Manufacturer%3AASUSTeK%20COMPUTER%20INC.%20Product%3ATUF%20GAMING%20B660M-PLUS%20WIFI%20D4%22%2C%0A%20%20%22cpu%22%3A%20%22Intel%20Core%20i5-12400F%206-Core%20Processor%22%2C%0A%20%20%22hard_disk%22%3A%20%5B%0A%20%20%20%20%22DRIVE_FIXED%28C%3A%5C%5C%29%3ATotal%20disk%20space%3A476.9GBFree%20disk%20space%3A198.3GB%22%2C%0A%20%20%20%20%22DRIVE_FIXED%28D%3A%5C%5C%29%3ATotal%20disk%20space%3A931.5GBFree%20disk%20space%3A612.4GB%22%0A%20%20%5D%2C%0A%20%20%22hyperv_opened%22%3A%201%2C%0A%20%20%22ip%22%3A%20%22202.173.8.23%22%2C%0A%20%20%22mac%22%3A%20%22FD%3A87%3A30%3A1F%3AC9%3A0B%22%2C%0A%20%20%22memory%22%3A%2016384%2C%0A%20%20%22os%22%3A%20%22Windows%2010%2064-bit%20Kernel%2010.0.19045%22%2C%0A%20%20%22screen%22%3A%20%7B%0A%20%20%20%20%22height%22%3A%201080%2C%0A%20%20%20%20%22width%22%3A%201920%0A%20%20%7D%2C%0A%20%20%22screen_list%22%3A%20%5B%0A%20%20%20%20%7B%0A%20%20%20%20%20%20%22dpr%22%3A%201%2C%0A%20%20%20%20%20%20%22height%22%3A%201080%2C%0A%20%20%20%20%20%20%22is_primary%22%3A%201%2C%0A%20%20%20%20%20%20%22width%22%3A%201920%0A%20%20%20%20%7D%0A%20%20%5D%2C%0A%20%20%22supported_install_arc%22%3A%20%22x86_64%22%2C%0A%20%20%22video%22%3A%20%5B%0A%20%20%20%20%22NVIDIA%20GeForce%20RTX%203060%22%0A%20%20%5D%2C%0A%20%20%22vt%22%3A%20%22%22%2C%0A%20%20%22vt_enabled%22%3A%201%2C%0A%20%20%22vt_supported%22%3A%200%0A%7D&n=MuMuInstaller_3.1.14_hoeCnNC&package=&product=&usage=0&uuid={}",
        uuid
    );

    let client = reqwest::Client::new();
    let resp = client
        .post("https://mumu.nie.netease.com/api/v1/download/nemux")
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(params)
        .send()
        .await
        .map_err(|e| format!("Fallo en la petición: {}", e))?;

    let body = resp
        .text()
        .await
        .map_err(|e| format!("No se pudo leer la respuesta: {}", e))?;
    let download_resp: DownloadResponse =
        serde_json::from_str(&body).map_err(|e| format!("No se pudo analizar la respuesta: {}", e))?;

    if download_resp.errcode != 100 {
        return Err(format!(
            "La API ha devuelto un error: {} - {}",
            download_resp.errcode, download_resp.errmsg
        ));
    }

    let link = download_resp
        .data
        .and_then(|d| d.mumu)
        .map(|m| m.link)
        .ok_or_else(|| "No se ha encontrado el enlace de descarga en la respuesta".to_string())?;

    Ok(link)
}

/// Obtiene el enlace de descarga de V5
async fn fetch_download_url_v5_inner() -> Result<String, String> {
    let uuid = uuid::Uuid::new_v4();
    let random_suffix = uuid
        .to_string()
        .split('-')
        .next()
        .unwrap_or("default")
        .to_string();
    let n = format!("MuMu_5.0.2_{}", random_suffix);

    let params = format!(
        "architecture=x86_64&channel=nochannel-mumu12&detectinfo=&downloader_version=5.0.2&has_installed=0&language=zh-Hans&machine=%7B%0A%20%20%22base_board%22%3A%20%22Manufacturer%3AASUSTeK%20COMPUTER%20INC.%20Product%3ATUF%20GAMING%20B660M-PLUS%20WIFI%20D4%22%2C%0A%20%20%22cpu%22%3A%20%22Intel%20Core%20i5-12400F%206-Core%20Processor%22%2C%0A%20%20%22hard_disk%22%3A%20%5B%0A%20%20%20%20%22DRIVE_FIXED%28C%3A%5C%5C%29%3ATotal%20disk%20space%3A476.9GBFree%20disk%20space%3A198.3GB%22%2C%0A%20%20%20%20%22DRIVE_FIXED%28D%3A%5C%5C%29%3ATotal%20disk%20space%3A931.5GBFree%20disk%20space%3A612.4GB%22%0A%20%20%5D%2C%0A%20%20%22hyperv_opened%22%3A%201%2C%0A%20%20%22ip%22%3A%20%22202.173.8.23%22%2C%0A%20%20%22mac%22%3A%20%22FD%3A87%3A30%3A1F%3AC9%3A0B%22%2C%0A%20%20%22memory%22%3A%2016384%2C%0A%20%20%22os%22%3A%20%22Windows%2010%2064-bit%20Kernel%2010.0.19045%22%2C%0A%20%20%22screen%22%3A%20%7B%0A%20%20%20%20%22height%22%3A%201080%2C%0A%20%20%20%20%22width%22%3A%201920%0A%20%20%7D%2C%0A%20%20%22screen_list%22%3A%20%5B%0A%20%20%20%20%7B%0A%20%20%20%20%20%20%22dpr%22%3A%201%2C%0A%20%20%20%20%20%20%22height%22%3A%201080%2C%0A%20%20%20%20%20%20%22is_primary%22%3A%201%2C%0A%20%20%20%20%20%20%22width%22%3A%201920%0A%20%20%20%20%7D%0A%20%20%5D%2C%0A%20%20%22supported_install_arc%22%3A%20%22x86_64%22%2C%0A%20%20%22video%22%3A%20%5B%0A%20%20%20%20%22NVIDIA%20GeForce%20RTX%203060%22%0A%20%20%5D%2C%0A%20%20%22vt%22%3A%20%22%22%2C%0A%20%20%22vt_enabled%22%3A%201%2C%0A%20%20%22vt_supported%22%3A%200%0A%7D&n={}&package=&product=&usage=0&uuid={}",
        n, uuid
    );

    let client = reqwest::Client::new();
    let resp = client
        .post("https://api.mumu.nie.netease.com/api/v1/download/nx")
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(params)
        .send()
        .await
        .map_err(|e| format!("Fallo en la petición: {}", e))?;

    let body = resp
        .text()
        .await
        .map_err(|e| format!("No se pudo leer la respuesta: {}", e))?;
    let download_resp: DownloadResponse =
        serde_json::from_str(&body).map_err(|e| format!("No se pudo analizar la respuesta: {}", e))?;

    if download_resp.errcode != 100 {
        return Err(format!(
            "La API ha devuelto un error: {} - {}",
            download_resp.errcode, download_resp.errmsg
        ));
    }

    let link = download_resp
        .data
        .and_then(|d| d.mumu)
        .map(|m| m.link)
        .ok_or_else(|| "No se ha encontrado el enlace de descarga en la respuesta".to_string())?;

    Ok(link)
}

/// Funcion interna
async fn fetch_download_url_v6_inner() -> Result<String, String> {
    let uuid_val = uuid::Uuid::new_v4();
    let random_suffix = uuid_val
        .to_string()
        .split('-')
        .next()
        .unwrap_or("default")
        .to_string();
    let n = format!("MuMu_6.0.1_{}", random_suffix);

    let params = format!(
        "architecture=x86_64&channel=nochannel-mumu12&detectinfo=&downloader_version=6.0.1&has_installed=1&language=zh-Hans&machine=%7B%0A%20%20%22base_board%22%3A%20%22Manufacturer%3AASUSTeK%20COMPUTER%20INC.%20Product%3ATUF%20GAMING%20B660M-PLUS%20WIFI%20D4%22%2C%0A%20%20%22cpu%22%3A%20%22Intel%20Core%20i5-12400F%206-Core%20Processor%22%2C%0A%20%20%22hard_disk%22%3A%20%5B%0A%20%20%20%20%22DRIVE_FIXED%28C%3A%5C%5C%29%3ATotal%20disk%20space%3A476.9GBFree%20disk%20space%3A198.3GB%22%2C%0A%20%20%20%20%22DRIVE_FIXED%28D%3A%5C%5C%29%3ATotal%20disk%20space%3A931.5GBFree%20disk%20space%3A612.4GB%22%0A%20%20%5D%2C%0A%20%20%22hyperv_opened%22%3A%201%2C%0A%20%20%22ip%22%3A%20%22202.173.8.23%22%2C%0A%20%20%22mac%22%3A%20%22FD%3A87%3A30%3A1F%3AC9%3A0B%22%2C%0A%20%20%22memory%22%3A%2016384%2C%0A%20%20%22os%22%3A%20%22Windows%2010%2064-bit%20Kernel%2010.0.19045%22%2C%0A%20%20%22screen%22%3A%20%7B%0A%20%20%20%20%22height%22%3A%201080%2C%0A%20%20%20%20%22width%22%3A%201920%0A%20%20%7D%2C%0A%20%20%22screen_list%22%3A%20%5B%0A%20%20%20%20%7B%0A%20%20%20%20%20%20%22dpr%22%3A%201%2C%0A%20%20%20%20%20%20%22height%22%3A%201080%2C%0A%20%20%20%20%20%20%22is_primary%22%3A%201%2C%0A%20%20%20%20%20%20%22width%22%3A%201920%0A%20%20%20%20%7D%0A%20%20%5D%2C%0A%20%20%22supported_install_arc%22%3A%20%22x86_64%22%2C%0A%20%20%22video%22%3A%20%5B%0A%20%20%20%20%22NVIDIA%20GeForce%20RTX%203060%22%0A%20%20%5D%2C%0A%20%20%22vt%22%3A%20%22%22%2C%0A%20%20%22vt_enabled%22%3A%201%2C%0A%20%20%22vt_supported%22%3A%200%0A%7D&n={}&package=&product=&usage=0&uuid={}",
        n, uuid_val
    );

    let client = reqwest::Client::new();
    let resp = client
        .post("https://api.mumu.nie.netease.com/api/v2/download/nx")
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(params)
        .send()
        .await
        .map_err(|e| format!("Fallo en la petición: {}", e))?;

    let body = resp
        .text()
        .await
        .map_err(|e| format!("No se pudo leer la respuesta: {}", e))?;
    let download_resp: V6DownloadResponse =
        serde_json::from_str(&body).map_err(|e| format!("No se pudo analizar la respuesta: {}", e))?;

    if download_resp.errcode != 100 {
        return Err(format!(
            "La API ha devuelto un error: {} - {}",
            download_resp.errcode, download_resp.errmsg
        ));
    }

    let data = download_resp
        .data
        .ok_or_else(|| "No se ha encontrado data en la respuesta".to_string())?;
    let version = data.version.clone().unwrap_or_default();
    let components = data.components.clone().unwrap_or_default();
    let default_engine = data.default_download_engine.clone().unwrap_or_default();

    // Devolver la lista de componentes y la información de version
    let result = serde_json::json!({
        "version": version,
        "components": components,
        "defaultDownloadEngine": default_engine,
    });
    Ok(result.to_string())
}
