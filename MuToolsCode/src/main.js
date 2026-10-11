// Punto de entrada principal
import { addLog, loadBackendLogs, clearLogs } from "./js/logger.js";
import { showToast } from "./js/toast.js";
import { getDailyQuote, onVersionClick } from "./js/daily-quotes.js";
import { switchPage } from "./js/router.js";
import { updateProgress, initInstallPage } from "./js/install.js";
import { loadSettings, loadAdminStatus, initSettingsPage } from "./js/settings.js";
import { initPackagesPage } from "./js/packages.js";
import { initOptimizePage } from "./js/optimize.js";
import { startMuMuAutoRefresh, onAutoRefreshSettingChanged, initMuMuInfoPage } from "./js/mumu-info.js";
import { initHelpPage } from "./js/help.js";
import { initAboutPage } from "./js/about.js";

const { invoke } = window.__TAURI__.core;

window.addEventListener("DOMContentLoaded", () => {
  addLog("Aplicación iniciada");

  // Escucha el evento de progreso de descarga del backend
  const { listen } = window.__TAURI__.event;
  listen("download-progress", (event) => {
    updateProgress(event.payload);
  }).catch(() => {});

  // Escucha el evento de aviso toast del backend
  listen("toast", (event) => {
    const payload = event.payload;
    if (typeof payload === "string") {
      showToast(payload, "info");
    } else if (payload && typeof payload === "object") {
      const msg = payload.message || payload.msg || "Notificación";
      const type = payload.type || "info";
      showToast(msg, type);
    }
  }).catch(() => {});

  // Carga los ajustes
  loadSettings();
  loadAdminStatus();

  // Obtiene la carpeta de datos
  invoke("get_data_dir").then(dataDir => {
    addLog(`Carpeta de datos: ${dataDir}`);
  }).catch(() => {});

  // Restaura la carpeta de instalación desde los ajustes
  const savedSettings = localStorage.getItem("mutools_settings");
  if (savedSettings) {
    try {
      const s = JSON.parse(savedSettings);
      document.getElementById("install-dir").value = s.installDir || "D:\\Program Files\\Netease\\MuMu";
    } catch (e) {}
  }

  // Navegación
  document.querySelectorAll(".nav-item").forEach((item) => {
    item.addEventListener("click", () => {
      switchPage(item.dataset.page);
    });
  });

  // Inicializa el enlace de eventos de cada página
  initInstallPage();
  initSettingsPage();
  initPackagesPage();
  initMuMuInfoPage();
  initOptimizePage();

  // Botón de registro
  document.getElementById("btn-refresh-logs").addEventListener("click", () => {
    loadBackendLogs();
    addLog("Actualizar registro");
  });
  document.getElementById("btn-clear-logs").addEventListener("click", clearLogs);

  // Interruptor de actualización automática
  const autoRefreshCheckbox = document.getElementById("settings-auto-refresh-mumu");
  if (autoRefreshCheckbox) {
    autoRefreshCheckbox.addEventListener("change", () => {
      try {
        const saved = JSON.parse(localStorage.getItem("mutools_settings") || "{}");
        saved.autoRefreshMuMu = autoRefreshCheckbox.checked;
        localStorage.setItem("mutools_settings", JSON.stringify(saved));
      } catch (e) {}
      onAutoRefreshSettingChanged();
    });
  }

  // Inicia la actualización automática
  startMuMuAutoRefresh();

  // Frase del día
  const quoteEl = document.getElementById("daily-quote-text");
  const quoteBox = document.querySelector(".daily-quote");
  if (quoteEl) {
    quoteEl.textContent = getDailyQuote();
  }
  if (quoteBox) {
    quoteBox.addEventListener("click", () => {
      if (quoteEl) {
        quoteEl.textContent = getDailyQuote();
      }
    });
  }

  // Huevo de pascua del número de versión en la página Acerca de
  const versionEl = document.getElementById("about-version");
  if (versionEl) {
    versionEl.addEventListener("click", onVersionClick);
  }

  // Página de ayuda
  initHelpPage();

  // Página Acerca de
  initAboutPage();
});
