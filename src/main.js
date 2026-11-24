const { invoke } = window.__TAURI__.core;
const eventModule = window.__TAURI__.event;

let lastState = null;
let smoothPosition = 0;

function formatTime(seconds) {
  if (isNaN(seconds) || seconds < 0) return "0:00";
  const mins = Math.floor(seconds / 60);
  const secs = Math.floor(seconds % 60);
  return `${mins}:${secs.toString().padStart(2, "0")}`;
}

function updateVerificationTable(state) {
  const tbody = document.getElementById("verification-body");
  if (!tbody) return;

  const track = state.track;
  const isDetected = state.is_detected;

  const fields = [
    {
      name: "Resolved Provider ID",
      status: "Active",
      statusClass: "badge-verified",
      value: `${state.providerId} (${state.providerName})`,
    },
    {
      name: "Source Application",
      status: isDetected ? "Connected" : "Idle",
      statusClass: isDetected ? "badge-verified" : "badge-empty",
      value: `${state.appName || "-"} [${state.bundleId || "-"}]`,
    },
    {
      name: "Song Title",
      status: track?.title ? "Resolved" : "None",
      statusClass: track?.title ? "badge-verified" : "badge-empty",
      value: track?.title || "No track title detected",
    },
    {
      name: "Artist",
      status: track?.artist ? "Resolved" : "None",
      statusClass: track?.artist ? "badge-verified" : "badge-empty",
      value: track?.artist || "No artist detected",
    },
    {
      name: "Album",
      status: track?.album ? "Resolved" : "Verified Empty",
      statusClass: track?.album ? "badge-verified" : "badge-empty",
      value: track?.album ? track.album : "None (Spotube omitted album metadata - not fabricated)",
    },
    {
      name: "Track Fingerprint",
      status: track ? "Computed" : "None",
      statusClass: track ? "badge-verified" : "badge-empty",
      value: track ? `${track.title.trim().toLowerCase()}::${track.artist.trim().toLowerCase()}::${track.durationSecs.toFixed(1)}` : "None",
    },
    {
      name: "Artwork",
      status: track?.artworkDataUrl ? "Verified" : "Not Provided",
      statusClass: track?.artworkDataUrl ? "badge-verified" : "badge-empty",
      value: track?.artworkDataUrl ? `JPEG Base64 (${Math.round(track.artworkDataUrl.length / 1024)} KB)` : "None",
    },
    {
      name: "Duration",
      status: track?.durationSecs > 0 ? "Verified" : "Waiting",
      statusClass: track?.durationSecs > 0 ? "badge-verified" : "badge-empty",
      value: track?.durationSecs > 0 ? `${track.durationSecs.toFixed(2)}s (${formatTime(track.durationSecs)})` : "0.00s",
    },
    {
      name: "Playback Position",
      status: isDetected ? "Live" : "Idle",
      statusClass: isDetected ? "badge-verified" : "badge-empty",
      value: `${state.positionSecs.toFixed(2)}s (${formatTime(state.positionSecs)})`,
    },
    {
      name: "Playback Status",
      status: state.status.toUpperCase(),
      statusClass: state.status === "playing" ? "badge-verified" : "badge-empty",
      value: state.status.toUpperCase(),
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

function renderPlaybackState(state) {
  if (!state) return;
  lastState = state;
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
  updateVerificationTable(state);
}

function updateProgress() {
  if (!lastState || !lastState.track || lastState.track.durationSecs <= 0) {
    document.getElementById("current-time").textContent = "0:00";
    document.getElementById("progress-bar-fill").style.width = "0%";
    return;
  }

  const duration = lastState.track.durationSecs;
  const pct = Math.min(100, Math.max(0, (smoothPosition / duration) * 100));
  document.getElementById("current-time").textContent = formatTime(smoothPosition);
  document.getElementById("progress-bar-fill").style.width = `${pct}%`;
}

async function fetchState() {
  try {
    const state = await invoke("get_playback_state");
    renderPlaybackState(state);
  } catch (err) {
    console.error("Failed to query playback state:", err);
  }
}

async function sendCommand(cmd) {
  try {
    await invoke("send_playback_command", { command: cmd });
    setTimeout(fetchState, 150);
  } catch (err) {
    console.error(`Failed to send command ${cmd}:`, err);
  }
}

async function handleProviderChange(event) {
  const value = event.target.value;
  const providerId = value === "auto" ? null : value;
  try {
    await invoke("set_active_provider", { providerId });
    await fetchState();
  } catch (err) {
    console.error("Failed to switch provider:", err);
  }
}

window.addEventListener("DOMContentLoaded", async () => {
  document.getElementById("btn-prev")?.addEventListener("click", () => sendCommand("prev"));
  document.getElementById("btn-play-pause")?.addEventListener("click", () => sendCommand("toggle"));
  document.getElementById("btn-next")?.addEventListener("click", () => sendCommand("next"));
  document.getElementById("btn-refresh")?.addEventListener("click", fetchState);
  document.getElementById("provider-select")?.addEventListener("change", handleProviderChange);

  // Subscribe to background push events if available
  if (eventModule && eventModule.listen) {
    try {
      await eventModule.listen("playback-state", (event) => {
        renderPlaybackState(event.payload);
      });
    } catch (e) {
      console.warn("Event listen not available:", e);
    }
  }

  // Immediate query
  await fetchState();

  // Smooth progress interpolation
  setInterval(() => {
    if (lastState && lastState.status === "playing" && lastState.track) {
      smoothPosition = Math.min(lastState.track.durationSecs, smoothPosition + 0.25);
      updateProgress();
    }
  }, 250);

  // Periodic fallback poll
  setInterval(fetchState, 1000);
});
