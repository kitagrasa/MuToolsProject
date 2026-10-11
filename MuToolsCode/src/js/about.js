// Página Acerca de

function initDisclaimerModal() {
  const overlay = document.getElementById("modal-disclaimer-overlay");
  if (!overlay) return;

  const btnOpen = document.getElementById("btn-about-disclaimer");
  const btnClose = document.getElementById("btn-disclaimer-close");
  const btnCloseBottom = document.getElementById("btn-disclaimer-close-bottom");
  const linkHome = document.getElementById("link-home-disclaimer");

  function open() { overlay.style.display = "flex"; }
  function close() { overlay.style.display = "none"; }

  if (btnOpen) btnOpen.addEventListener("click", open);
  if (btnClose) btnClose.addEventListener("click", close);
  if (btnCloseBottom) btnCloseBottom.addEventListener("click", close);
  if (linkHome) linkHome.addEventListener("click", open);

  overlay.addEventListener("click", (e) => {
    if (e.target === overlay) close();
  });
}

// Registro de cambios

const CHANGELOG_DATA = [
  {
    version: "Tauri-v1.1.0",
    date: "2026-08-31",
    items: [
      { text: "Corregida la optimización de startupImageDir en la versión V6", type: "fix" }
    ]
  },
  {
    version: "Tauri-v1.0.0",
    date: "2026-08-28",
    items: [
      { text: "Publicada la primera versión estable de Tauri", type: "plain" },
      { text: "Se descarta el método de optimización del cortafuegos", type: "breaking" },
      { text: "Añadido el aviso legal", type: "add" },
      { text: "Añadido el registro de cambios", type: "add" },
      { text: "Corregida la optimización de system.vdi en la versión V6", type: "fix" },
      { text: "Corregidos varios problemas del apartado de optimización del cortafuegos", type: "fix" },
      { text: "Reajustada la clasificación de la lista de optimizaciones", type: "opt" },
      { text: "Eliminado el código muerto y los recursos inservibles del proyecto", type: "opt" },
      { text: "Optimizado el diseño de la interfaz de gestión de paquetes de recursos", type: "opt" }
    ]
  },
  {
    version: "Tauri-v0.1.0",
    date: "2026-08-25",
    items: [
      { text: "Publicada la primera versión preliminar de Tauri", type: "plain" },
      { text: "Diseño de interfaz gráfica completamente nuevo", type: "add" },
      { text: "Permite descargar en línea el instalador de MuMu", type: "add" },
      { text: "Compatible con la versión MuMu12-V6", type: "add" },
      { text: "Añadido el apartado de ayuda de MuMu para consultar la documentación y contactar con soporte", type: "add" },
      { text: "Añadidas la frase del día y algunas sorpresas ocultas", type: "add" }
    ]
  },
  {
    version: "Python-v5.4",
    date: "0000-00-00",
    items: [
      { text: "Última versión en Python", type: "plain" },
      { text: "Eliminado el complemento de control remoto de MuMu del paquete de recursos", type: "breaking" },
      { text: "Eliminada la función Copia de seguridad Multinstancia Pro, que ya no recibe soporte", type: "breaking" },
      { text: "El módulo de paquetes de recursos ha cambiado y ya no es compatible con versiones anteriores.", type: "breaking" },
      { text: "Añadida la dirección del sitio web oficial de UU Remote", type: "add" },
      { text: "Permite guardar y cargar el estado de los paquetes de recursos", type: "add" },
      { text: "Añadida una herramienta de escaneo de códigos QR", type: "add" },
      { text: "La línea de comandos permite cargar automáticamente el archivo de configuración", type: "add" },
      { text: "Optimizada la visualización de la interfaz de gestión de recursos", type: "opt" },
      { text: "Eliminados algunos avisos innecesarios", type: "opt" },
      { text: "Optimizada la interfaz de gestión multinstancia de MuMu", type: "opt" }
    ]
  },
  {
    version: "Python-v5.3",
    date: "0000-00-00",
    items: [
      { text: "Añadido el tipo de canal de optimización opcional modo imitación Premium para 5.x", type: "add" },
      { text: "Añadido el parámetro -v a la línea de comandos", type: "add" },
      { text: "Añadida la detección automática de dominios duplicados al modificar el archivo hosts", type: "add" },
      { text: "MuTools ya tiene su propio icono nuevo", type: "add" },
      { text: "Corregido el fallo al desactivar la telemetría", type: "fix" },
      { text: "Corregido el error de permisos del registro", type: "fix" },
      { text: "Corregido el error de detección de rutas al iniciar", type: "fix" },
      { text: "Optimizada la compilación del programa para mejorar el rendimiento", type: "opt" },
      { text: "Optimizada la experiencia visual de la interfaz principal", type: "opt" },
      { text: "Optimizada la experiencia de uso de la pantalla de selección de optimizaciones, con marcas +-*", type: "opt" },
      { text: "El límite configurado para los archivos de registro pasa a 10 MB / 90 d", type: "opt" },
      { text: "Optimizada la lógica de detección de la versión de MuMu12", type: "opt" },
      { text: "Ajustado el aviso que se muestra cuando MuMu12 no está instalado", type: "opt" },
      { text: "Ajustada la visualización del número de versión", type: "opt" },
      { text: "Optimizada la carga de módulos", type: "opt" }
    ]
  },
  {
    version: "Python-v5.2",
    date: "0000-00-00",
    items: [
      { text: "Añadido el paquete de optimización system.vdi", type: "add" },
      { text: "Compatible con la actualización automática", type: "add" },
      { text: "Añadida la opción de optimización para desactivar la telemetría de MuMu", type: "add" },
      { text: "Añadido el aviso de actualización tras actualizar", type: "add" },
      { text: "Actualizado el paquete de optimización data de la versión V5.0", type: "add" },
      { text: "Añadido el tipo SUCCESS para los avisos y los registros", type: "add" },
      { text: "Corregido el fallo de decodificación de cadenas de la línea de comandos en algunos casos", type: "fix" },
      { text: "Corregidos los avisos y registros de error inesperados tras modificar el archivo hosts correctamente", type: "fix" },
      { text: "Corregido un aviso erróneo al restaurar el archivo hosts", type: "fix" },
      { text: "Optimizados los avisos al modificar startupImageDir", type: "opt" },
      { text: "Optimizadas las opciones de optimización con un clic y de revertir la optimización con un clic", type: "opt" },
      { text: "Optimizada la visualización de la salida del registro de cambios", type: "opt" },
      { text: "Reescrito el código de guardado de los ajustes para mejorar el rendimiento", type: "opt" },
      { text: "Optimizado el orden de detección de los mensajes de salida y los registros", type: "opt" },
      { text: "Simplificado el código antidepuración", type: "opt" },
      { text: "La pantalla de selección de recursos ahora muestra avisos de error más detallados", type: "opt" },
      { text: "Optimizada la eficiencia de reconocimiento y lectura de los parámetros de la línea de comandos", type: "opt" }
    ]
  },
  {
    version: "Python-v5.1",
    date: "0000-00-00",
    items: [
      { text: "Añadidos los avisos emergentes de la interfaz gráfica", type: "add" },
      { text: "Añadido el enlace de descarga de la versión internacional de MuMu", type: "add" },
      { text: "La línea de comandos permite ejecutar optimizaciones", type: "add" },
      { text: "Compatible con la obtención automática de bitWidth", type: "add" },
      { text: "Corregido el problema por el que la opción 9 no permitía ver el registro de cambios", type: "fix" },
      { text: "Corregido el cierre inesperado al crear rutas de forma automática", type: "fix" },
      { text: "Corregida la detección recursiva infinita del número de versión cuando el emulador aún no está instalado", type: "fix" },
      { text: "Si los parámetros de la línea de comandos son incorrectos se mostrará un aviso en lugar de iniciarse con normalidad", type: "opt" },
      { text: "Optimizado el acuerdo de usuario", type: "opt" }
    ]
  },
  {
    version: "Python-v5.0",
    date: "0000-00-00",
    items: [
      { text: "Añadida la modificación automática de MuMuPlayerUpdater.exe", type: "add" },
      { text: "Añadida la optimización para la versión MuMu 5.x", type: "add" },
      { text: "Añadido el sitio web oficial de versiones anteriores de MuMu", type: "add" },
      { text: "Añadida la herramienta de extracción de paquetes de recursos por línea de comandos", type: "add" },
      { text: "Añadida la búsqueda automática de actualizaciones", type: "add" },
      { text: "Añadido el elemento de detección vbox-env", type: "add" },
      { text: "Añadida la versión de 32 bits", type: "add" },
      { text: "Actualizados los instaladores sin conexión 5.2 y 5.5", type: "add" },
      { text: "Actualizados los instaladores en línea 5.0.1 y 5.0.2", type: "add" },
      { text: "Actualizada la versión del software UU Remote", type: "add" },
      { text: "Actualizada la herramienta de detección ColaBoxChecker a MuMuChecker", type: "add" },
      { text: "Añadida la detección de la versión del emulador MuMu", type: "add" },
      { text: "La obtención del número de versión permite leer el archivo de configuración", type: "add" },
      { text: "Integradas las funciones de la versión de prueba de Mupro", type: "add" },
      { text: "Añadida la función de registro detallado", type: "add" },
      { text: "Habilitada la depuración DEV; la línea de comandos permite activar el modo de depuración", type: "add" },
      { text: "Corregido el fallo al eliminar startupImage cuando se repetía la optimización", type: "fix" },
      { text: "Corregido el fallo de la versión 5.x en la tarea do-2-1", type: "fix" },
      { text: "Corregido el cierre inesperado al leer el registro sin gestionar los errores", type: "fix" },
      { text: "Corregida la lectura del registro en la versión 5.x", type: "fix" },
      { text: "Corregida la lectura de MuMuManager.exe en la versión 5.x", type: "fix" },
      { text: "Corregido el icono del paquete de optimización overlay", type: "fix" },
      { text: "Corregidos los fallos de detección e instalación de recursos en algunas versiones de Windows 11 22H2", type: "fix" },
      { text: "Corregido el error que identificaba los paquetes de recursos nuevos como dañados", type: "fix" },
      { text: "Corregido el error al leer el registro provocado por la redirección", type: "fix" },
      { text: "Optimizada la descarga y la ejecución; se lanza el paquete todo en uno con los recursos incluidos, listos para usar tras descomprimir", type: "opt" },
      { text: "Optimizada la instalación de paquetes de recursos, ahora permite copiar la ruta desde la pestaña Seguridad", type: "opt" },
      { text: "Optimizada la dirección de solicitud de actualizaciones", type: "opt" },
      { text: "Optimizada la lógica de uso de la interfaz, eliminando la necesidad de pulsar Enter repetidamente", type: "opt" },
      { text: "Optimizada la visualización de los mensajes de salida, con soporte para varios colores", type: "opt" },
      { text: "Optimizado el User-Agent enviado al servidor", type: "opt" },
      { text: "Simplificada la longitud del contenido introducido en la página de gestión de paquetes de recursos", type: "opt" },
      { text: "Eliminado el mensaje [Configuración guardada] que se mostraba repetidamente al cambiar la configuración", type: "opt" }
    ]
  },
  {
    version: "Python-v4.1",
    date: "0000-00-00",
    items: [
      { text: "Añadidos los ajustes de la aplicación", type: "add" },
      { text: "Añadida la función de instalación de componentes de MuMu", type: "add" },
      { text: "Añadida la función de comprobación del entorno de ejecución", type: "add" },
      { text: "Añadida la modificación de la lista de paquetes de recursos", type: "add" },
      { text: "Añadida la configuración global de datos", type: "add" },
      { text: "Añadida la descripción del paquete de optimización data", type: "add" },
      { text: "Añadida la limpieza con un clic de multinstancia (datos de MuMu)", type: "add" },
      { text: "Corregidos varios errores causados por rutas no válidas", type: "fix" },
      { text: "Corregida la ruta de 7z al importar copias de seguridad y copias comprimidas", type: "fix" },
      { text: "Corregidos varios errores de Copia de seguridad Pro", type: "fix" },
      { text: "Eliminado el aviso de la ruta de instalación", type: "fix" },
      { text: "Optimizada la visualización de la salida", type: "opt" },
      { text: "Optimizado el código duplicado", type: "opt" },
      { text: "Optimizadas las descripciones de los distintos botones", type: "opt" },
      { text: "Optimizados los archivos y carpetas innecesarios que se creaban", type: "opt" }
    ]
  },
  {
    version: "Python-v4.0",
    date: "0000-00-00",
    items: [
      { text: "Añadidas la copia de seguridad con un clic y la copia de seguridad mejorada", type: "add" },
      { text: "Añadido el diseño modular de paquetes de recursos", type: "add" },
      { text: "Añadida la opción de revertir la optimización con un clic", type: "add" },
      { text: "Añadidos los elementos de optimización opcionales", type: "add" },
      { text: "Añadida la protección del programa contra manipulaciones", type: "add" },
      { text: "Añadidos más métodos y acciones de optimización", type: "add" },
      { text: "Actualizado y reestructurado el código desastroso anterior", type: "add" },
      { text: "Actualizados los recursos relacionados con MuMu a la versión del 2025-04-27 y corregidos algunos errores curiosos de versiones anteriores", type: "add" },
      { text: "Corregido el error de exportación causado por el nombre del emulador; basta con cambiar el nombre del emulador para volver a la normalidad", type: "fix" },
      { text: "Optimizado el diseño de la interfaz", type: "opt" },
      { text: "Optimizado el modo de publicación de paquetes de recursos", type: "opt" },
      { text: "Optimizado el anuncio que se publica al actualizar la versión", type: "opt" },
      { text: "Optimizado el modo de actualización (debido al cambio del servicio de almacenamiento en la nube, ya no se admite la descarga automática de paquetes de actualización)", type: "opt" },
      { text: "Optimizada la modificación del archivo hosts, resolviendo los fallos al modificar y las entradas duplicadas", type: "opt" }
    ]
  }
];

const TYPE_CLASS = {
  add: "changelog-add",
  fix: "changelog-fix",
  opt: "changelog-opt",
  breaking: "changelog-breaking",
  plain: ""
};

let changelogIndex = 0;
let changelogItems = [];

/** Crea el nodo DOM de una versión a partir de los datos JSON */
function buildChangelogItem(entry) {
  const item = document.createElement("div");
  item.className = "changelog-item";

  const version = document.createElement("div");
  version.className = "changelog-version";
  version.textContent = entry.version;

  const date = document.createElement("div");
  date.className = "changelog-date";
  date.textContent = entry.date;

  const list = document.createElement("ul");
  list.className = "changelog-list";

  for (const li of entry.items) {
    const el = document.createElement("li");
    el.textContent = li.text;
    if (li.type && TYPE_CLASS[li.type]) {
      el.className = TYPE_CLASS[li.type];
    }
    list.appendChild(el);
  }

  item.appendChild(version);
  item.appendChild(date);
  item.appendChild(list);
  return item;
}

/** Renderiza los datos integrados en el contenedor */
function loadChangelogData() {
  const container = document.getElementById("about-changelog");
  if (!container) return;

  changelogItems = CHANGELOG_DATA.map((entry) => buildChangelogItem(entry));

  if (changelogItems.length > 0) {
    container.innerHTML = "";
    container.appendChild(changelogItems[0].cloneNode(true));
  }
}

function updateChangelogDisplay() {
  const container = document.getElementById("about-changelog");
  const counter = document.getElementById("changelog-counter");
  const prevBtn = document.getElementById("btn-changelog-prev");
  const nextBtn = document.getElementById("btn-changelog-next");

  if (!container || changelogItems.length === 0) return;

  container.innerHTML = "";
  const item = changelogItems[changelogIndex];
  container.appendChild(item.cloneNode(true));

  if (counter) {
    counter.textContent = `${changelogIndex + 1} / ${changelogItems.length}`;
  }
  if (prevBtn) {
    prevBtn.disabled = changelogIndex === 0;
  }
  if (nextBtn) {
    nextBtn.disabled = changelogIndex >= changelogItems.length - 1;
  }
}

function initChangelogModal() {
  const overlay = document.getElementById("modal-changelog-overlay");
  if (!overlay) return;

  const btnOpen = document.getElementById("btn-about-changelog");
  const btnClose = document.getElementById("btn-changelog-close");
  const btnCloseBottom = document.getElementById("btn-changelog-close-bottom");

  function open() {
    changelogIndex = 0;
    updateChangelogDisplay();
    overlay.style.display = "flex";
  }
  function close() { overlay.style.display = "none"; }

  if (btnOpen) btnOpen.addEventListener("click", open);
  if (btnClose) btnClose.addEventListener("click", close);
  if (btnCloseBottom) btnCloseBottom.addEventListener("click", close);

  overlay.addEventListener("click", (e) => {
    if (e.target === overlay) close();
  });

  // Anterior / Siguiente
  const prevBtn = document.getElementById("btn-changelog-prev");
  const nextBtn = document.getElementById("btn-changelog-next");
  if (prevBtn) {
    prevBtn.addEventListener("click", () => {
      if (changelogIndex > 0) {
        changelogIndex--;
        updateChangelogDisplay();
      }
    });
  }
  if (nextBtn) {
    nextBtn.addEventListener("click", () => {
      if (changelogIndex < changelogItems.length - 1) {
        changelogIndex++;
        updateChangelogDisplay();
      }
    });
  }
}

export function initAboutPage() {
  initDisclaimerModal();
  loadChangelogData();
  initChangelogModal();
}
