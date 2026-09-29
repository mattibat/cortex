<script lang="ts">
  import { app } from "../lib/store.svelte";
  import * as api from "../lib/api";
  import type { ChunkInfo, Note } from "../lib/api";
  import { convertFileSrc } from "@tauri-apps/api/core";
  import Icon from "../components/Icon.svelte";
  import RichText from "../components/RichText.svelte";
  import ChatPanel from "../components/ChatPanel.svelte";
  import { isMobile } from "../lib/platform";
  import { tick } from "svelte";
  import hljs from "highlight.js/lib/common";
  import "highlight.js/styles/vs2015.css";

  // ---- state ----
  let chunks = $state<ChunkInfo[]>([]);
  let loading = $state(true);
  let split = $state(58); // percent for left pane
  let dragging = $state(false);

  // Citation deep-link: when a source is opened from a ⟦… · loc⟧ chip, scroll to the
  // chunk whose loc matches the cited location and flash it. Best-effort — no match just
  // opens the source. Re-runs when chunks finish loading.
  $effect(() => {
    const loc = app.sourceJumpLoc;
    void chunks;
    if (!loc || chunks.length === 0) return;
    const lc = loc.toLowerCase();
    const target = chunks.find((c) => {
      const cl = (c.loc ?? "").toLowerCase();
      return !!cl && (cl === lc || cl.includes(lc) || lc.includes(cl));
    });
    app.sourceJumpLoc = null; // consume
    if (!target) return;
    void tick().then(() => {
      const el = document.getElementById("sv-chunk-" + target.ord);
      if (!el) return;
      el.scrollIntoView({ behavior: "smooth", block: "center" });
      el.classList.add("sv-chunk-flash");
      setTimeout(() => el.classList.remove("sv-chunk-flash"), 1600);
    });
  });

  // Mobile: zoom is locked app-wide (native-app feel), but a source IS the one place
  // you need pinch-zoom — to read a dense PDF/slide. Unlock the viewport while this
  // viewer is mounted and restore the lock on exit. Desktop is unaffected (no touch-zoom).
  $effect(() => {
    if (!isMobile) return;
    const meta = document.querySelector('meta[name="viewport"]');
    if (!meta) return;
    const locked = meta.getAttribute("content") ?? "width=device-width, initial-scale=1.0, maximum-scale=1, user-scalable=no";
    meta.setAttribute("content", "width=device-width, initial-scale=1.0, maximum-scale=5, user-scalable=yes");
    return () => meta.setAttribute("content", locked);
  });

  // ---- per-kind preview routing ----
  // `assetUrl` is the webview-loadable URL for the persisted original (or, for
  // pptx/docx, the backend-rendered PDF). Guarded against a null stored_path.
  const assetUrl = $derived(
    app.activeSource?.stored_path
      ? convertFileSrc(app.activeSource.stored_path)
      : null
  );
  // A document renders as an embedded PDF when its kind is a PDF-previewable
  // document. pptx/docx are rendered to PDF by the backend (their stored_path
  // points at that PDF), so they preview as slides via the same iframe.
  const PDF_KINDS = ["pdf", "pptx", "docx", "xlsx"];
  const isPdfDoc = $derived(
    !!app.activeSource && PDF_KINDS.includes(app.activeSource.kind) && !!assetUrl
  );
  const isImage = $derived(app.activeSource?.kind === "image" && !!assetUrl);
  const isAudio = $derived(app.activeSource?.kind === "audio" && !!assetUrl);
  const isTable = $derived(app.activeSource?.kind === "xlsx" && !!app.activeSource.content);
  // Readable text: explicit text-ish kinds, or any source with no stored file
  // but extracted content to show.
  const TEXT_KINDS = ["txt", "md", "web", "url", "epub"];
  const isText = $derived(
    !!app.activeSource &&
      !isPdfDoc &&
      !isImage &&
      !isAudio &&
      !isTable &&
      !!app.activeSource.content &&
      (TEXT_KINDS.includes(app.activeSource.kind) || !app.activeSource.stored_path)
  );
  // Anything with a dedicated preview skips the chunk-list fallback.
  const hasPreview = $derived(isPdfDoc || isImage || isAudio || isTable || isText);

  const OFFICE_KINDS = ["docx", "pptx", "xlsx"];
  let officeRenderingId = $state<string | null>(null);
  let officeError = $state<string | null>(null);
  let officeRequested: string | null = null;
  $effect(() => {
    const s = app.activeSource;
    if (!s || s.id !== officeRequested) officeError = null;
    if (!s || isMobile || !OFFICE_KINDS.includes(s.kind) || s.stored_path || !s.origin) return;
    if (s.id === officeRequested) return;
    const id = s.id;
    officeRequested = id;
    officeRenderingId = id;
    api.renderSourcePreview(id)
      .then((full) => { if (app.activeSource?.id === id) app.activeSource = full; })
      .catch((e) => { if (app.activeSource?.id === id) officeError = String(e); })
      .finally(() => { if (officeRenderingId === id) officeRenderingId = null; });
  });

  const isMarkdown = $derived(isText && app.activeSource?.kind === "md");
  const codeLang = $derived.by(() => {
    const s = app.activeSource;
    if (!isText || s?.kind !== "txt") return null;
    const ext = (s.origin ?? s.name).split(".").pop()?.toLowerCase() ?? "";
    return hljs.getLanguage(ext) ? ext : "plaintext";
  });
  const codeHtml = $derived(
    codeLang && app.activeSource?.content
      ? hljs.highlight(app.activeSource.content, { language: codeLang, ignoreIllegals: true }).value
      : ""
  );
  const codeLines = $derived(codeLang ? (app.activeSource?.content ?? "").split("\n").length : 0);

  const LOGO_WORD = "M23.004 1.5q.41 0 .703.293t.293.703v19.008q0 .41-.293.703t-.703.293H6.996q-.41 0-.703-.293T6 21.504V18H.996q-.41 0-.703-.293T0 17.004V6.996q0-.41.293-.703T.996 6H6V2.496q0-.41.293-.703t.703-.293zM6.035 11.203l1.442 4.735h1.64l1.57-7.876H9.036l-.937 4.653-1.325-4.5H5.38l-1.406 4.523-.938-4.675H1.312l1.57 7.874h1.641zM22.5 21v-3h-15v3zm0-4.5v-3.75H12v3.75zm0-5.25V7.5H12v3.75zm0-5.25V3h-15v3Z";
  const LOGO_EXCEL = "M23 1.5q.41 0 .7.3.3.29.3.7v19q0 .41-.3.7-.29.3-.7.3H7q-.41 0-.7-.3-.3-.29-.3-.7V18H1q-.41 0-.7-.3-.3-.29-.3-.7V7q0-.41.3-.7Q.58 6 1 6h5V2.5q0-.41.3-.7.29-.3.7-.3zM6 13.28l1.42 2.66h2.14l-2.38-3.87 2.34-3.8H7.46l-1.3 2.4-.05.08-.04.09-.64-1.28-.66-1.29H2.59l2.27 3.82-2.48 3.85h2.16zM14.25 21v-3H7.5v3zm0-4.5v-3.75H12v3.75zm0-5.25V7.5H12v3.75zm0-5.25V3H7.5v3zm8.25 15v-3h-6.75v3zm0-4.5v-3.75h-6.75v3.75zm0-5.25V7.5h-6.75v3.75zm0-5.25V3h-6.75v3Z";
  const LOGO_POWERPOINT = "M13.5 1.5q1.453 0 2.795.375 1.342.375 2.508 1.06 1.166.686 2.12 1.641.956.955 1.641 2.121.686 1.166 1.061 2.508Q24 10.547 24 12q0 1.453-.375 2.795-.375 1.342-1.06 2.508-.686 1.166-1.641 2.12-.955.956-2.121 1.641-1.166.686-2.508 1.061-1.342.375-2.795.375-1.29 0-2.52-.305-1.23-.304-2.337-.884-1.108-.58-2.063-1.418-.955-.838-1.693-1.893H.997q-.411 0-.704-.293T0 17.004V6.996q0-.41.293-.703T.996 6h3.89q.739-1.055 1.694-1.893.955-.837 2.063-1.418 1.107-.58 2.337-.884Q12.21 1.5 13.5 1.5zm.75 1.535v8.215h8.215q-.14-1.64-.826-3.076-.686-1.436-1.782-2.531-1.095-1.096-2.537-1.782-1.441-.685-3.07-.826zm-5.262 7.57q0-.68-.228-1.166-.229-.486-.627-.79-.399-.305-.938-.446-.539-.14-1.172-.14H2.848v7.863h1.84v-2.742H5.93q.574 0 1.119-.17t.978-.493q.434-.322.698-.802.263-.48.263-1.114zM13.5 21q1.172 0 2.262-.287t2.056-.82q.967-.534 1.776-1.278.808-.744 1.418-1.664.61-.92.984-1.986.375-1.067.469-2.227h-9.703V3.035q-1.735.14-3.27.908T6.797 6h4.207q.41 0 .703.293t.293.703v10.008q0 .41-.293.703t-.703.293H6.797q.644.715 1.412 1.271.768.557 1.623.944.855.387 1.781.586Q12.54 21 13.5 21zM5.812 9.598q.575 0 .915.228.34.229.34.838 0 .27-.124.44-.123.17-.31.275-.188.105-.422.146-.234.041-.445.041H4.687V9.598Z";
  const LOGO_VSCODE = "M23.15 2.587L18.21.21a1.494 1.494 0 0 0-1.705.29l-9.46 8.63-4.12-3.128a.999.999 0 0 0-1.276.057L.327 7.261A1 1 0 0 0 .326 8.74L3.899 12 .326 15.26a1 1 0 0 0 .001 1.479L1.65 17.94a.999.999 0 0 0 1.276.057l4.12-3.128 9.46 8.63a1.492 1.492 0 0 0 1.704.29l4.942-2.377A1.5 1.5 0 0 0 24 20.06V3.939a1.5 1.5 0 0 0-.85-1.352zm-5.146 14.861L10.826 12l7.178-5.448v10.896z";
  const LOGO_CURSOR = "M11.503.131 1.891 5.678a.84.84 0 0 0-.42.726v11.188c0 .3.162.575.42.724l9.609 5.55a1 1 0 0 0 .998 0l9.61-5.55a.84.84 0 0 0 .42-.724V6.404a.84.84 0 0 0-.42-.726L12.497.131a1.01 1.01 0 0 0-.996 0M2.657 6.338h18.55c.263 0 .43.287.297.515L12.23 22.918c-.062.107-.229.064-.229-.06V12.335a.59.59 0 0 0-.295-.51l-9.11-5.257c-.109-.063-.064-.23.061-.23";

  type ExternalApp = { label: string; color: string; svgPath: string; with?: string };
  function officeApp(kind: string): ExternalApp | null {
    if (kind === "docx") return { label: "Word", color: "#2B579A", svgPath: LOGO_WORD };
    if (kind === "pptx") return { label: "PowerPoint", color: "#D24726", svgPath: LOGO_POWERPOINT };
    if (kind === "xlsx") return { label: "Excel", color: "#217346", svgPath: LOGO_EXCEL };
    return null;
  }
  const officeAppInfo = $derived(app.activeSource ? officeApp(app.activeSource.kind) : null);
  const isEditorKind = $derived(
    app.activeSource?.kind === "txt" || app.activeSource?.kind === "md"
  );
  const EDITORS: ExternalApp[] = [
    { label: "VS Code", color: "#007ACC", svgPath: LOGO_VSCODE, with: "code" },
    { label: "Cursor", color: "#1a1a1a", svgPath: LOGO_CURSOR, with: "cursor" },
  ];
  let openWithMenu = $state(false);
  async function openExternal(withApp?: string) {
    openWithMenu = false;
    const origin = app.activeSource?.origin;
    if (!origin) return;
    try {
      await api.openSourceFile(origin, withApp);
    } catch (e) {
      app.pushToast({ kind: "error", title: "Öffnen fehlgeschlagen", body: String(e) });
    }
  }
  $effect(() => {
    if (!openWithMenu) return;
    const close = () => (openWithMenu = false);
    const id = setTimeout(() => window.addEventListener("mousedown", close), 0);
    return () => { clearTimeout(id); window.removeEventListener("mousedown", close); };
  });

  // ---- mobile PDF rendering ----
  // iOS WKWebView only renders the FIRST page of an <iframe> PDF and won't scroll,
  // so on mobile we render every page to a canvas with PDF.js into a scroll column.
  // Desktop keeps the native <iframe> preview.
  let pdfBox = $state<HTMLDivElement | null>(null);
  let pdfRendering = $state(false);
  let pdfError = $state<string | null>(null);
  $effect(() => {
    if (!(isMobile && isPdfDoc && assetUrl && pdfBox)) return;
    const url = assetUrl;
    const box = pdfBox;
    let cancelled = false;
    pdfRendering = true;
    pdfError = null;
    box.replaceChildren();
    (async () => {
      try {
        const pdfjs = await import("pdfjs-dist");
        const worker = await import("pdfjs-dist/build/pdf.worker.min.mjs?url");
        pdfjs.GlobalWorkerOptions.workerSrc = worker.default;
        // isEvalSupported/enableScripting harden the renderer (no eval, no in-PDF JS);
        // real runtime options in pdfjs v6 but absent from its public types — hence the cast.
        const doc = await pdfjs.getDocument({ url, isEvalSupported: false, enableScripting: false } as any).promise;
        const dpr = Math.min(window.devicePixelRatio || 1, 2);
        for (let n = 1; n <= doc.numPages; n++) {
          if (cancelled) return;
          const page = await doc.getPage(n);
          const base = page.getViewport({ scale: 1 });
          const cssWidth = box.clientWidth || 360;
          const vp = page.getViewport({ scale: (cssWidth / base.width) * dpr });
          const canvas = document.createElement("canvas");
          canvas.className = "sv-pdf-page";
          canvas.width = vp.width;
          canvas.height = vp.height;
          const ctx = canvas.getContext("2d");
          if (!ctx) continue;
          box.appendChild(canvas);
          await page.render({ canvasContext: ctx, viewport: vp, canvas }).promise;
        }
      } catch (e) {
        if (!cancelled) pdfError = String(e);
      } finally {
        if (!cancelled) pdfRendering = false;
      }
    })();
    return () => { cancelled = true; };
  });

  // ---- load chunks on mount / when the active source changes ----
  // Re-runs reactively whenever app.activeSource (or its id) changes. We only
  // need raw chunks when there's no richer preview to fall back to, plus audio
  // uses them as a transcript fallback.
  $effect(() => {
    const src = app.activeSource;
    if (!src) return;
    // Skip the (potentially large) chunk fetch when a real preview is shown and
    // we don't need a transcript fallback.
    if (hasPreview && !isAudio) {
      chunks = [];
      loading = false;
      return;
    }
    loading = true;
    api.listChunks(src.id)
      .then((c) => { chunks = c; })
      .catch(() => { chunks = []; })
      .finally(() => { loading = false; });
  });

  // ---- delete ----
  async function confirmDelete() {
    const src = app.activeSource;
    if (!src) return;
    if (await app.confirm({ title: "Diese Quelle löschen?", danger: true, okLabel: "Löschen" })) {
      await app.deleteSource(src.id); // store action closes the viewer on success
    }
  }

  // ---- edit ----
  function editSource() {
    const src = app.activeSource;
    if (!src) return;
    app.openEdit({
      kind: "source",
      id: src.id,
      name: src.name,
      subjectId: src.subject_id,
      topicId: src.topic_id,
      tags: src.tags ?? [],
      topicOptions: (app.activeSubject?.topics ?? []).map((t) => ({ id: t.id, label: t.name })),
    });
  }

  // transcript text for audio: prefer extracted content, fall back to chunks
  const transcript = $derived(
    app.activeSource?.content ?? (chunks.length ? chunks.map((c) => c.text).join("\n\n") : "")
  );

  // ---- lecture overview note (audio) ----
  // Transcription auto-saves a markdown summary as a note titled "Summary — {name}"
  // in the same subject (spawn_lecture_summary). No FK links it to the source, so
  // it's found by that title convention; a stale-response guard keeps a slow fetch
  // from landing on a different source.
  let summaryNote = $state<Note | null>(null);
  let showTranscript = $state(false);
  $effect(() => {
    const s = app.activeSource;
    summaryNote = null;
    showTranscript = false;
    if (!s || s.kind !== "audio") return;
    api.listNotes(s.subject_id)
      .then((notes) => {
        if (app.activeSource?.id !== s.id) return;
        summaryNote = notes.find((n) => n.title === `Zusammenfassung — ${s.name}`) ?? null;
      })
      .catch(() => {});
  });

  // ---- splitter drag ----
  $effect(() => {
    function onMove(e: MouseEvent) {
      if (!dragging) return;
      const sidebar = 248; // --sb-w default
      const pct = ((e.clientX - sidebar) / (window.innerWidth - sidebar)) * 100;
      split = Math.min(72, Math.max(38, pct));
    }
    function onUp() {
      dragging = false;
      document.body.style.cursor = "";
    }
    window.addEventListener("mousemove", onMove);
    window.addEventListener("mouseup", onUp);
    return () => {
      window.removeEventListener("mousemove", onMove);
      window.removeEventListener("mouseup", onUp);
    };
  });

  function startDrag() {
    dragging = true;
    document.body.style.cursor = "col-resize";
  }

  // ---- derived ----
  const src = $derived(app.activeSource);
  const dim = $derived(chunks.length > 0 ? chunks[0].dim : 0);
  const kindBadge = $derived(
    (src?.kind ?? "web") === "audio" ? "audio" : (src?.kind ?? "web")
  );
</script>

<div class="source-viewer">
  <!-- LEFT: embedding proof pane -->
  <div class="sv-pane sv-source" style:width="{app.chatOpen ? split : 100}%">
    <div class="sv-head">
      <button
        class="btn btn--icon btn--sm btn--ghost"
        onclick={() => app.closeSource()}
        title="Zurück"
      >
        <span style="display:block;transform:rotate(180deg)">
          <Icon name="chevron" size={13} color="currentColor" />
        </span>
      </button>

      {#if src}
        <span class="badge badge--{kindBadge}">
          <span class="dot"></span>
          {(src.kind ?? "web").toUpperCase()}
        </span>
        <span class="sv-name mono">{src.name}</span>
        <div class="grow"></div>
        {#if src.meta}
          <div class="sv-tools mono faint">{src.meta}</div>
        {/if}
      {/if}

      {#if src?.origin && !isMobile}
        {#if officeAppInfo}
          <button
            class="btn btn--sm btn--ghost sv-extbtn"
            onclick={() => openExternal(undefined)}
            title="In {officeAppInfo.label} öffnen"
          >
            <span class="sv-applogo" style="background:{officeAppInfo.color}">
              <svg viewBox="0 0 24 24" width="11" height="11"><path fill="#fff" d={officeAppInfo.svgPath} /></svg>
            </span>
            {officeAppInfo.label}
          </button>
        {:else if isEditorKind}
          <div class="sv-openwith">
            <button
              class="btn btn--sm btn--ghost sv-extbtn"
              onclick={(e) => { e.stopPropagation(); openWithMenu = !openWithMenu; }}
              title="Öffnen mit…"
            >
              Öffnen mit
              <Icon name="chevron" size={10} color="currentColor" />
            </button>
            {#if openWithMenu}
              <div class="sv-openwith-menu" role="menu">
                {#each EDITORS as ed (ed.with)}
                  <button class="sv-menu-item" role="menuitem" onclick={() => openExternal(ed.with)}>
                    <span class="sv-applogo sv-applogo--sm" style="background:{ed.color}">
                      <svg viewBox="0 0 24 24" width="12" height="12"><path fill="#fff" d={ed.svgPath} /></svg>
                    </span>
                    <span>{ed.label}</span>
                  </button>
                {/each}
              </div>
            {/if}
          </div>
        {/if}
      {/if}

      <button class="btn btn--icon btn--sm btn--ghost" title="Suchen">
        <Icon name="search" size={13} />
      </button>
      {#if src}
        <button
          class="btn btn--icon btn--sm btn--ghost"
          onclick={editSource}
          title="Quelle bearbeiten"
        >
          <Icon name="pencil" size={13} color="currentColor" />
        </button>
        <button
          class="btn btn--icon btn--sm btn--ghost sv-delete"
          onclick={confirmDelete}
          title="Quelle löschen"
        >
          <Icon name="x" size={13} color="currentColor" />
        </button>
      {/if}
    </div>

    <!-- Preview / embedding proof scrollable area -->
    <div class="sv-doc" class:sv-doc--flush={isPdfDoc || isImage || !!codeLang}>
      {#if officeError && !isPdfDoc}
        <p class="pdf-note mono sv-office-note">Originalvorschau nicht verfügbar: {officeError}</p>
      {/if}
      {#if src?.error}
        <!-- Failed ingest: surface the error prominently instead of an empty view -->
        <div class="pdf-page sv-error" style="width:100%;max-width:560px">
          <h3 class="pdf-h" style="color:var(--err, var(--warn))">Einlesen fehlgeschlagen</h3>
          <p class="read pdf-note">{src.error}</p>
          <p class="pdf-note mono" style="font-size:var(--t-xs);color:var(--fg-muted)">
            Diese Quelle konnte nicht verarbeitet werden. Lösche sie und füge sie erneut hinzu.
          </p>
        </div>
      {:else if isPdfDoc && assetUrl}
        <!-- PDF / rendered slide preview (also covers pptx & docx via rendered PDF) -->
        {#if isMobile}
          <!-- All pages via PDF.js (WKWebView's iframe shows only page 1) -->
          <div class="sv-pdf-pages" bind:this={pdfBox}></div>
          {#if pdfRendering}
            <p class="pdf-note mono" style="font-size:var(--t-xs);color:var(--fg-muted);text-align:center;padding:12px">Seiten werden dargestellt…</p>
          {/if}
          {#if pdfError}
            <p class="pdf-note mono" style="font-size:var(--t-xs);color:var(--err,var(--warn));text-align:center;padding:12px">PDF konnte nicht dargestellt werden: {pdfError}</p>
          {/if}
        {:else}
          <iframe class="sv-frame" src={assetUrl} title={src?.name ?? "Dokument"}></iframe>
        {/if}
      {:else if isImage && assetUrl}
        <!-- Image preview, centered & fit -->
        <div class="sv-img-wrap">
          <img class="sv-img" src={assetUrl} alt={src?.name ?? "Bild"} />
        </div>
      {:else if isAudio && assetUrl}
        <!-- Audio player + rendered overview note + transcript behind a toggle -->
        <div class="pdf-page" style="width:100%;max-width:560px">
          <audio class="sv-audio" controls src={assetUrl}></audio>
        </div>
        <div class="pdf-page" style="width:100%;max-width:560px">
          <h3 class="pdf-h">Überblick</h3>
          {#if summaryNote}
            <RichText text={summaryNote.body} />
          {:else}
            <p class="pdf-note mono" style="font-size:var(--t-xs);color:var(--fg-muted)">
              Noch kein Überblick – nach der Transkription wird automatisch eine Zusammenfassung
              erstellt und in den Notizen als „Zusammenfassung — {src?.name}“ gespeichert.
            </p>
          {/if}
        </div>
        <div class="pdf-page" style="width:100%;max-width:560px">
          <div class="sv-sec-head">
            <h3 class="pdf-h">Transkript</h3>
            <button
              class="btn btn--sm btn--ghost"
              onclick={() => (showTranscript = !showTranscript)}
            >
              {showTranscript ? "Transkript ausblenden" : "Transkript anzeigen"}
            </button>
          </div>
          {#if showTranscript}
            {#if transcript}
              <p class="read pdf-note sv-text">{transcript}</p>
            {:else}
              <p class="pdf-note mono" style="font-size:var(--t-xs);color:var(--fg-muted)">
                Noch kein Transkript vorhanden.
              </p>
            {/if}
          {/if}
        </div>
      {:else if src && officeRenderingId === src.id}
        <div class="pdf-page" style="width:100%;max-width:560px">
          <p class="pdf-note mono" style="font-size:var(--t-xs);color:var(--fg-muted)">
            Vorschau wird mit {officeAppInfo?.label ?? "Office"} erstellt…
          </p>
          <div class="pdf-line sk" style="width:60%;height:14px;margin:14px 0 18px"></div>
          <div class="pdf-line sk" style="width:90%"></div>
          <div class="pdf-line sk" style="width:80%"></div>
        </div>
      {:else if isTable && src?.content}
        <div class="pdf-page" style="width:100%;max-width:900px">
          <RichText text={src.content} />
        </div>
      {:else if codeLang && src?.content}
        <div class="sv-code hljs">
          <pre class="sv-code-gutter" aria-hidden="true">{Array.from({ length: codeLines }, (_, i) => i + 1).join("\n")}</pre>
          <pre class="sv-code-body"><code>{@html codeHtml}</code></pre>
        </div>
      {:else if isMarkdown && src?.content}
        <div class="pdf-page" style="width:100%;max-width:780px">
          <RichText text={src.content} />
        </div>
      {:else if isText && src?.content}
        <!-- Readable extracted text (txt / md / web / url, or content-only sources) -->
        <div class="pdf-page" style="width:100%;max-width:680px">
          <p class="read pdf-note sv-text">{src.content}</p>
        </div>
      {:else if loading}
        <div class="pdf-page" style="width:100%;max-width:560px">
          <div class="pdf-line sk" style="width:60%;height:14px;margin-bottom:18px"></div>
          <div class="pdf-line sk" style="width:90%"></div>
          <div class="pdf-line sk" style="width:80%"></div>
          <div class="pdf-line sk" style="width:70%"></div>
        </div>
      {:else}
        <!-- Fallback: raw chunk list (nothing richer to show) -->
        <div class="pdf-page" style="width:100%;max-width:560px">
          {#if chunks.length > 0}
            <h3 class="pdf-h" style="color:var(--ok)">
              ✓ {chunks.length} {chunks.length === 1 ? "Abschnitt" : "Abschnitte"} eingebettet · {dim}-dimensionale Vektoren
            </h3>
            <p class="pdf-note mono" style="font-size:var(--t-xs);color:var(--fg-muted)">
              Quelle vollständig verarbeitet und eingebettet. Jeder Abschnitt unten ist mit seinem Vektor gespeichert.
            </p>
          {:else}
            <h3 class="pdf-h" style="color:var(--warn)">nicht eingebettet / wird eingelesen</h3>
            <p class="pdf-note mono" style="font-size:var(--t-xs);color:var(--fg-muted)">
              Keine Abschnitte gefunden – die Quelle wird eventuell noch eingelesen oder konnte nicht verarbeitet werden.
            </p>
          {/if}
        </div>

        <!-- Chunk list -->
        {#each chunks as chunk (chunk.ord)}
          <div id={"sv-chunk-" + chunk.ord} class="pdf-page" style="width:100%;max-width:560px">
            <div class="pdf-pageno mono">#{chunk.ord}</div>
            {#if chunk.loc}
              <div class="pdf-formula mono" style="margin-bottom:10px">{chunk.loc}</div>
            {/if}
            <p class="read pdf-note">{chunk.text}</p>
          </div>
        {/each}
      {/if}
    </div>
  </div>

  <!-- Splitter + scoped chat — hidden when the chat is toggled off (c). The
       left pane expands to full width and App's "Ask c" FAB reopens it. -->
  {#if app.chatOpen}
    <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
    <div
      class="sv-splitter"
      role="separator"
      aria-orientation="vertical"
      tabindex="-1"
      onmousedown={startDrag}
    >
      <span></span>
    </div>

    <!-- RIGHT: scoped chat -->
    <div class="sv-pane sv-chat" style:width="{100 - split}%">
      <ChatPanel onClose={() => (app.chatOpen = false)} />
    </div>
  {/if}
</div>

<style>
  /* PDF / image previews fill the pane edge-to-edge instead of the padded,
     centered "document" column used by the chunk list. */
  .sv-doc--flush {
    padding: 0;
    gap: 0;
    align-items: stretch;
  }
  .sv-frame {
    flex: 1;
    width: 100%;
    border: none;
    background: var(--surface);
  }
  .sv-img-wrap {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: var(--sp-6);
    min-height: 0;
  }
  .sv-img {
    max-width: 100%;
    max-height: 100%;
    object-fit: contain;
    border-radius: var(--rad-2);
    border: 1px solid var(--border);
  }
  .sv-audio {
    width: 100%;
    display: block;
  }
  .sv-office-note {
    font-size: var(--t-xs);
    color: var(--warn);
    padding: 8px 16px;
    margin: 0;
  }
  .sv-code {
    flex: 1;
    display: flex;
    min-height: 100%;
    padding: 0;
    font-family: var(--font-mono);
    font-size: 13px;
    line-height: 1.55;
  }
  .sv-code pre {
    margin: 0;
    padding: 16px 0;
    font: inherit;
  }
  .sv-code-gutter {
    flex: none;
    padding: 16px 12px 16px 16px !important;
    text-align: right;
    color: #6e7681;
    user-select: none;
    border-right: 1px solid #2d2d2d;
  }
  .sv-code-body {
    flex: 1;
    overflow-x: auto;
    padding-left: 16px !important;
    padding-right: 16px !important;
    tab-size: 4;
  }
  .sv-extbtn {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 0 10px;
  }
  .sv-applogo {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 16px;
    height: 16px;
    border-radius: 4px;
    color: #fff;
    font-size: 10px;
    font-weight: 700;
    line-height: 1;
    flex: none;
  }
  .sv-applogo--sm {
    width: 18px;
    height: 18px;
    border-radius: 5px;
    font-size: 11px;
  }
  .sv-openwith {
    position: relative;
  }
  .sv-openwith-menu {
    position: absolute;
    top: calc(100% + 6px);
    right: 0;
    z-index: 20;
    min-width: 160px;
    padding: 5px;
    background: var(--surface-2);
    border: 1px solid var(--border-strong);
    border-radius: var(--rad-3);
    box-shadow: var(--shadow-pop);
  }
  .sv-menu-item {
    display: flex;
    align-items: center;
    gap: 10px;
    width: 100%;
    height: 30px;
    padding: 0 9px;
    border: none;
    background: none;
    cursor: pointer;
    border-radius: var(--rad-2);
    color: var(--fg-muted);
    font-family: var(--font-sans);
    font-size: var(--t-sm);
    text-align: left;
  }
  .sv-menu-item:hover {
    background: var(--surface-3);
    color: var(--fg-bright);
  }
  /* Section header row with an inline action (Transcript · View/Hide button). */
  .sv-sec-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--sp-3, 8px);
  }
  .sv-sec-head .pdf-h {
    margin: 0;
  }
  /* Preserve paragraphs/newlines for readable plaintext and transcripts. */
  .sv-text {
    white-space: pre-wrap;
    word-break: break-word;
  }
  .sv-error {
    border-color: color-mix(in oklab, var(--warn) 40%, var(--border));
  }
  .sv-delete:hover {
    color: var(--err, var(--warn));
  }
</style>
