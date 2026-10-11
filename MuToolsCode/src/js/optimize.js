// Página de optimización
import { showToast } from "./toast.js";
import { addLog } from "./logger.js";
const { invoke } = window.__TAURI__.core;

const optimizeGroups = [
  {
    id: "startup-ad",
    title: "Anuncio de inicio",
    items: [
      {
        id: "startup-image",
        title: "Modificar la carpeta startupImage",
        command: "disable_startup_image",
        undoCommand: "restore_startup_image",
        pros: ["Elimina el anuncio de inicio"],
        cons: ["No se pueden mostrar los eventos de MuMu"],
        notes: ["Se puede acceder a los eventos desde el centro de juegos"],
      },
      {
        id: "hosts-update",
        title: "Proxy de host a 127",
        command: "block_update_domains",
        undoCommand: "unblock_update_domains",
        pros: ["Elimina las actualizaciones del centro de mensajes y del emulador"],
        cons: ["No se pueden usar las actualizaciones descargadas del sitio web ni el instalador en línea"],
        notes: ["No afecta al entorno de red interno del emulador"],
      },
    ],
  },
  {
    id: "message‌-ad",
    title: "Anuncios del centro de mensajes",
    items: [
      {
        id: "hosts-update",
        title: "Proxy de host a 127",
        command: "block_update_domains",
        undoCommand: "unblock_update_domains",
        pros: ["Elimina las actualizaciones del centro de mensajes y del emulador"],
        cons: ["No se pueden usar las actualizaciones descargadas del sitio web ni el instalador en línea"],
        notes: ["No afecta al entorno de red interno del emulador"],
      },
      {
        id: "firewall-message",
        title: "[Obsoleto] Bloquear conexiones con el firewall",
        command: "add_firewall_rule_1",
        undoCommand: "remove_firewall_rule_1",
        pros: ["Elimina los anuncios del centro de mensajes"],
        cons: ["No se pueden usar algunas funciones del centro de juegos"],
        notes: ["Solo bloquea las conexiones de MuMuPlayer.exe a las IP de los servidores de anuncios"],
      },
    ],
  },
  {
    id: "desktop-ad",
    title: "Anuncios del escritorio del emulador",
    items: [
      {
        id: "data-package",
        title: "Usar el paquete data",
        command: "import_data_package",
        undoCommand: "undo_import_data_package",
        needsResource: "data",
        pros: ["Elimina los anuncios del escritorio del emulador"],
        cons: ["Los iconos de las aplicaciones multiinstancia no se muestran en el escritorio; se puede acceder desde el software de multiapertura"],
        notes: [],
      },
      {
        id: "firewall-desktop",
        title: "[Obsoleto] Bloquear conexiones con el firewall",
        command: "add_firewall_rule_2",
        undoCommand: "remove_firewall_rule_2",
        pros: ["Elimina los anuncios del escritorio del emulador y el centro de juegos integrado"],
        cons: ["No se pueden usar el instalador de Google, la aceleración de juegos, la ubicación virtual ni la obtención de teclas predeterminadas"],
        notes: ["Solo bloquea las conexiones del software VMMHeadless; si falla la obtención del android_id, se puede iniciar el emulador de antemano e intentar obtener el android_id"],
      },
    ],
  },
  {
    id: "block-update",
    title: "Bloquear las actualizaciones",
    items: [
      {
        id: "hosts-update-2",
        title: "Proxy de host a 127",
        command: "block_update_domains",
        undoCommand: "unblock_update_domains",
        pros: ["Elimina las actualizaciones del centro de mensajes y del emulador"],
        cons: ["No se pueden usar las actualizaciones descargadas del sitio web ni el instalador en línea"],
        notes: ["No afecta al entorno de red interno del emulador"],
      },
      {
        id: "disable-updater",
        title: "Modificar el archivo de actualización",
        command: "disable_updater",
        undoCommand: "restore_updater",
        pros: ["Bloquea las actualizaciones del emulador"],
        cons: ["No se puede probar la versión nueva"],
        notes: ["Cambia MuMuPlayerUpdater.exe a MuMuPlayerUpdater.exe.bak"],
      },
    ],
  },
  {
    id: "special-edition",
    title: "Conversión a modo imitación Premium / versión internacional",
    items: [
      {
        id: "overlay-4x",
        title: "[4.x] Conversión a modo imitación Premium",
        command: "apply_overlay_package",
        undoCommand: "undo_overlay_package",
        needsResource: "overlay",
        pros: ["Se convierte en modo imitación Premium: se eliminan las limitaciones de funciones del emulador de dicha edición, ocupa poco espacio y ofrece funciones potentes, además de quitar la barra de búsqueda"],
        cons: ["Puede haber problemas de compatibilidad con datos antiguos del paquete data; es necesario crear de nuevo las instancias múltiples"],
        notes: ["Creado a partir de la extracción y modificación de los archivos overlay del emulador de la edición Premium; la modificación del overlay se aplica a todas las instancias múltiples"],
      },
      {
        id: "fchannel-5x",
        title: "[5.x][6.x] Conversión a modo imitación Premium",
        command: "apply_fchannel",
        undoCommand: "undo_fchannel",
        pros: ["Se convierte en modo imitación Premium y apenas afecta a la experiencia de la versión original"],
        cons: [],
        notes: ["Modifica la información de configuración de la instalación; en la versión normal, cambiar solo este parámetro no descarga juegos automáticamente"],
      },
      {
        id: "patch-vdi-overseas",
        title: "[5.x][6.x] Modificar system.vdi",
        command: "patch_system_vdi_overseas",
        undoCommand: "undo_patch_system_vdi_overseas",
        pros: ["Modifica system.vdi para añadir el identificador de versión internacional"],
        cons: ["Ocupará temporalmente entre 1 y 2 GB de memoria"],
        notes: ["Modifica build.prod directamente en system.vdi"],
      }
    ],
  },
  {
    id: "telemetry",
    title: "Telemetría de MuMu",
    items: [
      {
        id: "block-report",
        title: "Proxy global a 127 (modificar hosts)",
        command: "block_report_domains",
        undoCommand: "unblock_report_domains",
        pros: ["Desactiva la telemetría de MuMu"],
        cons: ["Falla el envío de registros de errores"],
        notes: ["El bloqueo incluye, entre otros, la telemetría de MuMu 12"],
      },
    ],
  },
];

const optimizeFilePaths = {};

export function renderOptimizePage() {
  const accordion = document.getElementById("optimize-accordion");
  if (!accordion || accordion.dataset.rendered) return;
  accordion.innerHTML = "";

  const faqList = document.createElement("div");
  faqList.className = "optimize-faq-list";

  optimizeGroups.forEach((group) => {
    const faqItem = document.createElement("div");
    faqItem.className = "optimize-faq-item";

    const question = document.createElement("div");
    question.className = "optimize-faq-question";
    question.innerHTML = `
      <span>${group.title}</span>
      <div class="optimize-faq-right">
        <span class="opt-group-count">${group.items.length} elementos</span>
        <span class="optimize-faq-arrow">▼</span>
      </div>
    `;
    question.addEventListener("click", () => { faqItem.classList.toggle("open"); });
    faqItem.appendChild(question);

    const answer = document.createElement("div");
    answer.className = "optimize-faq-answer";

    group.items.forEach((item) => {
      const row = document.createElement("div");
      row.className = "opt-item";

      const left = document.createElement("div");
      left.className = "opt-item-left";

      const topRow = document.createElement("div");
      const cb = document.createElement("input");
      cb.type = "checkbox";
      cb.className = "opt-item-checkbox";
      cb.dataset.groupId = group.id;
      cb.dataset.itemId = item.id;
      cb.addEventListener("change", () => onOptItemCheckChanged(cb, item));
      topRow.appendChild(cb);

      const label = document.createElement("span");
      label.className = "opt-item-title";
      label.textContent = item.title;
      topRow.appendChild(label);
      left.appendChild(topRow);

      if (item.needsResource) {
        const fileRow = document.createElement("div");
        fileRow.className = "opt-item-file";
        fileRow.style.display = "none";
        const select = document.createElement("select");
        select.className = "form-input opt-resource-select";
        select.dataset.itemId = item.id;
        select.innerHTML = '<option value="">Cargando...</option>';
        select.disabled = true;
        fileRow.appendChild(select);
        left.appendChild(fileRow);
      }

      const detail = document.createElement("div");
      detail.className = "opt-item-detail";
      if (item.pros.length > 0) {
        item.pros.forEach((p) => {
          const d = document.createElement("div");
          d.className = "opt-detail opt-detail-pro";
          d.textContent = `+ ${p}`;
          detail.appendChild(d);
        });
      }
      if (item.cons.length > 0) {
        item.cons.forEach((c) => {
          const d = document.createElement("div");
          d.className = "opt-detail opt-detail-con";
          d.textContent = `- ${c}`;
          detail.appendChild(d);
        });
      }
      if (item.notes.length > 0) {
        item.notes.forEach((n) => {
          const d = document.createElement("div");
          d.className = "opt-detail opt-detail-note";
          d.textContent = `* ${n}`;
          detail.appendChild(d);
        });
      }
      left.appendChild(detail);

      row.appendChild(left);
      answer.appendChild(row);
    });

    faqItem.appendChild(answer);
    faqList.appendChild(faqItem);
  });

  accordion.appendChild(faqList);
  accordion.dataset.rendered = "true";
}

function onOptItemCheckChanged(cb, item) {
  if (item.needsResource) {
    const fileRow = cb.closest(".opt-item-left").querySelector(".opt-item-file");
    if (fileRow) {
      fileRow.style.display = cb.checked ? "flex" : "none";
      if (cb.checked) {
        loadOptimizeResources(item, fileRow);
      } else {
        delete optimizeFilePaths[item.id];
        const select = fileRow.querySelector(".opt-resource-select");
        if (select) select.value = "";
      }
    }
  }
}

async function loadOptimizeResources(item, fileRow) {
  const select = fileRow.querySelector(".opt-resource-select");
  if (!select) return;
  const cmdMap = { data: "list_data_optimize_packages", overlay: "list_overlay_optimize_packages" };
  const cmd = cmdMap[item.needsResource] || "list_data_optimize_packages";
  try {
    const list = await invoke(cmd);
    select.innerHTML = "";
    if (!list || list.length === 0) {
      select.innerHTML = '<option value="">No se encontraron paquetes de recursos coincidentes</option>';
      select.disabled = true;
      return;
    }
    select.innerHTML = '<option value="">Seleccione un paquete de recursos...</option>';
    list.forEach((pkg) => {
      const opt = document.createElement("option");
      opt.value = pkg.filePath;
      let label = pkg.displayName || pkg.packageName;
      if (pkg.version) label += ` v${pkg.version}`;
      if (pkg.fileName) label += ` (${pkg.fileName})`;
      opt.textContent = label;
      select.appendChild(opt);
    });
    select.disabled = false;
    select.addEventListener("change", () => {
      if (select.value) {
        optimizeFilePaths[item.id] = select.value;
      } else {
        delete optimizeFilePaths[item.id];
      }
    });
  } catch (e) {
    select.innerHTML = '<option value="">Error al cargar</option>';
    select.disabled = true;
    addLog(`Error al cargar la lista de paquetes de recursos: ${e}`);
  }
}

function getCheckedItems() {
  const checked = [];
  document.querySelectorAll(".opt-item-checkbox:checked").forEach((cb) => {
    const groupId = cb.dataset.groupId;
    const itemId = cb.dataset.itemId;
    const group = optimizeGroups.find((g) => g.id === groupId);
    if (!group) return;
    const item = group.items.find((i) => i.id === itemId);
    if (!item) return;
    checked.push({ group, item });
  });
  return checked;
}

async function runOptimizeBatch(mode) {
  const checked = getCheckedItems();
  if (checked.length === 0) {
    showToast("Seleccione al menos una optimización", "info");
    return;
  }

  const label = mode === "apply" ? "Iniciar optimización" : "Iniciar reversión de la optimización";
  addLog(`========== ${label} ==========`);
  showToast(label, "info");

  for (const { item } of checked) {
    const command = mode === "apply" ? item.command : item.undoCommand;
    if (!command) continue;

    const args = {};
    if (mode === "apply" && item.needsResource) {
      const filePath = optimizeFilePaths[item.id];
      if (!filePath) {
        addLog(`Se omite «${item.title}»: no se ha seleccionado ningún paquete de recursos`);
        showToast(`Se omite «${item.title}»: no se ha seleccionado ningún paquete de recursos`, "info");
        continue;
      }
      args.path = filePath;
    }

    try {
      addLog(`Ejecutando: ${item.title}`);
      showToast(`Ejecutando: ${item.title}`, "info");
      const result = await invoke(command, args);
      addLog(`Correcto: ${result}`);
    } catch (e) {
      addLog(`Error: ${item.title} ${e}`);
      showToast(`${item.title}: error ${e}`, "error");
    }
  }

  addLog(`========== ${label} finalizado ==========`);
  showToast(`${label} finalizado`, "success");
}

export function initOptimizePage() {
  const btnOptimizeStart = document.getElementById("btn-optimize-start");
  if (btnOptimizeStart) {
    btnOptimizeStart.addEventListener("click", () => runOptimizeBatch("apply"));
  }
  const btnOptimizeCancel = document.getElementById("btn-optimize-cancel");
  if (btnOptimizeCancel) {
    btnOptimizeCancel.addEventListener("click", () => runOptimizeBatch("cancel"));
  }
}
