// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
pub async fn request(url: &str) -> Result<String, reqwest::Error> {
    let response = reqwest::get(url).await;
    let body = response?.text().await?;
    Ok(body)
}

#[tauri::command]
async fn ping(hst: String, port: u16) -> Result<String, String> {
    let url = format!("http://{}:{}", hst, port);
    let body = request(&url).await.map_err(|e| e.to_string())?;
    Ok(body)
}



#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![ping])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
