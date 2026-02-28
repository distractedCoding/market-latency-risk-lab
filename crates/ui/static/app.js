const statusEl = document.getElementById("connection-status");
const lastEventEl = document.getElementById("last-event");
const feedEl = document.getElementById("event-feed");
const feedHealthEl = document.getElementById("feed-health");
const paperFillsCountEl = document.getElementById("paper-fills-count");
const paperFillsLastEl = document.getElementById("paper-fills-last");
let paperFillCount = 0;
const fetchFeedHealthIntervalMs = 5000;
let feedHealthPollInFlight = false;

function setStatus(text, className) {
  statusEl.textContent = text;
  statusEl.className = `pill ${className}`;
}

function pushEvent(payload) {
  const item = document.createElement("li");
  item.textContent = payload;
  feedEl.prepend(item);

  while (feedEl.children.length > 20) {
    feedEl.removeChild(feedEl.lastElementChild);
  }
}

function maybeParseJson(raw) {
  try {
    return JSON.parse(raw);
  } catch {
    return null;
  }
}

function updateFeedHealth(data) {
  if (!feedHealthEl) {
    return;
  }

  const mode = typeof data.mode === "string" ? data.mode : "?";
  const sourceCounts = Array.isArray(data.source_counts) ? data.source_counts : [];
  const validSourceCounts = sourceCounts.filter((entry) => {
    return (
      entry &&
      typeof entry.source === "string" &&
      entry.source.length > 0 &&
      Number.isFinite(entry.count)
    );
  });

  const totalSources = validSourceCounts.length;
  const topSource = validSourceCounts.reduce((best, current) => {
    if (!best || current.count > best.count) {
      return current;
    }
    return best;
  }, null);

  if (topSource) {
    feedHealthEl.textContent = `mode: ${mode} | sources: ${totalSources} | top source: ${topSource.source} (${topSource.count})`;
    return;
  }

  feedHealthEl.textContent = `mode: ${mode} | sources: ${totalSources}`;
}

function updatePaperFills(data) {
  if (!paperFillsCountEl || !paperFillsLastEl) {
    return;
  }

  paperFillCount += 1;
  paperFillsCountEl.textContent = String(paperFillCount);

  const side = data.side || "?";
  const size = data.size ?? data.qty ?? "?";
  const price = data.fill_px ?? "?";
  paperFillsLastEl.textContent = `${side} ${size} @ ${price}`;
}

function routeTelemetry(rawEvent) {
  const parsed = maybeParseJson(rawEvent);
  if (!parsed || typeof parsed !== "object") {
    return;
  }

  const eventType = parsed.event_type;
  if (eventType === "feed_health") {
    updateFeedHealth(parsed);
    return;
  }

  if (eventType === "paper_fill") {
    updatePaperFills(parsed);
  }
}

function connect() {
  const protocol = window.location.protocol === "https:" ? "wss:" : "ws:";
  const socketUrl = `${protocol}//${window.location.host}/ws/events`;
  const ws = new WebSocket(socketUrl);

  setStatus("Connecting...", "state-pending");

  ws.addEventListener("open", () => {
    setStatus("Connected", "state-open");
  });

  ws.addEventListener("message", (event) => {
    lastEventEl.textContent = event.data;
    pushEvent(event.data);
    routeTelemetry(event.data);
  });

  ws.addEventListener("close", () => {
    setStatus("Disconnected - retrying", "state-closed");
    window.setTimeout(connect, 1500);
  });

  ws.addEventListener("error", () => {
    ws.close();
  });
}

async function fetchFeedHealth() {
  if (!feedHealthEl || feedHealthPollInFlight) {
    return;
  }

  feedHealthPollInFlight = true;
  try {
    const response = await fetch("/feed/health");
    if (!response.ok) {
      return;
    }

    const payload = await response.json();
    if (payload && typeof payload === "object") {
      updateFeedHealth(payload);
    }
  } catch {} finally {
    feedHealthPollInFlight = false;
  }
}

fetchFeedHealth();
window.setInterval(fetchFeedHealth, fetchFeedHealthIntervalMs);
connect();
