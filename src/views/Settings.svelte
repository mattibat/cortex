<script lang="ts">
  import { app, THEMES, THEME_LABELS } from "../lib/store.svelte";
  import type { Theme } from "../lib/store.svelte";
  import { isMobile } from "../lib/platform";
  import * as api from "../lib/api";
  import { getVersion } from "@tauri-apps/api/app";
  import type { Memory } from "../lib/api";
  import Icon from "../components/Icon.svelte";
  import Picker from "../components/Picker.svelte";
  import ModelSearch from "../components/ModelSearch.svelte";
  import { loadOpenRouterModels, type OrModel } from "../lib/openrouter";
  import { stations } from "../lib/mock";
  import { keybinds, ACTION_LABELS, ACTION_ORDER, LEADER_ACTIONS } from "../lib/keybinds.svelte";
  import type { Action } from "../lib/keybinds.svelte";
  import type { Snippet } from "svelte";

  // Shape for the reusable homelab endpoint-service snippet (Integrations tab).
  type EndpointOpts = {
    title: string;
    desc: string;
    value: string;
    oninput: (v: string) => void;
    onsave: () => void;
    onTest: () => void;
    state: null | "testing" | "ok" | "fail";
    placeholder: string;
    failHint?: string;
    hint?: string;
    extra?: Snippet;
  };

  // Fixed system bindings (not rebindable) shown for reference in the Keybinds tab
  // so the page reflects every shortcut, not just the customizable single-key set.
  const SYSTEM_BINDS = [
    { keys: "Ctrl F", label: "Auf der Seite suchen" },
    { keys: "Ctrl P", label: "Befehlspalette" },
    { keys: "Esc",    label: "Overlay schließen / zurück" },
    { keys: "g d",    label: "Zur Übersicht" },
    { keys: "Alt 1–9", label: "Zu Fach N springen" },
  ];

  // ---- tab navigation ----
  const TABS = [
    { id: "profile",    label: "Profil",        icon: "diamond" },
    { id: "models",     label: "Modelle",       icon: "bolt" },
    { id: "keys",       label: "API-Schlüssel", icon: "lock" },
    { id: "appearance", label: "Aussehen",      icon: "grid" },
    { id: "keybinds",   label: "Tastenkürzel",  icon: "cmd" },
    { id: "homelab",    label: "Integrationen", icon: "globe" },
    { id: "calendar",   label: "Google Calendar", icon: "globe" },
    { id: "experimental", label: "Experimentell", icon: "bolt" },
    { id: "audio",      label: "Audio",         icon: "music" },
    { id: "data",       label: "Daten & Datenschutz", icon: "doc" },
    { id: "about",      label: "Über",          icon: "diamond" },
  ] as const;

  // Mobile is a homelab-first portable view: drop the desktop-only Keybinds tab
  // and lead with Integrations (homelab). Desktop order/contents are unchanged.
  const MOBILE_TABS = ["homelab", "keys", "models", "appearance", "calendar", "experimental", "audio", "data", "profile", "about"];
  const navTabs = isMobile
    ? MOBILE_TABS.map((id) => TABS.find((t) => t.id === id)).filter((t): t is (typeof TABS)[number] => !!t)
    : TABS;

  let tab = $state<string>("profile");
  // Honour a deep-link from elsewhere (e.g. the subject Overview → Moodle setup).
  $effect(() => {
    if (app.settingsTab) {
      tab = app.settingsTab;
      app.settingsTab = null;
    }
  });

  // This view owns the global keys while open
  $effect(() => {
    (window as any).__cortexViewKeys = true;
    return () => { (window as any).__cortexViewKeys = false; };
  });

  // ---- providers ----
  // Model catalog. Each provider's models are ordered by MY cost-to-quality read —
  // best value (cheap + capable) first, then premium/frontier, then reasoning — and
  // labelled with a human name + a one-word tier hint so the dropdown is scannable.
  // `custom` (OpenRouter / OpenAI-compatible) lets you type any slug not listed.
  type Model = { id: string; label: string };
  const PROVIDERS: { id: string; label: string; models: Model[] }[] = [
    { id: "gemini", label: "Gemini", models: [
      { id: "gemini-2.5-flash",      label: "Gemini 2.5 Flash – ⚡ bestes Preis-Leistungs-Verhältnis" },
      { id: "gemini-2.5-flash-lite", label: "Gemini 2.5 Flash-Lite – am günstigsten" },
      { id: "gemini-2.0-flash-001",  label: "Gemini 2.0 Flash – älter, günstig" },
      { id: "gemini-2.5-pro",        label: "Gemini 2.5 Pro – ★ Premium" },
    ] },
    { id: "openrouter", label: "OpenRouter", models: [
      // Curated shortlist + ordering (the live catalog with per-row pricing replaces
      // these labels once it loads). Order ≈ best value → all-rounders → premium →
      // reasoning; the editorial "cheap/fast" tags are dropped since the searchable
      // list shows price-per-token + context per row.
      { id: "deepseek/deepseek-v4-flash",         label: "DeepSeek V4 Flash" },
      { id: "google/gemini-2.5-flash",            label: "Gemini 2.5 Flash" },
      { id: "google/gemini-2.5-flash-lite",       label: "Gemini 2.5 Flash-Lite" },
      { id: "openai/gpt-5-mini",                  label: "GPT-5 mini" },
      { id: "openai/gpt-4o-mini",                 label: "GPT-4o mini" },
      { id: "anthropic/claude-3.5-haiku",         label: "Claude 3.5 Haiku" },
      { id: "stepfun/step-3.7-flash",             label: "Step 3.7 Flash" },
      { id: "deepseek/deepseek-chat",             label: "DeepSeek V3" },
      { id: "meta-llama/llama-3.3-70b-instruct",  label: "Llama 3.3 70B" },
      { id: "qwen/qwen-2.5-72b-instruct",         label: "Qwen 2.5 72B" },
      { id: "mistralai/mistral-large",            label: "Mistral Large" },
      { id: "x-ai/grok-2-1212",                   label: "Grok 2" },
      { id: "google/gemini-2.0-flash-001",        label: "Gemini 2.0 Flash" },
      { id: "openai/gpt-4o",                      label: "GPT-4o" },
      { id: "anthropic/claude-3.5-sonnet",        label: "Claude 3.5 Sonnet" },
      { id: "anthropic/claude-3.7-sonnet",        label: "Claude 3.7 Sonnet" },
      { id: "google/gemini-2.5-pro",              label: "Gemini 2.5 Pro" },
      { id: "anthropic/claude-sonnet-4.5",        label: "Claude Sonnet 4.5" },
      { id: "openai/gpt-5",                       label: "GPT-5" },
      { id: "anthropic/claude-opus-4.1",          label: "Claude Opus 4.1" },
      { id: "x-ai/grok-3",                        label: "Grok 3" },
      { id: "openai/o3-mini",                     label: "o3-mini" },
      { id: "openai/o3",                          label: "o3" },
      { id: "deepseek/deepseek-r1",               label: "DeepSeek R1" },
    ] },
    { id: "openai", label: "OpenAI", models: [
      { id: "gpt-4o-mini", label: "GPT-4o mini – günstig" },
      { id: "gpt-5-mini",  label: "GPT-5 mini – günstig + klug" },
      { id: "gpt-4o",      label: "GPT-4o – ausgewogen" },
      { id: "gpt-5",       label: "GPT-5 – ★ Spitzenmodell" },
      { id: "o3-mini",     label: "o3-mini – günstiges Reasoning" },
      { id: "o3",          label: "o3 – tiefes Reasoning" },
    ] },
    { id: "claude", label: "Claude", models: [
      { id: "claude-haiku-4-5-20251001",  label: "Claude Haiku 4.5 – schnell + günstig" },
      { id: "claude-3-5-haiku-20241022",  label: "Claude 3.5 Haiku – günstig" },
      { id: "claude-3-5-sonnet-20241022", label: "Claude 3.5 Sonnet – stark" },
      { id: "claude-3-7-sonnet-20250219", label: "Claude 3.7 Sonnet – stark" },
      { id: "claude-sonnet-4-6",          label: "Claude Sonnet 4.6 – ★ Premium" },
      { id: "claude-opus-4-8",            label: "Claude Opus 4.8 – top, teuer" },
    ] },
    { id: "ollama", label: "Ollama (lokal)", models: [
      { id: "mistral-small", label: "Mistral Small – leicht, lokal" },
      { id: "qwen2.5:32b",   label: "Qwen 2.5 32B – lokal" },
      { id: "llama3.3:70b",  label: "Llama 3.3 70B – lokal, schwer" },
    ] },
    { id: "custom", label: "Eigener Endpunkt", models: [] },
  ];
  const EMBED_PROVIDERS: { id: string; label: string; models: Model[] }[] = [
    { id: "gemini",  label: "Gemini",        models: [{ id: "text-embedding-004", label: "text-embedding-004" }] },
    { id: "openai",  label: "OpenAI",        models: [{ id: "text-embedding-3-small", label: "text-embedding-3-small – günstig" }, { id: "text-embedding-3-large", label: "text-embedding-3-large – beste Qualität" }] },
    { id: "ollama",  label: "Ollama (lokal)", models: [{ id: "nomic-embed-text", label: "nomic-embed-text – lokal" }, { id: "mxbai-embed-large", label: "mxbai-embed-large – lokal" }] },
  ];
  const MODEL_TASKS = [
    { id: "chat",       label: "Chat",                  desc: "Fragen & Antworten zu deinen Quellen" },
    { id: "cheatsheet", label: "Lernzettel erstellen",   desc: "Auf Vollständigkeit geprüft" },
    { id: "audio",      label: "Skript Audio-Überblick", desc: "Podcast-Dialog mit zwei Sprechern" },
    { id: "quiz",       label: "Quiz erstellen",        desc: "Multiple Choice · Kurzantwort · Lückentext" },
    { id: "flashcard",  label: "Karteikarten erstellen", desc: "Frage/Antwort + Wiederholungsplan" },
    { id: "embedding",  label: "Embedding",             desc: "Vektorindex für die Suche" },
  ] as const;

  type TaskId = typeof MODEL_TASKS[number]["id"];

  // ---- profile state ----
  let name       = $state("");
  let pronouns   = $state("");
  let level      = $state("postgrad");
  let field      = $state("");
  let about      = $state("");
  let style      = $state("balanced");
  let explain    = $state<string[]>(["worked-examples","analogies"]);

  // ---- long-term memory state ----
  let memories     = $state<Memory[]>([]);
  let newMemory    = $state("");
  let memoryBusy   = $state(false);

  async function loadMemory() {
    memories = await api.listMemory().catch(() => [] as Memory[]);
  }
  async function addMemoryFact() {
    const text = newMemory.trim();
    if (!text || memoryBusy) return;
    memoryBusy = true;
    try {
      await api.addMemory(text);
      newMemory = "";
      await loadMemory();
    } catch {
      app.pushToast({ kind: "error", title: "Erinnerung konnte nicht gespeichert werden" });
    } finally {
      memoryBusy = false;
    }
  }
  async function removeMemory(id: string) {
    try {
      await api.deleteMemory(id);
      await loadMemory();
    } catch {
      app.pushToast({ kind: "error", title: "Erinnerung konnte nicht gelöscht werden" });
    }
  }

  // ---- models state ----
  type TaskAssign = { provider: string; model: string; budget: string };
  // Defaults: DeepSeek V4 Flash (via OpenRouter) for ALL text generation —
  // extremely cheap ($0.09/$0.18 per Mtok), 1M context, fast and non-reasoning.
  // Falls back to any configured key if OpenRouter isn't set (see
  // llm::from_spec_or_any). Embeddings stay on Gemini (DeepSeek doesn't embed).
  let assign = $state<Record<TaskId, TaskAssign>>({
    chat:       { provider: "openrouter", model: "deepseek/deepseek-v4-flash", budget: "8000" },
    cheatsheet: { provider: "openrouter", model: "deepseek/deepseek-v4-flash", budget: "32000" },
    audio:      { provider: "openrouter", model: "deepseek/deepseek-v4-flash", budget: "16000" },
    quiz:       { provider: "openrouter", model: "deepseek/deepseek-v4-flash", budget: "8000" },
    flashcard:  { provider: "openrouter", model: "deepseek/deepseek-v4-flash", budget: "6000" },
    embedding:  { provider: "gemini",     model: "text-embedding-004",        budget: "—" },
  });

  function setTask(id: TaskId, patch: Partial<TaskAssign>) {
    assign = { ...assign, [id]: { ...assign[id], ...patch } };
  }

  // ---- keys state ----
  let keys = $state({
    openrouter: "",
    gemini: "",
    claude: "",
    openai: "",
    custom_endpoint: "",
    custom_api_key: "",
  });
  const keyMeta = [
    { id: "openrouter", label: "OpenRouter",              note: "openrouter.ai/keys",    placeholder: "sk-or-…" },
    { id: "gemini",     label: "Gemini",                  note: "Google AI Studio",       placeholder: "AIza…" },
    { id: "claude",     label: "Claude",                  note: "console.anthropic.com",  placeholder: "sk-ant-…" },
    { id: "openai",     label: "OpenAI",                  note: "platform.openai.com",    placeholder: "sk-…" },
    { id: "custom_endpoint", label: "URL des eigenen Endpunkts", note: "OpenAI-kompatible Basis-URL", placeholder: "https://…/v1" },
    { id: "custom_api_key", label: "API-Schlüssel des eigenen Endpunkts", note: "Bearer-Token für den eigenen Endpunkt", placeholder: "sk-…" },
  ] as const;
  // show/hide per key
  let showKey = $state<Record<string, boolean>>({
    openrouter: false,
    gemini: false,
    claude: false,
    openai: false,
    custom_endpoint: false,
    custom_api_key: false,
  });

  // ---- appearance state ----
  const THEME_OPTS: { id: Theme; n: string; c: string; b: string }[] = [
    { id: "osaka-jade",  n: "Osaka Jade",       c: "#2dd5b7", b: "#111c18" },
    { id: "tokyo-night", n: "Tokyo Night",      c: "#7aa2f7", b: "#1a1b26" },
    { id: "catppuccin",  n: "Catppuccin Mocha", c: "#94e2d5", b: "#1e1e2e" },
    { id: "gruvbox",     n: "Gruvbox",          c: "#fabd2f", b: "#282828" },
    { id: "nord",        n: "Nord",             c: "#88c0d0", b: "#2e3440" },
    { id: "dracula",     n: "Dracula",          c: "#bd93f9", b: "#282a36" },
    { id: "rose-pine",   n: "Rosé Pine",        c: "#ebbcba", b: "#191724" },
    { id: "everforest",  n: "Everforest",       c: "#a7c080", b: "#2d353b" },
    { id: "solarized",   n: "Solarized",        c: "#268bd2", b: "#002b36" },
    { id: "kanagawa",    n: "Kanagawa",         c: "#7e9cd8", b: "#1f1f28" },
  ];
  let readFont      = $state("mono");
  let density       = $state("regular");

  // Settings is the ONLY view that mutates the <html> root. Writing an attribute on
  // documentElement invalidates styles for the ENTIRE document, forcing WebKit
  // (WebKitGTK / iOS WKWebView) to re-match every element against the large global
  // stylesheet (~1740 rules) — ~2× slower than Blink, which is why only Settings
  // freezes/lags on WebKit. The store already applies these on boot, so these were
  // redundant no-op writes that still triggered a full-document recalc every open.
  // Only write when the value actually changes (a getAttribute read is cheap and
  // does NOT force layout); live appearance changes still apply instantly.
  $effect(() => {
    const el = document.documentElement;
    if (el.getAttribute("data-read") !== readFont) el.setAttribute("data-read", readFont);
  });
  $effect(() => {
    const el = document.documentElement;
    const v = density === "compact" ? "compact" : "regular";
    if (el.getAttribute("data-density") !== v) el.setAttribute("data-density", v);
  });

  // persist appearance on change
  $effect(() => {
    // track both values
    const rf = readFont;
    const d  = density;
    if (!loaded) return;
    api.setSettings({ reading_font: rf, density: d }).catch(() => {});
  });

  // ---- keybinds state ----
  // Binds live in the shared keybinds module (persisted + applied live).
  let listening = $state<Action | null>(null);

  function displayKey(k: string): string {
    return k === " " ? "Space" : k;
  }

  $effect(() => {
    if (!listening) return;
    (window as any).__cortexModalOpen = true;
    function onKey(e: KeyboardEvent) {
      // Ignore standalone modifier presses so the user can type a shifted key
      // (e.g. press Shift then ":") — keep listening until a real key arrives.
      if (["Shift", "Control", "Alt", "Meta", "AltGraph", "CapsLock"].includes(e.key)) {
        return;
      }
      // Swallow the event entirely so the global keyboard engine doesn't also
      // act on it (otherwise ":" would open the command palette).
      e.preventDefault();
      e.stopPropagation();
      e.stopImmediatePropagation();
      const k = e.key === "Escape" ? null : e.key;
      if (k && listening) {
        const action = listening;
        const ok = keybinds.set(action, k); // persists + applies live
        if (ok) {
          app.pushToast({
            kind: "success",
            title: "Tastenkürzel geändert",
            body: `${ACTION_LABELS[action]} → „${k === " " ? "Leertaste" : k}“. Gilt außerhalb dieser Ansicht.`,
          });
        } else {
          app.pushToast({
            kind: "warning",
            title: "Taste bereits belegt",
            body: `„${k === " " ? "Leertaste" : k}“ ist schon einer anderen Aktion zugewiesen – wähle eine andere Taste.`,
          });
        }
      }
      listening = null;
      (window as any).__cortexModalOpen = false;
    }
    window.addEventListener("keydown", onKey, true);
    return () => {
      window.removeEventListener("keydown", onKey, true);
      (window as any).__cortexModalOpen = false;
    };
  });

  // ---- local models (ollama) + web search + remote whisper ----
  let endpoint   = $state("http://localhost:11434");
  let searxng    = $state("");
  let whisperUrl = $state("");
  let whisperModel = $state("");
  let webImages  = $state(false);
  let testState  = $state<null | "testing" | "ok" | "fail">(null);
  let searxState = $state<null | "testing" | "ok" | "fail">(null);
  let whisperState = $state<null | "testing" | "ok" | "fail">(null);

  function saveWhisper() {
    api.setSetting("whisper_url", whisperUrl.trim()).catch(() => {});
  }
  function saveWhisperModel() {
    api.setSetting("whisper_model", whisperModel.trim()).catch(() => {});
  }
  async function testWhisper() {
    if (!whisperUrl.trim()) return;
    whisperState = "testing";
    whisperState = await testEndpoint(whisperUrl);
  }
  // End-to-end whisper validation: server reachable AND the configured model
  // installed — downloading it server-side on the spot when missing, so the
  // first real lecture never pays the multi-minute cold pull.
  let whisperCheckState = $state<null | "checking" | "ok" | "fail">(null);
  let whisperCheckNote = $state("");
  async function checkWhisper() {
    whisperCheckState = "checking";
    whisperCheckNote = "Server wird geprüft (der erste Modell-Download kann ein paar Minuten dauern)…";
    try {
      whisperCheckNote = await api.checkWhisperModel();
      whisperCheckState = "ok";
    } catch (e) {
      whisperCheckNote = String(e);
      whisperCheckState = "fail";
    }
  }

  // ---- transcription mode (Settings → Integrations → Transcription) ----
  // "local" = on this machine (zero setup) · "cloud" = OpenAI-compatible API
  // with a key (Groq/OpenAI/custom) · "homelab" = the /whisper service behind
  // the Homelab URL. Unset falls back to the legacy auto behavior (homelab if
  // configured, else local) — shown as whichever of those applies.
  let transcriptionMode = $state<"local" | "cloud" | "homelab">("local");
  let whisperCloudProvider = $state<"groq" | "openai" | "custom">("groq");
  let whisperCloudUrl = $state("");
  let whisperCloudModel = $state("");
  let whisperApiKey = $state("");
  const CLOUD_WHISPER_PRESETS = {
    groq: { url: "https://api.groq.com/openai/v1", model: "whisper-large-v3-turbo" },
    openai: { url: "https://api.openai.com/v1", model: "whisper-1" },
  } as const;
  function setTranscriptionMode(m: "local" | "cloud" | "homelab") {
    transcriptionMode = m;
    whisperCheckState = null;
    whisperCheckNote = "";
    api.setSetting("transcription_mode", m).catch(() => {});
    // Entering cloud mode with everything blank: land on the Groq preset so the
    // only thing left to do is paste a key.
    if (m === "cloud" && !whisperCloudUrl.trim() && !whisperCloudModel.trim()) {
      applyCloudPreset("groq");
    }
  }
  function applyCloudPreset(p: "groq" | "openai" | "custom") {
    whisperCloudProvider = p;
    api.setSetting("whisper_cloud_provider", p).catch(() => {});
    if (p !== "custom") {
      whisperCloudUrl = CLOUD_WHISPER_PRESETS[p].url;
      whisperCloudModel = CLOUD_WHISPER_PRESETS[p].model;
      saveCloudWhisper();
    }
  }
  function saveCloudWhisper() {
    api.setSettings({
      whisper_cloud_url: whisperCloudUrl.trim(),
      whisper_cloud_model: whisperCloudModel.trim(),
      whisper_api_key: whisperApiKey.trim(),
    }).catch(() => {});
  }

  // ---- homelab access token (auth for every service except /sync) ----
  let hlToken = $state("");
  function saveHlToken() {
    api.setSetting("homelab_token", hlToken.trim()).catch(() => {});
  }

  // ---- unified homelab access ----
  // One base URL fronts every service (search / whisper / ollama / sync) behind
  // a reverse proxy; Cortex appends each service's path. Tailscale/public bases
  // are the auto local→Tailscale→public fallbacks.
  let hlBase      = $state("");
  let hlTailscale = $state("");
  let hlPublic    = $state("");
  let hlState     = $state<"idle" | "testing" | "ok" | "fail">("idle");
  let hlReach     = $state<Record<string, "ok" | "fail">>({});
  let hlSaveTimer: ReturnType<typeof setTimeout> | undefined;

  // Real app version for the footer (the Tauri build version), loaded once.
  let appVersion = $state("");
  $effect(() => { void getVersion().then((v) => (appVersion = v)).catch(() => {}); });
  function saveHomelabBases() {
    clearTimeout(hlSaveTimer);
    api.setSettings({
      homelab_base: hlBase.trim(),
      homelab_tailscale_base: hlTailscale.trim(),
      homelab_public_base: hlPublic.trim(),
    }).catch(() => {});
  }
  // iOS WKWebView fires change/blur unreliably (and "Sync now" lives in another
  // section), so the base could stay unpersisted → sync reports "target not set".
  // Persist shortly after each keystroke so the URL is always in the DB by sync time.
  function saveHomelabBasesSoon() {
    clearTimeout(hlSaveTimer);
    hlSaveTimer = setTimeout(saveHomelabBases, 400);
  }
  // Per-service health grid (SearXNG / WhisperX / WebDAV / syncd / ingest / Ollama),
  // probed through the SAME resolved URLs the app actually uses.
  let svcStatus = $state<api.HomelabServiceStatus[] | null>(null);
  let svcTesting = $state(false);
  async function testHomelab() {
    const tiers: [string, string][] = [["local", hlBase], ["tailscale", hlTailscale], ["public", hlPublic]];
    if (!tiers.some(([, u]) => u.trim())) return;
    hlState = "testing";
    svcTesting = true;
    const reach: Record<string, "ok" | "fail"> = {};
    for (const [tier, url] of tiers) {
      if (!url.trim()) continue;
      reach[tier] = await testEndpoint(url.trim());
    }
    hlReach = reach;
    hlState = Object.values(reach).some((r) => r === "ok") ? "ok" : "fail";
    // Then check each service individually so a broken one is named, not guessed.
    try { svcStatus = await api.homelabStatus(); } catch { svcStatus = null; }
    svcTesting = false;
  }
  // Whether the homelab's live-sync service answered on the last test (null = unknown).
  const syncdOk = $derived(svcStatus?.find((s) => s.id === "syncd")?.ok ?? null);

  // ---- live homelab sync (smart per-record merge + binary file sync) ----
  // Three endpoint tiers tried in order (auto): local LAN → Tailscale → public.
  let syncUrl   = $state(""); // local / LAN
  let syncUrlTs = $state(""); // tailscale
  let syncUrlPub = $state(""); // public
  let syncMode  = $state<"auto" | "local" | "tailscale" | "public">("auto");
  let syncUser  = $state("");
  let syncPass  = $state("");
  let syncOn    = $state(false);
  let syncTestState = $state<null | "testing" | "ok" | "fail">(null);
  // Per-endpoint reachability for the Test-all button.
  let syncReach = $state<Record<string, boolean>>({});

  const anySyncUrl = $derived(!!(syncUrl.trim() || syncUrlTs.trim() || syncUrlPub.trim()));
  // On mobile the per-tier sync URL fields are hidden — live sync rides the single
  // Homelab base URL (→ its /sync WebDAV). So the base alone must be enough to enable
  // the toggle; gating only on anySyncUrl left mobile stuck on "live sync off".
  const canSync = $derived(anySyncUrl || !!hlBase.trim() || !!hlTailscale.trim() || !!hlPublic.trim());

  let syncSaveTimer: ReturnType<typeof setTimeout> | undefined;
  function saveSync() {
    clearTimeout(syncSaveTimer);
    api.setSettings({
      sync_url: syncUrl.trim(),
      sync_url_tailscale: syncUrlTs.trim(),
      sync_url_public: syncUrlPub.trim(),
      sync_mode: syncMode,
      sync_user: syncUser.trim(),
      sync_pass: syncPass,
      sync_enabled: syncOn ? "true" : "false",
    }).then(() => app.loadSyncStatus()).catch(() => {});
  }
  // Same iOS blur/change unreliability as the homelab bases — persist creds as typed.
  function saveSyncSoon() {
    clearTimeout(syncSaveTimer);
    syncSaveTimer = setTimeout(saveSync, 400);
  }
  function toggleSync() {
    syncOn = !syncOn;
    if (syncOn && !canSync) { syncOn = false; return; }
    saveSync();
  }
  function setSyncMode(m: "auto" | "local" | "tailscale" | "public") {
    syncMode = m;
    saveSync();
  }
  async function testSync() {
    if (!anySyncUrl) return;
    syncTestState = "testing";
    const reach: Record<string, boolean> = {};
    const tiers: [string, string][] = [["local", syncUrl], ["tailscale", syncUrlTs], ["public", syncUrlPub]];
    for (const [tier, url] of tiers) {
      if (!url.trim()) continue;
      try { reach[tier] = await api.syncTest(url.trim(), syncUser.trim(), syncPass); }
      catch { reach[tier] = false; }
    }
    syncReach = reach;
    try {
      syncTestState = Object.values(reach).some(Boolean) ? "ok" : "fail";
    } catch {
      syncTestState = "fail";
    }
  }
  function fmtSyncTime(ms: number): string {
    return ms ? new Date(ms).toLocaleString() : "nie";
  }
  function syncPill() {
    // Live (WebSocket) trumps the periodic states — changes are propagating
    // across devices within about a second.
    if (app.syncLive && app.syncState !== "off" && app.syncState !== "error") {
      return { cls: "ready", label: "Live" };
    }
    switch (app.syncState) {
      case "syncing": return { cls: "draft", label: "Synchronisiert…" };
      case "synced":  return { cls: "ready", label: "Synchronisiert" };
      case "error":   return { cls: "error", label: "Sync-Fehler" };
      case "idle":    return { cls: "ready", label: "An" };
      default:        return { cls: "pending", label: "Aus" };
    }
  }

  // ---- Experimental features ----
  let expMoodle = $state(false);
  function toggleExpMoodle() {
    expMoodle = !expMoodle;
    api.setSetting("exp_moodle", expMoodle ? "true" : "false").catch(() => {});
  }

  // ---- Moodle integration ----
  let mdUrl   = $state("");
  let mdUser  = $state("");
  let mdPass  = $state("");
  let mdToken = $state("");
  let mdAuthMode = $state<"password" | "token">("token"); // SU uses Microsoft SSO → token paste
  let mdBusy  = $state(false);
  let mdStatus = $state<api.MoodleStatus>({ configured: false, user_id: 0, last_sync: 0 });
  let mdData = $state<api.MoodleData>({ courses: [], grades: [], deadlines: [], announcements: [] });
  let mdSummary = $state<api.MoodleSummary | null>(null);
  // Upcoming deadlines (future, soonest first) for the synced-data preview.
  const mdUpcoming = $derived(
    mdData.deadlines
      .filter((d) => d.due_at * 1000 >= Date.now() - 12 * 3600 * 1000)
      .sort((a, b) => a.due_at - b.due_at)
  );

  async function loadMoodle() {
    try {
      mdStatus = await api.moodleStatus();
      if (mdStatus.configured) mdData = await api.moodleData();
    } catch { /* not configured yet */ }
  }
  async function mdConnect() {
    mdBusy = true;
    try {
      const name = mdAuthMode === "password"
        ? await api.moodleConnect(mdUrl, mdUser.trim(), mdPass)
        : await api.moodleSetToken(mdUrl, mdToken);
      mdPass = ""; // never keep the password around
      app.pushToast({ kind: "success", title: "Moodle verbunden", body: name || undefined });
      await loadMoodle();
    } catch (e) {
      app.pushToast({ kind: "error", title: "Moodle-Verbindung fehlgeschlagen", body: String(e) });
    } finally { mdBusy = false; }
  }
  async function mdSyncNow() {
    mdBusy = true;
    try {
      mdSummary = await api.moodleSync();
      app.pushToast({ kind: "success", title: "Moodle synchronisiert",
        body: `${mdSummary.courses} Kurse · ${mdSummary.grades} Noten · ${mdSummary.deadlines} Termine · ${mdSummary.announcements} Ankündigungen` });
      await loadMoodle();
    } catch (e) {
      app.pushToast({ kind: "error", title: "Moodle-Sync fehlgeschlagen", body: String(e) });
    } finally { mdBusy = false; }
  }
  async function mdAutolink() {
    try {
      const n = await api.moodleAutolink();
      app.pushToast({ kind: n ? "success" : "info", title: `${n} ${n === 1 ? "Fach" : "Fächer"} automatisch verknüpft` });
      await app.refresh();
    } catch (e) {
      app.pushToast({ kind: "error", title: "Automatisches Verknüpfen fehlgeschlagen", body: String(e) });
    }
  }
  async function mdDisconnect() {
    try {
      await api.moodleDisconnect();
      mdToken = "";
      await loadMoodle();
      app.pushToast({ kind: "info", title: "Moodle getrennt" });
    } catch { /* ignore */ }
  }
  function mdCourseName(courseId: string): string {
    return mdData.courses.find((c) => c.id === courseId)?.fullname || courseId;
  }
  async function mdLoginSso() {
    try {
      await api.moodleLoginSso(mdUrl);
    } catch (e) {
      app.pushToast({ kind: "error", title: "SSO-Anmeldung konnte nicht geöffnet werden", body: String(e) });
    }
  }
  // SSO login happens in a separate window; react to its result here.
  $effect(() => {
    let un1: (() => void) | undefined;
    let un2: (() => void) | undefined;
    api.onMoodleSsoDone(async (name) => {
      app.pushToast({ kind: "success", title: "Moodle verbunden", body: name || undefined });
      await loadMoodle();
    }).then((u) => (un1 = u));
    api.onMoodleSsoError((msg) =>
      app.pushToast({ kind: "error", title: "SSO-Anmeldung fehlgeschlagen", body: msg })
    ).then((u) => (un2 = u));
    return () => { un1?.(); un2?.(); };
  });

  // Diagrams/images from the homelab SearXNG. Mirrors app.webImagesEnabled and
  // persists an explicit choice so it survives the "default-on-when-connected".
  function setWebImages(on: boolean) {
    webImages = on;
    app.webImagesEnabled = on;
    api.setSetting("web_images_enabled", on ? "true" : "false").catch(() => {});
  }

  // ---- encrypted backups (age + rclone) ----
  let ageRecipient = $state("");
  let rcloneRemote = $state("");
  let backupInfo   = $state<api.BackupStatus | null>(null);
  let backingUp    = $state(false);
  async function refreshBackupStatus() {
    try { backupInfo = await api.backupStatus(); } catch { /* non-fatal */ }
  }
  function saveBackupConfig() {
    api.setSettings({
      backup_age_recipient: ageRecipient.trim(),
      backup_rclone_remote: rcloneRemote.trim(),
    }).then(refreshBackupStatus).catch(() => {});
  }
  async function runBackup() {
    if (backingUp) return;
    backingUp = true;
    try {
      const dest = await api.backupNow();
      app.pushToast({ kind: "success", title: "Backup hochgeladen", body: dest });
      await refreshBackupStatus();
    } catch (e) {
      app.pushToast({ kind: "error", title: "Backup fehlgeschlagen", body: String(e) });
    } finally {
      backingUp = false;
    }
  }
  function fmtBackupTime(ms: number | null): string {
    if (!ms) return "never";
    return new Date(ms).toLocaleString();
  }

  // Last transport error from a failed reachability test, so the homelab card can
  // explain WHY (e.g. iOS blocking cleartext LAN, DNS, timeout) instead of "fail".
  let lastTestError = $state("");
  async function testEndpoint(url: string): Promise<"ok" | "fail"> {
    try {
      const ok = await api.pingUrl(url);
      lastTestError = ok ? "" : "Reached the server, but it didn't return a success status.";
      return ok ? "ok" : "fail";
    } catch (e) {
      lastTestError = String(e);
      return "fail";
    }
  }

  async function testConnection() {
    testState = "testing";
    testState = await testEndpoint(endpoint);
  }

  function saveSearxng() {
    api.setSetting("searxng_url", searxng).catch(() => {});
  }

  let schoolFolderPath = $state("");
  let schoolSyncing = $state(false);
  let schoolSyncResult = $state<api.SchoolSyncResult | null>(null);
  async function chooseSchoolFolder() {
    try {
      const { open } = await import("@tauri-apps/plugin-dialog");
      const dir = await open({ directory: true, multiple: false });
      if (!dir || typeof dir !== "string") return;
      schoolFolderPath = dir;
      await api.setSetting("school_folder_path", dir);
      await runSchoolSync();
    } catch (e) {
      app.pushToast({ kind: "error", title: "Ordnerauswahl fehlgeschlagen", body: String(e) });
    }
  }
  async function runSchoolSync() {
    if (!schoolFolderPath.trim()) return;
    schoolSyncing = true;
    try {
      schoolSyncResult = await api.syncSchoolFolder();
      await app.refresh();
    } catch (e) {
      app.pushToast({ kind: "error", title: "Schule-Sync fehlgeschlagen", body: String(e) });
    } finally {
      schoolSyncing = false;
    }
  }

  // ---- Google Calendar ----
  let gClientId = $state("");
  let gClientSecret = $state("");
  let gStatus = $state<api.GoogleStatus | null>(null);
  let gBusy = $state(false);
  $effect(() => {
    if (tab === "calendar" && gStatus === null) loadGoogle();
    if (tab === "homelab" && depReport === null) loadDeps();
    if (tab === "data") loadArchived();
  });

  // ---- external dependency status ----
  let depReport = $state<api.DependencyReport | null>(null);
  let depLoading = $state(false);
  async function loadDeps() {
    depLoading = true;
    try { depReport = await api.dependencyStatus(); }
    catch { depReport = null; }
    finally { depLoading = false; }
  }
  async function copyDepCmd() {
    if (!depReport?.install_command) return;
    try {
      await navigator.clipboard.writeText(depReport.install_command);
      app.pushToast({ kind: "success", title: "Kopiert", body: "Installationsbefehl in die Zwischenablage kopiert." });
    } catch {
      app.pushToast({ kind: "warning", title: "Kopieren nicht möglich", body: depReport.install_command });
    }
  }
  // One-click install (macOS/Homebrew only — no sudo needed there).
  let depInstalling = $state(false);
  async function installDeps() {
    depInstalling = true;
    try {
      const msg = await api.installDependencies();
      app.pushToast({ kind: "success", title: "Abhängigkeiten", body: msg });
      await loadDeps();
    } catch (e) {
      app.pushToast({ kind: "error", title: "Installation fehlgeschlagen", body: String(e) });
    } finally {
      depInstalling = false;
    }
  }
  let gCalendars = $state<api.GoogleCalendar[]>([]);
  let gCalBusy = $state(false);
  async function loadGoogle() {
    try {
      gClientId = (await api.getSetting("google_client_id")) ?? "";
      gClientSecret = (await api.getSetting("google_client_secret")) ?? "";
      gStatus = await api.googleStatus();
      if (gStatus.connected) void loadGoogleCalendars();
    } catch {
      gStatus = { connected: false, email: null, configured: false };
    }
  }
  async function loadGoogleCalendars() {
    if (!gStatus?.connected) return;
    gCalBusy = true;
    try { gCalendars = await api.googleListCalendars(); }
    catch (e) { app.pushToast({ kind: "error", title: "Kalender konnten nicht geladen werden", body: String(e) }); }
    finally { gCalBusy = false; }
  }
  function toggleGoogleCal(id: string) {
    gCalendars = gCalendars.map((c) => (c.id === id ? { ...c, selected: !c.selected } : c));
    const csv = gCalendars.filter((c) => c.selected).map((c) => c.id).join(",");
    api.setSettings({ google_pull_calendars: csv }).catch(() => {});
  }
  function saveGoogleCreds() {
    api.setSettings({
      google_client_id: gClientId.trim(),
      google_client_secret: gClientSecret.trim(),
    }).catch(() => {});
  }
  async function connectGoogle() {
    gBusy = true;
    try {
      saveGoogleCreds();
      gStatus = await api.googleConnect();
      app.pushToast({ kind: "success", title: "Google Kalender verbunden", body: gStatus.email ?? undefined });
      void loadGoogleCalendars();
    } catch (e) {
      app.pushToast({ kind: "error", title: "Verbinden fehlgeschlagen", body: String(e) });
    } finally {
      gBusy = false;
    }
  }
  async function syncGoogle() {
    gBusy = true;
    try {
      const r = await api.googleSync();
      await app.refresh();            // pulled events filed to subjects; colours updated
      app.notifyEventsChanged();      // refresh the calendar view
      app.pushToast({ kind: "success", title: "Kalender synchronisiert", body: `${r.pulled} geholt · ${r.pushed} gesendet` });
    } catch (e) {
      app.pushToast({ kind: "error", title: "Synchronisierung fehlgeschlagen", body: String(e) });
    } finally {
      gBusy = false;
    }
  }
  async function disconnectGoogle() {
    try {
      gStatus = await api.googleDisconnect();
      app.pushToast({ kind: "info", title: "Google getrennt" });
    } catch (e) {
      app.pushToast({ kind: "error", title: "Trennen fehlgeschlagen", body: String(e) });
    }
  }

  async function testSearxng() {
    if (!searxng.trim()) return;
    searxState = "testing";
    searxState = await testEndpoint(searxng);
  }

  // Map a connection test state + whether the endpoint is set to a design-system
  // status pill (class modifier + label), so every integration shows state the
  // same way the rest of the app shows source/cheatsheet state.
  function connPill(state: null | "testing" | "ok" | "fail", configured: boolean) {
    if (state === "testing") return { cls: "draft", label: "Wird geprüft…" };
    if (state === "ok") return { cls: "ready", label: "Verbunden" };
    if (state === "fail") return { cls: "error", label: "Nicht erreichbar" };
    return configured
      ? { cls: "pending", label: "Nicht getestet" }
      : { cls: "pending", label: "Nicht gesetzt" };
  }

  // persist the Ollama endpoint on change, and re-probe its installed models
  $effect(() => {
    const ep = endpoint;
    if (!loaded) return;
    api.setSettings({ ollama_url: ep }).catch(() => {});
    invalidateOllamaModels();
  });

  // ---- focus timer (pomodoro) durations — bound to the app-wide timer ----
  const pomoFields = [
    { key: "workMin",            label: "Fokusdauer",    unit: " min", step: 5, min: 5, max: 90 },
    { key: "breakMin",           label: "Kurze Pause",   unit: " min", step: 1, min: 1, max: 30 },
    { key: "longBreakMin",       label: "Lange Pause",   unit: " min", step: 5, min: 5, max: 60 },
    { key: "sessionsBeforeLong", label: "Einheiten pro Runde", unit: "",    step: 1, min: 2, max: 8  },
  ] as const;
  function pomoVal(key: string): number {
    return (app.pomo as unknown as Record<string, number>)[key];
  }
  function setPomo(key: string, delta: number) {
    const f = pomoFields.find((x) => x.key === key)!;
    const next = Math.max(f.min, Math.min(f.max, pomoVal(key) + delta));
    (app.pomo as unknown as Record<string, number>)[key] = next;
    api.setSettings({ ["pomo_" + key]: String(next) }).catch(() => {});
  }

  // ---- audio state ----
  let autoplay = $state(false);
  let station  = $state("lofi");
  let voiceA   = $state("maya");
  let voiceB   = $state("theo");

  // Set true only once the mount hydration below has loaded persisted values.
  // The persist $effects fire immediately on mount with their *default* values;
  // without this guard, opening Settings would clobber saved settings (e.g. it
  // silently reset default_station to "lofi" on every visit).
  let loaded = $state(false);

  // Built-in + user-added stations, so the Default-station picker shows them all.
  const allStations = $derived([
    ...stations.map((s) => ({ id: s.id, label: s.name })),
    ...app.customStations.map((s) => ({ id: s.id, label: s.name })),
  ]);

  // Stream-tool detection for the YouTube-audio engine (mpv sidecar).
  let mediaTools = $state<api.MediaTools | null>(null);
  $effect(() => {
    if (tab === "audio" && mediaTools === null) {
      api.mediaToolsStatus().then((t) => (mediaTools = t)).catch(() => {});
    }
  });

  // persist audio on change. NOTE: never read app.music here — reading + writing
  // it in one effect creates a self-triggering reactive loop (crashes the view).
  // The live-player sync happens in the Picker's onChange handler instead.
  $effect(() => {
    const st = station;
    const ap = autoplay;
    if (!loaded) return;
    api.setSettings({ default_station: st, autoplay: String(ap) }).catch(() => {});
  });

  // persist host voices on change
  $effect(() => {
    const a = voiceA;
    const b = voiceB;
    if (!loaded) return;
    api.setSettings({ voice_a: a, voice_b: b }).catch(() => {});
  });

  // ---- window behaviour ----
  // Default ON: closing the window hides to the tray so ingest/generation/music
  // keep running; the backend treats anything but "false" as enabled.
  let closeToTray = $state(true);
  function toggleCloseToTray() {
    closeToTray = !closeToTray;
    api.setSetting("close_to_tray", closeToTray ? "true" : "false").catch(() => {});
  }

  // ---- data & privacy state ----
  let offlineMode = $state(false);
  let stats       = $state<api.DbStats | null>(null);
  let archivedSubjects = $state<api.Subject[]>([]);

  async function loadArchived() {
    archivedSubjects = await app.listArchivedSubjects();
  }
  async function restoreSubject(id: string) {
    await app.setSubjectArchived(id, false);
    await Promise.all([loadArchived(), loadStats()]);
  }

  function fmtBytes(n: number): string {
    if (n >= 1024 * 1024 * 1024) return (n / (1024 * 1024 * 1024)).toFixed(1) + " GB";
    if (n >= 1024 * 1024)        return (n / (1024 * 1024)).toFixed(1) + " MB";
    if (n >= 1024)               return (n / 1024).toFixed(0) + " KB";
    return n + " B";
  }

  async function loadStats() {
    stats = await api.dbStats().catch(() => null);
  }

  function toggleOffline() {
    offlineMode = !offlineMode;
    api.setSetting("offline_mode", offlineMode ? "true" : "false").catch(() => {});
  }

  async function clearCaches() {
    testState = null;
    searxState = null;
    try {
      await api.optimizeDb();
      await loadStats();
      app.pushToast({ kind: "success", title: "Speicher optimiert", body: "Ungenutzter Platz wurde freigegeben (VACUUM)." });
    } catch (e) {
      app.pushToast({ kind: "error", title: "Optimieren fehlgeschlagen", body: String(e) });
    }
  }

  async function exportData() {
    try {
      const { save } = await import("@tauri-apps/plugin-dialog");
      const dest = await save({
        defaultPath: "cortex-export.db",
        filters: [{ name: "SQLite-Datenbank", extensions: ["db"] }],
      });
      if (!dest) return;
      await api.exportDatabase(dest);
      app.pushToast({ kind: "success", title: "Exportiert", body: dest });
    } catch (e) {
      app.pushToast({ kind: "error", title: "Export fehlgeschlagen", body: String(e) });
    }
  }

  async function deleteEverything() {
    const ok = window.confirm(
      "ALLE Daten löschen?\n\nDie lokale Datenbank wird geleert – alle Fächer, Quellen, Lernzettel und Embeddings. Das lässt sich nicht rückgängig machen. Einstellungen und API-Schlüssel bleiben erhalten.",
    );
    if (!ok) return;
    try {
      await api.deleteAllData();
      await loadStats();
      app.pushToast({ kind: "success", title: "Alle Daten gelöscht", body: "Starte die App neu, um von vorne zu beginnen." });
    } catch {
      app.pushToast({ kind: "error", title: "Löschen fehlgeschlagen" });
    }
  }

  // ---- mount: hydrate from backend ----
  $effect(() => {
    // All three are independent; issue them concurrently.
    (async () => {
      const [s] = await Promise.all([
        api.getAllSettings().catch(() => ({}) as Record<string, string>),
        loadMemory(),
        loadStats(),
      ]);
      // API keys
      if (s.openrouter_api_key) keys = { ...keys, openrouter: s.openrouter_api_key };
      if (s.gemini_api_key)     keys = { ...keys, gemini: s.gemini_api_key };
      if (s.claude_api_key)     keys = { ...keys, claude: s.claude_api_key };
      if (s.openai_api_key)     keys = { ...keys, openai: s.openai_api_key };
      if (s.custom_endpoint)    keys = { ...keys, custom_endpoint: s.custom_endpoint };
      if (s.custom_api_key)     keys = { ...keys, custom_api_key: s.custom_api_key };

      // Models
      for (const taskId of ["chat","cheatsheet","audio","quiz","flashcard","embedding"] as TaskId[]) {
        const raw = s[`model_${taskId}`];
        if (raw) {
          const sep = raw.indexOf(":");
          if (sep !== -1) {
            const prov  = raw.slice(0, sep);
            const model = raw.slice(sep + 1);
            assign = { ...assign, [taskId]: { ...assign[taskId], provider: prov, model } };
          }
        }
        const budget = s[`budget_${taskId}`];
        if (budget) {
          assign = { ...assign, [taskId]: { ...assign[taskId], budget } };
        }
      }
      if (s.embed_provider) {
        assign = { ...assign, embedding: { ...assign.embedding, provider: s.embed_provider } };
      }

      // Local models + web search + remote whisper
      if (s.ollama_url)                    endpoint = s.ollama_url;
      if (s.searxng_url)                   searxng  = s.searxng_url;
      if (s.school_folder_path)            schoolFolderPath = s.school_folder_path;
      if (s.whisper_url)                   whisperUrl = s.whisper_url;
      if (s.whisper_model)                 whisperModel = s.whisper_model;
      // Transcription mode + cloud provider. Unset mode = legacy auto: homelab
      // when any whisper/homelab URL is configured, otherwise local.
      if (s.transcription_mode === "local" || s.transcription_mode === "cloud" || s.transcription_mode === "homelab") {
        transcriptionMode = s.transcription_mode;
      } else {
        transcriptionMode =
          (s.whisper_url || s.homelab_base || s.homelab_tailscale_base || s.homelab_public_base)
            ? "homelab" : "local";
      }
      if (s.whisper_cloud_provider === "groq" || s.whisper_cloud_provider === "openai" || s.whisper_cloud_provider === "custom") whisperCloudProvider = s.whisper_cloud_provider;
      if (s.whisper_cloud_url)   whisperCloudUrl = s.whisper_cloud_url;
      if (s.whisper_cloud_model) whisperCloudModel = s.whisper_cloud_model;
      if (s.whisper_api_key)     whisperApiKey = s.whisper_api_key;
      if (s.homelab_token)       hlToken = s.homelab_token;
      // Live sync
      if (s.homelab_base)           hlBase = s.homelab_base;
      if (s.homelab_tailscale_base) hlTailscale = s.homelab_tailscale_base;
      if (s.homelab_public_base)    hlPublic = s.homelab_public_base;
      if (s.sync_url)   syncUrl  = s.sync_url;
      if (s.sync_url_tailscale) syncUrlTs = s.sync_url_tailscale;
      if (s.sync_url_public)    syncUrlPub = s.sync_url_public;
      if (s.sync_mode === "auto" || s.sync_mode === "local" || s.sync_mode === "tailscale" || s.sync_mode === "public") syncMode = s.sync_mode;
      if (s.sync_user)  syncUser = s.sync_user;
      if (s.sync_pass)  syncPass = s.sync_pass;
      syncOn = s.sync_enabled === "true";
      // Encrypted backups
      if (s.backup_age_recipient) ageRecipient = s.backup_age_recipient;
      if (s.backup_rclone_remote) rcloneRemote = s.backup_rclone_remote;
      refreshBackupStatus();

      // Appearance
      if (s.reading_font)   readFont      = s.reading_font;
      if (s.density)        density       = s.density;
      // Window behaviour (default ON: closing hides to the tray)
      if (s.close_to_tray !== undefined) closeToTray = s.close_to_tray !== "false";

      // Audio: default station, autoplay, host voices
      if (s.default_station) station = s.default_station;
      if (s.autoplay !== undefined) autoplay = s.autoplay === "true";
      if (s.voice_a) voiceA = s.voice_a;
      if (s.voice_b) voiceB = s.voice_b;

      // Homelab: web images (diagrams) toggle mirrors the store's resolved value.
      webImages = app.webImagesEnabled;

      // Data & privacy
      if (s.offline_mode !== undefined) offlineMode = s.offline_mode === "true";

      // Profile
      if (s.profile_name)      name     = s.profile_name;
      if (s.profile_pronouns)  pronouns = s.profile_pronouns;
      if (s.profile_level)     level    = s.profile_level;
      if (s.profile_field)     field    = s.profile_field;
      if (s.profile_about)     about    = s.profile_about;
      if (s.profile_style)     style    = s.profile_style;
      if (s.profile_explain)   explain  = s.profile_explain.split(",").filter(Boolean);

      expMoodle = s.exp_moodle === "true";
      if (s.moodle_url) mdUrl = s.moodle_url;
      void loadMoodle();

      // Hydration complete — persist effects may now write without clobbering.
      loaded = true;

      // Auto-verify each stored key's connection, and pre-load Ollama's installed
      // models if any task uses Ollama (so the picker reflects reality immediately).
      verifyAllKeys();
      if (Object.values(assign).some((x) => x.provider === "ollama")) void ensureOllamaModels();
    })();
  });

  // ---- helpers ----
  function saveProfile() {
    api.setSettings({
      profile_name: name,
      profile_pronouns: pronouns,
      profile_level: level,
      profile_field: field,
      profile_about: about,
      profile_style: style,
      profile_explain: explain.join(","),
    }).then(() => app.pushToast({ kind: "success", title: "Profil gespeichert", body: "Die KI nutzt jetzt deine aktualisierten Angaben." }))
      .catch(() => app.pushToast({ kind: "error", title: "Speichern fehlgeschlagen" }));
  }

  function saveKeys() {
    api.setSettings({
      openrouter_api_key: keys.openrouter,
      gemini_api_key:     keys.gemini,
      claude_api_key:     keys.claude,
      openai_api_key:     keys.openai,
      custom_endpoint:    keys.custom_endpoint,
      custom_api_key:     keys.custom_api_key,
    }).then(() => app.pushToast({ kind: "success", title: "Schlüssel gespeichert", body: "Im Schlüsselbund des Systems abgelegt." }))
      .catch(() => app.pushToast({ kind: "error", title: "Speichern fehlgeschlagen" }));
  }

  // Live OpenRouter catalog for the searchable model picker — fetched once, on the
  // first time an OpenRouter model dropdown is opened.
  let orModels = $state<OrModel[]>([]);
  let orLoading = $state(false);
  let orLoaded = false;
  async function ensureOrModels() {
    if (orLoaded || orLoading) return;
    orLoading = true;
    try {
      orModels = await loadOpenRouterModels();
      orLoaded = true;
    } catch {
      app.pushToast({ kind: "error", title: "OpenRouter-Modelle", body: "Die Modellliste konnte nicht geladen werden – prüfe deine Verbindung." });
    } finally {
      orLoading = false;
    }
  }

  async function onModelProviderChange(taskId: TaskId, p: string) {
    if (p === "ollama") await ensureOllamaModels();
    const provList = providersFor(taskId);
    const np = provList.find((x) => x.id === p) ?? provList[0];
    // Ollama → default to the first INSTALLED model (empty if none pulled, which leaves
    // the picker blank as intended). Others → the provider's first curated model.
    const firstModel = p === "ollama" || p === "custom" ? "" : (np.models[0]?.id ?? "");
    setTask(taskId, { provider: p, model: firstModel });
    const kv: Record<string, string> = { [`model_${taskId}`]: p + ":" + firstModel };
    if (taskId === "embedding") kv.embed_provider = p;
    api.setSettings(kv).catch(() => {});
  }

  function onModelChange(taskId: TaskId, m: string) {
    setTask(taskId, { model: m });
    api.setSettings({ [`model_${taskId}`]: assign[taskId].provider + ":" + m }).catch(() => {});
  }

  // ── Ollama: live installed-model list (GET /api/tags) ──────────────────────
  // When a task's provider is Ollama, the model picker shows ONLY the models actually
  // installed on the configured server (local on desktop, or the homelab). An empty
  // list ⇒ an empty picker (nothing to choose) — never models that aren't pulled.
  let ollamaInstalled = $state<string[]>([]);
  let ollamaLoaded = false;
  async function ensureOllamaModels() {
    if (ollamaLoaded) return;
    ollamaLoaded = true;
    try { ollamaInstalled = await api.ollamaModels(); } catch { ollamaInstalled = []; }
  }
  // Changing the Ollama/homelab URL just INVALIDATES the cached list — it's re-probed
  // lazily next time the picker opens, so we don't fire a network call per keystroke.
  function invalidateOllamaModels() { ollamaLoaded = false; ollamaInstalled = []; }

  // Is local-model (Ollama) usage available here? On desktop, always (localhost). On
  // mobile there is no localhost Ollama, so it needs a homelab base (or an explicit
  // non-localhost Ollama URL) to be reachable.
  const hasHomelabOllama = $derived(
    !!(hlBase.trim() || hlTailscale.trim() || hlPublic.trim()) ||
    (!!endpoint.trim() && !/localhost|127\.0\.0\.1/.test(endpoint))
  );
  const ollamaAvailable = $derived(!isMobile || hasHomelabOllama);

  // Provider list for a task, with Ollama dropped on mobile when no homelab is set
  // (so it isn't offered when it can't possibly work).
  function providersFor(taskId: TaskId) {
    const base = taskId === "embedding" ? EMBED_PROVIDERS : PROVIDERS;
    return ollamaAvailable ? base : base.filter((p) => p.id !== "ollama");
  }

  // Options for a task's model picker: OpenRouter → live catalog; Ollama → installed
  // models (may be empty); else the provider's curated list.
  function modelOptionsFor(prov: { id: string; models: Model[] }) {
    if (prov.id === "openrouter" && orModels.length) return orModels;
    if (prov.id === "ollama") return ollamaInstalled.map((id) => ({ id, label: id }));
    return prov.models.map((m) => ({ id: m.id, label: m.label }));
  }

  // Ollama URL to display in API keys. Desktop: the editable endpoint. Mobile: the
  // homelab-derived URL (base + /ollama), shown read-only — Ollama is homelab-only there.
  const ollamaDisplayUrl = $derived(
    endpoint.trim() ? endpoint.trim()
    : (hlBase.trim() ? hlBase.trim().replace(/\/+$/, "") + "/ollama" : "")
  );

  // ── API-key verification (a real authed connection check, not just "key present") ──
  type VerifyState = api.VerifyResult | "checking" | null;
  let verify = $state<Record<string, VerifyState>>({});
  // Status-badge class + label, shared by the provider rows and the Ollama row.
  const statusClass = (v: VerifyState) =>
    "key-status " + (v === "checking" ? "checking" : v ? (v.ok ? "ok" : "bad") : "off");
  const statusLabel = (v: VerifyState, isSet = false) =>
    v === "checking" ? "wird geprüft…" : v ? (v.ok ? "verbunden" : v.detail) : (isSet ? "nicht geprüft" : "nicht gesetzt");
  const verifyIdForKey = (id: string) =>
    id === "custom_endpoint" || id === "custom_api_key" ? "custom" : id;
  async function verifyKey(id: string) {
    const vid = verifyIdForKey(id);
    verify = { ...verify, [vid]: "checking" };
    try { verify = { ...verify, [vid]: await api.verifyProvider(vid) }; }
    catch (e) { verify = { ...verify, [vid]: { ok: false, detail: String(e) } }; }
  }
  // Verify every provider that has a stored key (run on load + after Save keys).
  function verifyAllKeys() {
    for (const k of keyMeta) {
      if (keys[k.id as keyof typeof keys]?.trim()) void verifyKey(k.id);
    }
    if (ollamaAvailable) void verifyKey("ollama");
  }

  const levelLabels: Record<string, string> = {
    undergrad: "Schule / Ausbildung",
    postgrad:  "Studium",
    phd:       "Promotion / Forschung",
    self:      "Selbststudium",
  };

  function toggleExplain(id: string) {
    explain = explain.includes(id)
      ? explain.filter((x) => x !== id)
      : [...explain, id];
  }
</script>

<div class="settings">
  <!-- NAV SIDEBAR -->
  <aside class="set-nav">
    <div class="set-nav-h">
      <button
        class="btn btn--icon btn--sm btn--ghost"
        onclick={() => app.setView("subject")}
        title="Zurück"
      >
        <Icon name="chevron" size={14} style="transform:rotate(180deg)" />
      </button>
      <span class="mono" style="color:var(--fg-bright);font-weight:600">Einstellungen</span>
    </div>

    {#each navTabs as t}
      <button
        class={"set-nav-item" + (tab === t.id ? " on" : "")}
        onclick={() => (tab = t.id)}
      >
        <Icon name={t.icon} size={13} /> {t.label}
      </button>
    {/each}

    <div class="set-nav-foot mono faint">Cortex {appVersion ? "v" + appVersion : ""} · eigener Schlüssel · lokal</div>
  </aside>

  <!-- MAIN BODY -->
  <div class="set-body">

    {#if isMobile}
      <div class="set-mnav">
        <Picker
          value={tab}
          onChange={(id) => (tab = id)}
          options={navTabs.map((t) => ({ id: t.id, label: t.label }))}
          icon={navTabs.find((t) => t.id === tab)?.icon}
        />
      </div>
    {/if}

    <!-- ===== PROFILE ===== -->
    {#if tab === "profile"}
      <div class="set-pane">
        <header class="set-head">
          <div class="eyebrow">Profil</div>
          <h1 class="set-title">Wer du für die KI bist</h1>
          <p class="set-sub">Wird bei jedem Chat und jeder Erstellung mitgegeben, damit Antworten zu deinem Niveau und Stil passen. Bleibt auf diesem Gerät.</p>
        </header>

        <section class="set-group">
          <div class="set-group-h"><h3 class="set-group-t">Person</h3></div>
          <div class="set-card">
            <div class="set-row">
              <div class="set-row-l"><div class="set-row-t">Anzeigename</div></div>
              <div class="set-row-r"><input class="input" bind:value={name} /></div>
            </div>
            <div class="set-row">
              <div class="set-row-l"><div class="set-row-t">Pronomen</div></div>
              <div class="set-row-r"><input class="input" bind:value={pronouns} /></div>
            </div>
            <div class="set-row">
              <div class="set-row-l"><div class="set-row-t">Niveau</div></div>
              <div class="set-row-r">
                <Picker
                  value={level}
                  onChange={(v) => (level = v)}
                  options={[
                    { id: "undergrad", label: "Schule / Ausbildung" },
                    { id: "postgrad",  label: "Studium" },
                    { id: "phd",       label: "Promotion / Forschung" },
                    { id: "self",      label: "Selbststudium" },
                  ]}
                />
              </div>
            </div>
            <div class="set-row">
              <div class="set-row-l"><div class="set-row-t">Fachrichtung</div></div>
              <div class="set-row-r"><input class="input" bind:value={field} /></div>
            </div>
          </div>
        </section>

        <section class="set-group">
          <div class="set-group-h">
            <h3 class="set-group-t">Über dich</h3>
            <p class="set-group-d">Kontext, mit dem die KI Erklärungen an dich anpasst.</p>
          </div>
          <div class="set-card">
            <div class="set-row stacked">
              <div class="set-row-t">In deinen Worten</div>
              <textarea class="input set-textarea set-bio" bind:value={about} rows={6}></textarea>
            </div>
            <div class="set-row">
              <div class="set-row-l">
                <div class="set-row-t">Antwortstil</div>
                <div class="set-row-d">Wie ausführlich standardmäßig.</div>
              </div>
              <div class="set-row-r">
                <div class="seg">
                  {#each [{ id: "concise", label: "Knapp" }, { id: "balanced", label: "Ausgewogen" }, { id: "detailed", label: "Ausführlich" }] as opt}
                    <button type="button" class={"seg-opt" + (style === opt.id ? " on" : "")} onclick={() => (style = opt.id)}>{opt.label}</button>
                  {/each}
                </div>
              </div>
            </div>
            <div class="set-row stacked">
              <div class="set-row-l">
                <div class="set-row-t">Erklären mit</div>
                <div class="set-row-d">Wähle, womit du am schnellsten lernst.</div>
              </div>
              <div class="set-row-r">
                <div class="tag-suggest" style="margin-top:0">
                  {#each [{ id: "worked-examples", label: "Beispielaufgaben" }, { id: "analogies", label: "Vergleiche" }, { id: "formal-proofs", label: "formale Beweise" }, { id: "diagrams", label: "Diagramme" }, { id: "code", label: "Codebeispiele" }] as opt}
                    <button
                      type="button"
                      class={"tag-chip-add" + (explain.includes(opt.id) ? " on" : "")}
                      onclick={() => toggleExplain(opt.id)}
                    >
                      {#if explain.includes(opt.id)}<Icon name="check" size={9} />{/if}
                      {opt.label}
                    </button>
                  {/each}
                </div>
              </div>
            </div>
          </div>
        </section>

        <section class="set-group">
          <div class="set-group-h">
            <h3 class="set-group-t">Gedächtnis</h3>
            <p class="set-group-d">Dauerhafte Fakten, die die KI in jedem Chat erhält – etwa dein Prüfungstermin, dein Schulbuch oder wie du Antworten gern formuliert hättest.</p>
          </div>
          <div class="set-card">
            <div class="set-row stacked">
              <div class="set-row-r">
                <div class="row-inline">
                  <input
                    class="input"
                    bind:value={newMemory}
                    placeholder="z. B. Meine Abschlussprüfung ist am 20. Juni"
                    onkeydown={(e) => { if (e.key === "Enter") { e.preventDefault(); addMemoryFact(); } }}
                  />
                  <button class="btn btn--primary" onclick={addMemoryFact} disabled={!newMemory.trim() || memoryBusy}>
                    <Icon name="check" size={13} /> Merken
                  </button>
                </div>
              </div>
            </div>
            {#if memories.length === 0}
              <div class="set-row">
                <div class="set-row-l"><div class="set-row-d">Noch keine Erinnerungen. Füge oben einen Fakt hinzu, dann behält die KI ihn im Kopf.</div></div>
              </div>
            {:else}
              {#each memories as m (m.id)}
                <div class="set-row">
                  <div class="set-row-l"><div class="set-row-t">{m.content}</div></div>
                  <div class="set-row-r">
                    <button class="btn btn--icon btn--sm btn--ghost" onclick={() => removeMemory(m.id)} title="Vergessen">
                      <Icon name="x" size={13} />
                    </button>
                  </div>
                </div>
              {/each}
            {/if}
          </div>
        </section>

        <div class="set-preview">
          <div class="label" style="margin-bottom:8px">Was die KI erhält</div>
          <pre class="set-sysprompt mono">Nutzer: {name} ({pronouns}) · {levelLabels[level] ?? level}
Fachrichtung: {field}
Stil: {style}, bevorzugt {explain.join(", ") || "kein besonderes Format"}
Notizen: {about}</pre>
        </div>

        <div class="set-foot-actions">
          <button class="btn btn--primary" onclick={saveProfile}>
            <Icon name="check" size={13} /> Profil speichern
          </button>
        </div>
      </div>

    <!-- ===== MODELS ===== -->
    {:else if tab === "models"}
      <div class="set-pane set-pane--models">
        <header class="set-head">
          <div class="eyebrow">Modelle</div>
          <h1 class="set-title">Ein Modell für jede Aufgabe</h1>
          <p class="set-sub">Leite jede Aufgabe an den passendsten Anbieter. Token-Budgets begrenzen die Kosten pro Aufruf.</p>
        </header>

        <div class="set-card set-table">
          <div class="mt-head mono">
            <span>Aufgabe</span><span>Anbieter</span><span>Modell</span><span>Token-Budget</span>
          </div>
          {#each MODEL_TASKS as t}
            {@const a = assign[t.id]}
            {@const provList = providersFor(t.id)}
            {@const allProv = t.id === "embedding" ? EMBED_PROVIDERS : PROVIDERS}
            {@const prov = allProv.find((p) => p.id === a.provider) ?? provList[0]}
            {@const isOr = a.provider === "openrouter"}
            {@const isOllama = a.provider === "ollama"}
            {@const isCustom = a.provider === "custom"}
            <div class="mt-row">
              <div class="mt-task">
                <div class="mt-task-t">{t.label}</div>
                <div class="mt-task-d mono">{t.desc}</div>
              </div>
              <Picker
                value={a.provider}
                onChange={(p) => onModelProviderChange(t.id, p)}
                options={provList.map((p) => ({ id: p.id, label: p.label }))}
              />
              <!-- OpenRouter → live searchable catalog (curated list as offline/pre-fetch
                   fallback); Ollama → only models actually installed (empty ⇒ nothing to
                   pick); every other provider → its own curated list, still searchable. -->
              <ModelSearch
                value={a.model}
                onChange={(m) => onModelChange(t.id, m)}
                options={modelOptionsFor(prov)}
                loading={isOr && orLoading}
                onOpen={isOr ? ensureOrModels : (isOllama ? ensureOllamaModels : undefined)}
                allowCustom={isCustom}
                placeholder={isCustom ? "Modell-ID eingeben, z. B. qwen-plus" : (isOr ? "OpenRouter durchsuchen…" : (isOllama ? (ollamaInstalled.length ? "Installiertes Modell wählen" : "Keine Modelle installiert") : undefined))}
              />
              {#if t.id === "embedding"}
                <span class="mono faint mt-budget-na">n/a</span>
              {:else}
                <input
                  class="input mono mt-budget"
                  value={a.budget}
                  oninput={(e) => { const v = (e.target as HTMLInputElement).value; setTask(t.id, { budget: v }); api.setSettings({ ["budget_" + t.id]: v }).catch(() => {}); }}
                />
              {/if}
            </div>
          {/each}
        </div>

        <div class="set-note mono">
          <Icon name="diamond" size={11} color="var(--accent)" />
          Ollama-Aufgaben laufen komplett offline auf diesem Gerät oder deinem Homelab – kein Schlüssel nötig.
        </div>
      </div>

    <!-- ===== API KEYS ===== -->
    {:else if tab === "keys"}
      <div class="set-pane">
        <header class="set-head">
          <div class="eyebrow">API-Schlüssel</div>
          <h1 class="set-title">Eigene Schlüssel</h1>
          <p class="set-sub">Im Schlüsselbund des Systems gespeichert, nie synchronisiert. Nichts läuft über Cortex-Server.</p>
        </header>

        <section class="set-group">
          <div class="set-group-h"><h3 class="set-group-t">Anbieter</h3></div>
          <div class="set-card">
            {#each keyMeta as k}
              {@const isSet = !!keys[k.id as keyof typeof keys]}
              {@const v = verify[verifyIdForKey(k.id)]}
              <div class="set-row stacked">
                <div class="set-row-l">
                  <div class="set-row-t">
                    <span class="row-keytitle">
                      {k.label}
                      <span class={statusClass(v)}>{statusLabel(v, isSet)}</span>
                    </span>
                  </div>
                  <div class="set-row-d">{k.note}</div>
                </div>
                <div class="set-row-r">
                  <div class="masked">
                    <input
                      class="input mono"
                      type={showKey[k.id] ? "text" : "password"}
                      value={keys[k.id as keyof typeof keys]}
                      oninput={(e) => { keys = { ...keys, [k.id]: (e.target as HTMLInputElement).value }; verify = { ...verify, [k.id]: null }; }}
                      placeholder={k.placeholder}
                      spellcheck={false}
                    />
                    <button
                      type="button"
                      class="masked-eye"
                      onclick={() => { showKey = { ...showKey, [k.id]: !showKey[k.id] }; }}
                      title={showKey[k.id] ? "Verbergen" : "Anzeigen"}
                    >
                      <Icon name={showKey[k.id] ? "x" : "search"} size={13} />
                    </button>
                  </div>
                  {#if isSet}
                    <div style="display:flex;gap:6px;margin-top:6px">
                      <button
                        type="button"
                        class="btn btn--ghost btn--sm"
                        onclick={() => verifyKey(k.id)}
                      >Prüfen</button>
                      <button
                        type="button"
                        class="btn btn--ghost btn--sm"
                        onclick={() => { keys = { ...keys, [k.id]: "" }; verify = { ...verify, [k.id]: null }; }}
                      >Leeren</button>
                    </div>
                  {/if}
                </div>
              </div>
            {/each}
          </div>
        </section>

        <!-- Local models (Ollama) — keyless; a URL, not a key. On mobile Ollama is
             reached only through the Homelab (no localhost on a phone). -->
        {#if ollamaAvailable}
        {@const ov = verify.ollama}
        <section class="set-group">
          <div class="set-group-h"><h3 class="set-group-t">Lokale Modelle (Ollama)</h3></div>
          <div class="set-card">
            <div class="set-row stacked">
              <div class="set-row-l">
                <div class="set-row-t">
                  <span class="row-keytitle">
                    Ollama URL
                    <span class={statusClass(ov)}>{statusLabel(ov)}</span>
                  </span>
                </div>
                <div class="set-row-d">
                  {#if isMobile}
                    Läuft über dein Homelab – setze die Homelab-URL unter Integrationen. Ohne Schlüssel.
                  {:else}
                    Ohne Schlüssel, lokal. Standard ist <span class="mono">http://localhost:11434</span>; leer lassen, um dein Homelab zu nutzen.
                  {/if}
                </div>
              </div>
              <div class="set-row-r">
                {#if isMobile}
                  <input class="input mono" value={ollamaDisplayUrl} placeholder="Homelab-URL unter Integrationen setzen" readonly />
                {:else}
                  <input
                    class="input mono"
                    value={endpoint}
                    oninput={(e) => { endpoint = (e.target as HTMLInputElement).value; verify = { ...verify, ollama: null }; }}
                    placeholder="http://localhost:11434"
                    spellcheck={false}
                  />
                {/if}
                <button type="button" class="btn btn--ghost btn--sm" style="margin-top:6px" onclick={() => verifyKey("ollama")}>Prüfen</button>
              </div>
            </div>
          </div>
        </section>
        {/if}

        <div class="set-foot-actions">
          <button class="btn btn--primary" onclick={() => { saveKeys(); verifyAllKeys(); }}>
            <Icon name="check" size={13} /> Schlüssel speichern
          </button>
        </div>
      </div>

    <!-- ===== APPEARANCE ===== -->
    {:else if tab === "appearance"}
      <div class="set-pane">
        <header class="set-head">
          <div class="eyebrow">Aussehen</div>
          <h1 class="set-title">Nach deinem Geschmack</h1>
          <p class="set-sub">Cortex übernimmt live dein Omarchy-Design – oder du wählst selbst eins.</p>
        </header>

        <!-- Follow-Omarchy mirrors the desktop's Omarchy palette — meaningless on a phone. -->
        {#if !isMobile}
        <section class="set-group">
          <div class="set-group-h"><h3 class="set-group-t">Design</h3></div>
          <div class="set-card">
            <div class="set-row">
              <div class="set-row-l">
                <div class="set-row-t">Omarchy-Design folgen</div>
                <div class="set-row-d">Übernimmt bei jedem Start die aktuellen Omarchy-Farben deines Desktops. Wenn du unten ein Design wählst, wird das abgeschaltet.</div>
              </div>
              <div class="set-row-r">
                <button
                  type="button"
                  class={"st-toggle" + (app.followOmarchy ? " on" : "")}
                  role="switch"
                  aria-checked={app.followOmarchy}
                  aria-label="Omarchy-Design folgen"
                  onclick={async () => {
                    const matched = await app.setFollowOmarchy(!app.followOmarchy);
                    if (app.followOmarchy && !matched) {
                      app.pushToast({ kind: "warning", title: "Omarchy-Design nicht gefunden", body: "Dein Omarchy-Design konnte nicht gelesen werden oder hat keine Entsprechung in Cortex." });
                    } else if (matched) {
                      app.pushToast({ kind: "success", title: "Folgt Omarchy", body: `Übernommen → ${THEME_LABELS[matched]}.` });
                    }
                  }}
                ><span class="st-knob"></span></button>
              </div>
            </div>
          </div>
        </section>
        {/if}

        <section class="set-group">
          <div class="set-group-h"><h3 class="set-group-t">Design manuell wählen</h3></div>
          <div class="set-card">
            <div class="set-themes">
              {#each THEME_OPTS as t}
                <button
                  class={"set-theme" + (app.theme === t.id ? " on" : "")}
                  onclick={() => { if (app.followOmarchy) app.setFollowOmarchy(false); app.setTheme(t.id); }}
                  style="background:{t.b}"
                >
                  <div class="set-theme-sws">
                    <span style="background:{t.c}"></span>
                    <span style="background:{t.b};border:1px solid {t.c}"></span>
                  </div>
                  <span class="set-theme-n" style="color:{app.theme === t.id ? '#fff' : '#cbd5d0'}">{t.n}</span>
                  {#if app.theme === t.id}
                    <span class="set-theme-check" style="color:{t.c}">
                      <Icon name="check" size={13} />
                    </span>
                  {/if}
                </button>
              {/each}
            </div>
          </div>
        </section>

        <section class="set-group">
          <div class="set-group-h"><h3 class="set-group-t">Lesen</h3></div>
          <div class="set-card">
            <div class="set-row">
              <div class="set-row-l">
                <div class="set-row-t">Schrift der Lernzettel</div>
                <div class="set-row-d">Die Schrift für alles, was du zum Lernen liest.</div>
              </div>
              <div class="set-row-r">
                <div class="seg">
                  {#each [{ id: "mono", label: "Mono" }, { id: "serif", label: "Serif" }, { id: "sans", label: "Sans" }] as opt}
                    <button type="button" class={"seg-opt" + (readFont === opt.id ? " on" : "")} onclick={() => (readFont = opt.id)}>{opt.label}</button>
                  {/each}
                </div>
              </div>
            </div>
            <!-- Density is a desktop spacing affordance; touch uses one comfortable scale. -->
            {#if !isMobile}
            <div class="set-row">
              <div class="set-row-l">
                <div class="set-row-t">Dichte</div>
                <div class="set-row-d">Abstände in der ganzen App.</div>
              </div>
              <div class="set-row-r">
                <div class="seg">
                  {#each [{ id: "regular", label: "Normal" }, { id: "compact", label: "Kompakt" }] as opt}
                    <button type="button" class={"seg-opt" + (density === opt.id ? " on" : "")} onclick={() => (density = opt.id)}>{opt.label}</button>
                  {/each}
                </div>
              </div>
            </div>
            {/if}
          </div>
        </section>

        <!-- Window / tray is a desktop-only concept — hidden on mobile. -->
        {#if !isMobile}
        <section class="set-group">
          <div class="set-group-h"><h3 class="set-group-t">Fenster</h3></div>
          <div class="set-card">
            <div class="set-row">
              <div class="set-row-l">
                <div class="set-row-t">In den Infobereich schließen</div>
                <div class="set-row-d">Beim Schließen läuft Cortex im Infobereich weiter – Einlesen, Erstellen und Musik laufen weiter. Beenden über das Menü im Infobereich.</div>
              </div>
              <div class="set-row-r">
                <button type="button" class={"st-toggle" + (closeToTray ? " on" : "")} onclick={toggleCloseToTray} role="switch" aria-checked={closeToTray} aria-label="In den Infobereich schließen"><span class="st-knob"></span></button>
              </div>
            </div>
          </div>
        </section>

        <section class="set-group">
          <div class="set-group-h"><h3 class="set-group-t">Anzeige</h3></div>
          <div class="set-card">
            <div class="set-row">
              <div class="set-row-l">
                <div class="set-row-t">Skalierung</div>
                <div class="set-row-d">Alles größer oder kleiner machen – hilfreich bei hochauflösenden Bildschirmen.</div>
              </div>
              <div class="set-row-r" style="min-width:120px">
                <!-- Dropdown of presets (was a slider — the UI rescaling live under the
                     thumb made the slider itself jump around while dragging). -->
                <Picker
                  value={String(app.uiScale)}
                  onChange={(id) => app.setUiScale(+id)}
                  options={[
                    ...([80, 90, 100, 110, 125, 150].includes(app.uiScale) ? [] : [{ id: String(app.uiScale), label: `${app.uiScale}% (eigene)` }]),
                    { id: "80", label: "80%" },
                    { id: "90", label: "90%" },
                    { id: "100", label: "100% – Standard" },
                    { id: "110", label: "110%" },
                    { id: "125", label: "125%" },
                    { id: "150", label: "150%" },
                  ]}
                  placeholder="100%"
                />
              </div>
            </div>
          </div>
        </section>
        {/if}
      </div>

    <!-- ===== KEYBINDS ===== -->
    {:else if tab === "keybinds"}
      <div class="set-pane">
        <header class="set-head">
          <div class="eyebrow">Tastenkürzel</div>
          <h1 class="set-title">Helix-Stil, nach deinen Wünschen</h1>
          <p class="set-sub">Klicke auf ein Kürzel, um es neu zu belegen. Esc bricht die Eingabe ab.</p>
        </header>

        <div class="set-card">
          <div class="set-row">
            <div class="set-row-l">
              <div class="set-row-t">Vorlage</div>
              <div class="set-row-d">Ausgangspunkt für die Belegung.</div>
            </div>
            <div class="set-row-r">
              <div class="seg">
                {#each [{ id: "helix", label: "Helix" }, { id: "vim", label: "Vim" }, { id: "custom", label: "Eigene" }] as opt}
                  <button
                    type="button"
                    class={"seg-opt" + (keybinds.preset === opt.id ? " on" : "")}
                    disabled={opt.id === "custom"}
                    onclick={() => { if (opt.id === "helix" || opt.id === "vim") { keybinds.applyPreset(opt.id); app.pushToast({ kind: "success", title: `${opt.label}-Tastenkürzel übernommen`, body: "Die Kürzel gelten außerhalb dieser Ansicht." }); } }}
                  >{opt.label}</button>
                {/each}
              </div>
            </div>
          </div>
        </div>

        <div class="set-card set-binds">
          {#each ACTION_ORDER as action}
            <div class="bind-row">
              <span class="bind-label">{ACTION_LABELS[action]}</span>
              <button
                class={"bind-keys" + (listening === action ? " listening" : "")}
                onclick={() => (listening = action)}
              >
                {#if listening === action}
                  <span class="mono faint">Taste drücken…</span>
                {:else}
                  <span class="kbd">{displayKey(keybinds.map[action])}</span>
                {/if}
              </button>
            </div>
          {/each}
        </div>

        <div class="set-foot-actions">
          <button
            class="btn btn--ghost"
            onclick={() => {
              const p = keybinds.preset === "vim" ? "vim" : "helix";
              keybinds.applyPreset(p);
              app.pushToast({ kind: "info", title: "Zurückgesetzt", body: `Belegung auf die ${p === "vim" ? "Vim" : "Helix"}-Vorlage zurückgesetzt.` });
            }}
          >
            Auf Vorlage zurücksetzen
          </button>
        </div>

        <section class="set-group">
          <div class="set-group-h">
            <h3 class="set-group-t">Leertaste-Menü (Leertaste, dann…)</h3>
            <p class="set-group-d">Drücke <span class="kbd">Leertaste</span> für das Menü, dann eine Taste. Fest belegt und leicht zu merken – sie wirken nur, solange das Menü offen ist.</p>
          </div>
          <div class="set-card set-binds">
            {#each LEADER_ACTIONS as a}
              <div class="bind-row">
                <span class="bind-label">{a.label} <span class="faint">· {a.detail}</span></span>
                <span class="bind-keys"><span class="kbd">Leertaste</span> <span class="kbd">{a.key}</span></span>
              </div>
            {/each}
          </div>
        </section>

        <section class="set-group">
          <div class="set-group-h">
            <h3 class="set-group-t">System</h3>
            <p class="set-group-d">Eingebaute Kürzel nach Systemkonvention, die sich nicht ändern lassen.</p>
          </div>
          <div class="set-card set-binds">
            {#each SYSTEM_BINDS as b}
              <div class="bind-row">
                <span class="bind-label">{b.label}</span>
                <span class="bind-keys">
                  {#each b.keys.split(" ") as part}<span class="kbd">{part}</span> {/each}
                </span>
              </div>
            {/each}
          </div>
        </section>
      </div>

    <!-- ===== INTEGRATIONS ===== -->
    {:else if tab === "homelab"}
      {#snippet endpointService(o: EndpointOpts)}
        {@const p = connPill(o.state, !!o.value.trim())}
        <section class="set-group">
          <div class="set-group-h svc-h">
            <div>
              <h3 class="set-group-t">{o.title}</h3>
              <p class="set-group-d">{@html o.desc}</p>
            </div>
            <span class="status-pill status-pill--{p.cls}"><span class="dot"></span>{p.label}</span>
          </div>
          <div class="set-card">
            <div class="set-row stacked">
              <div class="row-inline">
                <input class="input mono" value={o.value} oninput={(e) => o.oninput(e.currentTarget.value)} onchange={o.onsave} onblur={o.onsave} placeholder={o.placeholder} />
                <button class="btn" onclick={o.onTest} disabled={o.state === "testing" || !o.value.trim()}>
                  <Icon name="refresh" size={12} /> Testen
                </button>
              </div>
              {#if o.state === "fail" && o.failHint}
                <div class="set-row-d" style="color:var(--err,#e5484d)">{o.failHint}</div>
              {/if}
              {#if o.hint}<div class="set-row-d">{@html o.hint}</div>{/if}
            </div>
            {#if o.extra}{@render o.extra()}{/if}
          </div>
        </section>
      {/snippet}

      {#snippet diagramsToggle()}
        <div class="set-row">
          <div class="set-row-l">
            <div class="set-row-t">Mit Diagrammen illustrieren</div>
            <div class="set-row-d">Ein passendes Diagramm pro Lernzettel-Abschnitt und Bilder im Chat. Standardmäßig an, sobald SearXNG verbunden ist.</div>
          </div>
          <div class="set-row-r">
            <button type="button" class={"st-toggle" + (webImages ? " on" : "")} onclick={() => setWebImages(!webImages)} disabled={!(searxng.trim() || hlBase.trim() || hlTailscale.trim() || hlPublic.trim())} role="switch" aria-checked={webImages} aria-label="diagrams"><span class="st-knob"></span></button>
          </div>
        </div>
      {/snippet}

      <div class="set-pane">
        <header class="set-head">
          <div class="eyebrow">Integrationen</div>
          <h1 class="set-title">Transkription, Suche & Sync – wie du willst</h1>
          <p class="set-sub">Alles funktioniert direkt auf diesem Computer. Für mehr Leistung kannst du einen Cloud-API-Schlüssel hinzufügen oder Cortex auf dein eigenes Homelab (das <span class="mono">homelab/</span>-Docker-Compose) zeigen lassen – nichts davon ist Pflicht.</p>
        </header>

        <!-- ═══ TRANSCRIPTION — outcome-first: pick WHERE audio becomes text ═══ -->
        <section class="set-group">
          <div class="set-group-h">
            <h3 class="set-group-t">Transkription</h3>
            <p class="set-group-d">Wo Aufnahmen zu Text werden.</p>
          </div>
          <div class="set-card">
            <div class="set-row stacked">
              <div class="seg">
                <button type="button" class={"seg-opt" + (transcriptionMode === "local" ? " on" : "")} onclick={() => setTranscriptionMode("local")}>Dieser Computer</button>
                <button type="button" class={"seg-opt" + (transcriptionMode === "cloud" ? " on" : "")} onclick={() => setTranscriptionMode("cloud")}>Cloud-API</button>
                <button type="button" class={"seg-opt" + (transcriptionMode === "homelab" ? " on" : "")} onclick={() => setTranscriptionMode("homelab")}>Mein Homelab</button>
              </div>
              {#if transcriptionMode === "local"}
                <div class="set-row-d">Keine Einrichtung – Whisper läuft auf diesem Gerät, die erste Transkription lädt das Modell automatisch. Am privatesten; bei langen Aufnahmen langsamer als die anderen beiden.</div>
              {:else if transcriptionMode === "cloud"}
                <div class="set-row-d">Kein Server, kein Hosting – nur ein API-Schlüssel. Mit dem kostenlosen Groq-Kontingent wird eine ganze Stunde mit <span class="mono">large-v3-turbo</span> in Sekunden zu Text.</div>
              {:else}
                <div class="set-row-d">Läuft auf dem WhisperX-Server hinter deiner Homelab-URL (unten eingestellt) – gebaut für stundenlange Aufnahmen, mit Sprechererkennung, wenn der Server ein HF-Token hat. Die Audiodaten verlassen deine Geräte nicht.</div>
              {/if}
            </div>


            {#if transcriptionMode === "cloud"}
              <div class="set-row stacked">
                <div class="set-row-t">Anbieter</div>
                <div class="seg">
                  <button type="button" class={"seg-opt" + (whisperCloudProvider === "groq" ? " on" : "")} onclick={() => applyCloudPreset("groq")}>Groq <span class="faint">kostenlos</span></button>
                  <button type="button" class={"seg-opt" + (whisperCloudProvider === "openai" ? " on" : "")} onclick={() => applyCloudPreset("openai")}>OpenAI</button>
                  <button type="button" class={"seg-opt" + (whisperCloudProvider === "custom" ? " on" : "")} onclick={() => applyCloudPreset("custom")}>Eigener</button>
                </div>
              </div>
              {#if whisperCloudProvider === "custom"}
                <div class="set-row stacked">
                  <div class="set-row-t">Endpunkt-URL</div>
                  <input class="input mono" bind:value={whisperCloudUrl} onchange={saveCloudWhisper} onblur={saveCloudWhisper} placeholder="https://api.example.com/v1" />
                  <div class="set-row-d">Jeder OpenAI-kompatible <span class="mono">/v1/audio/transcriptions</span>-Endpunkt.</div>
                </div>
              {/if}
              <div class="set-row stacked">
                <div class="set-row-t">API-Schlüssel</div>
                <input class="input mono" type="password" bind:value={whisperApiKey} onchange={saveCloudWhisper} onblur={saveCloudWhisper} placeholder={whisperCloudProvider === "openai" ? "sk-…" : "gsk_…"} />
                {#if whisperCloudProvider === "groq"}
                  <div class="set-row-d">Kostenlos erhältlich – ohne Kreditkarte:
                    <button class="btn btn--sm btn--ghost" onclick={() => api.openExternal("https://console.groq.com/keys")}><Icon name="external" size={11} /> console.groq.com/keys</button>
                  </div>
                {/if}
              </div>
              <div class="set-row stacked">
                <div class="set-row-t">Modell</div>
                <input class="input mono" bind:value={whisperCloudModel} onchange={saveCloudWhisper} onblur={saveCloudWhisper} placeholder="whisper-large-v3-turbo" />
              </div>
            {/if}

            {#if transcriptionMode === "homelab"}
              <div class="set-row stacked">
                <div class="set-row-t">Modell <span class="faint">nur ältere Server</span></div>
                <input class="input mono" bind:value={whisperModel} onchange={saveWhisperModel} onblur={saveWhisperModel} placeholder="deepdml/faster-whisper-large-v3-turbo-ct2" />
                <div class="set-row-d">Nur für OpenAI-kompatible Server (speaches). Der WhisperX-Server legt sein Modell stattdessen in <span class="mono">docker-compose</span> fest (<span class="mono">WHISPER_MODEL</span>, Standard <span class="mono">distil-large-v3</span>) – dafür leer lassen.</div>
              </div>
            {/if}

            {#if transcriptionMode !== "local"}
              <div class="set-row stacked">
                <div class="row-inline">
                  <button class="btn" onclick={checkWhisper} disabled={whisperCheckState === "checking"}>
                    <Icon name="refresh" size={12} /> {whisperCheckState === "checking" ? "Wird geprüft…" : "Einrichtung prüfen"}
                  </button>
                </div>
                {#if whisperCheckNote}
                  <div class="set-row-d" style="color:{whisperCheckState === 'fail' ? 'var(--err,#e5484d)' : whisperCheckState === 'ok' ? 'var(--ok)' : 'var(--fg-muted)'}">{whisperCheckNote}</div>
                {/if}
              </div>
            {/if}
          </div>
        </section>

        <section class="set-group">
          <div class="set-group-h svc-h">
            <div>
              <h3 class="set-group-t">Homelab-URL</h3>
              <p class="set-group-d">Eine Adresse für alles. Starte das <span class="mono">homelab/</span>-Docker-Compose und trage es hier ein – Suche, Transkription, Sofort-Sync, mobiles Einlesen und lokale Modelle laufen alle über diese eine URL (Cortex ergänzt <span class="mono">/searxng</span>, <span class="mono">/whisper</span>, <span class="mono">/sync</span>, <span class="mono">/syncd</span>, <span class="mono">/ingest</span>, <span class="mono">/ollama</span> selbst). Mit einer Tailscale- und/oder öffentlichen Adresse wählt Cortex automatisch die erste erreichbare: <strong>lokal → Tailscale → öffentlich</strong>.</p>
            </div>
            {#if hlBase.trim()}
              {@const p = connPill(hlState === "idle" ? null : hlState, true)}
              <span class="status-pill status-pill--{p.cls}"><span class="dot"></span>{p.label}</span>
            {/if}
          </div>
          <div class="set-card">
            <div class="set-row stacked">
              <div class="set-row-t">Lokale URL</div>
              <input class="input mono" bind:value={hlBase} oninput={saveHomelabBasesSoon} onchange={saveHomelabBases} onblur={saveHomelabBases} placeholder="http://192.168.1.10:8080" />
              <div class="set-row-d">Die LAN-Adresse deines Homelabs (Port des Caddy-Proxys, Standard <span class="mono">8080</span>).</div>
            </div>
            <div class="set-row stacked">
              <div class="set-row-t">Tailscale-URL <span class="faint">optional</span></div>
              <input class="input mono" bind:value={hlTailscale} oninput={saveHomelabBasesSoon} onchange={saveHomelabBases} onblur={saveHomelabBases} placeholder="https://homelab.tailnet-xxxx.ts.net" />
              <div class="set-row-d">Wird genutzt, wenn die lokale URL nicht erreichbar ist. Cortex tauscht nur den Host (und den Port, falls angegeben) und behält die Dienstpfade.</div>
            </div>
            <div class="set-row stacked">
              <div class="set-row-t">Öffentliche URL <span class="faint">optional</span></div>
              <input class="input mono" bind:value={hlPublic} oninput={saveHomelabBasesSoon} onchange={saveHomelabBases} onblur={saveHomelabBases} placeholder="https://lab.example.com" />
            </div>
            <div class="set-row stacked">
              <div class="set-row-t">Zugangs-Token <span class="faint">optional – nötig für eine öffentliche URL</span></div>
              <input class="input mono" type="password" bind:value={hlToken} onchange={saveHlToken} onblur={saveHlToken} placeholder="das CORTEX_TOKEN, mit dem dein Proxy gestartet wurde" />
              <div class="set-row-d">Schützt alle Homelab-Dienste mit einem gemeinsamen Geheimnis. Starte den Proxy mit <span class="mono">CORTEX_TOKEN=… docker compose up -d</span> und füge denselben Wert hier ein – Cortex sendet ihn bei jeder Anfrage mit (der Sync nutzt eigene WebDAV-Zugangsdaten).</div>
              {#if hlPublic.trim() && !hlToken.trim()}
                <div class="set-row-d" style="color:var(--warn)">Dein Homelab hat eine öffentliche URL, aber kein Zugangs-Token – jeder im Internet kann dein Whisper/Ollama/SearXNG nutzen. Setze <span class="mono">CORTEX_TOKEN</span> am Proxy und füge es hier ein.</div>
              {/if}
            </div>
            <div class="set-row stacked">
              <div class="row-inline" style="flex-wrap:wrap; gap:8px 12px; align-items:center">
                <button class="btn" onclick={testHomelab} disabled={hlState === 'testing' || !(hlBase.trim() || hlTailscale.trim() || hlPublic.trim())}>
                  <Icon name="refresh" size={12} /> {hlState === 'testing' ? "Wird getestet…" : "Homelab testen"}
                </button>
                {#if hlReach.local}<span class="hl-reach" style="color:{hlReach.local === 'ok' ? 'var(--ok)' : 'var(--err,#e5484d)'}">LAN {hlReach.local === 'ok' ? '✓' : '✗'}</span>{/if}
                {#if hlReach.tailscale}<span class="hl-reach" style="color:{hlReach.tailscale === 'ok' ? 'var(--ok)' : 'var(--err,#e5484d)'}">Tailscale {hlReach.tailscale === 'ok' ? '✓' : '✗'}</span>{/if}
                {#if hlReach.public}<span class="hl-reach" style="color:{hlReach.public === 'ok' ? 'var(--ok)' : 'var(--err,#e5484d)'}">Öffentlich {hlReach.public === 'ok' ? '✓' : '✗'}</span>{/if}
              </div>
            </div>
            <!-- Per-service health: each feature probed on the exact URL the app
                 uses, so a red row names the ONE thing to fix (and how). -->
            {#if svcTesting && !svcStatus}
              <div class="set-row-d faint" style="margin-top:6px">Dienste werden geprüft…</div>
            {:else if svcStatus}
              <div style="margin-top:10px">
                {#each svcStatus as s (s.id)}
                  <div class="dep-row" class:dep-missing={s.configured && !s.ok}>
                    <span class="dep-dot" class:on={s.ok}></span>
                    <div class="dep-main">
                      <span class="dep-name">{s.label}</span>
                      <span class="dep-detail mono">{s.detail}</span>
                    </div>
                    <span class="dep-status {s.ok ? 'ok' : s.configured ? 'miss' : ''}">{s.ok ? "läuft" : s.configured ? "Problem" : "nicht gesetzt"}</span>
                  </div>
                {/each}
              </div>
            {/if}
          </div>
        </section>

        <!-- Local CLI dependencies (pdftotext/libreoffice/age/…) never exist on a phone. -->
        {#if !isMobile}
        <section class="set-group">
          <div class="set-group-h svc-h">
            <div>
              <h3 class="set-group-t">Abhängigkeiten</h3>
              <p class="set-group-d">Externe Programme, die Cortex zum Einlesen, für Texterkennung, Transkription und Medien nutzt. Fehlendes installierst du mit dem Befehl unten.</p>
            </div>
            <button class="btn btn--sm btn--ghost" onclick={loadDeps} disabled={depLoading}><Icon name="refresh" size={12} /> {depLoading ? "Wird geprüft…" : "Erneut prüfen"}</button>
          </div>
          <div class="set-card">
            {#if depReport}
              {#each depReport.deps as d (d.name)}
                <div class="dep-row" class:dep-missing={!d.present}>
                  <span class="dep-dot" class:on={d.present}></span>
                  <div class="dep-main">
                    <span class="dep-name">{d.name}</span>
                    <span class="dep-detail mono">{d.detail}</span>
                  </div>
                  <span class="dep-status {d.present ? 'ok' : 'miss'}">{d.present ? "installiert" : "fehlt"}</span>
                </div>
              {/each}
              {#if depReport.install_command}
                <div class="set-row stacked" style="margin-top:10px">
                  <div class="set-row-t">Fehlendes installieren <span class="faint">· {depReport.manager}</span></div>
                  <div class="row-inline">
                    <input class="input mono" readonly value={depReport.install_command} />
                    <button class="btn" onclick={copyDepCmd}><Icon name="doc" size={12} /> Kopieren</button>
                    {#if depReport.manager === "brew"}
                      <button class="btn btn--primary" onclick={installDeps} disabled={depInstalling}><Icon name="plus" size={12} /> {depInstalling ? "Wird installiert…" : "Installieren"}</button>
                    {/if}
                  </div>
                  <div class="set-row-d">{depReport.manager === "brew" ? "Installation mit einem Klick über Homebrew (ohne sudo). LibreOffice ist groß – das kann ein paar Minuten dauern." : depReport.note}</div>
                </div>
              {:else}
                <div class="set-row-d" style="color:var(--ok); margin-top:8px">Alle Abhängigkeiten vorhanden 🎉</div>
              {/if}
            {:else}
              <div class="set-row-d faint">{depLoading ? "Installierte Programme werden geprüft…" : "Abhängigkeiten konnten nicht geprüft werden."}</div>
            {/if}
          </div>
        </section>
        {/if}

        <!-- One section of toggleable homelab features — same on desktop + mobile. No
             per-service URL overrides; everything rides the single Homelab URL above. -->
        <section class="set-group">
          <div class="set-group-h">
            <h3 class="set-group-t">Homelab-Funktionen</h3>
            <p class="set-group-d">Optionale Funktionen über die Dienste hinter deiner Homelab-URL – sie schalten sich ein, sobald eine Homelab-URL gesetzt ist.</p>
          </div>
          <div class="set-card">
            {@render diagramsToggle()}
          </div>
        </section>
        <section class="set-group">
          <div class="set-group-h svc-h">
            <div>
              <h3 class="set-group-t">Live-Sync</h3>
              <p class="set-group-d">Sofortiger Abgleich zwischen Geräten. Jede Änderung wird als kleines Delta pro Datensatz über einen WebSocket (den Homelab-Dienst <span class="mono">/syncd</span>) an deine anderen Geräte geschickt und ist nach etwa einer Sekunde da – die neueste Änderung gewinnt, und gelöscht wird nur, was du wirklich gelöscht hast. Quelldateien und ein vollständiger Snapshot laufen über den WebDAV-Speicher <span class="mono">/sync</span>, der auch als automatischer Ersatz dient, wenn <span class="mono">/syncd</span> nicht erreichbar ist.</p>
            </div>
            <span class="status-pill status-pill--{syncPill().cls}"><span class="dot"></span>{syncPill().label}</span>
          </div>
          <div class="set-card">
            <div class="set-row">
              <div class="set-row-l">
                <div class="set-row-t">Live-Sync aktivieren</div>
                <div class="set-row-d">Führt beim Start den entfernten Speicher zusammen (im Hintergrund – blockiert den Start nie) und überträgt dann deine Änderungen (gebündelt).</div>
              </div>
              <div class="set-row-r">
                <button type="button" class={"st-toggle" + (syncOn ? " on" : "")} onclick={toggleSync} disabled={!canSync} role="switch" aria-checked={syncOn} aria-label="Live-Sync"><span class="st-knob"></span></button>
              </div>
            </div>

            <div class="set-row stacked">
              <div class="set-row-d">Läuft über die <strong>Homelab-URL</strong> oben (deren Dienste <span class="mono">/syncd</span> + <span class="mono">/sync</span>), mit derselben Reihenfolge lokal → Tailscale → öffentlich. Ein Benutzername/Passwort unten gilt für beide – es ist das Paar aus <span class="mono">docker-compose.yml</span> für die Dienste <span class="mono">sync</span> und <span class="mono">syncd</span>.</div>
              {#if syncOn}
                {#if app.syncLive}
                  <div class="set-row-d" style="color:var(--ok)">Live – verbunden mit <span class="mono">/syncd</span>; Änderungen erreichen deine Geräte in etwa einer Sekunde.</div>
                {:else if syncdOk === false}
                  <div class="set-row-d" style="color:var(--warn)">Snapshot-Modus: Der Live-Sync-Dienst des Homelabs hat nicht geantwortet (siehe Dienstprüfung oben). Aktualisiere das Homelab – <span class="mono">git pull && docker compose up -d --build</span> – für Sofort-Sync; bis dahin wird weiter über WebDAV-Snapshots synchronisiert.</div>
                {:else if app.syncState === "error"}
                  <div class="set-row-d" style="color:var(--warn)">Der Sync schlägt fehl – klicke oben auf <strong>Homelab testen</strong>; die Dienstprüfung zeigt, was nicht stimmt (URL, Zugangsdaten oder ein Dienst, der nicht läuft).</div>
                {:else}
                  <div class="set-row-d faint">Gerade nicht live – der Snapshot-Sync läuft trotzdem im Hintergrund. <strong>Homelab testen</strong> oben prüft, ob dein Homelab den Live-Sync-Dienst <span class="mono">/syncd</span> hat.</div>
                {/if}
              {/if}
            </div>
            <div class="set-row stacked">
              <div class="set-row-t">Benutzername <span class="faint">optional</span></div>
              <input class="input mono" bind:value={syncUser} oninput={saveSyncSoon} onchange={saveSync} onblur={saveSync} placeholder="cortex" />
            </div>
            <div class="set-row stacked">
              <div class="set-row-t">Passwort <span class="faint">optional</span></div>
              <div class="row-inline">
                <input class="input mono" type="password" bind:value={syncPass} oninput={saveSyncSoon} onchange={saveSync} onblur={saveSync} placeholder="••••••••" />
                <button class="btn btn--primary" disabled={app.syncState === "syncing" || !canSync} onclick={() => app.syncManual()}>
                  <Icon name="upload" size={12} /> {app.syncState === "syncing" ? "Synchronisiert…" : "Jetzt synchronisieren"}
                </button>
              </div>
              <div class="set-row-d">Letzter Snapshot-Sync: <span class="mono">{fmtSyncTime(app.syncLastAt)}</span> (Live-Änderungen aktualisieren das nicht – es ist der regelmäßige Komplettabgleich). Dateien liegen am Ziel unter <span class="mono">files/</span>; die Datenbank wird pro Datensatz zusammengeführt.</div>
            </div>
          </div>
        </section>

        <!-- Encrypted backups use the age + rclone sidecars — desktop-only. -->
        {#if !isMobile}
        <section class="set-group">
          <div class="set-group-h svc-h">
            <div>
              <h3 class="set-group-t">Verschlüsselte Backups</h3>
              <p class="set-group-d">Die Datenbank sichern, mit <span class="mono">age</span> verschlüsseln und mit <span class="mono">rclone</span> hochladen. Nichts verlässt das Gerät unverschlüsselt.</p>
            </div>
            {#if backupInfo}
              <div class="svc-tools">
                <span class="badge {backupInfo.age_found ? 'badge--ok' : 'badge--err'}"><span class="dot"></span>age</span>
                <span class="badge {backupInfo.rclone_found ? 'badge--ok' : 'badge--err'}"><span class="dot"></span>rclone</span>
              </div>
            {/if}
          </div>
          <div class="set-card">
            <div class="set-row stacked">
              <div class="set-row-t">age-Empfänger (öffentlicher Schlüssel)</div>
              <input class="input mono" bind:value={ageRecipient} onchange={saveBackupConfig} onblur={saveBackupConfig} placeholder="age1ql3z7hjy54pw3hyww5ayyfg7zqgvc7w3j2elw8zmrj2kg5sfn9aqmcac8p" />
              <div class="set-row-d">Aus <span class="mono">age-keygen</span>. Nur der Besitzer dieses Schlüssels kann die Backups entschlüsseln.</div>
            </div>
            <div class="set-row stacked">
              <div class="set-row-t">rclone-Ziel</div>
              <div class="row-inline">
                <input class="input mono" bind:value={rcloneRemote} onchange={saveBackupConfig} onblur={saveBackupConfig} placeholder="homelab:cortex-backups" />
                <button class="btn btn--primary" disabled={backingUp} onclick={runBackup}>
                  <Icon name={backingUp ? "refresh" : "upload"} size={12} /> {backingUp ? "Backup läuft…" : "Jetzt sichern"}
                </button>
              </div>
              <div class="set-row-d">
                Ein rclone-Ziel mit Pfad, z. B. <span class="mono">homelab:cortex-backups</span> (einrichten mit <span class="mono">rclone config</span>).
                Letztes Backup: <span class="mono">{fmtBackupTime(backupInfo?.last_at ?? null)}</span>{#if backupInfo?.last_dest} → <span class="mono">{backupInfo.last_dest}</span>{/if}
              </div>
            </div>
          </div>
        </section>
        {/if}

      </div>

    <!-- ===== AUDIO ===== -->
    {:else if tab === "audio"}
      <div class="set-pane">
        <header class="set-head">
          <div class="eyebrow">Audio</div>
          <h1 class="set-title">Lernmusik & Stimmen</h1>
          <p class="set-sub">Standards für den Musikplayer und erstellte Audio-Überblicke.</p>
        </header>

        <!-- Music is cut on mobile (no mpv/yt-dlp sidecars) — hide its settings. -->
        {#if !isMobile}
        <section class="set-group">
          <div class="set-group-h"><h3 class="set-group-t">Lernmusik</h3></div>
          <div class="set-card">
            <div class="set-row">
              <div class="set-row-l"><div class="set-row-t">Standard-Sender</div></div>
              <div class="set-row-r">
                <Picker
                  value={station}
                  onChange={(v) => { station = v; app.music = { ...app.music, current: v }; }}
                  options={allStations}
                />
              </div>
            </div>
            <div class="set-row">
              <div class="set-row-l"><div class="set-row-t">Beim Start automatisch abspielen</div></div>
              <div class="set-row-r">
                <button type="button" class={"st-toggle" + (autoplay ? " on" : "")} onclick={() => (autoplay = !autoplay)} role="switch" aria-checked={autoplay} aria-label="Automatisch abspielen"><span class="st-knob"></span></button>
              </div>
            </div>
          </div>
        </section>
        {/if}

        <!-- YouTube streaming needs the mpv + yt-dlp sidecars, which don't ship on
             mobile — hide the whole section there. -->
        {#if !isMobile}
        <section class="set-group">
          <div class="set-group-h">
            <h3 class="set-group-t">YouTube-Streaming</h3>
            <p class="set-group-d">Füge im Musikbereich einen YouTube-Video- oder Livestream-Link ein, um ihn werbefrei zu streamen. Nutzt <span class="mono">mpv</span> + <span class="mono">yt-dlp</span> im Hintergrund (beim ersten Mal automatisch geladen). Gespeichert wird nur der Link.</p>
          </div>
          <div class="set-card">
            <div class="set-row">
              <div class="set-row-l"><div class="set-row-t">Programme</div></div>
              <div class="set-row-r">
                {#if mediaTools}
                  <span class="mono" style="color:{mediaTools.mpv ? 'var(--ok)' : 'var(--danger,#e06c75)'}">
                    <Icon name={mediaTools.mpv ? "check" : "x"} size={12} /> mpv
                  </span>
                  <span class="mono" style="margin-left:14px;color:{mediaTools.ffmpeg ? 'var(--ok)' : 'var(--danger,#e06c75)'}">
                    <Icon name={mediaTools.ffmpeg ? "check" : "x"} size={12} /> ffmpeg
                  </span>
                  <span class="mono" style="margin-left:14px;color:{mediaTools.ytdlp ? 'var(--ok)' : 'var(--warn)'}">
                    <Icon name={mediaTools.ytdlp ? "check" : "refresh"} size={12} /> yt-dlp
                  </span>
                {:else}
                  <span class="mono faint">…</span>
                {/if}
              </div>
            </div>
            {#if mediaTools && !mediaTools.mpv}
              <div class="set-row">
                <div class="set-row-l"><div class="set-row-d">Installiere mpv für YouTube-Streaming: <span class="mono">sudo pacman -S mpv</span></div></div>
              </div>
            {:else if mediaTools && !mediaTools.ytdlp}
              <div class="set-row">
                <div class="set-row-l"><div class="set-row-d">yt-dlp wird automatisch heruntergeladen, wenn du zum ersten Mal einen YouTube-Sender abspielst.</div></div>
              </div>
            {/if}
          </div>
        </section>
        {/if}

        <section class="set-group">
          <div class="set-group-h">
            <h3 class="set-group-t">Fokus-Timer</h3>
            <p class="set-group-d">Länge der Pomodoro-Einheiten (gilt für die ganze App).</p>
          </div>
          <div class="set-card">
            {#each pomoFields as f (f.key)}
              <div class="set-row">
                <div class="set-row-l"><div class="set-row-t">{f.label}</div></div>
                <div class="set-row-r">
                  <div style="display:flex;align-items:center;gap:8px">
                    <button class="btn btn--icon btn--sm" onclick={() => setPomo(f.key, -f.step)} aria-label="{f.label} verringern">−</button>
                    <span class="mono" style="min-width:62px;text-align:center;color:var(--fg-bright)">{pomoVal(f.key)}{f.unit}</span>
                    <button class="btn btn--icon btn--sm" onclick={() => setPomo(f.key, f.step)} aria-label="{f.label} erhöhen">+</button>
                  </div>
                </div>
              </div>
            {/each}
          </div>
        </section>

        <section class="set-group">
          <div class="set-group-h">
            <h3 class="set-group-t">Stimmen für Audio-Überblicke</h3>
            <p class="set-group-d">Die zwei Sprecher der erstellten Podcasts.</p>
          </div>
          <div class="set-card">
            <div class="set-row">
              <div class="set-row-l"><div class="set-row-t">Sprecher A</div></div>
              <div class="set-row-r">
                <Picker
                  value={voiceA}
                  onChange={(v) => (voiceA = v)}
                  options={[{ id: "maya", label: "Maya · warm" }, { id: "nova", label: "Nova · hell" }, { id: "io", label: "Io · neutral" }]}
                />
              </div>
            </div>
            <div class="set-row">
              <div class="set-row-l"><div class="set-row-t">Sprecher B</div></div>
              <div class="set-row-r">
                <Picker
                  value={voiceB}
                  onChange={(v) => (voiceB = v)}
                  options={[{ id: "theo", label: "Theo · ruhig" }, { id: "rex", label: "Rex · energisch" }, { id: "sol", label: "Sol · tief" }]}
                />
              </div>
            </div>
          </div>
        </section>
      </div>

    <!-- ===== GOOGLE CALENDAR ===== -->
    {:else if tab === "calendar"}
      <div class="set-pane">
        <header class="set-head">
          <div class="eyebrow">Google Kalender</div>
          <h1 class="set-title">Kalender synchronisieren</h1>
          <p class="set-sub">Beidseitiger Abgleich mit Google Kalender. Der Cortex-Kalender funktioniert auch ohne – die Verbindung spiegelt Termine nur in beide Richtungen.</p>
        </header>

        <section class="set-group">
          <div class="set-group-h">
            <h3 class="set-group-t">Status</h3>
          </div>
          <div class="set-card">
            <div class="set-row">
              <div class="set-row-l">
                <div class="set-row-t">Verbindung</div>
                <div class="set-row-d mono faint">
                  {#if gStatus?.connected}
                    Verbunden{gStatus.email ? " · " + gStatus.email : ""}
                  {:else if gStatus?.configured}
                    Zugangsdaten gespeichert – noch nicht verbunden
                  {:else}
                    Nicht eingerichtet
                  {/if}
                </div>
              </div>
              <div class="set-row-r">
                {#if gStatus?.connected}
                  <div class="row-inline">
                    <button class="btn" onclick={syncGoogle} disabled={gBusy}>
                      <Icon name="refresh" size={12} /> Jetzt synchronisieren
                    </button>
                    <button class="btn btn--danger" onclick={disconnectGoogle} disabled={gBusy}>Trennen</button>
                  </div>
                {:else}
                  <button class="btn btn--primary" onclick={connectGoogle} disabled={gBusy}>
                    <Icon name="globe" size={12} /> {gBusy ? "Wird verbunden…" : "Mit Google verbinden"}
                  </button>
                {/if}
              </div>
            </div>
          </div>
        </section>

        {#if gStatus?.connected}
          <section class="set-group">
            <div class="set-group-h">
              <h3 class="set-group-t">Zu synchronisierende Kalender</h3>
              <p class="set-group-d">Wähle, aus welchen Google-Kalendern Termine geholt werden – hake deinen <strong>Stundenplan-/Schulkalender</strong> an, damit Unterricht, Abgaben und Prüfungen im Cortex-Kalender landen.</p>
            </div>
            <div class="set-card">
              {#if gCalBusy && gCalendars.length === 0}
                <div class="set-row"><div class="set-row-d faint">Kalender werden geladen…</div></div>
              {:else if gCalendars.length === 0}
                <div class="set-row">
                  <div class="set-row-l"><div class="set-row-d faint">Keine Kalender gefunden.</div></div>
                  <div class="set-row-r"><button class="btn btn--sm" onclick={loadGoogleCalendars}>Neu laden</button></div>
                </div>
              {:else}
                {#each gCalendars as cal (cal.id)}
                  <label class="gcal-row" class:on={cal.selected}>
                    <input type="checkbox" checked={cal.selected} onchange={() => toggleGoogleCal(cal.id)} />
                    <span class="gcal-swatch" style:background={cal.color || "var(--border-strong)"}></span>
                    <span class="gcal-name">{cal.summary}</span>
                    {#if cal.primary}<span class="gcal-tag mono">primary</span>{/if}
                    {#if cal.selected}<span class="gcal-on mono">syncing</span>{/if}
                  </label>
                {/each}
              {/if}
            </div>
          </section>
        {/if}

        <section class="set-group">
          <div class="set-group-h">
            <h3 class="set-group-t">Zugangsdaten</h3>
            <p class="set-group-d">Lege in Google Cloud unter APIs &amp; Dienste → Anmeldedaten einen OAuth-Client vom Typ „Desktop-App“ an, aktiviere die Calendar API und füge dann ID und Secret hier ein.</p>
          </div>
          <div class="set-card">
            <div class="set-row stacked">
              <div class="set-row-l"><div class="set-row-t">Client-ID</div></div>
              <div class="set-row-r">
                <input class="input mono" bind:value={gClientId} onblur={saveGoogleCreds} placeholder="…apps.googleusercontent.com" />
              </div>
            </div>
            <div class="set-row stacked">
              <div class="set-row-l"><div class="set-row-t">Client-Secret</div></div>
              <div class="set-row-r">
                <input class="input mono" type="password" bind:value={gClientSecret} onblur={saveGoogleCreds} placeholder="GOCSPX-…" />
              </div>
            </div>
          </div>
        </section>
      </div>

    <!-- ===== EXPERIMENTAL ===== -->
    {:else if tab === "experimental"}
      <div class="set-pane">
        <header class="set-head">
          <div class="eyebrow">Einstellungen</div>
          <h2 class="set-title">Experimentell</h2>
          <p class="set-sub">Frühe Funktionen, die noch holprig sein oder sich ändern können. Schalte eine ein, um sie auszuprobieren.</p>
        </header>

        <section class="set-group">
          <div class="set-group-h svc-h">
            <div>
              <h3 class="set-group-t">Schulportal (Moodle)</h3>
              <p class="set-group-d">Noten, Aufgaben, Termine und Ankündigungen aus deinem Moodle in Cortex holen.</p>
            </div>
            <button class={"st-toggle" + (expMoodle ? " on" : "")} type="button" onclick={toggleExpMoodle} role="switch" aria-checked={expMoodle} aria-label="Moodle aktivieren"><span class="st-knob"></span></button>
          </div>

          {#if expMoodle}
            <div class="set-card">
              <div class="set-row">
                <div class="set-row-l">
                  <div class="set-row-t">Verbindung</div>
                  <div class="set-row-d">{mdStatus.configured ? `Verbunden · letzter Sync ${fmtSyncTime(mdStatus.last_sync)}` : "Nicht verbunden"}</div>
                </div>
                <span class="status-pill status-pill--{mdStatus.configured ? 'ready' : 'pending'}"><span class="dot"></span>{mdStatus.configured ? "Connected" : "Off"}</span>
              </div>

              <div class="set-row stacked">
                <div class="set-row-t">Moodle-Adresse</div>
                <input class="input mono" bind:value={mdUrl} placeholder="https://moodle.deine-schule.de" />
              </div>

              <div class="set-row">
                <div class="set-row-l">
                  <div class="set-row-t">Anmeldeverfahren</div>
                  <div class="set-row-d">Viele Schulen nutzen SSO (Microsoft/SAML), dann funktionieren Benutzername/Passwort oft nicht – füge stattdessen ein Webservice-Token ein.</div>
                </div>
                <div class="set-row-r" style="gap:6px">
                  <button class={"btn btn--sm" + (mdAuthMode === 'token' ? ' btn--primary' : ' btn--ghost')} type="button" onclick={() => (mdAuthMode = 'token')}>Token</button>
                  <button class={"btn btn--sm" + (mdAuthMode === 'password' ? ' btn--primary' : ' btn--ghost')} type="button" onclick={() => (mdAuthMode = 'password')}>Passwort</button>
                </div>
              </div>

              {#if mdAuthMode === 'password'}
                <div class="set-row stacked">
                  <div class="set-row-t">Benutzername</div>
                  <input class="input mono" bind:value={mdUser} placeholder="Benutzername" />
                </div>
                <div class="set-row stacked">
                  <div class="set-row-t">Passwort</div>
                  <input class="input mono" type="password" bind:value={mdPass} placeholder="••••••••" />
                </div>
              {:else}
                <div class="set-row stacked">
                  <div class="set-row-t">Webservice-Token</div>
                  <input class="input mono" bind:value={mdToken} placeholder="Moodle-Token einfügen" />
                  <div class="set-row-d">Du bekommst es über die offizielle Moodle-App oder eine Anmeldung im Browser. Wird lokal gespeichert; dein Passwort wird nie an Cortex gesendet.</div>
                </div>
              {/if}

              <div class="set-row">
                <div class="set-row-l"><div class="set-row-d">Verbinden, dann synchronisieren, um deine Daten zu holen.</div></div>
                <div class="set-row-r" style="gap:8px">
                  {#if mdStatus.configured}
                    <button class="btn btn--ghost btn--sm" type="button" onclick={mdDisconnect} disabled={mdBusy}>Trennen</button>
                    <button class="btn btn--primary btn--sm" type="button" onclick={mdSyncNow} disabled={mdBusy}><Icon name="refresh" size={12} /> {mdBusy ? "Synchronisiert…" : "Jetzt synchronisieren"}</button>
                  {:else}
                    <button class="btn btn--sm" type="button" onclick={mdLoginSso}>Im Browser anmelden (SSO)</button>
                    <button class="btn btn--primary btn--sm" type="button" onclick={mdConnect} disabled={mdBusy || (mdAuthMode==='token' ? !mdToken.trim() : !mdUser.trim())}>{mdBusy ? "Wird verbunden…" : "Verbinden"}</button>
                  {/if}
                </div>
              </div>

              {#if mdStatus.configured}
                <div class="set-row">
                  <div class="set-row-l">
                    <div class="set-row-t">Fächer mit Kursen verknüpfen</div>
                    <div class="set-row-d">Ordnet deine Cortex-Fächer anhand von Kürzel/Name automatisch den Moodle-Kursen zu.</div>
                  </div>
                  <div class="set-row-r"><button class="btn btn--sm" type="button" onclick={mdAutolink}>Automatisch verknüpfen</button></div>
                </div>
              {/if}

              {#if mdData.courses.length}
                <div class="set-row stacked">
                  <div class="set-row-t">Synchronisierte Daten</div>
                  <div class="md-stats">
                    <span class="md-stat"><b>{mdData.courses.length}</b> courses</span>
                    <span class="md-stat"><b>{mdData.grades.length}</b> grades</span>
                    <span class="md-stat"><b>{mdData.deadlines.length}</b> deadlines</span>
                    <span class="md-stat"><b>{mdData.announcements.length}</b> announcements</span>
                  </div>
                  {#if mdUpcoming.length}
                    <div class="md-up-h mono">Anstehende Termine</div>
                    <ul class="md-up">
                      {#each mdUpcoming.slice(0, 6) as d (d.id)}
                        <li>
                          <span class="md-up-date mono">{new Date(d.due_at * 1000).toLocaleDateString(undefined, { day: "numeric", month: "short" })}</span>
                          {#if d.url}
                            <button class="md-up-name md-up-link" onclick={() => d.url && api.openExternal(d.url)} title={d.name}>{d.name}</button>
                          {:else}
                            <span class="md-up-name" title={d.name}>{d.name}</span>
                          {/if}
                          {#if d.course_id}<span class="md-up-course" title={mdCourseName(d.course_id)}>{mdCourseName(d.course_id)}</span>{/if}
                        </li>
                      {/each}
                    </ul>
                  {/if}
                </div>
              {/if}
            </div>
          {/if}
        </section>
      </div>

    <!-- ===== DATA & PRIVACY ===== -->
    {:else if tab === "data"}
      <div class="set-pane">
        <header class="set-head">
          <div class="eyebrow">Daten & Datenschutz</div>
          <h1 class="set-title">Standardmäßig lokal</h1>
          <p class="set-sub">Alles liegt in einer SQLite-Datenbank auf diesem Gerät. Sie gehört dir.</p>
        </header>

        <section class="set-group">
          <div class="set-group-h"><h3 class="set-group-t">Speicher</h3></div>
          <div class="set-card">
            <div class="set-row">
              <div class="set-row-l">
                <div class="set-row-t">Datenbank</div>
                <div class="set-row-d">~/.cortex/cortex.db</div>
              </div>
              <div class="set-row-r">
                <span class="mono faint">
                  {#if stats}
                    {fmtBytes(stats.db_bytes)} · {stats.subjects} {stats.subjects === 1 ? "Fach" : "Fächer"} · {stats.sources} {stats.sources === 1 ? "Quelle" : "Quellen"}
                  {:else}
                    …
                  {/if}
                </span>
              </div>
            </div>
            <div class="set-row">
              <div class="set-row-l">
                <div class="set-row-t">Vektorindex</div>
                <div class="set-row-d">Lokale Embeddings für die Suche</div>
              </div>
              <div class="set-row-r">
                <span class="mono faint">{stats ? `${stats.chunks} ${stats.chunks === 1 ? "Abschnitt" : "Abschnitte"}` : "…"}</span>
              </div>
            </div>
            <div class="set-row">
              <div class="set-row-l">
                <div class="set-row-t">Offline-Modus</div>
                <div class="set-row-d">Alle Netzwerkzugriffe blockieren; nur Ollama.</div>
              </div>
              <div class="set-row-r">
                <button type="button" class={"st-toggle" + (offlineMode ? " on" : "")} onclick={toggleOffline} role="switch" aria-checked={offlineMode} aria-label="offline"><span class="st-knob"></span></button>
              </div>
            </div>
          </div>
        </section>

        <section class="set-group">
          <div class="set-group-h">
            <h3 class="set-group-t">Schule</h3>
            <p class="set-group-d">Ein Ordner mit Montag…Freitag → Stunde-Unterordnern. Jeder Tag wird ein eigenes Fach, jede Stunde ein Thema darin.</p>
          </div>
          <div class="set-card">
            <div class="set-row">
              <div class="set-row-l">
                <div class="set-row-t">Schulordner</div>
                <div class="set-row-d mono faint">{schoolFolderPath || "nicht gesetzt"}</div>
              </div>
              <div class="set-row-r">
                <button type="button" class="btn" onclick={chooseSchoolFolder}>Ordner wählen…</button>
              </div>
            </div>
            {#if schoolFolderPath}
              <div class="set-row">
                <div class="set-row-l">
                  <div class="set-row-t">Synchronisieren</div>
                  <div class="set-row-d">
                    {#if schoolSyncResult}
                      Zuletzt: {schoolSyncResult.added} neu · {schoolSyncResult.updated} aktualisiert · {schoolSyncResult.moved} verschoben · {schoolSyncResult.removed} entfernt · {schoolSyncResult.skipped} übersprungen{schoolSyncResult.errors.length ? ` · ${schoolSyncResult.errors.length} Fehler` : ""}
                    {:else}
                      Läuft automatisch beim Start, beim Wechsel zurück in die App und alle 2 Minuten.
                    {/if}
                  </div>
                </div>
                <div class="set-row-r">
                  <button type="button" class="btn" onclick={runSchoolSync} disabled={schoolSyncing}>{schoolSyncing ? "…" : "Jetzt synchronisieren"}</button>
                </div>
              </div>
            {/if}
          </div>
        </section>

        {#if archivedSubjects.length}
          <section class="set-group">
            <div class="set-group-h">
              <h3 class="set-group-t">Archivierte Fächer</h3>
              <p class="set-group-d">In der App ausgeblendet, aber gespeichert. Jederzeit wiederherstellbar.</p>
            </div>
            <div class="set-card">
              {#each archivedSubjects as s (s.id)}
                <div class="set-row">
                  <div class="set-row-l">
                    <div class="set-row-t">
                      <span style="color:{app.subjectColor(s)}">{s.glyph}</span> {s.name}
                    </div>
                    <div class="set-row-d">
                      {s.code ? s.code + " · " : ""}{s.sourceCount} {s.sourceCount === 1 ? "Quelle" : "Quellen"}
                    </div>
                  </div>
                  <div class="set-row-r">
                    <button class="btn" onclick={() => restoreSubject(s.id)}>Wiederherstellen</button>
                  </div>
                </div>
              {/each}
            </div>
          </section>
        {/if}

        <section class="set-group">
          <div class="set-group-h"><h3 class="set-group-t">Verwalten</h3></div>
          <div class="set-card">
            <div class="set-row">
              <div class="set-row-l">
                <div class="set-row-t">Alles exportieren</div>
                <div class="set-row-d">Fächer, Quellen, Lernzettel → ein portables Archiv.</div>
              </div>
              <div class="set-row-r">
                <button class="btn" onclick={exportData}>
                  <Icon name="external" size={12} /> Exportieren
                </button>
              </div>
            </div>
            <div class="set-row">
              <div class="set-row-l">
                <div class="set-row-t">Speicher optimieren</div>
                <div class="set-row-d">Ungenutzten Speicherplatz freigeben (VACUUM). Sicher.</div>
              </div>
              <div class="set-row-r">
                <button class="btn" onclick={clearCaches}>Optimieren</button>
              </div>
            </div>
            <div class="set-row">
              <div class="set-row-l">
                <div class="set-row-t">Alle Daten löschen</div>
                <div class="set-row-d">Nicht umkehrbar. Leert die lokale Datenbank.</div>
              </div>
              <div class="set-row-r">
                <button class="btn btn--danger" onclick={deleteEverything}>Löschen…</button>
              </div>
            </div>
          </div>
        </section>
      </div>

    <!-- ===== ABOUT ===== -->
    {:else if tab === "about"}
      <div class="set-pane">
        <header class="set-head">
          <div class="eyebrow">Über</div>
          <h1 class="set-title">Cortex</h1>
          <p class="set-sub">Ein Lern-Betriebssystem für den Desktop.</p>
        </header>

        <div class="set-card">
          <div class="set-row">
            <div class="set-row-l"><div class="set-row-t">Version</div></div>
            <div class="set-row-r"><span class="mono faint">{appVersion || "…"}</span></div>
          </div>
          <div class="set-row">
            <div class="set-row-l">
              <div class="set-row-t">Updates</div>
              <div class="set-row-d">Auf GitHub nach einer neueren Version suchen und sie installieren.</div>
            </div>
            <div class="set-row-r">
              <button class="btn" onclick={() => app.checkForUpdates()} disabled={app.updateChecking}>
                <Icon name="refresh" size={12} /> {app.updateChecking ? "Wird geprüft…" : "Nach Updates suchen"}
              </button>
            </div>
          </div>
          <div class="set-row">
            <div class="set-row-l">
              <div class="set-row-t">Cortex unterstützen</div>
              <div class="set-row-d">Cortex wird von einem Studenten entwickelt. Wenn es dir Zeit spart, hält ein Kaffee die Updates am Laufen.</div>
            </div>
            <div class="set-row-r">
              <button class="btn" onclick={() => api.openExternal("https://ko-fi.com/aidanmcconnon")}>
                <Icon name="heart" size={12} color="var(--err)" /> Auf Ko-fi unterstützen
              </button>
            </div>
          </div>
          <div class="set-row">
            <div class="set-row-l"><div class="set-row-t">Technik</div></div>
            <div class="set-row-r"><span class="mono faint">Rust · Tauri · Svelte</span></div>
          </div>
          <div class="set-row">
            <div class="set-row-l"><div class="set-row-t">Design-Quelle</div></div>
            <div class="set-row-r">
              <span class="mono faint">
                Omarchy · {THEME_LABELS[app.theme]}
              </span>
            </div>
          </div>
          <div class="set-row">
            <div class="set-row-l"><div class="set-row-t">Lizenz</div></div>
            <div class="set-row-r"><span class="mono faint">Quelltext verfügbar · eigener Schlüssel</span></div>
          </div>
        </div>

        <div class="set-note mono">
          <Icon name="diamond" size={11} color="var(--accent)" />
          Offline zuerst. Deine Notizen verlassen dieses Gerät nur, wenn du ein Cloud-Modell wählst.
        </div>
      </div>
    {/if}

  </div>
</div>
