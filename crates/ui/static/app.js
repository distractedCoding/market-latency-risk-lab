const statusEl = document.getElementById("connection-status");
const lastEventEl = document.getElementById("last-event");
const feedEl = document.getElementById("event-feed");
const feedHealthEl = document.getElementById("feed-health");
const paperFillsCountEl = document.getElementById("paper-fills-count");
const paperFillsLastEl = document.getElementById("paper-fills-last");
let paperFillCount = 0;

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

  const status = data.status || data.health || data.state;
  const mode = data.mode || "?";
  const source = data.source || data.feed_source || "?";
  if (!status && mode === "?" && source === "?") {
    return;
  }

  const lagMs = Number.isFinite(data.lag_ms) ? ` (${data.lag_ms} ms lag)` : "";
  const statusLabel = status ? `${status}${lagMs}` : "unknown";
  feedHealthEl.textContent = `${statusLabel} | mode: ${mode} | source: ${source}`;
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
  if (!feedHealthEl) {
    return;
  }

  try {
    const response = await fetch("/feed/health");
    if (!response.ok) {
      return;
    }

    const payload = await response.json();
    if (payload && typeof payload === "object") {
      updateFeedHealth(payload);
    }
  } catch {}
}

fetchFeedHealth();
connect();
