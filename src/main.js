const { invoke } = window.__TAURI__.core;
const eventModule = window.__TAURI__.event;

// Runtime state cache
let lastPlaybackState = null;
let lastLyricsState = null;
let lastRenderedFingerprint = null;
let lastRenderedIndex = null;
let lastArtworkUrl = null;
let smoothPosition = 0;
let isFullView = false;
let currentOffsetMs = 0;
let touchBarInfo = null;

// Helpers
function formatTime(seconds) {
  if (isNaN(seconds) || seconds < 0) return "0:00";
  const mins = Math.floor(seconds / 60);
  const secs = Math.floor(seconds % 60);
  return `${mins}:${secs.toString().padStart(2, "0")}`;
}

// 1. Playback UI update (optimized: minimal DOM modifications)
function renderPlayback(state) {
  if (!state) return;
  lastPlaybackState = state;
  smoothPosition = state.positionSecs || 0;

  // Status Indicator Dot
  const statusDot = document.getElementById("status-dot");
  if (statusDot) {
    statusDot.className = `status-dot status-${state.status}`;
    statusDot.title = `Status: ${state.status.toUpperCase()} (${state.providerName || "Idle"})`;
  }

  // Source App Badge
  const sourceBadge = document.getElementById("source-app-badge");
  if (sourceBadge) {
    const appLabel = state.appName || state.providerName || "Spotube";
    if (sourceBadge.textContent !== appLabel) {
      sourceBadge.textContent = appLabel;
    }
  }

  // Track Meta Elements
  const titleEl = document.getElementById("track-title");
  const artistEl = document.getElementById("track-artist");
  const artworkImg = document.getElementById("artwork-img");
  const artworkPlaceholder = document.getElementById("artwork-placeholder");

  if (state.track) {
    const titleText = state.track.title || "Unknown Title";
    const artistText = state.track.artist || "Unknown Artist";

    if (titleEl && titleEl.textContent !== titleText) {
      titleEl.textContent = titleText;
      titleEl.title = titleText;
    }
    if (artistEl && artistEl.textContent !== artistText) {
      artistEl.textContent = artistText;
    }

    // Lazy load artwork: only touch DOM when data URL actually changes
    const newArtwork = state.track.artworkDataUrl || null;
    if (newArtwork !== lastArtworkUrl) {
      lastArtworkUrl = newArtwork;
      if (newArtwork && artworkImg) {
        artworkImg.src = newArtwork;
        artworkImg.classList.remove("hidden");
        artworkPlaceholder?.classList.add("hidden");
      } else {
        artworkImg?.classList.add("hidden");
        artworkPlaceholder?.classList.remove("hidden");
      }
    }
  } else {
    const emptyTitle = state.is_detected ? `${state.providerName} (Idle)` : "No Track Playing";
    if (titleEl && titleEl.textContent !== emptyTitle) {
      titleEl.textContent = emptyTitle;
    }
    if (artistEl && artistEl.textContent !== "Connect music player to begin") {
      artistEl.textContent = "Connect music player to begin";
    }
    if (lastArtworkUrl !== null) {
      lastArtworkUrl = null;
      artworkImg?.classList.add("hidden");
      artworkPlaceholder?.classList.remove("hidden");
    }
  }

  // Transport Play/Pause Icon
  const iconPlay = document.getElementById("icon-play");
  const iconPause = document.getElementById("icon-pause");
  if (iconPlay && iconPause) {
    if (state.status === "playing") {
      iconPlay.classList.add("hidden");
      iconPause.classList.remove("hidden");
    } else {
      iconPlay.classList.remove("hidden");
      iconPause.classList.add("hidden");
    }
  }

  updateProgress();
}

// 2. Lyrics UI update (optimized: update only when active line changes)
function renderLyrics(lyricsState) {
  if (!lyricsState) return;
  lastLyricsState = lyricsState;

  const focusView = document.getElementById("lyrics-focus-view");
  const fullView = document.getElementById("lyrics-full-view");
  const emptyState = document.getElementById("lyrics-empty");
  const loadingState = document.getElementById("lyrics-loading");
  const fallbackState = document.getElementById("lyrics-fallback");
  const syncBadge = document.getElementById("sync-status");

  // Sync Status Badge
  if (syncBadge) {
    if (lyricsState.lyricsType === "synced") {
      syncBadge.className = "sync-status-badge status-synced";
      syncBadge.textContent = "Synced · LRCLIB";
    } else if (lyricsState.lyricsType === "instrumental") {
      syncBadge.className = "sync-status-badge status-instrumental";
      syncBadge.textContent = "Instrumental";
    } else if (lyricsState.lyricsType === "plain") {
      syncBadge.className = "sync-status-badge";
      syncBadge.textContent = "Plain Lyrics";
    } else {
      syncBadge.className = "sync-status-badge";
      syncBadge.textContent = lyricsState.trackFingerprint ? "No Lyrics Found" : "Idle";
    }
  }

  // 1. Handle State Overlays
  if (!lastPlaybackState?.track) {
    emptyState?.classList.remove("hidden");
    loadingState?.classList.add("hidden");
    fallbackState?.classList.add("hidden");
    focusView?.classList.add("hidden");
    fullView?.classList.add("hidden");
    return;
  }

  if (lyricsState.lyricsType === "instrumental") {
    emptyState?.classList.add("hidden");
    loadingState?.classList.add("hidden");
    fallbackState?.classList.remove("hidden");
    document.getElementById("fallback-title").textContent = "Instrumental Track";
    document.getElementById("fallback-desc").textContent = "No vocal lyrics recorded for this song.";
    focusView?.classList.add("hidden");
    fullView?.classList.add("hidden");
    return;
  }

  if (!lyricsState.hasLyrics) {
    emptyState?.classList.add("hidden");
    if (lyricsState.trackFingerprint) {
      fallbackState?.classList.remove("hidden");
      document.getElementById("fallback-title").textContent = "No Lyrics Available";
      document.getElementById("fallback-desc").textContent = "LRCLIB does not have synchronized lyrics for this track.";
      loadingState?.classList.add("hidden");
    } else {
      loadingState?.classList.remove("hidden");
      fallbackState?.classList.add("hidden");
    }
    focusView?.classList.add("hidden");
    fullView?.classList.add("hidden");
    return;
  }

  // Track has lyrics -> Hide all overlays
  emptyState?.classList.add("hidden");
  loadingState?.classList.add("hidden");
  fallbackState?.classList.add("hidden");

  // Show active view
  if (isFullView) {
    focusView?.classList.add("hidden");
    fullView?.classList.remove("hidden");
  } else {
    focusView?.classList.remove("hidden");
    fullView?.classList.add("hidden");
  }

  // 2. Track Change: Rebuild full scroll list once
  const isNewTrack = lastRenderedFingerprint !== lyricsState.trackFingerprint;
  if (isNewTrack) {
    lastRenderedFingerprint = lyricsState.trackFingerprint;
    lastRenderedIndex = null;

    const fullList = document.getElementById("lyrics-full-list");
    if (fullList) {
      fullList.innerHTML = lyricsState.lines
        .map(
          (line, idx) => `
        <div class="full-lyric-line" id="full-line-${idx}" data-time="${line.timeSecs}">
          ${line.text || "♪"}
        </div>
      `
        )
        .join("");
    }
  }

  // 3. Skip re-render if active line has not changed!
  const currIdx = lyricsState.currentLineIndex;
  if (!isNewTrack && currIdx === lastRenderedIndex) {
    return;
  }
  lastRenderedIndex = currIdx;

  // 4. Update 3-Line Focus View
  const prevEl = document.getElementById("lyric-prev");
  const currEl = document.getElementById("lyric-curr");
  const nextEl = document.getElementById("lyric-next");

  const lines = lyricsState.lines || [];
  let prevText = "";
  let currText = "♪ TLyrics";
  let nextText = "";

  if (currIdx !== null && currIdx !== undefined) {
    if (currIdx > 0 && lines[currIdx - 1]) {
      prevText = lines[currIdx - 1].text || "";
    }
    if (lines[currIdx]) {
      currText = lines[currIdx].text || "♪";
    }
    if (currIdx + 1 < lines.length && lines[currIdx + 1]) {
      nextText = lines[currIdx + 1].text || "";
    }
  } else if (lines.length > 0) {
    // Waiting for first line
    currText = "♪ (Intro)";
    nextText = lines[0]?.text || "";
  }

  if (prevEl && prevEl.textContent !== prevText) prevEl.textContent = prevText;
  if (currEl && currEl.textContent !== currText) currEl.textContent = currText;
  if (nextEl && nextEl.textContent !== nextText) nextEl.textContent = nextText;

  // 5. Update Full Scroll View highlight & scroll
  if (isFullView) {
    const fullLines = document.querySelectorAll(".full-lyric-line");
    fullLines.forEach((el, idx) => {
      el.classList.remove("active", "past");
      if (currIdx !== null && currIdx !== undefined) {
        if (idx === currIdx) {
          el.classList.add("active");
          el.scrollIntoView({ behavior: "smooth", block: "center" });
        } else if (idx < currIdx) {
          el.classList.add("past");
        }
      }
    });
  }
}

// 3. Smooth Progress Bar
function updateProgress() {
  const duration = lastPlaybackState?.track?.durationSecs || 0;
  const timeDisplay = document.getElementById("time-display");
  const progressFill = document.getElementById("progress-fill");

  if (!duration || duration <= 0) {
    if (timeDisplay) timeDisplay.textContent = "0:00 / 0:00";
    if (progressFill) progressFill.style.width = "0%";
    return;
  }

  const pct = Math.min(100, Math.max(0, (smoothPosition / duration) * 100));
  if (timeDisplay) {
    timeDisplay.textContent = `${formatTime(smoothPosition)} / ${formatTime(duration)}`;
  }
  if (progressFill) {
    progressFill.style.width = `${pct}%`;
  }
}

// Backend Command Dispatches
async function fetchAll() {
  try {
    const pState = await invoke("get_playback_state");
    renderPlayback(pState);

    const lState = await invoke("get_synced_lyrics");
    renderLyrics(lState);
  } catch (err) {
    console.warn("Polling error:", err);
  }
}

async function sendPlaybackCommand(cmd) {
  try {
    await invoke("send_playback_command", { command: cmd });
    setTimeout(fetchAll, 120);
  } catch (err) {
    console.error(`Command ${cmd} failed:`, err);
  }
}

// Settings Actions
async function updateLyricsOffset(deltaMs, isReset = false) {
  if (isReset) {
    currentOffsetMs = 0;
  } else {
    currentOffsetMs = Math.max(-5000, Math.min(5000, currentOffsetMs + deltaMs));
  }

  const displayEl = document.getElementById("offset-val");
  if (displayEl) {
    displayEl.textContent = `${currentOffsetMs >= 0 ? "+" : ""}${currentOffsetMs} ms`;
  }

  try {
    await invoke("set_lyrics_offset", { offsetMs: currentOffsetMs });
    localStorage.setItem("tlyrics_offset_ms", currentOffsetMs.toString());
  } catch (e) {
    console.warn("Failed to set lyrics offset in backend:", e);
  }
}

function updateFontSize(sizePx) {
  document.documentElement.style.setProperty("--lyric-font-size", `${sizePx}px`);
  const fontVal = document.getElementById("font-size-val");
  if (fontVal) fontVal.textContent = `${sizePx}px`;
  localStorage.setItem("tlyrics_font_size", sizePx.toString());
}

async function clearCache() {
  const btn = document.getElementById("btn-clear-cache");
  try {
    await invoke("clear_lyrics_cache");
    if (btn) {
      const orig = btn.textContent;
      btn.textContent = "Cache Cleared!";
      btn.style.backgroundColor = "rgba(16, 185, 129, 0.2)";
      btn.style.color = "#10b981";
      setTimeout(() => {
        btn.textContent = orig;
        btn.style.backgroundColor = "";
        btn.style.color = "";
      }, 1500);
    }
    lastRenderedFingerprint = null;
    await fetchAll();
  } catch (e) {
    console.error("Failed to clear cache:", e);
  }
}

async function copyText(text, btnEl, successText) {
  try {
    await navigator.clipboard.writeText(text);
    const orig = btnEl.textContent;
    btnEl.textContent = successText;
    btnEl.style.color = "#38bdf8";
    setTimeout(() => {
      btnEl.textContent = orig;
      btnEl.style.color = "";
    }, 1500);
  } catch (e) {
    console.error("Clipboard copy failed:", e);
  }
}

// Initialization & Event Binding
window.addEventListener("DOMContentLoaded", async () => {
  // Load saved preferences
  const savedFontSize = localStorage.getItem("tlyrics_font_size") || "24";
  const fontSlider = document.getElementById("setting-font-size");
  if (fontSlider) {
    fontSlider.value = savedFontSize;
    updateFontSize(parseInt(savedFontSize, 10));
  }

  const savedOffset = parseInt(localStorage.getItem("tlyrics_offset_ms") || "0", 10);
  if (!isNaN(savedOffset)) {
    currentOffsetMs = savedOffset;
    await updateLyricsOffset(0);
  }

  // Playback Control Buttons
  document.getElementById("btn-prev")?.addEventListener("click", () => sendPlaybackCommand("prev"));
  document.getElementById("btn-play-pause")?.addEventListener("click", () => sendPlaybackCommand("toggle"));
  document.getElementById("btn-next")?.addEventListener("click", () => sendPlaybackCommand("next"));

  // Toggle 3-Line vs Full View
  const btnToggleView = document.getElementById("btn-toggle-view");
  btnToggleView?.addEventListener("click", () => {
    isFullView = !isFullView;
    btnToggleView.classList.toggle("active", isFullView);
    if (lastLyricsState) {
      renderLyrics(lastLyricsState);
    }
  });

  // Settings Slide-over Toggle
  const settingsPanel = document.getElementById("settings-panel");
  const btnToggleSettings = document.getElementById("btn-toggle-settings");
  const btnCloseSettings = document.getElementById("btn-close-settings");

  btnToggleSettings?.addEventListener("click", () => {
    settingsPanel?.classList.toggle("hidden");
  });
  btnCloseSettings?.addEventListener("click", () => {
    settingsPanel?.classList.add("hidden");
  });

  // Provider selector
  const providerSelect = document.getElementById("setting-provider");
  providerSelect?.addEventListener("change", async (e) => {
    const val = e.target.value;
    const providerId = val === "auto" ? null : val;
    try {
      await invoke("set_active_provider", { providerId });
      await fetchAll();
    } catch (err) {
      console.error("Failed to switch provider:", err);
    }
  });

  // Font Size Slider
  fontSlider?.addEventListener("input", (e) => {
    updateFontSize(e.target.value);
  });

  // Offset Steppers
  document.getElementById("btn-offset-minus")?.addEventListener("click", () => updateLyricsOffset(-50));
  document.getElementById("btn-offset-plus")?.addEventListener("click", () => updateLyricsOffset(50));
  document.getElementById("btn-offset-reset")?.addEventListener("click", () => updateLyricsOffset(0, true));

  // Clear Cache Button
  document.getElementById("btn-clear-cache")?.addEventListener("click", clearCache);

  // Global Touch Bar Actions
  const tbBadge = document.getElementById("tb-badge");
  document.getElementById("btn-present-tb")?.addEventListener("click", async () => {
    try {
      await invoke("present_global_touchbar");
      if (tbBadge) {
        tbBadge.textContent = "Presented";
        tbBadge.style.color = "var(--status-playing)";
      }
    } catch (e) {
      console.error(e);
    }
  });

  document.getElementById("btn-dismiss-tb")?.addEventListener("click", async () => {
    try {
      await invoke("dismiss_global_touchbar");
      if (tbBadge) {
        tbBadge.textContent = "Dismissed";
        tbBadge.style.color = "var(--text-muted)";
      }
    } catch (e) {
      console.error(e);
    }
  });

  // Touch Bar Toggle Switch
  document.getElementById("setting-touchbar")?.addEventListener("change", async (e) => {
    if (e.target.checked) {
      await invoke("present_global_touchbar").catch(console.error);
      if (tbBadge) tbBadge.textContent = "Active";
    } else {
      await invoke("dismiss_global_touchbar").catch(console.error);
      if (tbBadge) tbBadge.textContent = "Disabled";
    }
    syncAppSettings();
  });

  // App Settings Synchronization
  let currentAppSettings = {
    launchAtLogin: false,
    startHidden: false,
    backgroundMode: true,
    touchbarEnabled: true,
    lastWindowVisible: true,
  };

  async function loadAppSettings() {
    try {
      const res = await invoke("get_app_settings");
      if (res) {
        currentAppSettings = res;
        const autostartEl = document.getElementById("setting-autostart");
        const startHiddenEl = document.getElementById("setting-start-hidden");
        const bgModeEl = document.getElementById("setting-bg-mode");
        const touchbarEl = document.getElementById("setting-touchbar");

        if (autostartEl) autostartEl.checked = !!res.launchAtLogin;
        if (startHiddenEl) startHiddenEl.checked = !!res.startHidden;
        if (bgModeEl) bgModeEl.checked = !!res.backgroundMode;
        if (touchbarEl) touchbarEl.checked = !!res.touchbarEnabled;
      }
    } catch (e) {
      console.warn("Failed to load app settings:", e);
    }
  }

  async function syncAppSettings() {
    try {
      const autostartEl = document.getElementById("setting-autostart");
      const startHiddenEl = document.getElementById("setting-start-hidden");
      const bgModeEl = document.getElementById("setting-bg-mode");
      const touchbarEl = document.getElementById("setting-touchbar");

      currentAppSettings.launchAtLogin = autostartEl ? autostartEl.checked : false;
      currentAppSettings.startHidden = startHiddenEl ? startHiddenEl.checked : false;
      currentAppSettings.backgroundMode = bgModeEl ? bgModeEl.checked : true;
      currentAppSettings.touchbarEnabled = touchbarEl ? touchbarEl.checked : true;

      await invoke("save_app_settings", { newSettings: currentAppSettings });
    } catch (e) {
      console.warn("Failed to save app settings:", e);
    }
  }

  document.getElementById("setting-autostart")?.addEventListener("change", syncAppSettings);
  document.getElementById("setting-start-hidden")?.addEventListener("change", syncAppSettings);
  document.getElementById("setting-bg-mode")?.addEventListener("change", syncAppSettings);

  await loadAppSettings();

  // Fetch Touch Bar background snippets
  try {
    touchBarInfo = await invoke("get_touchbar_integration_info");
    document.getElementById("btn-copy-btt")?.addEventListener("click", (e) => {
      if (touchBarInfo?.bttScript) {
        copyText(touchBarInfo.bttScript, e.target, "Copied BTT Script!");
      }
    });
    document.getElementById("btn-copy-mtmr")?.addEventListener("click", (e) => {
      if (touchBarInfo?.mtmrJson) {
        copyText(touchBarInfo.mtmrJson, e.target, "Copied MTMR JSON!");
      }
    });
  } catch (e) {
    console.warn("Touch Bar info unavailable:", e);
  }

  // Keyboard Shortcuts
  window.addEventListener("keydown", (e) => {
    if (e.target.tagName === "INPUT" || e.target.tagName === "SELECT") return;

    if (e.code === "Space") {
      e.preventDefault();
      sendPlaybackCommand("toggle");
    } else if (e.code === "ArrowRight" && (e.metaKey || e.ctrlKey)) {
      e.preventDefault();
      sendPlaybackCommand("next");
    } else if (e.code === "ArrowLeft" && (e.metaKey || e.ctrlKey)) {
      e.preventDefault();
      sendPlaybackCommand("prev");
    } else if (e.code === "Escape") {
      settingsPanel?.classList.add("hidden");
    }
  });

  // Listen to background push events
  if (eventModule && eventModule.listen) {
    try {
      await eventModule.listen("playback-state", (event) => {
        renderPlayback(event.payload);
      });
      await eventModule.listen("lyrics-state", (event) => {
        renderLyrics(event.payload);
      });
      await eventModule.listen("open-settings", () => {
        settingsPanel?.classList.remove("hidden");
      });
    } catch (e) {
      console.warn("Event listen not available:", e);
    }
  }

  // Initial fetch
  await fetchAll();

  // Smooth progress interpolation (250ms tick, runs only when playing)
  setInterval(() => {
    if (lastPlaybackState && lastPlaybackState.status === "playing" && lastPlaybackState.track) {
      smoothPosition = Math.min(lastPlaybackState.track.durationSecs, smoothPosition + 0.25);
      updateProgress();
    }
  }, 250);

  // Fallback periodic sync (every 1.5s) to guarantee UI consistency
  setInterval(fetchAll, 1500);
});
