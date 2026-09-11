import { invoke } from "@tauri-apps/api/core";
declare global { interface Window { PutmanUI: any } }

//Rust'taki HttpResponse struct'ının TS tarafı.
interface HttpResponse {
  status: number;
  statusText: string;
  timeMs: number;
  sizeBytes: number;
  headers: { key: string; value: string }[];
  body: string;
}

window.addEventListener("DOMContentLoaded", () => {
  async function baslangictaYukle() {
    window.PutmanUI.clearCollections();
    window.PutmanUI.clearHistory();

    try {
      klasorler = await invoke<Klasor[]>("load_collections");
      agaciCiz();
    } catch (e) {
      window.PutmanUI.toast("Koleksiyonlar yüklenemeid: " + e);
      klasorler = [];
    }

    try {
      gecmis = await invoke<any[]>("load_history");
      window.PutmanUI.setHistory(gecmis);
    } catch (e) {
      window.PutmanUI.toast("Geçmiş yüklenemedi: " + e);
      gecmis = [];
    }
  }

  baslangictaYukle();

  document.addEventListener("putman:send", async (e) => {
    const req = (e as CustomEvent).detail;   // = getRequest() çıktısı

    try {
      const res = await invoke<HttpResponse>("send_request", { req });

      window.PutmanUI.setResponse(res);
      window.PutmanUI.setStatus(
          `${res.status} ${res.statusText} — ${res.timeMs} ms`,
          res.status < 400 ? "ok" : "err"
      );
      const kayit = {
        id: "h_" + Date.now().toString(36),
        method: req.method,
        url: req.url,
        status: res.status,
        timeMs: res.timeMs,
        at: Date.now(),
      };
      window.PutmanUI.addHistory(kayit);
      gecmis.unshift(kayit);              // en yeni başa
      if (gecmis.length > 200) gecmis.length = 200;   // sınırla
      gecmisKaydet();

    } catch (err) {
      // Rust Err(String) döndürdüyse buraya düşer
      window.PutmanUI.setResponse({
        status: 0,
        statusText: "HATA",
        timeMs: 0,
        sizeBytes: 0,
        headers: [],
        body: String(err),
      });
      window.PutmanUI.setStatus(String(err), "err");
    }
  });


  //geçmiş temizleme
  document.addEventListener("putman:history-cleared", () => {
    gecmis = [];
    gecmisKaydet();
  });


  document.addEventListener("putman:folder-created", (e) => {
    const { folderId, name } = (e as CustomEvent).detail;
    klasorler.push({ id: folderId, name, open: true, requests: [] });
    koleksiyonKaydet();
  });

  document.addEventListener("putman:folder-renamed", (e) => {
    const { folderId, name } = (e as CustomEvent).detail;
    const f = klasorler.find((x) => x.id === folderId);
    if (f) f.name = name;
    koleksiyonKaydet();
  });

  document.addEventListener("putman:folder-deleted", (e) => {
    const { folderId } = (e as CustomEvent).detail;
    klasorler = klasorler.filter((x) => x.id !== folderId);
    koleksiyonKaydet();
  });

// istek oluşturulur.
  document.addEventListener("putman:request-created", (e) => {
    const { requestId, folderId, name, method } = (e as CustomEvent).detail;
    const f = klasorler.find((x) => x.id === folderId);
    if (f) {
      f.requests.push({
        requestId, folderId, name, method,
        url: "", description: "",
        params: [], headers: [],
        auth: { type: "none" }, body: { type: "none", text: "", fields: [] },
        settings: {
          timeoutMs: 30000, maxRedirects: 10,
          followRedirects: true, verifySsl: true, encodeUrl: true,
        },
      });
    }
    koleksiyonKaydet();
  });

  document.addEventListener("putman:request-renamed", (e) => {
    const { requestId, folderId, name } = (e as CustomEvent).detail;
    const r = klasorler.find((x) => x.id === folderId)
        ?.requests.find((x) => x.requestId === requestId);
    if (r) r.name = name;
    koleksiyonKaydet();
  });

  document.addEventListener("putman:request-deleted", (e) => {
    const { requestId, folderId } = (e as CustomEvent).detail;
    const f = klasorler.find((x) => x.id === folderId);
    if (f) f.requests = f.requests.filter((x) => x.requestId !== requestId);
    koleksiyonKaydet();
  });

// seçili içeriği forma yükler
  document.addEventListener("putman:request-selected", (e) => {
    const { requestId, folderId } = (e as CustomEvent).detail;
    const r = klasorler.find((x) => x.id === folderId)
        ?.requests.find((x) => x.requestId === requestId);
    if (r) window.PutmanUI.setRequest(r);
  });

// oTOMATİK kAYIT
  document.addEventListener("putman:request-changed", (e) => {
    const req = (e as CustomEvent).detail;
    if (!req.requestId) return;          // ağaçta seçili istek yok, kaydedecek yer yok

    const f = klasorler.find((x) => x.id === req.folderId);
    if (!f) return;
    const i = f.requests.findIndex((x) => x.requestId === req.requestId);
    if (i >= 0) f.requests[i] = req;
    koleksiyonKaydet();
  });





  interface Klasor {
    id: string;
    name: string;
    open: boolean;
    requests: any[];        // RequestSpec listesi
  }

  let klasorler: Klasor[] = [];
  let gecmis: any[] = [];

  //tekrar ekrana cizilir
  function agaciCiz() {
    window.PutmanUI.setCollections(
        klasorler.map((f) => ({
          id: f.id,
          name: f.name,
          open: f.open,
          requests: f.requests.map((r) => ({
            id: r.requestId,
            name: r.name || "Adsız istek",
            method: r.method || "GET",
          })),
        }))
    );
  }


  let kolTimer: number | undefined;
  function koleksiyonKaydet() {
    clearTimeout(kolTimer);
    kolTimer = window.setTimeout(() => {
      invoke("save_collections", { folders: klasorler })
          .catch((e) => window.PutmanUI.toast("Kaydedilemedi: " + e));
    }, 400);
  }

  let gecTimer: number | undefined;
  function gecmisKaydet() {
    clearTimeout(gecTimer);
    gecTimer = window.setTimeout(() => {
      invoke("save_history", { items: gecmis })
          .catch((e) => window.PutmanUI.toast("Geçmiş kaydedilemedi: " + e));
    }, 400);
  }



  //google'a ping

  document.querySelector("#ping-google")?.addEventListener("click", async () => {

    const basla = performance.now();
    try {
      window.PutmanUI.setLoading(true);
      const sonuc = await invoke<string>("ping", { hst: "google.com", port: 80 });
      window.PutmanUI.setResponse({
        status: 200,
        statusText: "OK",
        timeMs: Math.round(performance.now() - basla),
        sizeBytes: sonuc.length,
        body: sonuc,
      });
      window.PutmanUI.setLoading(false);
    } catch (err) {
      window.PutmanUI.setStatus(String(err), "err");
    }
  });

  window.PutmanUI.setStatus("Hazır", "ok");
});

//link değişmesine ve linkdeki parametreler silinmesine rağmen parametre kısmında parametreler kaldı.
//name parametresi ile linke gidildiğinde hata verdi
//Header yok, her yer kabul etmez.
//Her cevapta görünen json sıraları değişiyor. Alfabetik sıralansa daha iyi olur.