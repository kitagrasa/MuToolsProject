// Página de ajustes
import { showToast } from "./toast.js";
import { addLog } from "./logger.js";
import { onAutoRefreshSettingChanged } from "./mumu-info.js";
const { invoke } = window.__TAURI__.core;

export function loadSettings() {
  try {
    const saved = localStorage.getItem("mutools_settings");
    if (saved) {
      const settings = JSON.parse(saved);
      document.getElementById("settings-log-path").value = settings.logPath || "./log";
      document.getElementById("settings-download-path").value = settings.downloadPath || "./download";
      document.getElementById("settings-install-dir").value = settings.installDir || "D:\\Program Files\\Netease\\MuMu";

      const dataDirMode = settings.dataDirMode || "appdata";
      const dataDirCustom = settings.dataDirCustom || "";
      document.getElementById("settings-data-dir-mode").value = dataDirMode;
      document.getElementById("settings-data-dir-custom").value = dataDirCustom;
      updateDataDirCustomUI();

      const adminElevation = settings.adminElevation !== undefined ? settings.adminElevation : true;
      document.getElementById("settings-admin-elevation").checked = adminElevation;

      document.getElementById("settings-aria2-connections").value = settings.aria2MaxConnections || 5;
      document.getElementById("settings-aria2-split").value = settings.aria2Split || 5;
      document.getElementById("settings-auto-delete-installer").checked = settings.autoDeleteInstaller || false;
      document.getElementById("settings-auto-refresh-mumu").checked = settings.autoRefreshMuMu || false;
    } else {
      document.getElementById("settings-admin-elevation").checked = true;
      document.getElementById("settings-aria2-connections").value = 5;
      document.getElementById("settings-aria2-split").value = 5;
    }
    loadAria2ConfigFromBackend();
    loadAutoDeleteConfigFromBackend();
  } catch (e) {
    console.error("No se pudieron cargar los ajustes:", e);
  }
}

async function loadAria2ConfigFromBackend() {
  try {
    const cfg = await invoke("get_aria2_config");
    document.getElementById("settings-aria2-connections").value = cfg.maxConnections;
    document.getElementById("settings-aria2-split").value = cfg.split;
  } catch (e) {
    console.error("No se pudo cargar la configuración de aria2:", e);
  }
}

async function loadAutoDeleteConfigFromBackend() {
  try {
    const enabled = await invoke("get_auto_delete_installer");
    document.getElementById("settings-auto-delete-installer").checked = enabled;
  } catch (e) {
    console.error("No se pudo cargar la configuración de eliminación automática:", e);
  }
}

export function saveSettings() {
  const logPath = document.getElementById("settings-log-path").value.trim();
  const downloadPath = document.getElementById("settings-download-path").value.trim();
  const installDir = document.getElementById("settings-install-dir").value.trim();
  const dataDirMode = document.getElementById("settings-data-dir-mode").value;
  const dataDirCustom = document.getElementById("settings-data-dir-custom").value.trim();
  const adminElevation = document.getElementById("settings-admin-elevation").checked;
  const aria2MaxConnections = parseInt(document.getElementById("settings-aria2-connections").value) || 1;
  const aria2Split = parseInt(document.getElementById("settings-aria2-split").value) || 5;
  const autoDeleteInstaller = document.getElementById("settings-auto-delete-installer").checked;
  const autoRefreshMuMu = document.getElementById("settings-auto-refresh-mumu").checked;
  const settings = {
    logPath, downloadPath, installDir, dataDirMode, dataDirCustom,
    adminElevation, aria2MaxConnections, aria2Split, autoDeleteInstaller, autoRefreshMuMu
  };
  localStorage.setItem("mutools_settings", JSON.stringify(settings));

  invoke("save_data_config", { dataDirMode, dataDirCustom }).catch(e => console.error("No se pudo guardar la configuración de la carpeta de datos:", e));
  invoke("save_admin_elevation", { enabled: adminElevation }).catch(e => console.error("No se pudo guardar el ajuste de elevación de privilegios:", e));
  invoke("save_aria2_config", { maxConnections: aria2MaxConnections, split: aria2Split }).catch(e => console.error("No se pudo guardar la configuración de aria2:", e));
  invoke("save_auto_delete_installer", { enabled: autoDeleteInstaller }).catch(e => console.error("No se pudo guardar el ajuste de eliminación automática:", e));

  document.getElementById("install-dir").value = installDir;

  // Cambio en el ajuste de actualización automática
  onAutoRefreshSettingChanged();

  const prevSettings = (() => {
    try { return JSON.parse(localStorage.getItem("mutools_settings") || "{}"); } catch { return {}; }
  })();
  const prevElevation = prevSettings.adminElevation !== undefined ? prevSettings.adminElevation : true;
  if (prevElevation !== adminElevation) {
    showToast("Ajuste de elevación guardado, tendrá efecto al reiniciar la aplicación", "info");
    addLog(`El ajuste de elevación de privilegios ha cambiado a: ${adminElevation} (requiere reiniciar para tener efecto)`);
  } else {
    showToast("Ajustes guardados", "success");
  }
}

export function updateDataDirCustomUI() {
  const mode = document.getElementById("settings-data-dir-mode").value;
  const customGroup = document.getElementById("settings-data-dir-custom-group");
  customGroup.style.display = mode === "custom" ? "" : "none";
}

// Estado de los permisos de administrador

export async function loadAdminStatus() {
  const valueEl = document.getElementById("admin-status-value");
  if (!valueEl) return;
  valueEl.textContent = "Detectando...";
  valueEl.className = "admin-status-value unknown";
  try {
    const elevated = await invoke("check_admin_status");
    if (elevated) {
      valueEl.textContent = "Permisos de administrador";
      valueEl.className = "admin-status-value elevated";
    } else {
      valueEl.textContent = "Sin privilegios elevados";
      valueEl.className = "admin-status-value not-elevated";
    }
  } catch (e) {
    valueEl.textContent = "Error de detección";
    valueEl.className = "admin-status-value unknown";
    console.error("No se pudo detectar el estado de administrador:", e);
  }
}

// Vinculación de eventos

export function initSettingsPage() {
  document.getElementById("btn-save-settings").addEventListener("click", saveSettings);

  const dataDirModeSelect = document.getElementById("settings-data-dir-mode");
  if (dataDirModeSelect) {
    dataDirModeSelect.addEventListener("change", updateDataDirCustomUI);
  }
}
