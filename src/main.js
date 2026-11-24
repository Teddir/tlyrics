const { invoke } = window.__TAURI__.core;
const eventModule = window.__TAURI__.event;

let lastPlaybackState = null;
let lastLyricsState = null;
let currentRenderedFingerprint = null;
let smoothPosition = 0;
let touchBarInfo = null;

function formatTime(seconds) {
  if (isNaN(seconds) || seconds < 0) return "0:00";
  const mins = Math.floor(seconds / 60);
  const secs = Math.floor(seconds % 60);
  return `${mins}:${secs.toString().padStart(2, "0")}`;
}

function updateVerificationTable(playbackState, lyricsState) {
  const tbody = document.getElementById("verification-body");
  if (!tbody) return;

  const track = playbackState?.track;
  const isDetected = playbackState?.is_detected;

  const fields = [
    {
      name: "Active Provider",
      status: "Active",
      statusClass: "badge-verified",
      value: `${playbackState?.providerId || "-"} (${playbackState?.providerName || "-"})`,
    },
    {
      name: "Source Application",
      status: isDetected ? "Connected" : "Idle",
      statusClass: isDetected ? "badge-verified" : "badge-empty",
      value: `${playbackState?.appName || "-"} [${playbackState?.bundleId || "-"}]`,
    },
    {
      name: "Current Song",
      status: track?.title ? "Resolved" : "None",
      statusClass: track?.title ? "badge-verified" : "badge-empty",
      value: track ? `${track.title} - ${track.artist}` : "None",
    },
    {
      name: "Touch Bar Status",
      status: "Active",
      statusClass: "badge-verified",
      value: "Native Swift AppKit (NSTouchBar) + Background IPC Stream",
    },
    {
      name: "Touch Bar Current Line",
      status: lyricsState?.currentLine ? "Live" : "Standby",
      statusClass: lyricsState?.currentLine ? "badge-verified" : "badge-empty",
      value: lyricsState?.currentLine?.text || "♪ TLyrics",
    },
    {
      name: "Touch Bar Next Line",
      status: lyricsState?.nextLine ? "Queued" : "None",
      statusClass: lyricsState?.nextLine ? "badge-verified" : "badge-empty",
      value: lyricsState?.nextLine ? `‣ ${lyricsState.nextLine.text}` : "None",
    },
    {
      name: "Background IPC File",
      status: "Streaming",
      statusClass: "badge-verified",
      value: "~/.cache/tlyrics/current_lyric.txt",
    },
    {
      name: "Lyrics Sync Mode",
      status: lyricsState ? lyricsState.lyricsType.toUpperCase() : "SEARCHING",
      statusClass: lyricsState?.lyricsType === "synced" ? "badge-verified" : "badge-empty",
      value: lyricsState ? `${lyricsState.lyricsType} (${lyricsState.lines.length} lines)` : "Searching LRCLIB...",
    },
    {
      name: "Playback Position",
      status: isDetected ? "Live" : "Idle",
      statusClass: isDetected ? "badge-verified" : "badge-empty",
      value: playbackState ? `${playbackState.positionSecs.toFixed(2)}s (${formatTime(playbackState.positionSecs)})` : "0.00s",
    },
  ];

  tbody.innerHTML = fields
    .map(
      (f) => `
    <tr>
      <td><strong>${f.name}</strong></td>
      <td><span class="${f.statusClass}">${f.status}</span></td>
      <td class="table-value">${f.value}</td>
    </tr>
  `
    )
    .join("");
}

function renderPlayback(state) {
  if (!state) return;
  lastPlaybackState = state;
  smoothPosition = state.positionSecs || 0;

  // Status Badge
  const statusBadge = document.getElementById("status-badge");
  const statusText = document.getElementById("status-text");
  statusBadge.className = `status-badge status-${state.status}`;
  if (state.status === "playing") {
    statusText.textContent = `Playing (${state.providerName})`;
  } else if (state.status === "paused") {
    statusText.textContent = "Paused";
  } else if (state.status === "stopped") {
    statusText.textContent = "Idle";
  } else {
    statusText.textContent = "Inactive";
  }

  // App & Source
  document.getElementById("provider-name").textContent = state.providerName || "Spotube";
  document.getElementById("bundle-id").textContent = state.bundleId ? `(${state.bundleId})` : "";

  // Track Info
  const songTitleEl = document.getElementById("song-title");
  const songArtistEl = document.getElementById("song-artist");
  const songAlbumEl = document.getElementById("song-album");
  const artworkImg = document.getElementById("artwork-img");
  const artworkPlaceholder = document.getElementById("artwork-placeholder");
  const iconPlay = document.getElementById("icon-play");
  const iconPause = document.getElementById("icon-pause");

  if (state.track) {
    songTitleEl.textContent = state.track.title || "Unknown Title";
    songArtistEl.textContent = state.track.artist || "Unknown Artist";
    if (state.track.album && state.track.album.trim().length > 0) {
      songAlbumEl.textContent = state.track.album;
    } else {
      songAlbumEl.textContent = "None (Not provided by Spotube)";
    }

    if (state.track.artworkDataUrl) {
      artworkImg.src = state.track.artworkDataUrl;
      artworkImg.classList.remove("hidden");
      artworkPlaceholder.classList.add("hidden");
    } else {
      artworkImg.classList.add("hidden");
      artworkPlaceholder.classList.remove("hidden");
    }

    document.getElementById("total-time").textContent = formatTime(state.track.durationSecs);
  } else {
    songTitleEl.textContent = state.is_detected ? `${state.providerName} connected (Idle)` : "Waiting for playback...";
    songArtistEl.textContent = "-";
    songAlbumEl.textContent = "None";
    artworkImg.classList.add("hidden");
    artworkPlaceholder.classList.remove("hidden");
    document.getElementById("total-time").textContent = "0:00";
  }

  // Play/Pause icon
  if (state.status === "playing") {
    iconPlay.classList.add("hidden");
    iconPause.classList.remove("hidden");
  } else {
    iconPlay.classList.remove("hidden");
    iconPause.classList.add("hidden");
  }

  updateProgress();
  updateVerificationTable(lastPlaybackState, lastLyricsState);
}

function renderLyrics(lyricsState) {
  if (!lyricsState) return;
  lastLyricsState = lyricsState;

  const badgeEl = document.getElementById("lyrics-badge");
  const nextLineEl = document.getElementById("lyrics-next-line");
  const listContainer = document.getElementById("lyrics-list");

  // Touch Bar mirror elements
  const tbCurrLyric = document.getElementById("tb-curr-lyric");
  const tbNextLyric = document.getElementById("tb-next-lyric");

  const currText = lyricsState.currentLine?.text || (lastPlaybackState?.track ? "♪ Instrumental / Intro" : "♪ TLyrics");
  const nextText = lyricsState.nextLine?.text ? `‣ ${lyricsState.nextLine.text}` : "";

  tbCurrLyric.textContent = currText;
  tbNextLyric.textContent = nextText;

  // Next Line preview in header
  if (lyricsState.nextLine) {
    nextLineEl.textContent = lyricsState.nextLine.text;
  } else {
    nextLineEl.textContent = lyricsState.hasLyrics ? "(End of track)" : "-";
  }

  // Badge Status
  badgeEl.className = "lyrics-badge";
  if (lyricsState.lyricsType === "synced") {
    badgeEl.classList.add("badge-synced");
    badgeEl.textContent = "Synced (LRCLIB)";
  } else if (lyricsState.lyricsType === "plain") {
    badgeEl.classList.add("badge-plain");
    badgeEl.textContent = "Plain Lyrics";
  } else if (lyricsState.lyricsType === "instrumental") {
    badgeEl.classList.add("badge-instrumental");
    badgeEl.textContent = "Instrumental";
  } else {
    badgeEl.classList.add("badge-searching");
    badgeEl.textContent = lyricsState.trackFingerprint ? "No Lyrics Found" : "Searching...";
  }

  // Re-render lines if track changed
  if (currentRenderedFingerprint !== lyricsState.trackFingerprint) {
    currentRenderedFingerprint = lyricsState.trackFingerprint;

    if (!lyricsState.hasLyrics || lyricsState.lines.length === 0) {
      if (lyricsState.lyricsType === "instrumental") {
        listContainer.innerHTML = `<div class="lyrics-empty">Instrumental track (No lyrics)</div>`;
      } else if (!lyricsState.trackFingerprint) {
        listContainer.innerHTML = `<div class="lyrics-empty">Waiting for active track playback...</div>`;
      } else {
        listContainer.innerHTML = `<div class="lyrics-empty">No lyrics available for this track on LRCLIB</div>`;
      }
      return;
    }

    listContainer.innerHTML = lyricsState.lines
      .map(
        (line, idx) => `
      <div class="lyric-line" id="lyric-line-${idx}" data-time="${line.timeSecs}">
        ${line.text || "♪"}
      </div>
    `
      )
      .join("");
  }

  // Highlight and scroll active line
  const activeIdx = lyricsState.currentLineIndex;
  const lineEls = listContainer.querySelectorAll(".lyric-line");

  lineEls.forEach((el, idx) => {
    el.classList.remove("active", "past", "future");
    if (activeIdx !== null && activeIdx !== undefined) {
      if (idx === activeIdx) {
        el.classList.add("active");
        el.scrollIntoView({ behavior: "smooth", block: "center" });
      } else if (idx < activeIdx) {
        el.classList.add("past");
      } else {
        el.classList.add("future");
      }
    } else {
      el.classList.add("future");
    }
  });

  updateVerificationTable(lastPlaybackState, lastLyricsState);
}

function updateProgress() {
  if (!lastPlaybackState || !lastPlaybackState.track || lastPlaybackState.track.durationSecs <= 0) {
    document.getElementById("current-time").textContent = "0:00";
    document.getElementById("progress-bar-fill").style.width = "0%";
    return;
  }

  const duration = lastPlaybackState.track.durationSecs;
  const pct = Math.min(100, Math.max(0, (smoothPosition / duration) * 100));
  document.getElementById("current-time").textContent = formatTime(smoothPosition);
  document.getElementById("progress-bar-fill").style.width = `${pct}%`;
}

async function fetchAll() {
  try {
    const pState = await invoke("get_playback_state");
    renderPlayback(pState);

    const lState = await invoke("get_synced_lyrics");
    renderLyrics(lState);
  } catch (err) {
    console.error("Failed to query state:", err);
  }
}

async function sendCommand(cmd) {
  try {
    await invoke("send_playback_command", { command: cmd });
    setTimeout(fetchAll, 150);
  } catch (err) {
    console.error(`Failed to send command ${cmd}:`, err);
  }
}

async function handleProviderChange(event) {
  const value = event.target.value;
  const providerId = value === "auto" ? null : value;
  try {
    await invoke("set_active_provider", { providerId });
    await fetchAll();
  } catch (err) {
    console.error("Failed to switch provider:", err);
  }
}

async function copyToClipboard(text, buttonEl, successMsg) {
  try {
    await navigator.clipboard.writeText(text);
    const orig = buttonEl.textContent;
    buttonEl.textContent = successMsg;
    buttonEl.style.backgroundColor = "var(--status-playing)";
    buttonEl.style.color = "#000";
    setTimeout(() => {
      buttonEl.textContent = orig;
      buttonEl.style.backgroundColor = "";
      buttonEl.style.color = "";
    }, 1500);
  } catch (e) {
    console.error("Copy failed:", e);
  }
}

window.addEventListener("DOMContentLoaded", async () => {
  // Main transport controls
  document.getElementById("btn-prev")?.addEventListener("click", () => sendCommand("prev"));
  document.getElementById("btn-play-pause")?.addEventListener("click", () => sendCommand("toggle"));
  document.getElementById("btn-next")?.addEventListener("click", () => sendCommand("next"));
  document.getElementById("btn-refresh")?.addEventListener("click", fetchAll);
  document.getElementById("provider-select")?.addEventListener("change", handleProviderChange);

  // Touch Bar strip mirror buttons
  document.getElementById("tb-btn-prev")?.addEventListener("click", () => sendCommand("prev"));
  document.getElementById("tb-btn-play")?.addEventListener("click", () => sendCommand("toggle"));
  document.getElementById("tb-btn-next")?.addEventListener("click", () => sendCommand("next"));

  // Fetch Touch Bar background snippets
  try {
    touchBarInfo = await invoke("get_touchbar_integration_info");
    document.getElementById("btn-copy-btt")?.addEventListener("click", (e) => {
      if (touchBarInfo?.bttScript) {
        copyToClipboard(touchBarInfo.bttScript, e.target, "Copied BTT Script!");
      }
    });
    document.getElementById("btn-copy-mtmr")?.addEventListener("click", (e) => {
      if (touchBarInfo?.mtmrJson) {
        copyToClipboard(touchBarInfo.mtmrJson, e.target, "Copied MTMR JSON!");
      }
    });
  } catch (e) {
    console.warn("Touch Bar info unavailable:", e);
  }

  // Subscribe to background push events if available
  if (eventModule && eventModule.listen) {
    try {
      await eventModule.listen("playback-state", (event) => {
        renderPlayback(event.payload);
      });
      await eventModule.listen("lyrics-state", (event) => {
        renderLyrics(event.payload);
      });
    } catch (e) {
      console.warn("Event listen not available:", e);
    }
  }

  // Initial fetch
  await fetchAll();

  // Smooth progress interpolation
  setInterval(() => {
    if (lastPlaybackState && lastPlaybackState.status === "playing" && lastPlaybackState.track) {
      smoothPosition = Math.min(lastPlaybackState.track.durationSecs, smoothPosition + 0.25);
      updateProgress();
    }
  }, 250);

  // Fallback periodic poll
  setInterval(fetchAll, 1000);
});
