
use std::ffi::OsStr;
use std::iter;
use std::os::windows::ffi::OsStrExt;
use std::process::exit;

use windows_sys::Win32::Foundation::{CloseHandle, HANDLE};
use windows_sys::Win32::Security::{
    GetTokenInformation, TokenElevation, TOKEN_ELEVATION, TOKEN_QUERY,
};
use windows_sys::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};
use windows_sys::Win32::UI::Shell::ShellExecuteW;
use windows_sys::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;

/// Convierte un &str en una cadena ancha UTF-16 terminada en null
fn to_wide(s: &str) -> Vec<u16> {
    OsStr::new(s).encode_wide().chain(iter::once(0)).collect()
}

/// Detecta los permisos de administrador
pub fn is_admin() -> bool {
    unsafe {
        let mut token: HANDLE = std::ptr::null_mut();
        if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token) == 0 {
            return false;
        }

        let mut elevation = TOKEN_ELEVATION { TokenIsElevated: 0 };
        let mut ret_len: u32 = 0;
        let ok = GetTokenInformation(
            token,
            TokenElevation,
            &mut elevation as *mut _ as *mut std::ffi::c_void,
            std::mem::size_of::<TOKEN_ELEVATION>() as u32,
            &mut ret_len,
        );
        let _ = CloseHandle(token);

        ok != 0 && elevation.TokenIsElevated != 0
    }
}

/// Reinicia la aplicación con permisos de administrador
pub fn relaunch_as_admin(args: &[String]) -> bool {
    let exe = std::env::current_exe()
        .ok()
        .and_then(|p| p.to_str().map(|s| s.to_string()))
        .unwrap_or_default();
    if exe.is_empty() {
        return false;
    }

    let verb = to_wide("runas");
    let file = to_wide(&exe);

    // Construye la cadena de argumentos de la línea de comandos
    let mut params = String::new();
    for a in args {
        if !params.is_empty() {
            params.push(' ');
        }
        if a.contains(' ') {
            params.push('"');
            params.push_str(a);
            params.push('"');
        } else {
            params.push_str(a);
        }
    }
    let params_wide = if params.is_empty() {
        Vec::new()
    } else {
        to_wide(&params)
    };
    let params_ptr = if params_wide.is_empty() {
        std::ptr::null()
    } else {
        params_wide.as_ptr()
    };

    let result = unsafe {
        ShellExecuteW(
            std::ptr::null_mut(),
            verb.as_ptr(),
            file.as_ptr(),
            params_ptr,
            std::ptr::null(),
            SW_SHOWNORMAL,
        )
    };

    // Un valor de retorno de ShellExecuteW > 32 indica exito
    let success = result as usize > 32;
    if success {
        exit(0);
    }
    success
}

/// Parametro marcador para que el proceso hijo detecte si es un reinicio con elevación de privilegios
pub const ELEVATION_FLAG: &str = "--elevated";

/// Indica si la línea de comandos incluye el marcador de elevación de privilegios
pub fn has_elevation_flag() -> bool {
    std::env::args().any(|a| a == ELEVATION_FLAG)
}

/// Comprobacion de seguridad para evitar un bucle infinito
pub fn get_elevation_attempt() -> u32 {
    std::env::var("MUTOOLS_ELEVATION_ATTEMPT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0)
}

/// Escribe el número de intentos de elevación en una variable de entorno para que la lea el proceso hijo
pub fn set_elevation_attempt_env(n: u32) {
    std::env::set_var("MUTOOLS_ELEVATION_ATTEMPT", n.to_string());
}

/// Realiza la comprobacion de elevación de privilegios al arrancar
pub fn check_and_elevate(admin_elevation: bool) {
    if !admin_elevation {
        return;
    }

    // Ya se tienen permisos de administrador, no hace falta elevar privilegios
    if is_admin() {
        if has_elevation_flag() {
            log::info!("Reinicio con permisos de administrador correcto");
        }
        return;
    }

    // En entorno dev se omite el reinicio con elevación de privilegios
    if is_dev_env() {
        log::info!("Entorno dev sin permisos de administrador: se omite la elevación automática de privilegios");
        return;
    }

    // Evita un bucle infinito de reinicios
    let attempt = get_elevation_attempt();
    if attempt >= 3 {
        log::warn!("Se ha intentado {} veces; se descarta la elevación de privilegios", attempt);
        return;
    }

    // Construye los argumentos del proceso hijo
    let mut args: Vec<String> = std::env::args().skip(1).collect();
    // Descarta el marcador que ya pudiera estar presente
    args.retain(|a| a != ELEVATION_FLAG);
    args.push(ELEVATION_FLAG.to_string());

    // Define la variable de entorno que transmite el número de intentos de elevación
    set_elevation_attempt_env(attempt + 1);

    log::info!("Intento {} de reiniciar con permisos de administrador", attempt + 1);

    // Intenta el reinicio con elevación de privilegios
    if !relaunch_as_admin(&args) {
        log::warn!("La elevación automática de privilegios ha fallado; se continuara sin privilegios elevados");
    }
}

/// Detecta si se esta en dev
fn is_dev_env() -> bool {
    if cfg!(debug_assertions) {
        return true;
    }
    false
}

/// Para uso desde la línea de comandos
pub fn current_is_admin() -> bool {
    is_admin()
}
