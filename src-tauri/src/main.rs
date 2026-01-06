#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::Emitter;
use windows::Media::Control::GlobalSystemMediaTransportControlsSessionManager;
use windows::Storage::Streams::{DataReader, IRandomAccessStreamWithContentType};
use std::time::Duration;
use base64::{Engine as _, engine::general_purpose};

#[derive(Clone, serde::Serialize)]
struct MediaInfo {
    title: String,
    artist: String,
    status: String,
    cover: Option<String>,
}

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            let app_handle = app.handle().clone();

            // CAMBIO IMPORTANTE: Usamos un hilo nativo con un runtime local.
            // Esto evita el error de "Send" porque todo se queda en el mismo hilo.
            std::thread::spawn(move || {
                let rt = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .expect("No se pudo iniciar el runtime");

                rt.block_on(async {
                    loop {
                        // Intentamos obtener la info
                        if let Ok(info) = get_media_info().await {
                            let _ = app_handle.emit("media-update", info);
                        }
                        // Esperamos 1.5 segundos
                        tokio::time::sleep(Duration::from_millis(1500)).await;
                    }
                });
            });

            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

async fn get_media_info() -> Result<MediaInfo, ()> {
    // 1. Conectarse al manager de Windows
    let manager = GlobalSystemMediaTransportControlsSessionManager::RequestAsync()
        .map_err(|_| ())?.await.map_err(|_| ())?;

    let session = manager.GetCurrentSession().map_err(|_| ())?;
    
    // 2. Obtener propiedades
    let properties = session.TryGetMediaPropertiesAsync()
        .map_err(|_| ())?.await.map_err(|_| ())?;

    let title = properties.Title().unwrap_or_default().to_string();
    let artist = properties.Artist().unwrap_or_default().to_string();

    // 3. Obtener la portada (Ahora seguro porque estamos en un hilo local)
    let mut cover_base64 = None;

    // Obtenemos la referencia del thumbnail
    let thumbnail_ref_option = properties.Thumbnail().ok();
    
    if let Some(thumbnail_ref) = thumbnail_ref_option {
        // Abrimos el stream de lectura
        let stream_op_option = thumbnail_ref.OpenReadAsync().ok();

        if let Some(stream_op) = stream_op_option {
             if let Ok(stream) = stream_op.await {
                 // Leemos los bytes
                 if let Ok(base64_img) = read_thumbnail(stream).await {
                     cover_base64 = Some(format!("data:image/png;base64,{}", base64_img));
                 }
             }
        }
    }

    // 4. Obtener Estado (Play/Pause)
    let timeline = session.GetPlaybackInfo().map_err(|_| ())?;
    let status_code = timeline.PlaybackStatus().map_err(|_| ()).unwrap_or_default().0;
    
    let status_str = match status_code {
        4 => "Playing",
        5 => "Paused",
        _ => "Stopped",
    };

    Ok(MediaInfo {
        title: if title.is_empty() { "Sin música".to_string() } else { title },
        artist: if artist.is_empty() { "Escuchando...".to_string() } else { artist },
        status: status_str.to_string(),
        cover: cover_base64,
    })
}

// Función auxiliar para leer los bits de la imagen
async fn read_thumbnail(stream: IRandomAccessStreamWithContentType) -> Result<String, ()> {
    let size = stream.Size().map_err(|_| ())? as u32;
    if size == 0 { return Err(()); }

    let reader = DataReader::CreateDataReader(&stream).map_err(|_| ())?;
    reader.LoadAsync(size).map_err(|_| ())?.await.map_err(|_| ())?;

    let mut buffer = vec![0u8; size as usize];
    reader.ReadBytes(&mut buffer).map_err(|_| ())?;

    Ok(general_purpose::STANDARD.encode(&buffer))
}