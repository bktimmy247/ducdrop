<script lang="ts">
  import { onMount } from "svelte";
  import {
    startDownload,
    openFolder,
    readClipboard,
    onProgress,
    type DownloadMode,
    type DownloadProgress,
  } from "$lib/tauri";

  let url = $state("");
  let busy = $state(false);
  let toast = $state<{ text: string; kind: "error" | "info" } | null>(null);
  let items = $state<DownloadProgress[]>([]);

  let toastTimer: ReturnType<typeof setTimeout> | null = null;
  function showToast(text: string, kind: "error" | "info" = "info") {
    toast = { text, kind };
    if (toastTimer) clearTimeout(toastTimer);
    toastTimer = setTimeout(() => (toast = null), 4000);
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
    }).then((fn) => (unlisten = fn));
    return () => unlisten?.();
  });

  function looksLikeUrl(v: string): boolean {
    return /^https?:\/\/\S+$/i.test(v.trim());
  }

  async function paste() {
    const t = await readClipboard();
    if (t) {
      url = t.trim();
      showToast("Đã dán link từ clipboard");
    } else {
      showToast("Clipboard trống", "error");
    }
  }

  async function download(mode: DownloadMode) {
    const v = url.trim();
    if (!v) {
      showToast("Anh dán link vào đã nhé", "error");
      return;
    }
    if (!looksLikeUrl(v)) {
      showToast("Link chưa đúng. Cần bắt đầu bằng http…", "error");
      return;
    }
    busy = true;
    try {
      await startDownload(v, mode);
      showToast("Đã thêm vào hàng chờ tải");
      url = "";
    } catch (e) {
      showToast("Không bắt đầu được: " + String(e), "error");
    } finally {
      busy = false;
    }
  }

  async function reveal(item: DownloadProgress) {
    if (item.filepath) await openFolder(item.filepath);
  }

  const modeLabel: Record<string, string> = {
    smart: "Tải thông minh",
    best: "Đẹp nhất",
    audio: "Chỉ nghe (MP3)",
    small: "Nhẹ máy",
  };
</script>

<main>
  <header class="top">
    <div class="brand">
      <div class="logo">⬇</div>
      <div>
        <div class="name">DucDrop</div>
        <div class="tag">Thả link. Bấm tải. Xong.</div>
      </div>
    </div>
  </header>

  <section class="hero">
    <div class="paste-row">
      <input
        class="url"
        type="text"
        placeholder="Dán link video, nhạc, ảnh… vào đây"
        bind:value={url}
        onkeydown={(e) => e.key === "Enter" && download("smart")}
      />
      <button class="paste" onclick={paste} title="Dán từ clipboard">Dán</button>
    </div>

    <button class="smart" disabled={busy} onclick={() => download("smart")}>
      Tải thông minh
    </button>

    <div class="modes">
      <button class="mode" disabled={busy} onclick={() => download("best")}>
        <span class="m-ico">🎬</span> Đẹp nhất
      </button>
      <button class="mode" disabled={busy} onclick={() => download("audio")}>
        <span class="m-ico">🎧</span> Chỉ nghe (MP3)
      </button>
      <button class="mode" disabled={busy} onclick={() => download("small")}>
        <span class="m-ico">📱</span> Nhẹ máy
      </button>
    </div>
  </section>

  <section class="list">
    {#if items.length === 0}
      <div class="empty">
        <div class="empty-ico">⬇</div>
        <div class="empty-title">Chưa có gì tải cả</div>
        <div class="empty-sub">Thả link. Bấm tải. Xong.</div>
      </div>
    {:else}
      {#each items as item (item.id)}
        <div class="card">
          {#if item.thumbnail}
            <img class="thumb" src={item.thumbnail} alt="" />
          {:else}
            <div class="thumb ph">🎞</div>
          {/if}
          <div class="body">
            <div class="title">{item.title || item.url}</div>
            <div class="meta">
              {#if item.status === "completed"}
                <span class="done">✓ Xong</span>
              {:else if item.status === "error"}
                <span class="err">Lỗi: {item.message || "không tải được"}</span>
              {:else}
                <span class="pct">{Math.round(item.percent)}%</span>
                {#if item.speed}<span class="spd">{item.speed}</span>{/if}
              {/if}
            </div>
            {#if item.status === "downloading" || item.status === "queued"}
              <div class="bar"><div class="fill" style="width:{item.percent}%"></div></div>
            {/if}
          </div>
          {#if item.status === "completed"}
            <button class="open" onclick={() => reveal(item)}>Mở thư mục</button>
          {/if}
        </div>
      {/each}
    {/if}
  </section>

  {#if toast}
    <div class="toast" class:err={toast.kind === "error"}>{toast.text}</div>
  {/if}
</main>

<style>
  main {
    max-width: 720px;
    margin: 0 auto;
    padding: 26px 22px 40px;
    display: flex;
    flex-direction: column;
    gap: 22px;
    min-height: 100vh;
  }
  .top {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }
  .brand {
    display: flex;
    align-items: center;
    gap: 12px;
  }
  .logo {
    width: 42px;
    height: 42px;
    border-radius: 12px;
    background: linear-gradient(160deg, var(--accent-hi), var(--accent-press));
    display: grid;
    place-items: center;
    font-size: 22px;
    font-weight: 700;
    box-shadow: var(--shadow);
  }
  .name {
    font-size: 18px;
    font-weight: 700;
    letter-spacing: 0.2px;
  }
  .tag {
    font-size: 12.5px;
    color: var(--muted);
  }
  .hero {
    background: var(--card);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    padding: 20px;
    display: flex;
    flex-direction: column;
    gap: 14px;
    box-shadow: var(--shadow);
  }
  .paste-row {
    display: flex;
    gap: 10px;
  }
  .url {
    flex: 1;
    background: var(--bg-soft);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    padding: 14px 16px;
    color: var(--text);
    font-size: 15px;
    outline: none;
    user-select: text;
  }
  .url:focus {
    border-color: var(--accent);
  }
  .paste {
    background: var(--card-hi);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    padding: 0 18px;
    font-weight: 600;
    font-size: 14px;
  }
  .paste:hover {
    background: #24304a;
  }
  .smart {
    background: linear-gradient(160deg, var(--accent-hi), var(--accent-press));
    border-radius: var(--radius-sm);
    padding: 16px;
    font-size: 16.5px;
    font-weight: 700;
    letter-spacing: 0.3px;
    box-shadow: 0 8px 20px rgba(47, 129, 247, 0.35);
    transition: transform 0.08s ease, filter 0.15s ease;
  }
  .smart:hover {
    filter: brightness(1.07);
  }
  .smart:active {
    transform: translateY(1px);
  }
  .smart:disabled {
    filter: grayscale(0.4) brightness(0.8);
    cursor: default;
  }
  .modes {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 10px;
  }
  .mode {
    background: var(--card-hi);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    padding: 12px 8px;
    font-size: 13.5px;
    font-weight: 600;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 5px;
    transition: background 0.15s ease;
  }
  .mode:hover {
    background: #24304a;
  }
  .mode:disabled {
    opacity: 0.5;
    cursor: default;
  }
  .m-ico {
    font-size: 18px;
  }
  .list {
    display: flex;
    flex-direction: column;
    gap: 10px;
    flex: 1;
  }
  .empty {
    text-align: center;
    padding: 60px 20px;
    color: var(--muted);
  }
  .empty-ico {
    font-size: 46px;
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
  .card {
    display: flex;
    align-items: center;
    gap: 14px;
    background: var(--card);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    padding: 12px;
  }
  .thumb {
    width: 68px;
    height: 46px;
    border-radius: 8px;
    object-fit: cover;
    background: var(--bg-soft);
    flex-shrink: 0;
  }
  .thumb.ph {
    display: grid;
    place-items: center;
    font-size: 22px;
  }
  .body {
    flex: 1;
    min-width: 0;
  }
  .title {
    font-size: 14px;
    font-weight: 600;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .meta {
    font-size: 12.5px;
    color: var(--muted);
    margin-top: 4px;
    display: flex;
    gap: 12px;
  }
  .done {
    color: var(--green);
    font-weight: 600;
  }
  .err {
    color: var(--red);
  }
  .pct {
    color: var(--accent-hi);
    font-weight: 600;
  }
  .bar {
    margin-top: 8px;
    height: 5px;
    background: var(--bg-soft);
    border-radius: 4px;
    overflow: hidden;
  }
  .fill {
    height: 100%;
    background: linear-gradient(90deg, var(--accent), var(--accent-hi));
    transition: width 0.2s ease;
  }
  .open {
    background: var(--card-hi);
    border: 1px solid var(--border);
    border-radius: 9px;
    padding: 9px 13px;
    font-size: 13px;
    font-weight: 600;
    flex-shrink: 0;
  }
  .open:hover {
    background: #24304a;
  }
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
  }
  .toast.err {
    background: #3a1d20;
    border-color: #5a2a2c;
    color: #ffd7d3;
  }
</style>
