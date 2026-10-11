// Página de gestión de paquetes de recursos
import { showToast, formatSize, escapeHtml } from "./toast.js";
import { addLog } from "./logger.js";
const { invoke } = window.__TAURI__.core;

let pendingDelete = null; // { packageName, descFile?, descName? }

// Confirmación de eliminación

export function showDeleteConfirm(packageName, descFile, descName) {
  pendingDelete = { packageName, descFile: descFile || null, descName: descName || null };
  const msg = descFile
    ? `¿Seguro que quieres eliminar el subpaquete "${descName || descFile}"?`
    : `¿Seguro que quieres eliminar el paquete de recursos "${packageName}"?`;
  document.getElementById("modal-delete-message").textContent = msg;
  document.getElementById("modal-delete-overlay").style.display = "flex";
}

export function hideDeleteConfirm() {
  pendingDelete = null;
  document.getElementById("modal-delete-overlay").style.display = "none";
}

async function confirmDeletePackage() {
  const del = pendingDelete;
  if (!del) return;
  const confirmBtn = document.getElementById("btn-delete-confirm");
  confirmBtn.disabled = true;
  try {
    if (del.descFile) {
      await invoke("delete_sub_package", { packageName: del.packageName, descFile: del.descFile });
      addLog(`Subpaquete eliminado: ${del.descName || del.descFile}`);
    } else {
      await invoke("delete_resource_package", { name: del.packageName });
      addLog(`Paquete de recursos eliminado: ${del.packageName}`);
    }
    hideDeleteConfirm();
    await loadPackages();
  } catch (e) {
    showToast(`Error al eliminar: ${e}`, "error");
    addLog(`Error al eliminar: ${e}`);
  } finally {
    confirmBtn.disabled = false;
  }
}

// Cargar la lista de paquetes de recursos

export async function loadPackages() {
  try {
    const list = await invoke("list_resource_packages");
    const container = document.getElementById("pkg-list");
    container.innerHTML = "";
    if (!list || list.length === 0) {
      const empty = document.createElement("div");
      empty.className = "pkg-empty";
      empty.textContent = "No hay paquetes de recursos";
      container.appendChild(empty);
      return;
    }
    list.forEach((pkg) => {
      const card = document.createElement("div");
      card.className = "pkg-card";

      const header = document.createElement("div");
      header.className = "pkg-card-header";

      const hasSubItems = pkg.descInfos && pkg.descInfos.length > 0;

      const arrow = document.createElement("span");
      arrow.className = "pkg-expand-arrow";
      arrow.textContent = hasSubItems ? "▼" : "✕";
      if (!hasSubItems) {
        arrow.style.opacity = "0.3";
        arrow.style.cursor = "default";
      }
      header.appendChild(arrow);

      const info = document.createElement("div");
      info.className = "pkg-info";

      const name = document.createElement("div");
      name.className = "pkg-name";
      name.title = pkg.path;
      name.textContent = pkg.manifest.name || pkg.name;
      info.appendChild(name);

      if (pkg.manifest.description) {
        const desc = document.createElement("div");
        desc.className = "pkg-desc";
        desc.textContent = pkg.manifest.description;
        info.appendChild(desc);
      }

      const meta = document.createElement("div");
      meta.className = "pkg-meta";
      let metaParts = [];
      if (hasSubItems) {
        metaParts.push(`${pkg.descInfos.length} subpaquetes`);
      } else {
        metaParts.push(pkg.kind);
      }
      metaParts.push(formatSize(pkg.size));
      meta.textContent = metaParts.join(" · ");
      info.appendChild(meta);

      header.appendChild(info);

      const actions = document.createElement("div");
      actions.className = "pkg-actions";

      const openDirBtn = document.createElement("button");
      openDirBtn.className = "btn-secondary";
      openDirBtn.textContent = "Abrir carpeta";
      openDirBtn.addEventListener("click", (e) => {
        e.stopPropagation();
        invoke("open_resource_dir", { name: pkg.name })
          .then(() => addLog(`Abrir la carpeta del paquete de recursos: ${pkg.name}`))
          .catch((e) => addLog(`Error al abrir la carpeta del paquete de recursos: ${e}`));
      });

      const openLinkBtn = document.createElement("button");
      openLinkBtn.className = "btn-secondary";
      openLinkBtn.textContent = "Abrir enlace";
      openLinkBtn.addEventListener("click", (e) => {
        e.stopPropagation();
        const url = pkg.manifest?.webpage;
        if (url) {
          invoke("open_url", { url })
            .then(() => addLog(`Abrir enlace: ${url}`))
            .catch((e) => addLog(`Error al abrir el enlace: ${e}`));
        } else {
          showToast("Este paquete de recursos no tiene ningún enlace asociado", "info");
        }
      });

      actions.appendChild(openDirBtn);
      actions.appendChild(openLinkBtn);

      if (!hasSubItems) {
        const deleteBtn = document.createElement("button");
        deleteBtn.className = "btn-secondary btn-danger";
        deleteBtn.textContent = "Eliminar";
        deleteBtn.addEventListener("click", (e) => { e.stopPropagation(); showDeleteConfirm(pkg.name); });
        actions.appendChild(deleteBtn);
      }

      header.appendChild(actions);

      if (hasSubItems) {
        card.classList.add("pkg-card-expandable");
        header.addEventListener("click", () => { card.classList.toggle("open"); });
      }

      card.appendChild(header);

      if (hasSubItems) {
        const body = document.createElement("div");
        body.className = "pkg-card-body";

        pkg.descInfos.forEach((di) => {
          const subItem = document.createElement("div");
          subItem.className = "pkg-sub-item";

          const subItemRow = document.createElement("div");
          subItemRow.className = "pkg-sub-item-row";

          const subInfo = document.createElement("div");
          subInfo.className = "pkg-sub-info";

          const subName = document.createElement("div");
          subName.className = "pkg-sub-name";
          subName.textContent = di.name || "Sin nombre";
          subInfo.appendChild(subName);

          const subMeta = document.createElement("div");
          subMeta.className = "pkg-sub-meta";
          const parts = [];
          if (di.builder) parts.push(di.builder);
          if (di.displayVersion) parts.push(`Versión: ${di.displayVersion}`);
          if (di.buildVersion) parts.push(`Versión de compilación: ${di.buildVersion}`);
          if (di.time) parts.push(new Date(di.time * 1000).toLocaleString());
          subMeta.textContent = parts.join(" · ");
          subInfo.appendChild(subMeta);

          subItemRow.appendChild(subInfo);

          const subDeleteBtn = document.createElement("button");
          subDeleteBtn.className = "btn-secondary btn-danger btn-sm";
          subDeleteBtn.textContent = "Eliminar";
          subDeleteBtn.addEventListener("click", (e) => {
            e.stopPropagation();
            showDeleteConfirm(pkg.name, di.descFile, di.name);
          });
          subItemRow.appendChild(subDeleteBtn);

          subItem.appendChild(subItemRow);

          if (di.description) {
            const subDesc = document.createElement("div");
            subDesc.className = "pkg-sub-desc";
            subDesc.textContent = di.description;
            subItem.appendChild(subDesc);
          }

          if (di.fileList && di.fileList.length > 0) {
            const flInfo = document.createElement("div");
            flInfo.className = "pkg-sub-files";
            flInfo.textContent = `Archivos: ${di.fileList.map(f => `${f.filename} (${formatSize(f.size)})`).join(", ")}`;
            subItem.appendChild(flInfo);
          }

          body.appendChild(subItem);
        });

        card.appendChild(body);
      }

      container.appendChild(card);
    });
  } catch (e) {
    showToast(`Error al cargar la lista de paquetes de recursos: ${e}`, "error");
  }
}

// Ventana modal de instalación

function showModal() {
  document.getElementById("modal-install-overlay").style.display = "flex";
  document.getElementById("modal-install-method").value = "local";
  document.getElementById("modal-install-path").value = "";
  document.getElementById("modal-install-url").value = "";
  updateModalInstallMethodUI();
}

function hideModal() {
  document.getElementById("modal-install-overlay").style.display = "none";
}

function updateModalInstallMethodUI() {
  const method = document.getElementById("modal-install-method").value;
  const pathGroup = document.getElementById("modal-install-path").closest(".form-group");
  const urlGroup = document.getElementById("modal-install-url").closest(".form-group");
  pathGroup.style.display = method === "local" ? "" : "none";
  urlGroup.style.display = method === "online" ? "" : "none";
}

async function browseFile() {
  try {
    const path = await invoke("browse_file");
    if (path) {
      document.getElementById("modal-install-path").value = path;
      addLog(`Archivo local seleccionado: ${path}`);
    }
  } catch (e) {
    addLog(`Error al seleccionar el archivo: ${e}`);
  }
}

async function doInstall() {
  const btn = document.getElementById("btn-modal-install");
  btn.disabled = true;

  try {
    const method = document.getElementById("modal-install-method").value;
    const path = document.getElementById("modal-install-path").value.trim();
    const url = document.getElementById("modal-install-url").value.trim();

    if (method === "local" && path) {
      hideModal();
      showToast("Instalando el paquete de recursos local...", "info");
      addLog(`Instalar paquete de recursos local: ${path}`);
      const pkg = await invoke("install_resource_from_path", { sourcePath: path });
      showToast(`Instalación correcta: ${pkg.manifest.name || pkg.name}`, "success");
      addLog(`Paquete de recursos instalado correctamente: ${pkg.manifest.name || pkg.name}`);
      await loadPackages();
    } else if (method === "online" && url) {
      hideModal();
      showToast("Descargando e instalando...", "info");
      addLog(`Descargar paquete de recursos: ${url}`);
      const pkg = await invoke("install_resource_from_url", { url });
      showToast(`Instalación correcta: ${pkg.manifest.name || pkg.name}`, "success");
      addLog(`Paquete de recursos descargado e instalado correctamente: ${pkg.manifest.name || pkg.name}`);
      await loadPackages();
    } else {
      showToast(method === "local" ? "Selecciona un archivo de paquete de recursos local" : "Introduce la URL remota", "error");
      btn.disabled = false;
    }
  } catch (e) {
    showToast(`Error de instalación: ${e}`, "error");
    addLog(`Error al instalar el paquete de recursos: ${e}`);
  }
  btn.disabled = false;
}



// Actualizar la lista

export function refreshPackages() {
  addLog("Actualizar la lista de paquetes de recursos");
  loadPackages();
}

// Vinculación de eventos

export function initPackagesPage() {
  document.getElementById("btn-pkg-install").addEventListener("click", showModal);
  document.getElementById("btn-pkg-refresh").addEventListener("click", refreshPackages);
  document.getElementById("btn-pkg-open-folder").addEventListener("click", async () => {
    try { await invoke("open_resource_dir", { name: "" }); }
    catch (e) { addLog(`Error al abrir la carpeta de recursos: ${e}`); }
  });

  // Modal
  document.getElementById("btn-modal-close").addEventListener("click", hideModal);
  document.getElementById("btn-modal-cancel").addEventListener("click", hideModal);
  document.getElementById("modal-install-method").addEventListener("change", updateModalInstallMethodUI);
  document.getElementById("btn-modal-browse-file").addEventListener("click", browseFile);
  document.getElementById("btn-modal-install").addEventListener("click", doInstall);
  document.getElementById("modal-install-overlay").addEventListener("click", (e) => {
    if (e.target === e.currentTarget) hideModal();
  });

  // Delete modal
  document.getElementById("btn-delete-close").addEventListener("click", hideDeleteConfirm);
  document.getElementById("btn-delete-cancel").addEventListener("click", hideDeleteConfirm);
  document.getElementById("btn-delete-confirm").addEventListener("click", confirmDeletePackage);
  document.getElementById("modal-delete-overlay").addEventListener("click", (e) => {
    if (e.target === e.currentTarget) hideDeleteConfirm();
  });

  // Detail modal
  document.getElementById("btn-modal-detail-close").addEventListener("click", () => {
    document.getElementById("modal-detail-overlay").style.display = "none";
  });
  document.getElementById("btn-modal-detail-close-bottom").addEventListener("click", () => {
    document.getElementById("modal-detail-overlay").style.display = "none";
  });
  document.getElementById("modal-detail-overlay").addEventListener("click", (e) => {
    if (e.target === e.currentTarget) {
      document.getElementById("modal-detail-overlay").style.display = "none";
    }
  });
}
