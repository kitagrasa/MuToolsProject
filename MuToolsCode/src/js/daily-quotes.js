import { showToast } from "./toast.js";

const dailyQuotes = [
  "La primera versión de MuTools se desarrolló en Python y se publicó por primera vez el 2024-10-19",
  "pipicat es un gatito, pero ya se marchó al cielo de los gatos",
  "pipicat613 es un novato",
  "La nueva versión de MuTools se reescribió con Tauri V2",
  "pipicat613 es un NPC de Earth Online",
  "¡Ah! En realidad hay un huevo de pascua en la página Acerca de",
  "¡Nunca montes el equipo, depures ni escribas código por la noche!",
  "Acuérdate de comer bien antes de escribir código",
  "MuMu12-V4 vendió skins, pero no conseguí la edición limitada T^T",
  "El archivo de configuración Overlay oficial de MuMu12-V4 tenía 'flase' en lugar de 'false'...",
  "En realidad, MuTools no era más que un archivo por lotes .bat",
  "Arriba, arriba, abajo, abajo, izquierda, derecha, BABA",
  "El primer servidor de actualizaciones de MuTools escribía 'updata' en lugar de 'update'",
  "¿Vamos al muelle a por unas patatas fritas?",
  "¡Suerte de hoy +100!",
  "¿Quieres venir a mi restaurante y pedir un coche?",
  "Los chinos pueden hacer import antigravity",
  "import this",
  "The quick brown fox jumps over the lazy dog.",
  "TurboWarp = Scratch Pro Max",
  "p'y't'hon = pitón = ?",
  "True + True = 2",
  "Ctrl+S Ctrl+S! Ctrl+S!!!!!",
  "Reiniciar resuelve el 60 %, reinstalar el 30 % y el 10 % restante exige cambiar de equipo",
  "Venta al por mayor de montañas de código shit AAA",
  "Mojibake por todas partes y memoria sin inicializar",
  "La versión v0.1.0-Preview de MuTools-Tauri en realidad se hizo en solo 5 días",

  "La IA dice: mi cartera es como una cebolla, cada vez que la abro me hace llorar a mares",
  "La IA dice: no soy perezoso, solo estoy activando el modo de ahorro de energía",
  "La IA dice: antes creía que el dinero lo compraba todo, luego descubrí que no me llega el dinero",
  "La IA dice: mi cama es un lugar mágico, en ella siempre encuentro el sueño que había perdido",
  "La IA dice: lo de adelgazar suelo pensarlo siempre con la barriga llena",
  "La IA dice: no estoy gordo, solo estoy delgado de forma poco evidente",
  "La IA dice: mi cerebro es como un navegador, con demasiadas pestañas abiertas y además colgado",
  "La IA dice: la vida es como una caja de bombones, nunca sabes cuál está amargo",
  "La IA dice: cada día salto entre «hoy empiezo a esforzarme» y «mañana ya veremos»",
  "La IA dice: la batería de mi móvil dura más que mi fuerza de voluntad",
  "La IA dice: no es procrastinación, solo espero la inspiración (en realidad espero la fecha límite)",
  "La IA dice: mi sueño es dormir, la realidad es ir a trabajar",
  "La IA dice: mi memoria es como la de un pez dorado, a los siete segundos ya olvidé qué iba a hacer",
  "La IA dice: no es que no quiera hacer ejercicio, solo espero el momento adecuado (como el fin del mundo)",
  "La IA dice: mi cartera es como un agujero negro, el dinero entra y no vuelve a salir",
  "La IA dice: cada día lucho entre «hoy me acuesto pronto» y «cinco minutos más de móvil»",
  "La IA dice: mi cerebro es como un cubo de basura, lleno de todo tipo de cosas inútiles",
  "La IA dice: no es que no me esfuerce, solo espero un milagro",
  "La IA dice: mi móvil es como un duendecillo pegajoso, lo suelto y ya está sonando",
  "La IA dice: cada día doy vueltas entre «hoy empiezo la dieta» y «mañana ya veremos»",
  "La IA dice: mi memoria es como un cubo agujereado, todo lo que meto se escapa",
  "La IA dice: no es que no quiera estudiar, solo espero el ambiente adecuado (como cuando no tengo el móvil)",
  "La IA dice: mi cartera es como un globo, se pincha con solo soplar",
  "La IA dice: cada día dudo entre «hoy madrugo» y «cinco minutos más de sueño»",
  "La IA dice: mi cerebro es como un laberinto, entras y ya no sales",
  "La IA dice: no es que no haga ejercicio, solo espero el tiempo adecuado (como cuando no llueve)",
  "La IA dice: mi móvil es como un despertador, suena y no hay quien lo pare",
  "La IA dice: cada día me debato entre «hoy empiezo a ahorrar» y «mañana ya veremos»",
  "La IA dice: mi memoria es como un colador, todo se filtra",
  "La IA dice: no es que no quiera trabajar, solo espero el momento adecuado (como cuando me jubile)",

  "La IA dice: mi cartera es como una cebolla, cada vez que la abro me dan ganas de llorar",
  "La IA dice: no estoy distraído, le estoy haciendo una actualización de sistema al cerebro",
  "La IA dice: adelgazar es muy fácil, lo difícil es tener que volver a adelgazar cada día",
  "La IA dice: mi madre dice que soy vago y no lo acepto: puedo quedarme tumbado sin moverme todo el día, eso se llama disciplina",
  "La IA dice: sonó el despertador, negociamos intensamente y al final cedió: lo apagué",
  "La IA dice: creía que era especial, luego descubrí que todo el mundo piensa lo mismo",
  "La IA dice: la mayor mentira de la vida es «solo miro el móvil cinco minutos»",
  "La IA dice: no me pierdo, solo estoy explorando rutas desconocidas",
  "La IA dice: cada vez que juro acostarme pronto, el móvil parece enterarse y se vuelve mucho más divertido",
  "La IA dice: mis ahorros y mi línea del pelo van igual, cada vez dan más ansiedad",
  "La IA dice: los demás son valores con potencial; yo soy un valor con flotabilidad, siempre hacia abajo",
  "La IA dice: mi secreto para cocinar es: si se puede comer, vale; si está bueno, es un accidente",
  "La IA dice: he decidido ponerme en forma, empezando por sacarme un carné que casi no usaré",
  "La IA dice: mi vida es como la señal WiFi, aparece y desaparece, y encima se desconecta a menudo",
  "La IA dice: no es procrastinación, le estoy dando a las cosas el tiempo de fermentación necesario",
  "La IA dice: los demás viven de su talento; yo vivo de mi talento… para pedir comida a domicilio",
  "La IA dice: cada día mejoro, ayer no sabía quedarme en la cama hasta el mediodía y hoy ya sé",
  "La IA dice: tengo muy buena memoria, solo que no recuerdo lo que debería recordar",
  "La IA dice: creía que el dinero compraba la felicidad, luego vi que no me llega para comprarla",
  "La IA dice: no estoy gordo, la vida me ha alimentado demasiado bien",
  "La IA dice: cada vez que digo «solo un bocado», acabo bocado tras bocado hasta dejar el plato vacío",
  "La IA dice: mi ideal es ambicioso, la realidad es escuálida y yo estoy en medio, bien redondeado",
  "La IA dice: no soy antisocial, solo estoy ahorrando energía social",
  "La IA dice: cada día echo un pulso con la cama y la cama siempre gana",
  "La IA dice: creía que ya era maduro y aun así compro dos por lo del «segundo a mitad de precio»",
  "La IA dice: mis planes nunca alcanzan a los cambios, y los cambios no alcanzan a mis cambios de opinión de última hora",
  "La IA dice: no estoy vagueando, le estoy dando un toque de misterio al trabajo",
  "La IA dice: la vida de los demás es poesía y horizontes lejanos; la mía es humedad y horizontes lejanos, el paquete se mojó y aun así tengo que ir a por él",
  "La IA dice: juro que no volveré a trasnochar, salvo si hay móvil, hay internet y mañana no madrugo",
  "La IA dice: la vida es corta y hay que divertirse ya, así que elijo divertirme primero y lo amargo lo dejo para después",

  "La IA dice: le dije a mi jefe que dimitía, me dijo «piénsalo bien, fuera no es fácil encontrar trabajo» y le respondí «lo he pensado bien: me voy a ser jefe»",
  "La IA dice: mi meta en la vida es ganar suficiente dinero y luego pagar a alguien para que cumpla mis metas",
  "La IA dice: cada vez que quiero adelgazar, mi estómago celebra una reunión de urgencia y aprueba por unanimidad «otra comidita»",
  "La IA dice: tengo dos apps del tiempo en el móvil que nunca se ponen de acuerdo, y acabo creyendo que vivo en dos ciudades",
  "La IA dice: no soy vago, solo mantengo mis constantes vitales con el mínimo consumo de energía",
  "La IA dice: los demás gastan el sueldo en un mes; yo lo gasto en un día, llega a la cuenta, pago los créditos y ya anocheció",
  "La IA dice: intenté acostarme y levantarme pronto y descubrí que la única ventaja de madrugar es poder usar más rato el móvil",
  "La IA dice: le pregunté a mi madre por qué me trajo al mundo tan pobre y dijo que era para entrenar mi imaginación",
  "La IA dice: mi cartera es como una cebolla, en cuanto la abres te hace llorar",
  "La IA dice: juré empezar a ahorrar, pero el dinero dijo que no le gusta que lo guarden, que prefiere fluir libremente",
  "La IA dice: pagué un año de gimnasio, no me creció músculo, pero aprendí a negociar con los comerciales",
  "La IA dice: cuando como hot pot lo que más odio es que digan «qué cantidad comes», y le respondo: «tampoco me he comido tu fogón»",
  "La IA dice: mi vida es como la señal WiFi, a veces fuerte y a veces débil, y siempre hay quien quiere aprovecharse",
  "La IA dice: le dije al peluquero que solo me recortara un poco y lo entendió como «cámbiame de vida»",
  "La IA dice: compré muchos libros, no para leerlos, sino para que el conocimiento se filtre al cerebro a través de la almohada",
  "La IA dice: mi cocina es muy consistente, consistentemente mala, pero mi madre dice que sabe a casa",
  "La IA dice: antes de pesarme respiro hondo, no para pesar menos, sino para no asustar a la báscula",
  "La IA dice: ir a trabajar me pesa más que ir al cementerio, porque allí al menos no hay que fichar",
  "La IA dice: tuve un cactus, se murió, sospecho que fue de verme tan feo",
  "La IA dice: me dije «hoy voy a esforzarme de verdad» y me esforcé pasando todo el día con el móvil",
  "La IA dice: mi procrastinación está curada, mañana te cuento el método",
  "La IA dice: subí a la montaña con un amigo, dijo que las vistas de la cima eran buenas y al llegar descubrí que lo mejor seguían siendo las fotos del móvil",
  "La IA dice: bebo té con leche para animarme, luego me preocupa engordar, me desanimo y no queda otra que pedir otro vaso",
  "La IA dice: en mi galería no abundan los selfis, sino las capturas de contraseñas y los números de seguimiento de paquetes",
  "La IA dice: dije que me acostaría pronto, el móvil dijo «no, no quieres» y le creí",
  "La IA dice: mi nivel de inglés es muy estable, establemente solo sé decir «sorry» y «how much»",
  "La IA dice: me apunté al gimnasio para que los demás crean que me esfuerzo, cuando en realidad estoy tumbado en casa",
  "La IA dice: mi mayor éxito en la vida ha sido evitar con éxito todas las oportunidades de triunfar",
  "La IA dice: le dije a un amigo que iba a montar un negocio, me preguntó de qué y respondí «de comer tierra en todas sus variantes»",
  "La IA dice: cada vez que quiero ahorrar, todo lo del carrito me escribe a la vez: amo, ¿ya no nos quieres?",

];

export function getDailyQuote() {
  const index = Math.floor(Math.random() * dailyQuotes.length);
  return dailyQuotes[index];
}

// Huevo de pascua de la página Acerca de
let versionClickCount = 0;
let versionClickTimer = null;

export function onVersionClick() {
  versionClickCount++;
  if (versionClickTimer) {
    clearTimeout(versionClickTimer);
  }
  versionClickTimer = setTimeout(() => {
    versionClickCount = 0;
  }, 2000);

  if (versionClickCount >= 6) {
    versionClickCount = 0;
    if (versionClickTimer) {
      clearTimeout(versionClickTimer);
      versionClickTimer = null;
    }
    const roll = Math.random();
    if (roll < 0.05) {
      showToast("Ciallo～(∠・ω< )⌒★", "info");
    } else {
      showToast("Ciallo～(∠・ω< )⌒☆", "info");
    }
  }
}