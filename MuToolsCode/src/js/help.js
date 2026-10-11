// Página de ayuda

// Mapa de URL de las páginas remotas
const helpRemoteUrls = {
  official: "https://mumu.163.com/help/win/",
  support: "https://mumu.163.com/redirect/customerservice"
};

// Registro de los iframe ya cargados
const helpLoadedIframes = { official: false, support: false };

const helpFaqData = [
  {
    category: "Preguntas frecuentes",
    question: "¿Qué hago si la instalación falla?",
    answer: `1. Consulta el mensaje de error en la página <strong>Registro</strong><br>
2. Asegúrate de que la carpeta de instalación tenga suficiente espacio libre<br>
3. Confirma que la aplicación tiene permisos de administrador<br>
4. Prueba a instalar con el paquete y el instalador oficiales<br>
5. Contacta con el <strong>servicio de atención oficial</strong> para obtener ayuda`
  },
  {
    category: "Preguntas frecuentes",
    question: "¿Por qué se necesitan permisos de administrador?",
    answer: `Muchas de las <strong>funciones de MuTools</strong> necesitan permisos de administrador para funcionar correctamente<br>
Salvo casos excepcionales, concede siempre permisos de administrador al programa`
  },
  {
    category: "Preguntas frecuentes",
    question: "¿Qué hago si el emulador MuMu se bloquea o se cierra solo con frecuencia?",
    answer: `1. Consulta la documentación oficial para ver si existe una solución<br>
2. Actualiza MuMu a la última versión<br>
2. Vuelve a una versión anterior de MuMu<br>
4. Contacta con el <strong>servicio de atención oficial</strong> para obtener ayuda`
  },
  {
    category: "Preguntas frecuentes",
    question: "¿Qué hago si la instalación del emulador MuMu se queda en el 98%?",
    answer: `La barra de progreso de la instalación es simulada, quedarse en el 98% significa que la instalación no ha terminado`
  }
];

const helpCategories = [...new Set(helpFaqData.map(item => item.category))];

function filterByCategory(category) {
  document.querySelectorAll(".help-category-tag").forEach(t => {
    t.classList.toggle("active", t.dataset.category === category);
  });
  const searchInput = document.getElementById("help-search-input");
  if (searchInput) searchInput.value = "";
  document.getElementById("help-search-clear").style.display = "none";
  document.getElementById("help-no-results").style.display = "none";

  if (category === "all") {
    renderFaqList(helpFaqData);
  } else {
    const filtered = helpFaqData.filter(item => item.category === category);
    renderFaqList(filtered);
  }
}

function performSearch(query) {
  const results = helpFaqData.filter(item => {
    return item.question.toLowerCase().includes(query) ||
      item.category.toLowerCase().includes(query) ||
      item.answer.toLowerCase().includes(query);
  });

  document.getElementById("help-no-results").style.display = results.length === 0 ? "" : "none";

  if (results.length > 0) {
    renderFaqList(results, query);
  }
}

function renderFaqList(items, highlightQuery) {
  const container = document.getElementById("help-faq-list");
  if (!container) return;
  container.innerHTML = "";

  items.forEach(item => {
    const faqItem = document.createElement("div");
    faqItem.className = "help-faq-item";

    const question = document.createElement("div");
    question.className = "help-faq-question";

    const qText = document.createElement("span");
    qText.innerHTML = highlightQuery
      ? highlightText(item.question, highlightQuery)
      : item.question;
    question.appendChild(qText);

    const rightGroup = document.createElement("span");
    rightGroup.className = "help-faq-right";

    const catTag = document.createElement("span");
    catTag.className = "help-faq-category";
    catTag.textContent = item.category;
    rightGroup.appendChild(catTag);

    const arrow = document.createElement("span");
    arrow.className = "help-faq-arrow";
    arrow.textContent = "▼";
    rightGroup.appendChild(arrow);

    question.appendChild(rightGroup);

    question.addEventListener("click", () => {
      faqItem.classList.toggle("open");
    });

    const answer = document.createElement("div");
    answer.className = "help-faq-answer";
    answer.innerHTML = highlightQuery
      ? highlightText(item.answer, highlightQuery)
      : item.answer;

    faqItem.appendChild(question);
    faqItem.appendChild(answer);
    container.appendChild(faqItem);
  });
}

function highlightText(text, query) {
  if (!query) return text;
  const escaped = query.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
  const regex = new RegExp(`(${escaped})`, 'gi');
  return text.replace(regex, '<span class="help-highlight">$1</span>');
}

export function initHelpPage() {
  // Help tab switching
  document.querySelectorAll(".help-tab").forEach(tab => {
    tab.addEventListener("click", () => {
      const tabName = tab.dataset.helpTab;
      document.querySelectorAll(".help-tab").forEach(t => t.classList.remove("active"));
      tab.classList.add("active");
      document.querySelectorAll(".help-panel").forEach(p => p.classList.remove("active"));
      document.getElementById(`help-panel-${tabName}`).classList.add("active");
    });
  });

  // Render categories
  const categoriesEl = document.getElementById("help-categories");
  if (categoriesEl) {
    const allTag = document.createElement("button");
    allTag.className = "help-category-tag active";
    allTag.textContent = "Todas";
    allTag.dataset.category = "all";
    allTag.addEventListener("click", () => filterByCategory("all"));
    categoriesEl.appendChild(allTag);

    helpCategories.forEach(cat => {
      const tag = document.createElement("button");
      tag.className = "help-category-tag";
      tag.textContent = cat;
      tag.dataset.category = cat;
      tag.addEventListener("click", () => filterByCategory(cat));
      categoriesEl.appendChild(tag);
    });
  }

  // Render FAQ list
  renderFaqList(helpFaqData);

  // Search
  const searchInput = document.getElementById("help-search-input");
  const searchClear = document.getElementById("help-search-clear");
  if (searchInput) {
    searchInput.addEventListener("input", () => {
      const query = searchInput.value.trim().toLowerCase();
      searchClear.style.display = query ? "" : "none";
      if (query) {
        document.querySelectorAll(".help-category-tag").forEach(t => {
          t.classList.toggle("active", t.dataset.category === "all");
        });
        performSearch(query);
      } else {
        renderFaqList(helpFaqData);
        document.getElementById("help-no-results").style.display = "none";
      }
    });
  }
  if (searchClear) {
    searchClear.addEventListener("click", () => {
      searchInput.value = "";
      searchClear.style.display = "none";
      renderFaqList(helpFaqData);
      document.getElementById("help-no-results").style.display = "none";
      document.querySelectorAll(".help-category-tag").forEach(t => {
        t.classList.toggle("active", t.dataset.category === "all");
      });
    });
  }

  // Carga diferida de las páginas remotas
  document.querySelectorAll(".help-open-btn").forEach(btn => {
    btn.addEventListener("click", () => {
      const type = btn.dataset.helpOpen;   // "official" | "support"
      const mode = btn.dataset.helpMode;   // "embed" | "browser"
      const url = helpRemoteUrls[type];
      if (!url) return;

      if (mode === "browser") {
        window.__TAURI__.opener.openUrl(url).catch(err => {
          console.error("No se puede abrir en el navegador:", err);
        });
      } else if (mode === "embed") {
        const placeholder = document.getElementById(`help-placeholder-${type}`);
        const iframeWrap = document.getElementById(`help-iframe-wrap-${type}`);
        const iframe = document.getElementById(`help-iframe-${type}`);
        if (placeholder) placeholder.style.display = "none";
        if (iframeWrap) iframeWrap.style.display = "";
        if (iframe && !helpLoadedIframes[type]) {
          iframe.src = url;
          helpLoadedIframes[type] = true;
        }
      }
    });
  });
}
