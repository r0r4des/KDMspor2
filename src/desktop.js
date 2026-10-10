/* KDMspor masaüstü köprüsü.
   Uygulamanın kendisi tarayıcıdakiyle birebir aynı dosya. Burada yalnızca
   masaüstünde mümkün olan iki şeyi ekliyoruz: dosyayı kaydetme penceresi ve
   doğrudan klasöre yazma, bir de izlenen klasör.
   __TAURI__ yoksa hiçbir şey yapmadan çekiliyoruz; sayfa tarayıcıdaki gibi
   çalışmaya devam eder.                                                   */
(function () {
  const T = window.__TAURI__;
  if (!T || !T.core) return;
  const { invoke } = T.core;
  const dialog = T.dialog;
  const fsapi = T.fs;
  const listen = T.event && T.event.listen;
  const LSW = "kdmspor.watchDir";
  const get = k => { try { return localStorage.getItem(k) || ""; } catch (e) { return ""; } };
  const set = (k, v) => { try { localStorage.setItem(k, v); } catch (e) { } };

  /* ---- indirme: uygulamanın saveFile() çağrısı buraya düşer ---- */
  window.claude = {
    use: async name => name !== "downloads" ? null : {
      async save({ filename, data }) {
        const ext = (filename.split(".").pop() || "").toLowerCase();
        const path = await dialog.save({
          defaultPath: filename,
          filters: [{ name: ext.toUpperCase(), extensions: [ext] }]
        });
        if (!path) { const e = new Error("iptal"); e.code = "declined"; throw e; }
        let bytes;
        if (typeof data === "string") bytes = new TextEncoder().encode(data);
        else if (data instanceof Blob) bytes = new Uint8Array(await data.arrayBuffer());
        else bytes = new Uint8Array(data);
        await fsapi.writeFile(path, bytes);
        return { status: "saved" };
      }
    }
  };

  /* ---- arayüze masaüstü bölümü ekle ---- */
  function panel() {
    const api = window.kdmspor;
    if (!api) return;
    const wrap = document.querySelector(".wrap");
    if (!wrap || document.getElementById("deskStep")) return;

    const sec = document.createElement("section");
    sec.className = "step";
    sec.id = "deskStep";
    sec.innerHTML =
      '<div class="stephd"><span class="stepn">+</span><span class="steptitle">Masaüstü</span>' +
      '<span class="stepnote" id="deskWatchNote"></span></div>' +
      '<div class="stepbody">' +
      '<p class="hint">Klasöre yazarken zip açmana gerek kalmaz. İzlenen klasöre bir KDM zip\'i düştüğünde uygulama kendiliğinden okur.</p>' +
      '<div class="row" style="margin-top:0">' +
      '<button class="btn sec" id="deskWrite" type="button">Doğru KDM\'leri klasöre yaz</button>' +
      '<button class="btn sec" id="deskWatch" type="button">İzlenecek klasörü seç</button>' +
      '<button class="btn sec" id="deskUnwatch" type="button">İzlemeyi durdur</button>' +
      '<span class="note" id="deskNote"></span>' +
      '</div></div>';
    const report = document.getElementById("report");
    wrap.insertBefore(sec, report);

    const note = m => { const n = document.getElementById("deskNote"); if (n) n.textContent = m || ""; };
    const watchNote = () => {
      const d = get(LSW);
      const n = document.getElementById("deskWatchNote");
      if (n) n.textContent = d ? "izleniyor: " + d : "";
    };
    watchNote();

    document.getElementById("deskWrite").addEventListener("click", async () => {
      const a = api.analyse();
      const items = api.foldering(a);
      if (!items.length) { note("Yazılacak dosya yok. KDM'leri bu oturumda yükle."); return; }
      const root = await dialog.open({ directory: true, title: "Klasörlerin yazılacağı yer" });
      if (!root) { note("Klasör seçilmedi."); return; }
      note("Yazılıyor…");
      try {
        const n = await invoke("write_tree", {
          root,
          items: items.map(it => ({
            film: it.film, venue: it.venue, sub: it.eski ? "Eski" : "",
            name: it.raw.name, text: it.raw.text
          })),
          manifest: api.manifest(items)
        });
        note(n + " dosya yazıldı: " + root + "/KDM");
      } catch (e) { note("Yazılamadı: " + e); }
    });

    document.getElementById("deskWatch").addEventListener("click", async () => {
      const d = await dialog.open({ directory: true, title: "KDM'lerin düştüğü klasör" });
      if (!d) return;
      try {
        await invoke("watch_folder", { path: d });
        set(LSW, d); watchNote(); note("İzleniyor.");
      } catch (e) { note("İzlenemedi: " + e); }
    });

    document.getElementById("deskUnwatch").addEventListener("click", async () => {
      try { await invoke("stop_watch"); } catch (e) { }
      set(LSW, ""); watchNote(); note("İzleme durdu.");
    });

    if (listen) {
      listen("kdm-dropped", async ev => {
        const paths = ev.payload || [];
        if (!paths.length) return;
        try {
          const texts = await invoke("read_kdm_files", { paths });
          if (!texts.length) return;
          await api.addKdmTexts(texts.map(t => [t.name, t.text]), []);
          note(texts.length + " dosya izlenen klasörden alındı.");
        } catch (e) { note("Okunamadı: " + e); }
      });
    }

    // önceki oturumdan kalan izleme varsa sürdür
    const prev = get(LSW);
    if (prev) invoke("watch_folder", { path: prev }).catch(() => { set(LSW, ""); watchNote(); });
  }

  if (window.kdmspor) panel();
  else document.addEventListener("kdmspor-ready", panel, { once: true });
})();
