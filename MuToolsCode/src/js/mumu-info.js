import { addLog } from "./logger.js";
import { escapeHtml } from "./toast.js";
const { invoke } = window.__TAURI__.core;

let mumuAutoRefreshTimer = null;
let mumuAutoRefreshEnabled = false;

export async function loadMuMuInfo() {
  const container = document.getElementById("mumu-info-content");
  if (!container) return;

  container.innerHTML = '<div class="mumu-info-loading">Cargando...</div>';

  try {
    const info = await invoke("get_mumu_info");
    renderMuMuInfo(info);
    addLog("Información de MuMu cargada correctamente");
  } catch (e) {
    container.innerHTML = `<div class="mumu-info-error">Error al cargar: ${e}</div>`;
    addLog(`Error al cargar la información de MuMu: ${e}`);
  }
}

function renderMuMuInfo(info) {
  const container = document.getElementById("mumu-info-content");
  if (!container) return;

  const majorVer = info.majorVersion || "?";
  const fullVer = info.fullVersion || "Desconocido";
  const channel = info.channel || "Desconocido";
  const installDir = info.installDir || "No detectado";
  const mumuManager = info.mumuManagerPath || "No encontrado";
  const appdataDir = info.appdataDir || "Desconocido";
  const registryKey = info.registryKey || "No detectado";
  const uninstallStr = info.uninstallString || "No detectado";

  const isMissing = (v) => v === "No detectado" || v === "No encontrado" || v === "Desconocido";

  let html = '<div class="mumu-info-card">';
  html += '<div class="mumu-info-hero">';
  html += '<div class="mumu-info-versions">';
  html += `<div class="mumu-info-major">MuMu ${escapeHtml(majorVer)}</div>`;
  html += `<div class="mumu-info-full">Versión completa: ${escapeHtml(fullVer)}</div>`;
  html += `<div class="mumu-info-channel">Canal: ${escapeHtml(channel)}</div>`;
  html += '</div></div>';

  html += '<div class="mumu-info-details">';
  const detailItems = [
    { label: "Carpeta raíz de instalación", value: installDir },
    { label: "MuMuManager", value: mumuManager },
    { label: "Carpeta AppData", value: appdataDir },
    { label: "Clave del registro", value: registryKey },
    { label: "Ruta de desinstalación", value: uninstallStr },
  ];
  detailItems.forEach((item) => {
    const cls = isMissing(item.value) ? "mumu-info-detail-row missing" : "mumu-info-detail-row";
    html += `<div class="${cls}"><span class="mumu-info-detail-label">${item.label}</span><span class="mumu-info-detail-value" title="${escapeHtml(item.value)}">${escapeHtml(item.value)}</span></div>`;
  });
  html += '</div>';

  if (info.errors && info.errors.length > 0) {
    html += '<div class="mumu-info-errors">';
    info.errors.forEach((err) => {
      html += `<div class="mumu-info-error-item">${escapeHtml(err)}</div>`;
    });
    html += '</div>';
  }

  html += '</div>';
  container.innerHTML = html;
}

// Actualización automática

export function startMuMuAutoRefresh() {
  stopMuMuAutoRefresh();
  const enabled = getAutoRefreshSetting();
  mumuAutoRefreshEnabled = enabled;
  if (enabled) {
    addLog("Actualización automática de la información de MuMu");
    loadMuMuInfo().catch(() => { });
  }
}

export function stopMuMuAutoRefresh() {
  if (mumuAutoRefreshTimer) {
    clearInterval(mumuAutoRefreshTimer);
    mumuAutoRefreshTimer = null;
  }
  mumuAutoRefreshEnabled = false;
}

export function refreshMuMuInfoAfterInstall() {
  if (getAutoRefreshSetting()) {
    loadMuMuInfo().catch(() => { });
  }
}

export function getAutoRefreshSetting() {
  try {
    const saved = localStorage.getItem("mutools_settings");
    if (saved) {
      const settings = JSON.parse(saved);
      return settings.autoRefreshMuMu || false;
    }
  } catch (e) { }
  return false;
}

export function onAutoRefreshSettingChanged() {
  const enabled = getAutoRefreshSetting();
  if (enabled !== mumuAutoRefreshEnabled) {
    if (enabled) {
      startMuMuAutoRefresh();
    } else {
      stopMuMuAutoRefresh();
    }
  }
}

// Vinculación de eventos

export function initMuMuInfoPage() {
  document.getElementById("btn-mumu-refresh").addEventListener("click", () => {
    addLog("Actualización manual de la información de MuMu");
    loadMuMuInfo();
  });
}
