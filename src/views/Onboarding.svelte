<script lang="ts">
  import { app, THEMES } from "../lib/store.svelte";
  import type { Theme } from "../lib/store.svelte";
  import Icon from "../components/Icon.svelte";
  import logo from "../assets/cortex-logo.png";

  let { onFinish }: { onFinish?: () => void } = $props();

  let step = $state(0);
  let apiKey = $state("");
  let theme = $state<Theme>("osaka-jade");
  let homelab = $state(false);
  let homelabEndpoint = $state("");
  let subjName = $state("");

  const steps = ["Willkommen", "Dein Schlüssel", "Design", "Homelab", "Erstes Fach"];

  function next() {
    step = Math.min(steps.length - 1, step + 1);
  }

  function pickTheme(t: Theme) {
    theme = t;
    app.setTheme(t);
  }

  function finish() {
    onFinish?.();
  }

  const THEME_OPTS: { id: Theme; n: string; c: string; b: string }[] = [
    { id: "osaka-jade",  n: "Osaka Jade",  c: "#2dd5b7", b: "#111c18" },
    { id: "tokyo-night", n: "Tokyo Night",  c: "#7aa2f7", b: "#1a1b26" },
    { id: "catppuccin",  n: "Catppuccin",   c: "#94e2d5", b: "#1e1e2e" },
  ];

  const SUGGESTIONS = ["Mathematik", "Deutsch", "Informatik"];
</script>

<div class="onb">
  <!-- Left rail: step list -->
  <div class="onb-rail">
    <div class="onb-brand">
      <img class="ds-logo-glyph" src={logo} alt="Cortex" /> Cortex
    </div>
    <div class="onb-steps">
      {#each steps as s, i}
        <div class={"onb-step" + (i === step ? " on" : "") + (i < step ? " done" : "")}>
          <span class="os-num">
            {#if i < step}
              <Icon name="check" size={11} />
            {:else}
              {i + 1}
            {/if}
          </span>
          {s}
        </div>
      {/each}
    </div>
    <div class="onb-skip">
      <button class="btn btn--ghost btn--sm" onclick={finish}>Einrichtung überspringen</button>
    </div>
  </div>

  <!-- Main content -->
  <div class="onb-main">
    <div class="onb-card">
      <!-- Step 0: Welcome -->
      {#if step === 0}
        <div class="onb-pane">
          <div class="eyebrow">Erster Start</div>
          <h1 class="read onb-h">Ein ruhiger Ort für schwierige Fächer.</h1>
          <p class="onb-p read">
            Cortex macht aus Unterricht, PDFs, Aufnahmen und dem Web strukturiertes Lernmaterial –
            verlässliche Lernzettel, Karteikarten und einen Chat, der genau bei dem bleibt,
            was du fragst. Fünf kurze Schritte; alles lässt sich später ändern.
          </p>
          <div class="onb-feats">
            <div class="onb-feat">
              <Icon name="book" size={15} color="var(--accent)" />
              <span>Auf Vollständigkeit geprüfte Lernzettel – nichts geht still verloren</span>
            </div>
            <div class="onb-feat">
              <Icon name="record" size={15} color="var(--accent)" />
              <span>Aufnahme &amp; Transkription des Unterrichts eingebaut</span>
            </div>
            <div class="onb-feat">
              <Icon name="cmd" size={15} color="var(--accent)" />
              <span>Tastaturorientierte Navigation im Helix-Stil</span>
            </div>
          </div>
          <button class="btn btn--primary onb-cta" onclick={next}>
            Loslegen <Icon name="arrowR" size={14} />
          </button>
        </div>

      <!-- Step 1: BYOK -->
      {:else if step === 1}
        <div class="onb-pane">
          <div class="eyebrow">Eigener Schlüssel</div>
          <h1 class="read onb-h">Füge deinen Gemini-API-Schlüssel ein.</h1>
          <p class="onb-p read">
            Cortex nutzt deinen eigenen Schlüssel – er bleibt auf diesem Gerät im Schlüsselbund des Systems.
            Nichts läuft über fremde Server. Claude, OpenAI oder ein lokales Ollama-Modell kannst du später in den Einstellungen hinzufügen.
          </p>
          <!-- svelte-ignore a11y_label_has_associated_control -->
          <label class="onb-label mono">GEMINI_API_KEY</label>
          <input
            class="input"
            bind:value={apiKey}
            placeholder="AIza…"
            style="font-family:var(--font-mono)"
          />
          <div class="onb-row">
            <Icon name="diamond" size={11} color="var(--fg-faint)" />
            <span class="mono faint">Im Schlüsselbund gespeichert · nie synchronisiert</span>
          </div>
          <div class="onb-actions">
            <button class="btn btn--ghost" onclick={next}>Später hinzufügen</button>
            <button class="btn btn--primary" onclick={next}>Weiter <Icon name="arrowR" size={14} /></button>
          </div>
        </div>

      <!-- Step 2: Theme -->
      {:else if step === 2}
        <div class="onb-pane">
          <div class="eyebrow">Aussehen</div>
          <h1 class="read onb-h">Wir haben dein Omarchy-Design erkannt.</h1>
          <p class="onb-p read">
            Cortex passt sich live an deinen Desktop an. Gefunden: <b class="accent">Osaka Jade</b> –
            behalte es oder wähle ein anderes. Wechselst du später dein Omarchy-Design, zieht Cortex automatisch mit.
          </p>
          <div class="onb-themes">
            {#each THEME_OPTS as t}
              <button
                class={"onb-theme" + (theme === t.id ? " on" : "")}
                onclick={() => pickTheme(t.id)}
                style="background:{t.b}"
              >
                <span class="ot-sw" style="background:{t.c}"></span>
                <span class="ot-name" style="color:{theme === t.id ? '#fff' : '#cbd'}">{t.n}</span>
                {#if theme === t.id}
                  <span class="ot-check" style="color:{t.c}">
                    <Icon name="check" size={13} />
                  </span>
                {/if}
              </button>
            {/each}
          </div>
          <div class="onb-actions">
            <span class="mono faint">Über Omarchy erkannt</span>
            <button class="btn btn--primary" onclick={next}>Weiter <Icon name="arrowR" size={14} /></button>
          </div>
        </div>

      <!-- Step 3: Homelab -->
      {:else if step === 3}
        <div class="onb-pane">
          <div class="eyebrow">Optional</div>
          <h1 class="read onb-h">Hast du ein Homelab?</h1>
          <p class="onb-p read">
            Lagere aufwendige Aufgaben – Whisper-Transkription, große Modelle, Backups – auf einen
            Rechner in deinem Netzwerk aus. Ohne Homelab bleibt Cortex komplett lokal; es macht große
            Aufgaben nur schneller.
          </p>
          <button
            class={"onb-toggle" + (homelab ? " on" : "")}
            onclick={() => (homelab = !homelab)}
          >
            <span class="ot-knob"></span>
            <span class="mono">{homelab ? "Homelab für aufwendige Aufgaben nutzen" : "Alles auf diesem Gerät ausführen"}</span>
          </button>
          {#if homelab}
            <div class="onb-homelab">
              <!-- svelte-ignore a11y_label_has_associated_control -->
              <label class="onb-label mono">ENDPOINT</label>
              <input
                class="input"
                bind:value={homelabEndpoint}
                placeholder="http://homelab.local:11434"
                style="font-family:var(--font-mono)"
              />
              <button class="btn btn--sm" style="margin-top:10px">
                <Icon name="refresh" size={12} /> Verbindung testen
              </button>
            </div>
          {/if}
          <div class="onb-actions">
            <button class="btn btn--ghost" onclick={next}>Überspringen</button>
            <button class="btn btn--primary" onclick={next}>Weiter <Icon name="arrowR" size={14} /></button>
          </div>
        </div>

      <!-- Step 4: First subject -->
      {:else if step === 4}
        <div class="onb-pane">
          <div class="eyebrow">Letzter Schritt</div>
          <h1 class="read onb-h">Erstelle dein erstes Fach.</h1>
          <p class="onb-p read">
            Ein Fach enthält Themen, Quellen und einen mitwachsenden Lernzettel. Benenne es nach einem
            Schulfach – danach fügst du Material hinzu.
          </p>
          <!-- svelte-ignore a11y_label_has_associated_control -->
          <label class="onb-label mono">NAME DES FACHS</label>
          <!-- svelte-ignore a11y_autofocus -->
          <input
            class="input"
            bind:value={subjName}
            placeholder="z. B. Mathematik"
            autofocus
          />
          <div class="onb-suggest">
            {#each SUGGESTIONS as x}
              <button class="btn btn--sm btn--ghost" onclick={() => (subjName = x)}>{x}</button>
            {/each}
          </div>
          <button class="btn btn--primary onb-cta" onclick={finish}>
            <Icon name="check" size={14} /> Cortex starten
          </button>
        </div>
      {/if}
    </div>

    <!-- Progress bar -->
    <div class="onb-progress">
      <div class="onb-progress-bar" style="width:{((step + 1) / steps.length) * 100}%"></div>
    </div>
  </div>
</div>
