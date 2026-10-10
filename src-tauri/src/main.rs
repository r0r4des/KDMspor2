// KDMspor — masaüstü tarafı.
// Üç iş: izlenen klasöre düşen KDM dosyalarını haber vermek, arşivleri açıp
// içlerindeki XML'leri arayüze vermek, doğrulanmış KDM'leri Film/Salon/dosya
// düzeniyle diske yazmak.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager, State};

#[derive(Default)]
struct AppState {
    watcher: Mutex<Option<RecommendedWatcher>>,
}

#[derive(Serialize, Clone)]
struct KdmText {
    name: String,
    text: String,
}

fn is_xmlish(name: &str) -> bool {
    let n = name.to_lowercase();
    n.ends_with(".xml") || n.ends_with(".kdm") || n.ends_with(".cpl")
}

fn collect_from_zip(bytes: &[u8], out: &mut Vec<KdmText>, depth: u8) {
    if depth > 5 {
        return;
    }
    let reader = std::io::Cursor::new(bytes);
    let mut zip = match zip::ZipArchive::new(reader) {
        Ok(z) => z,
        Err(_) => return,
    };
    for i in 0..zip.len() {
        let mut f = match zip.by_index(i) {
            Ok(f) => f,
            Err(_) => continue,
        };
        if f.is_dir() {
            continue;
        }
        let full = f.name().to_string();
        let name = full.rsplit('/').next().unwrap_or("").to_string();
        if name.starts_with("._") {
            continue;
        }
        let lower = name.to_lowercase();
        let mut buf = Vec::new();
        if f.read_to_end(&mut buf).is_err() {
            continue;
        }
        if lower.ends_with(".zip") {
            collect_from_zip(&buf, out, depth + 1);
        } else if is_xmlish(&name) {
            if let Ok(t) = String::from_utf8(buf) {
                out.push(KdmText { name, text: t });
            }
        }
    }
}

/// Verilen dosyaları (xml, kdm, zip) açıp içlerindeki XML metinlerini döndürür.
#[tauri::command]
fn read_kdm_files(paths: Vec<String>) -> Result<Vec<KdmText>, String> {
    let mut out: Vec<KdmText> = Vec::new();
    for p in paths {
        let path = PathBuf::from(&p);
        let name = path
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default();
        if name.starts_with("._") {
            continue;
        }
        let bytes = match fs::read(&path) {
            Ok(b) => b,
            Err(_) => continue,
        };
        if name.to_lowercase().ends_with(".zip") {
            collect_from_zip(&bytes, &mut out, 0);
        } else if is_xmlish(&name) {
            if let Ok(t) = String::from_utf8(bytes) {
                out.push(KdmText { name, text: t });
            }
        }
    }
    Ok(out)
}

#[derive(Deserialize)]
struct TreeItem {
    film: String,
    venue: String,
    sub: Option<String>,
    name: String,
    text: String,
}

fn safe(s: &str) -> String {
    let cleaned: String = s
        .chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '-',
            c if (c as u32) < 32 => ' ',
            c => c,
        })
        .collect();
    let t = cleaned.trim().to_string();
    if t.is_empty() {
        "Adsiz".to_string()
    } else {
        t.chars().take(80).collect()
    }
}

/// Film / Salon / dosya düzeninde seçilen klasöre yazar.
#[tauri::command]
fn write_tree(root: String, items: Vec<TreeItem>, manifest: Option<String>) -> Result<usize, String> {
    let base = Path::new(&root).join("KDM");
    fs::create_dir_all(&base).map_err(|e| e.to_string())?;
    let mut n: usize = 0;
    for it in &items {
        let mut dir = base.join(safe(&it.film)).join(safe(&it.venue));
        if let Some(s) = &it.sub {
            if !s.is_empty() {
                dir = dir.join(safe(s));
            }
        }
        fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        fs::write(dir.join(safe(&it.name)), it.text.as_bytes()).map_err(|e| e.to_string())?;
        n += 1;
    }
    if let Some(m) = manifest {
        fs::write(base.join("_ozet.csv"), m.as_bytes()).map_err(|e| e.to_string())?;
    }
    Ok(n)
}

/// Bir klasörü izler; yeni .zip/.xml/.kdm düştüğünde arayüze haber verir.
#[tauri::command]
fn watch_folder(app: AppHandle, state: State<AppState>, path: String) -> Result<(), String> {
    let app2 = app.clone();
    let mut w = notify::recommended_watcher(move |res: notify::Result<notify::Event>| {
        let ev = match res {
            Ok(e) => e,
            Err(_) => return,
        };
        if !matches!(
            ev.kind,
            notify::EventKind::Create(_) | notify::EventKind::Modify(_)
        ) {
            return;
        }
        let hits: Vec<String> = ev
            .paths
            .iter()
            .filter(|p| {
                let n = p
                    .file_name()
                    .map(|s| s.to_string_lossy().to_lowercase())
                    .unwrap_or_default();
                !n.starts_with("._")
                    && (n.ends_with(".zip") || n.ends_with(".xml") || n.ends_with(".kdm"))
            })
            .map(|p| p.to_string_lossy().to_string())
            .collect();
        if !hits.is_empty() {
            // kopyalama bitsin diye kısa bekleme
            std::thread::sleep(Duration::from_millis(800));
            let _ = app2.emit("kdm-dropped", hits);
        }
    })
    .map_err(|e| e.to_string())?;

    w.watch(Path::new(&path), RecursiveMode::Recursive)
        .map_err(|e| e.to_string())?;
    *state.watcher.lock().map_err(|e| e.to_string())? = Some(w);
    Ok(())
}

#[tauri::command]
fn stop_watch(state: State<AppState>) -> Result<(), String> {
    *state.watcher.lock().map_err(|e| e.to_string())? = None;
    Ok(())
}

fn main() {
    tauri::Builder::default()
        .manage(AppState::default())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![
            read_kdm_files,
            write_tree,
            watch_folder,
            stop_watch
        ])
        .setup(|app| {
            if let Some(win) = app.get_webview_window("main") {
                let _ = win.set_title("KDMspor");
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("uygulama başlatılamadı");
}
