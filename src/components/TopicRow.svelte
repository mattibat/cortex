<script lang="ts">
  // One editable topic row for the subject panel: glyph (emoji picker), name and
  // tags, all editable inline. Glyph saves immediately on pick; name/tags save on
  // blur. Each row owns its local state (seeded from the topic) so a background
  // store refresh after a save doesn't clobber an in-progress edit.
  import { app, topicGlyph } from "../lib/store.svelte";
  import type { Topic } from "../lib/api";
  import Icon from "./Icon.svelte";
  import EmojiPicker from "./EmojiPicker.svelte";

  let { topic, subjectId }: { topic: Topic; subjectId: string } = $props();

  // Local editable copies, seeded from the topic in an effect (matches EditModal's
  // pattern). Reseeds when the underlying topic changes (e.g. after a save+refresh);
  // doesn't fire mid-typing since those edits touch local state, not `topic.*`.
  let name = $state("");
  let tags = $state("");
  let glyph = $state("");
  $effect(() => {
    name = topic.name;
    tags = (topic.tags ?? []).join(", ");
    glyph = topic.glyph || topicGlyph(topic.id);
  });

  const tagList = () => tags.split(",").map((s) => s.trim()).filter(Boolean);

  async function save() {
    await app.updateTopic(topic.id, name.trim() || topic.name, subjectId, glyph || undefined, tagList());
  }
  async function pick(g: string) {
    glyph = g;
    await save();
  }
  async function del() {
    if (!(await app.confirm({ title: `Thema „${topic.name}“ löschen?`, body: "Die Quellen darin werden keinem Thema mehr zugeordnet.", danger: true, okLabel: "Löschen" }))) return;
    await app.deleteTopic(topic.id, subjectId);
  }
</script>

<div class="tr">
  <EmojiPicker value={glyph} onPick={pick} size={28} />
  <input class="input tr-name" bind:value={name} onblur={save} placeholder="Name des Themas" />
  <input class="input tr-tags mono" bind:value={tags} onblur={save} placeholder="Tags, durch Komma getrennt" />
  <button class="btn btn--icon btn--sm btn--ghost" title="Thema löschen" onclick={del}>
    <Icon name="x" size={12} />
  </button>
</div>

<style>
  .tr { display: flex; align-items: center; gap: 8px; }
  .tr-name { flex: 1 1 42%; min-width: 0; }
  .tr-tags { flex: 1 1 48%; min-width: 0; font-size: 12px; }
</style>
