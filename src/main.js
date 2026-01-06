const { listen } = window.__TAURI__.event;
let lastSearchedArtist = "";

document.addEventListener('DOMContentLoaded', async () => {
    
    const titleLabel = document.getElementById('title');
    const artistLabel = document.getElementById('artist');
    const coverImg = document.getElementById('cover');
    const container = document.getElementById('widget-container');

    // ARRASTRE
    container.addEventListener('mousedown', (e) => {
        if (e.button === 0) {
            window.__TAURI__.window.getCurrentWindow().startDragging();
        }
    });

    // ESCUCHAR AL BACKEND
    await listen('media-update', async (event) => {
        const data = event.payload; 
        
        // --- TEXTOS ---
        // Usamos una propiedad personalizada (dataset) para guardar el texto original
        // y así poder comparar limpiamente sin que el texto duplicado nos confunda.
        
        if (titleLabel.dataset.originalText !== data.title) {
            titleLabel.dataset.originalText = data.title;
            verificarDesbordamiento(titleLabel, data.title);
        }

        if (artistLabel.dataset.originalText !== data.artist) {
            artistLabel.dataset.originalText = data.artist;
            verificarDesbordamiento(artistLabel, data.artist);
        }

        // --- PORTADA ---
        if (data.cover) {
            if (coverImg.src !== data.cover) {
                coverImg.src = data.cover;
                lastSearchedArtist = ""; 
            }
        } else {
            if (data.artist !== lastSearchedArtist && data.artist !== "Escuchando...") {
                lastSearchedArtist = data.artist;
                const artistImage = await buscarFotoArtista(data.artist);
                if (artistImage) {
                    coverImg.src = artistImage;
                }
            }
        }
    });
});

// --- LA FUNCIÓN MAGICA PARA EL BUCLE SIN ESPACIOS ---
function verificarDesbordamiento(elemento, textoOriginal) {
    // 1. Reseteamos el elemento al estado base para medir
    elemento.classList.remove('scrolling');
    elemento.innerHTML = textoOriginal; 
    
    // 2. Medimos
    const anchoContenedor = elemento.parentElement.clientWidth;
    const anchoTexto = elemento.scrollWidth;

    // 3. Si el texto es más grande que el contenedor...
    if (anchoTexto > anchoContenedor) {
        // ...DUPLICAMOS el texto con un separador en medio.
        // Esto permite que cuando el primero salga, el segundo entre pegado.
        const separador = "&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;"; // Un espacio visible
        elemento.innerHTML = `<span>${textoOriginal}</span><span>${separador}</span><span>${textoOriginal}</span><span>${separador}</span>`;
        
        // Activamos la animación
        elemento.classList.add('scrolling');
    }
}

async function buscarFotoArtista(nombreArtista) {
    try {
        const query = nombreArtista.split(',')[0].split('feat')[0].trim();
        const response = await fetch(`https://api.deezer.com/search/artist?q=${encodeURIComponent(query)}`);
        const json = await response.json();
        if (json.data && json.data.length > 0) {
            return json.data[0].picture_medium; 
        }
    } catch (error) {
        console.error("Error API Deezer:", error);
    }
    return null;
}