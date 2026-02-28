const statusEl = document.getElementById("connection-status");
const lastEventEl = document.getElementById("last-event");
const feedEl = document.getElementById("event-feed");

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
  });

  ws.addEventListener("close", () => {
    setStatus("Disconnected - retrying", "state-closed");
    window.setTimeout(connect, 1500);
  });

  ws.addEventListener("error", () => {
    ws.close();
  });
}

connect();
