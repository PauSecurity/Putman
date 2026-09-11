use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};
use std::path::PathBuf;
use tauri::Manager;

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KeyValue {
    #[serde(default)] pub enabled: bool, // <- #[serde(default)] boş gelirse "" ile doldurur.
    #[serde(default)] pub key: String,
    #[serde(default)] pub value: String,
    #[serde(default)] pub description: String,
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Auth {
    #[serde(rename = "type", default)] pub kind: String,
    #[serde(default)] pub token: String,
    #[serde(default)] pub username: String,
    #[serde(default)] pub password: String,
    #[serde(default)] pub key: String,
    #[serde(default)] pub value: String,
    #[serde(rename = "in", default)] pub location: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    #[serde(default)] pub timeout_ms: u64,
    #[serde(default)] pub max_redirects: usize,
    #[serde(default)] pub follow_redirects: bool,
    #[serde(default)] pub verify_ssl: bool,
    #[serde(default)] pub encode_url: bool,
}

//Default settings değerleri
impl Default for Settings {
    fn default() -> Self {
        Settings {
            timeout_ms: 30_000,
            max_redirects: 10,
            follow_redirects: true,
            verify_ssl: true,
            encode_url: true,
        }
    }
}


 // body sekmesi. getRequest() şunu üretiyor:
//   { type: "none"|"json"|"text"|"form"|"urlencoded", text: "...", fields: [...] }
#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Body {
    #[serde(rename = "type", default)] pub kind: String,
    #[serde(default)] pub text: String,
    #[serde(default)] pub fields: Vec<KeyValue>,
}

//index.html içindeki getRequest()'in tamamının Rust'taki şekli
#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RequestSpec {
    #[serde(default)] pub request_id: Option<String>,
    #[serde(default)] pub folder_id: Option<String>,
    #[serde(default)] pub name: String,
    #[serde(default)] pub description: String,
    pub method: String,
    pub url: String,
    #[serde(default)] pub params: Vec<KeyValue>,
    #[serde(default)] pub headers: Vec<KeyValue>,
    #[serde(default)] pub auth: Auth,
    #[serde(default)] pub body: Body,
    #[serde(default)] pub settings: Settings,
}






//Alttaki ikisi cevap struct'ları
#[derive(Debug, Serialize)]
pub struct HeaderPair {
    pub key: String,
    pub value: String,
}

// index.html içindeki setResponse()'un beklediği şekl
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HttpResponse {
    pub status: u16,
    pub status_text: String,
    pub time_ms: u64,
    pub size_bytes: usize,
    pub headers: Vec<HeaderPair>,
    pub body: String,
}







//Kayıtlar
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Folder {
    pub id: String,
    pub name: String,
    #[serde(default)] pub open: bool,
    #[serde(default)] pub requests: Vec<RequestSpec>,
}

//geçmiş
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoryItem {
    pub id: String,
    pub method: String,
    pub url: String,
    #[serde(default)] pub status: u16,
    #[serde(default)] pub time_ms: u64,
    #[serde(default)] pub at: i64,        // Unix ms — PutmanUI "3 dk önce" diye gösterir
}









#[tauri::command]
async fn send_request(req: RequestSpec) -> Result<HttpResponse, String> {

    // İSTEMCİYİ KUR (settings sekmesi burada devrede)
    let mut builder = reqwest::Client::builder();

    if req.settings.timeout_ms > 0 {
        builder = builder.timeout(Duration::from_millis(req.settings.timeout_ms));
    }

    builder = if req.settings.follow_redirects {
        builder.redirect(reqwest::redirect::Policy::limited(req.settings.max_redirects))
    } else {
        builder.redirect(reqwest::redirect::Policy::none())
    };

    if !req.settings.verify_ssl {
        builder = builder.danger_accept_invalid_certs(true);
    }

    let client = builder.build().map_err(|e| e.to_string())?;




    // metod çözümü
    let method = reqwest::Method::from_bytes(req.method.as_bytes())
        .map_err(|_| format!("Gecersiz method: {}", req.method))?;



    //parametreler alınır
    let query: Vec<(String, String)> = req.params.iter()
        .filter(|p| p.enabled && !p.key.is_empty())
        .map(|p| (p.key.clone(), p.value.clone()))
        .collect();

    // isteğin iskeleti oluşturulur.
    let tam_url = if query.is_empty() || req.settings.encode_url {
        req.url.clone()
    } else {
        let ham: String = query.iter()
            .map(|(k, v)| format!("{k}={v}"))
            .collect::<Vec<_>>()
            .join("&");
        let ayrac = if req.url.contains('?') { '&' } else { '?' };
        format!("{}{}{}", req.url, ayrac, ham)
    };
    let mut rb = client.request(method, &tam_url);
    if req.settings.encode_url && !query.is_empty() {
        rb = rb.query(&query);
    }


    // headers
    for h in req.headers.iter().filter(|h| h.enabled && !h.key.is_empty()) {
        rb = rb.header(h.key.as_str(), h.value.as_str());
    }

     //auth kısmı
    match req.auth.kind.as_str() {
        "bearer" => rb = rb.bearer_auth(&req.auth.token),
        "basic"  => rb = rb.basic_auth(&req.auth.username, Some(&req.auth.password)),
        "apikey" => {
            if req.auth.location == "query" {
                rb = rb.query(&[(req.auth.key.as_str(), req.auth.value.as_str())]);
            } else {
                rb = rb.header(req.auth.key.as_str(), req.auth.value.as_str());
            }
        }
        _ => {} // auth boş
    }

    // body
    // Kullanıcı headers sekmesinde kendi Content-Type'ını yazmış mı?
    let kullanici_ct = req.headers.iter()
        .any(|h| h.enabled && h.key.eq_ignore_ascii_case("content-type"));

    match req.body.kind.as_str() {
        "json" => {
            let metin = req.body.text.trim();
            if !metin.is_empty() {
                // Bozuk JSON'u ağa çıkmadan yakala
                serde_json::from_str::<serde_json::Value>(metin)
                    .map_err(|e| format!("Gövde geçerli JSON değil: {e}"))?;

                if !kullanici_ct {
                    rb = rb.header("content-type", "application/json");
                }
                rb = rb.body(metin.to_string());
            }
        }

        "text" => {
            if !req.body.text.is_empty() {
                if !kullanici_ct {
                    rb = rb.header("content-type", "text/plain; charset=utf-8");
                }
                rb = rb.body(req.body.text.clone());
            }
        }

        "urlencoded" => {
            let alanlar: Vec<(String, String)> = req.body.fields.iter()
                .filter(|f| f.enabled && !f.key.is_empty())
                .map(|f| (f.key.clone(), f.value.clone()))
                .collect();
            if !alanlar.is_empty() {
                rb = rb.form(&alanlar);   // Content-Type'ı kendi koyar
            }
        }

        "form" => {
            let mut form = reqwest::multipart::Form::new();
            let mut bos = true;
            for f in req.body.fields.iter().filter(|f| f.enabled && !f.key.is_empty()) {
                form = form.text(f.key.clone(), f.value.clone());
                bos = false;
            }
            if !bos {
                rb = rb.multipart(form);  // boundary'yi kendi üretir
            }
        }

        _ => {} // "none"
    }


    // asıl gönderilen kısım
    let started = Instant::now();
    let resp = rb.send().await.map_err(|e| e.to_string())?;

    // cevap parse'ı
    let status = resp.status();

    let headers: Vec<HeaderPair> = resp.headers().iter()
        .map(|(k, v)| HeaderPair {
            key: k.as_str().to_string(),
            value: v.to_str().unwrap_or("<okunamayan değer>").to_string(),
        })
        .collect();

    let body = resp.text().await.map_err(|e| e.to_string())?;
    let time_ms = started.elapsed().as_millis() as u64;

    Ok(HttpResponse {
        status: status.as_u16(),
        status_text: status.canonical_reason().unwrap_or("").to_string(),
        time_ms,
        size_bytes: body.len(),
        headers,
        body,
    })
}




// app_data_dir/<ad> yolunu verir ve dizinin var olduğundan emin olur.
fn veri_yolu(app: &tauri::AppHandle, ad: &str) -> Result<PathBuf, String> {
    let dizin = app.path().app_data_dir()
        .map_err(|e| format!("Veri dizini bulnuamadı: {e}"))?;
    std::fs::create_dir_all(&dizin)
        .map_err(|e| format!("Veri dizini oluşturulamadı: {e}"))?;
    Ok(dizin.join(ad))
}

// dosya yoksa boş liste döner — ilk açılışta hata vermemesi için.
fn json_oku<T: serde::de::DeserializeOwned + Default>(
    app: &tauri::AppHandle, ad: &str
) -> Result<T, String> {
    let yol = veri_yolu(app, ad)?;
    if !yol.exists() {
        return Ok(T::default());
    }
    let metin = std::fs::read_to_string(&yol)
        .map_err(|e| format!("{} okunamadı: {e}", yol.display()))?;
    serde_json::from_str(&metin)
        .map_err(|e| format!("{} bozuk JSON: {e}", yol.display()))
}

fn json_yaz<T: Serialize>(app: &tauri::AppHandle, ad: &str, veri: &T) -> Result<(), String> {
    let yol = veri_yolu(app, ad)?;
    let metin = serde_json::to_string_pretty(veri)
        .map_err(|e| format!("Serileştirilemeid: {e}"))?;

    let gecici = yol.with_extension("tmp");
    std::fs::write(&gecici, metin)
        .map_err(|e| format!("Yazılamadı: {e}"))?;
    std::fs::rename(&gecici, &yol)
        .map_err(|e| format!("Taşınamadı: {e}"))?;
    Ok(())
}



#[tauri::command]
fn load_collections(app: tauri::AppHandle) -> Result<Vec<Folder>, String> {
    json_oku(&app, "collections.json")
}
#[tauri::command]
fn save_collections(app: tauri::AppHandle, folders: Vec<Folder>) -> Result<(), String> {
    json_yaz(&app, "collections.json", &folders)
}

#[tauri::command]
fn load_history(app: tauri::AppHandle) -> Result<Vec<HistoryItem>, String> {
    json_oku(&app, "history.json")
}

#[tauri::command]
fn save_history(app: tauri::AppHandle, items: Vec<HistoryItem>) -> Result<(), String> {
    json_yaz(&app, "history.json", &items)
}



//alttaki iki fonksiyon ping için.
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
        .invoke_handler(tauri::generate_handler![ping,send_request,load_collections, save_collections, load_history, save_history])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
