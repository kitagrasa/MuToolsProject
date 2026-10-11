// Módulo de registro

const { invoke } = window.__TAURI__.core;

let allLogEntries = [];

export function getLogEntries() {
  return allLogEntries;
}

export function formatTime() {
  const d = new Date();
  const pad = (n) => String(n).padStart(2, "0");
  return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())} ${pad(d.getHours())}:${pad(d.getMinutes())}:${pad(d.getSeconds())}`;
}

export function addLog(message, source = "Interfaz") {
  const time = formatTime();
  const entry = `[${time}] [${source}] ${message}`;
  allLogEntries.push(entry);
  renderMergedLogs();
  invoke("log_action", { message: `[${source}] ${message}` }).catch(() => { });
}

export function renderMergedLogs() {
  const container = document.getElementById("merged-logs");
  if (!container) return;
  container.innerHTML = "";
  if (allLogEntries.length === 0) {
    container.textContent = "No hay registros";
    return;
  }
  allLogEntries.forEach((entry) => {
    const div = document.createElement("div");
    div.className = "log-entry";
    div.textContent = entry;
    container.appendChild(div);
  });
  container.scrollTop = container.scrollHeight;
}

export async function loadBackendLogs() {
  try {
    const logs = await invoke("get_logs");
    if (logs && logs.length > 0) {
      logs.forEach((entry) => {
        if (!allLogEntries.some((e) => e === entry)) {
          allLogEntries.push(entry);
        }
      });
      renderMergedLogs();
    }
  } catch (e) {
    console.error("No se pudieron cargar los registros del backend:", e);
  }
}

export function clearLogs() {
  allLogEntries = [];
  renderMergedLogs();
  invoke("clear_logs").catch(() => { });
  addLog("Registro vaciado");
}
