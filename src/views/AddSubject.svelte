<script lang="ts">
  import { app, SUBJECT_COLORS, GLYPHS } from "../lib/store.svelte";
  import * as api from "../lib/api";
  import Icon from "../components/Icon.svelte";
  import EmojiPicker from "../components/EmojiPicker.svelte";
  import { isMobile } from "../lib/platform";

  let name = $state("");
  let code = $state("");
  let color = $state(SUBJECT_COLORS[0]);
  let glyph = $state(GLYPHS[0]);
  let topics = $state(["", "", ""]);

  const colors = SUBJECT_COLORS;

  function setTopic(i: number, v: string) {
    topics = topics.map((x, idx) => (idx === i ? v : x));
  }

  const ready = $derived(name.trim().length > 0);

  async function create() {
    if (!ready) return;
    try {
      const subj = await api.createSubject(name.trim(), code.trim() || undefined, glyph, color);
      // Create each non-empty starter topic sequentially. Resilient: a failing
      // topic surfaces a toast but never blocks the rest or the navigation.
      for (const t of topics) {
        const tn = t.trim();
        if (!tn) continue;
        try {
          await api.createTopic(subj.id, tn);
        } catch (te) {
          app.pushToast({ kind: "error", title: "Thema konnte nicht hinzugefügt werden", body: `${tn}: ${String(te)}` });
        }
      }
      await app.refresh();
      app.pushToast({ kind: "success", title: "Fach erstellt", body: name.trim() + " ist bereit." });
      app.setView("dashboard");
    } catch (e) {
      app.pushToast({ kind: "error", title: "Fach konnte nicht erstellt werden", body: String(e) });
    }
  }
</script>

<div class="workspace-scroll">
  <div class="addpage addpage--subject">
    <div class="addpage-head">
      {#if !isMobile}
        <button class="btn btn--icon btn--sm btn--ghost" onclick={() => app.setView("dashboard")} title="Zurück">
          <Icon name="chevron" size={14} style="transform:rotate(180deg)" />
        </button>
      {/if}
      <div>
        <div class="eyebrow">Neues Fach</div>
        <h1 class="addpage-title">Fach erstellen</h1>
        <div class="mono faint" style="font-size:var(--t-xs)">enthält Themen, Quellen und einen mitwachsenden Lernzettel</div>
      </div>
    </div>

    <div class="addsubj-preview">
      <span class="subj-glyph" style="border-color:{color};color:{color};font-size:15px;display:inline-flex;align-items:center;justify-content:center">
        {glyph}
      </span>
      <div>
        <div class="read" style="font-size:var(--r-lg);color:var(--fg-bright)">{name || "Untitled subject"}</div>
        <div class="mono faint" style="font-size:var(--t-2xs)">{code || "kein Kürzel"} · {topics.filter(t => t.trim()).length} Themen</div>
      </div>
    </div>

    <div class="addsubj-form">
      <div class="field">
        <!-- svelte-ignore a11y_label_has_associated_control -->
        <label class="onb-label mono">NAME DES FACHS</label>
        <!-- svelte-ignore a11y_autofocus -->
        <input class="input" autofocus bind:value={name} placeholder="z. B. Mathematik" />
      </div>

      <div class="field">
        <!-- svelte-ignore a11y_label_has_associated_control -->
        <label class="onb-label mono">KÜRZEL <span class="faint">optional</span></label>
        <input class="input mono" bind:value={code} placeholder="MA-11" />
      </div>

      <div class="field">
        <!-- svelte-ignore a11y_label_has_associated_control -->
        <label class="onb-label mono">FARBE</label>
        <div class="color-row">
          {#each colors as c}
            <button
              class={"color-dot" + (color === c ? " on" : "")}
              style="background:{c}"
              onclick={() => (color = c)}
            >
              {#if color === c}
                <Icon name="check" size={12} color="#07140f" />
              {/if}
            </button>
          {/each}
        </div>
      </div>

      <div class="field">
        <!-- svelte-ignore a11y_label_has_associated_control -->
        <label class="onb-label mono">SYMBOL</label>
        <EmojiPicker value={glyph} onPick={(e) => (glyph = e)} />
      </div>

      <div class="field">
        <!-- svelte-ignore a11y_label_has_associated_control -->
        <label class="onb-label mono">ERSTE THEMEN <span class="faint">optional – hier legst du Material ab</span></label>
        <div class="topic-inputs">
          {#each topics as t, i}
            <input
              class="input"
              value={t}
              oninput={e => setTopic(i, (e.target as HTMLInputElement).value)}
              placeholder={["Analysis", "Lineare Algebra", "Stochastik"][i] || "Thema"}
            />
          {/each}
        </div>
      </div>
    </div>

    <div class="add-foot">
      <button class="btn btn--ghost" onclick={() => app.setView("dashboard")}>Abbrechen</button>
      <button class="btn btn--primary" disabled={!ready} onclick={create}>
        <Icon name="check" size={13} /> Fach erstellen
      </button>
    </div>
  </div>
</div>
