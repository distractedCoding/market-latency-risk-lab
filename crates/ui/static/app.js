const statusEl = document.getElementById("connection-status");
const lastEventEl = document.getElementById("last-event");
const feedEl = document.getElementById("event-feed");
const feedHealthEl = document.getElementById("feed-health");

const paperFillsCountEl = document.getElementById("paper-fills-count");
const paperFillsLastEl = document.getElementById("paper-fills-last");

const moneyMadeEl = document.getElementById("money-made");
const openPositionEl = document.getElementById("open-position");
const totalFillsEl = document.getElementById("total-fills");
const equityLatestEl = document.getElementById("equity-latest");
const equityChartEl = document.getElementById("equity-chart");

const pricesUpdatedEl = document.getElementById("prices-updated");
const pricePolyMarketEl = document.getElementById("price-poly-market");
const priceCoinbaseEl = document.getElementById("price-coinbase");
const priceBinanceEl = document.getElementById("price-binance");
const priceKrakenEl = document.getElementById("price-kraken");
const pricePolyMidEl = document.getElementById("price-poly-mid");
const pricePolyBidEl = document.getElementById("price-poly-bid");
const pricePolyAskEl = document.getElementById("price-poly-ask");

const trendCoinbaseEl = document.getElementById("trend-coinbase");
const trendBinanceEl = document.getElementById("trend-binance");
const trendKrakenEl = document.getElementById("trend-kraken");
const trendPolyMidEl = document.getElementById("trend-poly-mid");

const fetchFeedHealthIntervalMs = 5000;
const fetchPortfolioIntervalMs = 3000;
const fetchPriceSnapshotIntervalMs = 3000;
const priceFreshnessCheckIntervalMs = 1500;
const stalePriceThresholdMs = 10000;
const maxEquityPoints = 180;

let paperFillCount = 0;
let feedHealthPollInFlight = false;
let portfolioPollInFlight = false;
let priceSnapshotPollInFlight = false;
let lastPriceUpdateAtMs = 0;
let lastPriceStatusLabel = "Waiting for snapshot...";
const equityPoints = [];
const lastDirectionalValues = new Map();

function setStatus(text, className) {
  if (!statusEl) {
    return;
  }
  statusEl.textContent = text;
  statusEl.className = `pill ${className}`;
}

function pushEvent(payload) {
  if (!feedEl) {
    return;
  }

  const item = document.createElement("li");
  item.textContent = payload;
  feedEl.prepend(item);

  while (feedEl.children.length > 24) {
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

function asFiniteNumber(value) {
  if (typeof value !== "number" || !Number.isFinite(value)) {
    return null;
  }
  return value;
}

function formatMoney(value) {
  if (!Number.isFinite(value)) {
    return "$0.00";
  }
  const sign = value >= 0 ? "+" : "-";
  return `${sign}$${Math.abs(value).toFixed(2)}`;
}

function formatFixed(value, digits) {
  if (!Number.isFinite(value)) {
    return "0.00";
  }
  return value.toFixed(digits);
}

function formatUsd(value) {
  if (!Number.isFinite(value)) {
    return "--";
  }
  return `$${value.toFixed(2)}`;
}

function formatProbability(value) {
  if (!Number.isFinite(value)) {
    return "--";
  }
  return `${value.toFixed(3)} (${(value * 100).toFixed(1)}%)`;
}

function setTrend(trendEl, direction) {
  if (!trendEl) {
    return;
  }
  trendEl.textContent = direction;
  trendEl.className = `trend-pill trend-${direction}`;
}

function setDirectionalValue(key, value, valueEl, trendEl, formatter, tolerance) {
  if (!valueEl || !trendEl) {
    return;
  }

  if (!Number.isFinite(value)) {
    valueEl.textContent = "--";
    valueEl.classList.remove("value-up", "value-down", "pulse-up", "pulse-down");
    setTrend(trendEl, "flat");
    return;
  }

  valueEl.textContent = formatter(value);

  const previous = lastDirectionalValues.get(key);
  let direction = "flat";
  if (Number.isFinite(previous)) {
    if (value > previous + tolerance) {
      direction = "up";
    } else if (value < previous - tolerance) {
      direction = "down";
    }
  }

  setTrend(trendEl, direction);
  valueEl.classList.remove("value-up", "value-down", "pulse-up", "pulse-down");
  if (direction === "up") {
    valueEl.classList.add("value-up", "pulse-up");
  }
  if (direction === "down") {
    valueEl.classList.add("value-down", "pulse-down");
  }
  if (direction !== "flat") {
    window.setTimeout(() => {
      valueEl.classList.remove("pulse-up", "pulse-down");
    }, 430);
  }

  lastDirectionalValues.set(key, value);
}

function updatePriceFreshnessLabel() {
  if (!pricesUpdatedEl) {
    return;
  }

  if (!lastPriceUpdateAtMs) {
    pricesUpdatedEl.textContent = "Waiting for snapshot...";
    pricesUpdatedEl.classList.add("stale");
    return;
  }

  const ageMs = Date.now() - lastPriceUpdateAtMs;
  if (ageMs > stalePriceThresholdMs) {
    const staleSeconds = Math.floor(ageMs / 1000);
    pricesUpdatedEl.textContent = `${lastPriceStatusLabel} | stale ${staleSeconds}s`;
    pricesUpdatedEl.classList.add("stale");
    return;
  }

  pricesUpdatedEl.textContent = lastPriceStatusLabel;
  pricesUpdatedEl.classList.remove("stale");
}

function updatePriceSnapshot(snapshot) {
  const coinbase = asFiniteNumber(snapshot.coinbase_btc_usd);
  const binance = asFiniteNumber(snapshot.binance_btc_usdt);
  const kraken = asFiniteNumber(snapshot.kraken_btc_usd);
  const polyBid = asFiniteNumber(snapshot.polymarket_yes_bid);
  const polyAsk = asFiniteNumber(snapshot.polymarket_yes_ask);
  const polyMid = asFiniteNumber(snapshot.polymarket_yes_mid);

  setDirectionalValue(
    "coinbase",
    coinbase,
    priceCoinbaseEl,
    trendCoinbaseEl,
    formatUsd,
    0.01,
  );
  setDirectionalValue(
    "binance",
    binance,
    priceBinanceEl,
    trendBinanceEl,
    formatUsd,
    0.01,
  );
  setDirectionalValue("kraken", kraken, priceKrakenEl, trendKrakenEl, formatUsd, 0.01);
  setDirectionalValue(
    "poly-mid",
    polyMid,
    pricePolyMidEl,
    trendPolyMidEl,
    formatProbability,
    0.0005,
  );

  if (pricePolyBidEl) {
    pricePolyBidEl.textContent = formatProbability(polyBid);
  }
  if (pricePolyAskEl) {
    pricePolyAskEl.textContent = formatProbability(polyAsk);
  }

  if (pricePolyMarketEl) {
    const market =
      typeof snapshot.polymarket_market_id === "string" && snapshot.polymarket_market_id
        ? snapshot.polymarket_market_id
        : "--";
    pricePolyMarketEl.textContent = `market: ${market}`;
  }

  const tick = Number.isFinite(snapshot.ts) ? Math.floor(snapshot.ts) : null;
  const now = new Date();
  if (tick === null) {
    lastPriceStatusLabel = `updated ${now.toLocaleTimeString()}`;
  } else {
    lastPriceStatusLabel = `tick ${tick} | ${now.toLocaleTimeString()}`;
  }

  lastPriceUpdateAtMs = Date.now();
  updatePriceFreshnessLabel();
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
  const qty = Number.isFinite(data.qty) ? data.qty : "?";
  const price = Number.isFinite(data.fill_px) ? data.fill_px.toFixed(3) : "?";
  const market = typeof data.market_id === "string" ? data.market_id : "?";
  paperFillsLastEl.textContent = `${side} ${qty} @ ${price} (${market})`;
}

function pushEquityPoint(value) {
  if (!Number.isFinite(value)) {
    return;
  }

  equityPoints.push(value);
  if (equityPoints.length > maxEquityPoints) {
    equityPoints.shift();
  }
  renderEquityChart();
}

function renderEquityChart() {
  if (!equityChartEl) {
    return;
  }

  const context = equityChartEl.getContext("2d");
  if (!context) {
    return;
  }

  const width = equityChartEl.width;
  const height = equityChartEl.height;
  const pad = 16;
  const innerWidth = width - pad * 2;
  const innerHeight = height - pad * 2;

  context.clearRect(0, 0, width, height);

  context.fillStyle = "#f7fbff";
  context.fillRect(0, 0, width, height);

  context.strokeStyle = "#dce7f4";
  context.lineWidth = 1;
  for (let row = 0; row < 4; row += 1) {
    const y = pad + (innerHeight / 3) * row;
    context.beginPath();
    context.moveTo(pad, y);
    context.lineTo(width - pad, y);
    context.stroke();
  }

  if (equityPoints.length < 2) {
    return;
  }

  const min = Math.min(...equityPoints);
  const max = Math.max(...equityPoints);
  const spread = Math.max(max - min, 0.0001);

  context.beginPath();
  for (let i = 0; i < equityPoints.length; i += 1) {
    const x = pad + (i / (equityPoints.length - 1)) * innerWidth;
    const normalized = (equityPoints[i] - min) / spread;
    const y = height - pad - normalized * innerHeight;
    if (i === 0) {
      context.moveTo(x, y);
    } else {
      context.lineTo(x, y);
    }
  }

  context.lineWidth = 2;
  context.strokeStyle = "#0b6ef9";
  context.stroke();

  const last = equityPoints[equityPoints.length - 1];
  const dotX = width - pad;
  const dotY = height - pad - ((last - min) / spread) * innerHeight;
  context.beginPath();
  context.arc(dotX, dotY, 3.5, 0, Math.PI * 2);
  context.fillStyle = "#0b6ef9";
  context.fill();
}

function updatePortfolioSummary(summary) {
  const pnl = Number(summary.pnl);
  const equity = Number(summary.equity);
  const positionQty = Number(summary.position_qty);
  const fills = Number(summary.fills);

  if (moneyMadeEl) {
    moneyMadeEl.textContent = formatMoney(pnl);
    moneyMadeEl.style.color = pnl >= 0 ? "#148f57" : "#c43227";
  }
  if (openPositionEl) {
    openPositionEl.textContent = formatFixed(positionQty, 2);
  }
  if (totalFillsEl && Number.isFinite(fills)) {
    totalFillsEl.textContent = String(Math.floor(fills));
  }
  if (paperFillsCountEl && Number.isFinite(fills)) {
    paperFillCount = Math.max(paperFillCount, Math.floor(fills));
    paperFillsCountEl.textContent = String(paperFillCount);
  }
  if (equityLatestEl) {
    equityLatestEl.textContent = `equity: ${formatFixed(equity, 2)}`;
  }

  pushEquityPoint(equity);
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
    return;
  }

  if (eventType === "portfolio_snapshot") {
    updatePortfolioSummary(parsed);
    return;
  }

  if (eventType === "price_snapshot") {
    updatePriceSnapshot(parsed);
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
    if (lastEventEl) {
      lastEventEl.textContent = event.data;
    }
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
  } catch {
  } finally {
    feedHealthPollInFlight = false;
  }
}

async function fetchPortfolioSummary() {
  if (portfolioPollInFlight) {
    return;
  }

  portfolioPollInFlight = true;
  try {
    const response = await fetch("/portfolio/summary");
    if (!response.ok) {
      return;
    }
    const payload = await response.json();
    if (payload && typeof payload === "object") {
      updatePortfolioSummary(payload);
    }
  } catch {
  } finally {
    portfolioPollInFlight = false;
  }
}

async function fetchPriceSnapshot() {
  if (priceSnapshotPollInFlight) {
    return;
  }

  priceSnapshotPollInFlight = true;
  try {
    const response = await fetch("/prices/snapshot");
    if (!response.ok) {
      return;
    }
    const payload = await response.json();
    if (payload && typeof payload === "object") {
      updatePriceSnapshot(payload);
    }
  } catch {
  } finally {
    priceSnapshotPollInFlight = false;
  }
}

fetchFeedHealth();
fetchPortfolioSummary();
fetchPriceSnapshot();
updatePriceFreshnessLabel();

window.setInterval(fetchFeedHealth, fetchFeedHealthIntervalMs);
window.setInterval(fetchPortfolioSummary, fetchPortfolioIntervalMs);
window.setInterval(fetchPriceSnapshot, fetchPriceSnapshotIntervalMs);
window.setInterval(updatePriceFreshnessLabel, priceFreshnessCheckIntervalMs);

connect();
