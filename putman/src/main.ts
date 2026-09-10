import { invoke } from "@tauri-apps/api/core";
declare global { interface Window { PutmanUI: any } }

window.addEventListener("DOMContentLoaded", () => {
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
});
