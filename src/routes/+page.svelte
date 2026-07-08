<script lang="ts">
  import { onMount } from "svelte";
  import {
    startDownload,
    cancelDownload,
    cancelAllDownloads,
    catchPreview,
    openFolder,
    getDownloadsDir,
    readClipboard,
    onProgress,
    getEngineHealth,
    type DownloadMode,
    type DownloadProgress,
    type DownloadStatus,
    type EngineHealth,
    type PreviewInfo,
  } from "$lib/tauri";
  import {
    hero,
    ctas,
    modes as modeCopy,
    states,
    loading,
    engine as engineCopy,
    humanError,
    hints,
  } from "$lib/ducdrop-copy";

  // --- state ---
  let url = $state("");
  let busy = $state(false);
  let toast = $state<{ text: string; kind: "error" | "info" } | null>(null);
  let items = $state<DownloadProgress[]>([]);
  let preview = $state<PreviewInfo | null>(null);
  let engineHealth = $state<EngineHealth[]>([]);
  let advancedOpen = $state(false);
  let logs = $state<string[]>([]);
  let dropHover = $state(false);

  let toastTimer: ReturnType<typeof setTimeout> | null = null;
  function showToast(text: string, kind: "error" | "info" = "info") {
    toast = { text, kind };
    if (toastTimer) clearTimeout(toastTimer);
    toastTimer = setTimeout(() => (toast = null), 4000);
  }

  function log(line: string) {
    logs = [line, ...logs].slice(0, 120);
  }

  onMount(() => {
    let unlisten: (() => void) | null = null;
    onProgress((p) => {
      const idx = items.findIndex((i) => i.id === p.id);
      if (idx === -1) items = [p, ...items];
      else {
        items[idx] = { ...items[idx], ...p };
        items = [...items];
      }
      if (p.message) log(`[${p.id}] ${p.status}: ${p.message}`);
      else log(`[${p.id}] ${p.status} ${Math.round(p.percent)}%`);
    }).then((fn) => (unlisten = fn));

    getEngineHealth().then((h) => {
      engineHealth = h;
      h.forEach((e) => log(`engine ${e.name}: ${e.message} (${e.ready ? "ok" : "missing"})`));
    });

    return () => unlisten?.();
  });

  function looksLikeUrl(v: string): boolean {
    return /^https?:\/\/\S+$/i.test(v.trim());
  }

  function linkLines(): string[] {
    return url
      .split(/\r?\n/)
      .map((x) => x.trim())
      .filter(Boolean);
  }

  function primaryLink(): string {
    return linkLines()[0] ?? "";
  }

  function hasManyLinks(): boolean {
    return linkLines().length > 1;
  }

  function guessSource(u: string): string {
    const lower = u.toLowerCase();
    if (lower.includes("youtube") || lower.includes("youtu.be")) return "YouTube";
    if (lower.includes("tiktok")) return "TikTok";
    if (lower.includes("facebook") || lower.includes("fb.watch")) return "Facebook";
    if (lower.includes("instagram")) return "Instagram";
    return "Web";
  }

  async function paste() {
    const t = await readClipboard();
    if (t) {
      url = t.trim();
      showToast(hints.pasted);
    } else {
      showToast("Clipboard trống", "error");
    }
  }

  async function catchLink() {
    const v = primaryLink();
    if (!v) {
      showToast(hints.noUrl, "error");
      return;
    }
    if (hasManyLinks()) showToast(hints.multi, "info");
    if (!looksLikeUrl(v)) {
      showToast(hints.badUrl, "error");
      return;
    }
    busy = true;
    preview = null;
    try {
      log(`catch: ${v}`);
      const p = await catchPreview(v);
      preview = p ?? {
        title: v,
        source: guessSource(v),
        thumbnail: undefined,
        duration: undefined,
        modes: ["video", "audio", "thumbnail", "subtitle", "channel"],
        note: states.previewUnsupported,
      };
      if (preview.note) showToast(preview.note, "info");
    } catch (e) {
      log(`catch error: ${String(e)}`);
      preview = {
        title: v,
        source: guessSource(v),
        modes: ["video", "audio", "thumbnail", "subtitle", "channel"],
        note: states.previewUnsupported,
      };
    } finally {
      busy = false;
    }
  }

  async function download(mode: DownloadMode) {
    const links = linkLines();
    if (links.length === 0) {
      showToast(hints.noUrl, "error");
      return;
    }
    const invalid = links.find((x) => !looksLikeUrl(x));
    if (invalid) {
      showToast(hints.badUrl, "error");
      return;
    }

    const runMode: DownloadMode = mode;
    const selected = links.length > 1 ? links : [links[0]];

    busy = true;
    try {
      for (const link of selected) {
        await startDownload(link, runMode);
      }
      showToast(selected.length > 1 ? `Đã thêm ${selected.length} link vào hàng chờ` : hints.added);
      preview = null;
    } catch (e) {
      const msg = humanError(String(e));
      showToast(msg, "error");
      log(`download error: ${String(e)}`);
    } finally {
      busy = false;
    }
  }

  function isActive(item: DownloadProgress): boolean {
    return item.status === "queued" || item.status === "downloading";
  }

  async function stopItem(item: DownloadProgress) {
    if (!isActive(item)) return;
    items = items.map((x) =>
      x.id === item.id ? { ...x, status: "cancelled", message: "Đã dừng lệnh tải." } : x,
    );
    try {
      await cancelDownload(item.id);
      showToast("Đã dừng lệnh tải", "info");
    } catch (e) {
      showToast(humanError(String(e)), "error");
      log(`cancel error: ${String(e)}`);
    }
  }

  async function removeItem(item: DownloadProgress) {
    if (isActive(item)) await stopItem(item);
    items = items.filter((x) => x.id !== item.id);
  }

  async function clearQueue() {
    const hasActive = items.some(isActive);
    if (hasActive) {
      try {
        await cancelAllDownloads();
        showToast("Đã dừng toàn bộ lệnh tải và xóa danh sách", "info");
      } catch (e) {
        showToast(humanError(String(e)), "error");
        log(`cancel all error: ${String(e)}`);
      }
    }
    items = [];
  }

  async function reveal(item: DownloadProgress) {
    if (item.filepath) await openFolder(item.filepath);
  }

  async function openDownloadsDir() {
    const dir = await getDownloadsDir();
    await openFolder(dir);
  }

  // --- drag / drop ---
  function onDragOver(e: DragEvent) {
    e.preventDefault();
    dropHover = true;
  }
  function onDragLeave(e: DragEvent) {
    const target = e.currentTarget as Element | null;
    if (target && !target.contains(e.relatedTarget as Node | null)) {
      dropHover = false;
    }
  }
  async function onDrop(e: DragEvent) {
    e.preventDefault();
    dropHover = false;
    const text = e.dataTransfer?.getData("text/plain")?.trim();
    if (text && looksLikeUrl(text)) {
      url = text;
      await catchLink();
    } else {
      showToast("Kéo thả chưa đúng. Em chỉ nhận link (http...) nhé.", "error");
    }
  }

  // --- helpers ---
  const allModes: { key: DownloadMode; label: string; icon: string; desc: string }[] = [
    { key: "video", label: modeCopy.video.label, icon: modeCopy.video.icon, desc: modeCopy.video.desc },
    { key: "audio", label: modeCopy.audio.label, icon: modeCopy.audio.icon, desc: modeCopy.audio.desc },
    { key: "thumbnail", label: modeCopy.thumbnail.label, icon: modeCopy.thumbnail.icon, desc: modeCopy.thumbnail.desc },
    { key: "subtitle", label: modeCopy.subtitle.label, icon: modeCopy.subtitle.icon, desc: modeCopy.subtitle.desc },
    { key: "channel", label: modeCopy.channel.label, icon: modeCopy.channel.icon, desc: modeCopy.channel.desc },
  ];

  function modeAvailable(key: DownloadMode): boolean {
    if (!preview) return true;
    return preview.modes.includes(key);
  }

  function engineClass(ready: boolean): string {
    return ready ? "badge badge--ok" : "badge badge--err";
  }

  function statusLabel(s: DownloadStatus): string {
    switch (s) {
      case "queued":
        return states.queued;
      case "downloading":
        return states.running;
      case "completed":
        return states.completed;
      case "error":
        return states.failed;
      case "cancelled":
        return "Đã dừng";
      default:
        return s;
    }
  }

  function statusIcon(s: DownloadStatus): string {
    switch (s) {
      case "queued":
        return "⏳";
      case "downloading":
        return "⬇️";
      case "completed":
        return "✓";
      case "error":
        return "✕";
      case "cancelled":
        return "⏹";
      default:
        return "•";
    }
  }

  function fmtPercent(n: number): number {
    return Math.max(0, Math.min(100, Math.round(n)));
  }
</script>

<main>
  <header class="top">
    <div class="brand">
      <div class="logo" aria-hidden="true">
        <img src="/ducdrop-logo.png" alt="" />
      </div>
      <div>
        <div class="name">{hero.brand}</div>
        <div class="tag">{hero.tagline}</div>
      </div>
    </div>

    <div class="top-actions">
      <button class="ghost" onclick={openDownloadsDir} title={ctas.openFolder}>
        📂 {ctas.openFolder}
      </button>
    </div>
  </header>

  <section
    class="hero"
    class:drop-hover={dropHover}
    ondragover={onDragOver}
    ondragleave={onDragLeave}
    ondrop={onDrop}
    aria-label="Khu vực thả link"
  >
    <div class="hero-head">
      <div class="hero-text">
        <h1>{hero.dropTitle}</h1>
        <p class="sub">{hero.dropHint}</p>
      </div>

    </div>

    <div class="paste-row">
      <textarea
        class="url"
        rows="3"
        placeholder="Dán một link hoặc nhiều link, mỗi dòng một link"
        bind:value={url}
        onkeydown={(e) => e.key === "Enter" && (e.ctrlKey || e.metaKey) && catchLink()}
      ></textarea>
      <button class="secondary" onclick={paste} title="Dán từ clipboard">
        {ctas.paste}
      </button>
      <button class="primary" disabled={busy} onclick={catchLink}>
        {busy ? loading.catch : ctas.catch}
      </button>
    </div>

      <aside class="owner-card" aria-label="Thông tin cộng đồng Cường Đức Agentic">
        <div class="owner-copy">
          <div class="owner-kicker">Cộng đồng</div>
          <div class="owner-name">Cường Đức Agentic</div>
          <a class="owner-link" href="https://zalo.me/g/6mkvta67ijedc8abfgsc" target="_blank" rel="noreferrer">
            zalo.me/g/6mkvta67ijedc8abfgsc
          </a>
        </div>
        <img class="owner-qr" src="/cuong-duc-zalo-qr.png" alt="QR nhóm Zalo Cường Đức Agentic" />
      </aside>

    {#if hasManyLinks()}
      <div class="link-count">Đã nhận {linkLines().length} link. Chọn một chế độ bên dưới để tải lần lượt.</div>
    {/if}

    <p class="platform-hint">{hero.platforms}</p>

    {#if preview}
      <div class="preview">
        {#if preview.thumbnail}
          <img class="preview-thumb" src={preview.thumbnail} alt="" />
        {/if}
        <div class="preview-body">
          <div class="preview-title">{preview.title || url}</div>
          <div class="preview-meta">
            <span class="badge badge--source">{preview.source}</span>
            {#if preview.duration}
              <span class="duration">⏱ {preview.duration}</span>
            {/if}
          </div>
          {#if preview.note}
            <div class="preview-note">{preview.note}</div>
          {/if}
        </div>
      </div>
    {/if}

    <div class="modes">
      {#each allModes as m}
        <button
          class="mode"
          disabled={busy || !modeAvailable(m.key)}
          onclick={() => download(m.key)}
          title={m.label}
        >
          <span class="m-ico" aria-hidden="true">{m.icon}</span>
          <span class="m-label">{m.label}</span>
          <span class="m-desc">{m.desc}</span>
          {#if !modeAvailable(m.key)}
            <span class="m-disabled">Chưa hỗ trợ</span>
          {/if}
        </button>
      {/each}
    </div>
  </section>

  <section class="queue" aria-label="Hàng chờ tải">
    <div class="section-head">
      <div class="section-title">Hàng chờ &amp; trạng thái</div>
      {#if items.length > 0}
        <button class="ghost small danger" onclick={clearQueue}>Dừng & xóa danh sách</button>
      {/if}
    </div>

    {#if items.length === 0}
      <div class="empty">
        <div class="empty-ico" aria-hidden="true">⬇</div>
        <div class="empty-title">{states.empty}</div>
        <div class="empty-sub">{states.emptySub}</div>
      </div>
    {:else}
      <div class="items">
        {#each items as item (item.id)}
          <div class="task" class:task--done={item.status === "completed"} class:task--err={item.status === "error"} class:task--cancelled={item.status === "cancelled"}>
            {#if item.thumbnail}
              <img class="task-thumb" src={item.thumbnail} alt="" />
            {:else}
              <div class="task-thumb ph" aria-hidden="true">🎞</div>
            {/if}
            <div class="task-body">
              <div class="task-title">{item.title || item.url}</div>
              <div class="task-meta">
                <span class="task-status" class:ok={item.status === "completed"} class:bad={item.status === "error"} class:muted={item.status === "cancelled"}>
                  {statusIcon(item.status)} {statusLabel(item.status)}
                </span>
                {#if item.status === "downloading" || item.status === "queued"}
                  <span class="task-pct">{fmtPercent(item.percent)}%</span>
                  {#if item.speed}
                    <span class="task-spd">{item.speed}</span>
                  {/if}
                {/if}
              </div>
              {#if item.message && (item.status === "error" || item.status === "cancelled")}
                <div class="task-msg" class:muted={item.status === "cancelled"}>{item.status === "error" ? humanError(item.message) : item.message}</div>
              {/if}
              {#if item.status === "downloading" || item.status === "queued"}
                <div class="bar">
                  <div class="fill" style="width: {fmtPercent(item.percent)}%"></div>
                </div>
              {/if}
            </div>
            <div class="task-actions">
              {#if isActive(item)}
                <button class="danger small" onclick={() => stopItem(item)}>Dừng</button>
              {/if}
              {#if item.status === "completed"}
                <button class="secondary small" onclick={() => reveal(item)}>
                  {ctas.openFolder}
                </button>
              {/if}
              <button class="ghost small" onclick={() => removeItem(item)}>Xóa lệnh</button>
            </div>
          </div>
        {/each}
      </div>
    {/if}
  </section>

  <section class="engines" aria-label="Tình trạng engine">
    <div class="engine-title">
      <span>{engineCopy.title}</span>
      <button
        class="ghost tiny"
        onclick={() => getEngineHealth().then((h) => (engineHealth = h))}
        title="Kiểm tra lại"
      >
        🔄
      </button>
    </div>
    <div class="engine-list">
      {#each engineHealth as e}
        <div class={engineClass(e.ready)} role="status">
          <span class="dot" class:ok={e.ready} class:bad={!e.ready}></span>
          <span>{e.name}</span>
          {#if e.version}
            <span class="ver">{e.version}</span>
          {/if}
        </div>
      {:else}
        <div class="badge badge--warn">{engineCopy.checking}</div>
      {/each}
    </div>
  </section>

  <section class="advanced">
    <button class="advanced-toggle" onclick={() => (advancedOpen = !advancedOpen)}>
      {advancedOpen ? "▾" : "▸"} {ctas.advanced}
    </button>
    {#if advancedOpen}
      <div class="advanced-panel">
        {#if logs.length === 0}
          <div class="no-logs">Chưa có log kỹ thuật.</div>
        {:else}
          <div class="logs">
            {#each logs as line}
              <div class="log-line">{line}</div>
            {/each}
          </div>
        {/if}
      </div>
    {/if}
  </section>

  {#if toast}
    <div class="toast" class:err={toast.kind === "error"}>{toast.text}</div>
  {/if}
</main>

<style>
  main {
    max-width: 980px;
    margin: 0 auto;
    padding: 28px 28px 44px;
    display: flex;
    flex-direction: column;
    gap: 22px;
    min-height: 100vh;
    position: relative;
    isolation: isolate;
  }
  main::before {
    content: "";
    position: fixed;
    inset: 0;
    z-index: -2;
    background:
      radial-gradient(740px 420px at 18% 0%, rgba(24, 199, 255, 0.28), transparent 62%),
      radial-gradient(560px 420px at 92% 10%, rgba(124, 58, 237, 0.26), transparent 58%),
      linear-gradient(140deg, #06101d 0%, #091729 45%, #100d24 100%);
  }
  main::after {
    content: "";
    position: fixed;
    inset: 0;
    z-index: -1;
    pointer-events: none;
    opacity: 0.34;
    background-image:
      linear-gradient(rgba(255,255,255,0.055) 1px, transparent 1px),
      linear-gradient(90deg, rgba(255,255,255,0.045) 1px, transparent 1px);
    background-size: 46px 46px;
    mask-image: radial-gradient(circle at 50% 20%, black, transparent 72%);
  }

  /* header */
  .top {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 14px;
    padding: 14px 16px;
    border: 1px solid rgba(255,255,255,0.08);
    border-radius: 22px;
    background: rgba(7, 17, 31, 0.62);
    box-shadow: 0 14px 44px rgba(0,0,0,0.22);
    backdrop-filter: blur(18px);
  }
  .brand {
    display: flex;
    align-items: center;
    gap: 13px;
  }
  .logo {
    width: 50px;
    height: 50px;
    border-radius: 18px;
    background: rgba(255,255,255,0.06);
    border: 1px solid rgba(255,255,255,0.14);
    display: grid;
    place-items: center;
    box-shadow: 0 0 0 8px rgba(24,199,255,0.08), 0 18px 42px rgba(24,199,255,0.18);
    overflow: hidden;
  }
  .logo img {
    width: 34px;
    height: 34px;
    object-fit: contain;
  }
  .name {
    font-size: 20px;
    font-weight: 800;
    letter-spacing: -0.3px;
    background: linear-gradient(90deg, var(--accent-hi), var(--violet-hi));
    -webkit-background-clip: text;
    -webkit-text-fill-color: transparent;
    background-clip: text;
  }
  .tag {
    font-size: 13px;
    color: var(--muted);
    margin-top: 2px;
  }
  .top-actions {
    display: flex;
    gap: 8px;
  }

  /* buttons */
  .primary {
    background: linear-gradient(150deg, var(--accent), var(--violet));
    color: #fff;
    border-radius: var(--radius-sm);
    padding: 13px 22px;
    font-size: 15px;
    font-weight: 700;
    letter-spacing: 0.2px;
    box-shadow: 0 8px 24px rgba(24, 199, 255, 0.28);
    transition: transform 0.08s ease, filter 0.15s ease;
    white-space: nowrap;
  }
  .primary:hover {
    filter: brightness(1.08);
  }
  .primary:active {
    transform: translateY(1px);
  }
  .primary:disabled {
    filter: grayscale(0.5) brightness(0.75);
    cursor: default;
  }
  .secondary {
    background: var(--card-hi);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    padding: 12px 18px;
    font-size: 14px;
    font-weight: 600;
    transition: background 0.15s ease, border-color 0.15s ease;
    white-space: nowrap;
  }
  .secondary:hover {
    background: #263550;
    border-color: var(--border-hi);
  }
  .secondary.small {
    padding: 9px 13px;
    font-size: 13px;
  }
  .ghost {
    background: transparent;
    border: 1px solid transparent;
    color: var(--muted);
    border-radius: var(--radius-sm);
    padding: 9px 12px;
    font-size: 13px;
    font-weight: 600;
    transition: color 0.15s ease, background 0.15s ease;
  }
  .ghost:hover {
    color: var(--text);
    background: rgba(255, 255, 255, 0.05);
  }
  .ghost.small {
    padding: 6px 10px;
    font-size: 12px;
  }
  .ghost.danger {
    color: var(--rose);
    border-color: rgba(244, 63, 94, 0.24);
  }
  .ghost.tiny {
    padding: 4px 8px;
    font-size: 13px;
  }

  /* hero */
  .hero {
    background:
      linear-gradient(160deg, rgba(24,199,255,0.18), rgba(124,58,237,0.12) 42%, rgba(12,20,34,0.92) 100%),
      rgba(15,23,42,0.72);
    border: 1px solid rgba(137, 207, 255, 0.18);
    border-radius: 30px;
    padding: 34px;
    display: flex;
    flex-direction: column;
    gap: 20px;
    box-shadow: 0 26px 80px rgba(0,0,0,0.42), inset 0 1px 0 rgba(255,255,255,0.08);
    transition: border-color 0.2s ease, box-shadow 0.2s ease;
    position: relative;
    overflow: hidden;
    backdrop-filter: blur(22px);
  }
  .hero::before {
    content: "";
    position: absolute;
    width: 260px;
    height: 260px;
    right: -70px;
    top: -80px;
    border-radius: 44% 56% 56% 44% / 44% 42% 58% 56%;
    background: radial-gradient(circle at 36% 24%, rgba(255,255,255,0.42), transparent 20%), linear-gradient(160deg, rgba(24,199,255,0.36), rgba(124,58,237,0.28));
    filter: blur(0.2px);
    opacity: 0.72;
  }
  .hero::after {
    content: "";
    position: absolute;
    inset: auto 28px 18px 28px;
    height: 1px;
    background: linear-gradient(90deg, transparent, rgba(24,199,255,0.45), rgba(124,58,237,0.4), transparent);
  }
  .hero > * {
    position: relative;
    z-index: 1;
  }
  .hero.drop-hover {
    border-color: var(--accent);
    box-shadow: var(--shadow-glow), var(--shadow);
  }
  .hero-head {
    display: block;
  }
  .hero-text {
    text-align: left;
    padding-left: 10px;
  }
  .drop-ico {
    display: flex;
    justify-content: center;
    margin-bottom: 10px;
    filter: drop-shadow(0 6px 18px rgba(24, 199, 255, 0.2));
  }
  .hero h1 {
    margin: 0;
    font-size: clamp(30px, 5vw, 48px);
    font-weight: 900;
    letter-spacing: -1.4px;
    line-height: 1.02;
    text-shadow: 0 14px 50px rgba(0,0,0,0.4);
  }
  .sub {
    margin: 8px 0 0;
    color: var(--muted);
    font-size: 14.5px;
    line-height: 1.5;
    max-width: 560px;
    margin-inline: 0;
  }
  .platform-hint {
    margin: 0;
    color: var(--muted);
    font-size: 12.5px;
    text-align: center;
  }

  .paste-row {
    display: flex;
    gap: 10px;
    align-items: stretch;
    padding: 8px;
    border-radius: 18px;
    background: rgba(3, 10, 22, 0.62);
    border: 1px solid rgba(255,255,255,0.08);
    box-shadow: inset 0 1px 0 rgba(255,255,255,0.05), 0 20px 50px rgba(0,0,0,0.24);
  }
  .url {
    flex: 1;
    background: rgba(255,255,255,0.055);
    border: 1px solid rgba(255,255,255,0.08);
    border-radius: 14px;
    padding: 15px 17px;
    color: var(--text);
    font-size: 15px;
    outline: none;
    user-select: text;
    min-width: 0;
    resize: vertical;
    min-height: 78px;
    line-height: 1.45;
  }
  .url:focus {
    border-color: var(--accent);
    box-shadow: 0 0 0 3px rgba(24, 199, 255, 0.12);
  }
  .url::placeholder {
    color: #607294;
  }

  .smart-row {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 12px;
    flex-wrap: wrap;
  }
  .smart-primary {
    min-width: 260px;
    border-radius: 999px;
    padding: 17px 28px;
    font-size: 17px;
    font-weight: 900;
    letter-spacing: -0.2px;
    color: #04101e;
    background: linear-gradient(135deg, #5ce1ff, #2ee59d 52%, #f8fbff);
    box-shadow: 0 18px 45px rgba(24,199,255,0.3), 0 0 0 10px rgba(46,229,157,0.06);
  }
  .smart-primary span {
    margin-right: 8px;
  }
  .like-last,
  .link-count {
    border-radius: 999px;
    padding: 11px 15px;
    border: 1px solid rgba(255,255,255,0.1);
    background: rgba(255,255,255,0.06);
    color: var(--muted);
    font-weight: 700;
    font-size: 13px;
  }
  .link-count {
    color: var(--mint);
    border-color: rgba(46,229,157,0.22);
  }

  /* preview */
  .preview {
    display: flex;
    gap: 14px;
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    padding: 14px;
    align-items: center;
  }
  .preview-thumb {
    width: 100px;
    height: 68px;
    border-radius: var(--radius-xs);
    object-fit: cover;
    background: var(--card);
    flex-shrink: 0;
  }
  .preview-body {
    flex: 1;
    min-width: 0;
  }
  .preview-title {
    font-size: 15px;
    font-weight: 700;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .preview-meta {
    display: flex;
    gap: 10px;
    align-items: center;
    margin-top: 6px;
    font-size: 12.5px;
    color: var(--muted);
  }
  .badge--source {
    background: rgba(124, 58, 237, 0.16);
    color: var(--violet-hi);
  }
  .duration {
    color: var(--muted);
  }
  .preview-note {
    margin-top: 8px;
    font-size: 12.5px;
    color: var(--amber);
    line-height: 1.4;
  }

  /* modes */
  .modes {
    display: grid;
    grid-template-columns: repeat(5, 1fr);
    gap: 10px;
  }
  .mode {
    background: linear-gradient(180deg, rgba(255,255,255,0.075), rgba(255,255,255,0.035));
    border: 1px solid rgba(255,255,255,0.1);
    border-radius: 18px;
    padding: 16px 10px;
    font-size: 13.5px;
    font-weight: 600;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 4px;
    transition: background 0.15s ease, border-color 0.15s ease, transform 0.08s ease;
    position: relative;
    overflow: hidden;
  }
  .mode:hover:not(:disabled) {
    background: #1f2e4a;
    border-color: var(--accent);
    transform: translateY(-1px);
  }
  .mode:disabled {
    opacity: 0.45;
    cursor: default;
  }
  .m-ico {
    font-size: 22px;
    margin-bottom: 2px;
  }
  .m-label {
    color: var(--text);
  }
  .m-desc {
    font-size: 11.5px;
    color: var(--muted);
    font-weight: 500;
  }
  .m-disabled {
    position: absolute;
    bottom: 0;
    left: 0;
    right: 0;
    font-size: 10px;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    background: rgba(244, 63, 94, 0.14);
    color: var(--rose);
    padding: 3px 0;
  }

  /* queue */
  .queue {
    background: linear-gradient(180deg, rgba(20,29,44,0.82), rgba(10,16,28,0.72));
    border: 1px solid rgba(255,255,255,0.08);
    border-radius: 24px;
    padding: 20px;
    box-shadow: 0 20px 70px rgba(0,0,0,0.3), inset 0 1px 0 rgba(255,255,255,0.05);
    flex: 1;
    backdrop-filter: blur(16px);
  }
  .section-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 14px;
  }
  .section-title {
    font-size: 14px;
    font-weight: 700;
    color: var(--muted);
    text-transform: uppercase;
    letter-spacing: 0.6px;
  }
  .empty {
    text-align: center;
    padding: 54px 20px;
    color: var(--muted);
  }
  .empty-ico {
    font-size: 48px;
    opacity: 0.35;
  }
  .empty-title {
    margin-top: 12px;
    font-size: 16px;
    font-weight: 600;
    color: #b6c0d2;
  }
  .empty-sub {
    font-size: 13px;
    margin-top: 4px;
  }
  .items {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }
  .task {
    display: flex;
    align-items: center;
    gap: 12px;
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    padding: 12px;
    transition: border-color 0.15s ease;
  }
  .task--done {
    border-color: rgba(46, 229, 157, 0.25);
  }
  .task--err {
    border-color: rgba(244, 63, 94, 0.25);
  }
  .task--cancelled {
    border-color: rgba(147, 163, 191, 0.24);
    opacity: 0.78;
  }
  .task-thumb {
    width: 72px;
    height: 50px;
    border-radius: 8px;
    object-fit: cover;
    background: var(--card);
    flex-shrink: 0;
  }
  .task-thumb.ph {
    display: grid;
    place-items: center;
    font-size: 22px;
  }
  .task-body {
    flex: 1;
    min-width: 0;
  }
  .task-title {
    font-size: 14px;
    font-weight: 600;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .task-meta {
    font-size: 12.5px;
    color: var(--muted);
    margin-top: 5px;
    display: flex;
    gap: 12px;
    align-items: center;
  }
  .task-status {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    font-weight: 600;
  }
  .task-status.ok {
    color: var(--mint);
  }
  .task-status.bad {
    color: var(--rose);
  }
  .task-status.muted {
    color: var(--muted);
  }
  .task-pct {
    color: var(--accent-hi);
    font-weight: 700;
  }
  .task-spd {
    color: var(--muted);
  }
  .task-msg {
    margin-top: 6px;
    font-size: 12.5px;
    color: var(--rose);
    line-height: 1.4;
  }
  .task-msg.muted {
    color: var(--muted);
  }
  .task-actions {
    display: flex;
    align-items: center;
    gap: 8px;
    flex-wrap: wrap;
    justify-content: flex-end;
  }
  .danger.small {
    padding: 9px 13px;
    font-size: 13px;
    border-radius: var(--radius-sm);
    color: #fff;
    background: linear-gradient(180deg, rgba(244, 63, 94, 0.92), rgba(190, 18, 60, 0.88));
    box-shadow: 0 10px 24px rgba(244, 63, 94, 0.18);
  }
  .bar {
    margin-top: 8px;
    height: 5px;
    background: var(--card);
    border-radius: 4px;
    overflow: hidden;
  }
  .fill {
    height: 100%;
    background: linear-gradient(90deg, var(--accent), var(--violet-hi));
    transition: width 0.25s ease;
  }

  /* engines */
  .engines {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    background: var(--card);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    padding: 12px 18px;
    box-shadow: var(--shadow);
  }
  .engine-title {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 13px;
    font-weight: 700;
    color: var(--muted);
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }
  .engine-list {
    display: flex;
    gap: 10px;
    flex-wrap: wrap;
  }
  .dot {
    width: 7px;
    height: 7px;
    border-radius: 50%;
    background: var(--muted);
  }
  .dot.ok {
    background: var(--mint);
    box-shadow: 0 0 8px var(--mint);
  }
  .dot.bad {
    background: var(--rose);
    box-shadow: 0 0 8px var(--rose);
  }
  .ver {
    font-weight: 500;
    opacity: 0.85;
  }

  /* advanced */
  .advanced {
    background: var(--card);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    overflow: hidden;
    box-shadow: var(--shadow);
  }
  .advanced-toggle {
    width: 100%;
    text-align: left;
    padding: 14px 18px;
    background: var(--surface);
    color: var(--muted);
    font-size: 13px;
    font-weight: 600;
    border-bottom: 1px solid transparent;
    transition: color 0.15s ease, border-color 0.15s ease;
  }
  .advanced-toggle:hover {
    color: var(--text);
    border-color: var(--border);
  }
  .advanced-panel {
    padding: 14px 18px;
  }
  .logs {
    max-height: 240px;
    overflow-y: auto;
    background: var(--ink);
    border: 1px solid var(--border);
    border-radius: var(--radius-xs);
    padding: 10px 14px;
    font-family: "Consolas", "SFMono-Regular", monospace;
    font-size: 12px;
    color: #a9b9d7;
  }
  .log-line {
    white-space: pre-wrap;
    word-break: break-all;
    padding: 2px 0;
    border-bottom: 1px solid rgba(255, 255, 255, 0.04);
  }
  .no-logs {
    color: var(--muted);
    font-size: 13px;
  }

  .owner-card {
    width: 100%;
    display: grid;
    grid-template-columns: minmax(0, 1fr) 88px;
    gap: 14px;
    align-items: center;
    padding: 12px 14px;
    border-radius: 20px;
    background: rgba(7, 17, 31, 0.66);
    border: 1px solid rgba(24, 199, 255, 0.18);
    box-shadow: inset 0 1px 0 rgba(255,255,255,0.08);
    backdrop-filter: blur(18px);
    position: relative;
    z-index: 1;
  }
  .owner-kicker {
    color: var(--accent-hi);
    font-size: 12px;
    font-weight: 900;
    text-transform: uppercase;
    letter-spacing: 0.8px;
    margin-bottom: 5px;
  }
  .owner-name {
    font-size: 16px;
    font-weight: 900;
    color: var(--text);
    letter-spacing: -0.3px;
  }
  .owner-phone,
  .owner-link {
    display: block;
    color: var(--muted);
    text-decoration: none;
    font-size: 13px;
    margin-top: 5px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .owner-phone {
    color: var(--mint);
    font-weight: 800;
  }
  .owner-link:hover,
  .owner-phone:hover {
    color: var(--accent-hi);
  }
  .owner-qr {
    width: 88px;
    height: 88px;
    border-radius: 14px;
    object-fit: contain;
    padding: 6px;
    border: 2px solid rgba(255,255,255,0.9);
    background: #fff;
  }

  /* toast */
  .toast {
    position: fixed;
    left: 50%;
    bottom: 26px;
    transform: translateX(-50%);
    background: #1c2740;
    border: 1px solid var(--border);
    color: var(--text);
    padding: 12px 20px;
    border-radius: 12px;
    font-size: 14px;
    box-shadow: var(--shadow);
    max-width: 90vw;
    text-align: center;
  }
  .toast.err {
    background: rgba(244, 63, 94, 0.12);
    border-color: rgba(244, 63, 94, 0.35);
    color: #ffd7d3;
  }

  /* responsive */
  @media (max-width: 860px) {
    .hero-head {
      display: block;
    }
    .hero-text {
      padding-left: 0;
      text-align: center;
    }
    .sub {
      margin-inline: auto;
    }
  }
  @media (max-width: 640px) {
    main {
      padding: 18px 14px 30px;
    }
    .paste-row {
      flex-wrap: wrap;
    }
    .url {
      min-width: 100%;
    }
    .modes {
      grid-template-columns: repeat(2, 1fr);
    }
    .engines {
      flex-direction: column;
      align-items: flex-start;
    }
    .preview {
      flex-direction: column;
      align-items: flex-start;
    }
    .owner-card {
      grid-template-columns: minmax(0, 1fr) 76px;
      padding: 10px 12px;
    }
    .owner-qr {
      width: 76px;
      height: 76px;
    }
    .preview-thumb {
      width: 100%;
      height: auto;
      aspect-ratio: 16 / 10;
    }
  }
</style>


