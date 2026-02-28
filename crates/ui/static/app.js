const statusEl = document.getElementById("connection-status");

const kpiBalanceEl = document.getElementById("kpi-balance");
const kpiTotalPnlEl = document.getElementById("kpi-total-pnl");
const kpiExecLatencyEl = document.getElementById("kpi-exec-latency");
const kpiWinRateEl = document.getElementById("kpi-win-rate");
const kpiBtcUsdEl = document.getElementById("kpi-btc-usd");

const settingsFormEl = document.getElementById("settings-form");
const settingsModeEl = document.getElementById("settings-execution-mode");
const settingsPausedEl = document.getElementById("settings-trading-paused");
const settingsLagEl = document.getElementById("settings-lag-threshold");
const settingsRiskEl = document.getElementById("settings-risk-per-trade");
const settingsDailyEl = document.getElementById("settings-daily-loss-cap");
const settingsMarketEl = document.getElementById("settings-market");
const settingsHorizonEl = document.getElementById("settings-horizon");
const settingsStatusEl = document.getElementById("settings-status");

const forecastCurrentEl = document.getElementById("forecast-current");
const forecastTargetEl = document.getElementById("forecast-target");
const forecastDeltaEl = document.getElementById("forecast-delta");
const forecastUpdatedEl = document.getElementById("forecast-updated");

const feedHealthEl = document.getElementById("feed-health");
const logsEl = document.getElementById("execution-logs");

const equityLatestEl = document.getElementById("equity-latest");
const equityChartEl = document.getElementById("equity-chart");

const fetchFeedHealthIntervalMs = 5000;
const fetchPortfolioIntervalMs = 3000;
const fetchPriceSnapshotIntervalMs = 4000;
const fetchSettingsIntervalMs = 10000;
const fetchStatsIntervalMs = 3000;
const fetchForecastIntervalMs = 3000;
const fetchLogsIntervalMs = 6000;
const maxChartPoints = 180;
const maxChatItems = 140;

let feedHealthPollInFlight = false;
let portfolioPollInFlight = false;
let priceSnapshotPollInFlight = false;
let settingsPollInFlight = false;
let settingsPatchInFlight = false;
let statsPollInFlight = false;
let forecastPollInFlight = false;
let logsPollInFlight = false;

let latestBtcUsd = null;

const equityPoints = [];
const seenLogKeys = new Set();

function setStatus(text, className) {
  if (!statusEl) {
    return;
  }
  statusEl.textContent = text;
  statusEl.className = `pill ${className}`;
}

function formatUsd(value) {
  if (!Number.isFinite(value)) {
    return "--";
  }
  return `$${value.toFixed(2)}`;
}

function formatSignedUsd(value) {
  if (!Number.isFinite(value)) {
    return "$0.00";
  }
  const sign = value >= 0 ? "+" : "-";
  return `${sign}$${Math.abs(value).toFixed(2)}`;
}

function formatPct(value) {
  if (!Number.isFinite(value)) {
    return "0.00%";
  }
  return `${value.toFixed(2)}%`;
}

function formatTs(ts) {
  if (!Number.isFinite(ts)) {
    return new Date().toLocaleTimeString();
  }
  if (ts > 1_000_000_000_000) {
    return new Date(ts).toLocaleTimeString();
  }
  return `t${Math.floor(ts)}`;
}

function asFiniteNumber(value) {
  if (typeof value !== "number" || !Number.isFinite(value)) {
    return null;
  }
  return value;
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
  context.fillStyle = "#f8fbff";
  context.fillRect(0, 0, width, height);

  context.strokeStyle = "#d8e5f2";
  context.lineWidth = 1;
  for (let i = 0; i < 4; i += 1) {
    const y = pad + (innerHeight / 3) * i;
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
}

function pushEquityPoint(value) {
  if (!Number.isFinite(value)) {
    return;
  }

  equityPoints.push(value);
  if (equityPoints.length > maxChartPoints) {
    equityPoints.shift();
  }
  renderEquityChart();
}

function updateSettingsStatus(text, isError) {
  if (!settingsStatusEl) {
    return;
  }
  settingsStatusEl.textContent = text;
  settingsStatusEl.classList.toggle("stale", Boolean(isError));
}

function updateSettingsForm(settings) {
  if (!settings || typeof settings !== "object") {
    return;
  }

  if (settingsModeEl && typeof settings.execution_mode === "string") {
    settingsModeEl.value = settings.execution_mode;
  }
  if (settingsPausedEl) {
    settingsPausedEl.checked = Boolean(settings.trading_paused);
  }
  if (settingsLagEl && Number.isFinite(settings.lag_threshold_pct)) {
    settingsLagEl.value = String(settings.lag_threshold_pct);
  }
  if (settingsRiskEl && Number.isFinite(settings.risk_per_trade_pct)) {
    settingsRiskEl.value = String(settings.risk_per_trade_pct);
  }
  if (settingsDailyEl && Number.isFinite(settings.daily_loss_cap_pct)) {
    settingsDailyEl.value = String(settings.daily_loss_cap_pct);
  }
  if (settingsMarketEl && typeof settings.market === "string") {
    settingsMarketEl.textContent = `Market: ${settings.market}`;
  }
  if (settingsHorizonEl && Number.isFinite(settings.forecast_horizon_minutes)) {
    settingsHorizonEl.textContent = `Forecast Horizon: ${Math.floor(settings.forecast_horizon_minutes)}m`;
  }

  updateSettingsStatus("Settings synced", false);
}

function collectSettingsPayload() {
  return {
    execution_mode: settingsModeEl ? settingsModeEl.value : "paper",
    trading_paused: settingsPausedEl ? settingsPausedEl.checked : false,
    lag_threshold_pct: settingsLagEl ? Number(settingsLagEl.value) : null,
    risk_per_trade_pct: settingsRiskEl ? Number(settingsRiskEl.value) : null,
    daily_loss_cap_pct: settingsDailyEl ? Number(settingsDailyEl.value) : null,
  };
}

async function applySettings(event) {
  event.preventDefault();

  if (settingsPatchInFlight) {
    return;
  }

  settingsPatchInFlight = true;
  updateSettingsStatus("Applying settings...", false);
  try {
    const response = await fetch("/settings", {
      method: "PATCH",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify(collectSettingsPayload()),
    });

    if (!response.ok) {
      let detail = "Failed to apply settings";
      try {
        const errPayload = await response.json();
        if (errPayload && typeof errPayload.error === "string") {
          detail = errPayload.error;
        }
      } catch {
      }
      updateSettingsStatus(detail, true);
      return;
    }

    const payload = await response.json();
    updateSettingsForm(payload);
    updateSettingsStatus("Settings applied", false);
  } catch {
    updateSettingsStatus("Network error while applying settings", true);
  } finally {
    settingsPatchInFlight = false;
  }
}

function updateStrategyStats(stats) {
  if (!stats || typeof stats !== "object") {
    return;
  }

  const balance = asFiniteNumber(stats.balance);
  const totalPnl = asFiniteNumber(stats.total_pnl);
  const execLatencyUs = asFiniteNumber(stats.exec_latency_us);
  const winRate = asFiniteNumber(stats.win_rate);
  const btcUsd = asFiniteNumber(stats.btc_usd);

  if (kpiBalanceEl) {
    kpiBalanceEl.textContent = formatUsd(balance);
  }
  if (kpiTotalPnlEl) {
    kpiTotalPnlEl.textContent = formatSignedUsd(totalPnl);
    kpiTotalPnlEl.style.color = totalPnl >= 0 ? "#0f8f54" : "#be382f";
  }
  if (kpiExecLatencyEl) {
    kpiExecLatencyEl.textContent = Number.isFinite(execLatencyUs)
      ? `${Math.round(execLatencyUs)} us`
      : "0 us";
  }
  if (kpiWinRateEl) {
    kpiWinRateEl.textContent = Number.isFinite(winRate) ? `${winRate.toFixed(1)}%` : "0.0%";
  }
  if (kpiBtcUsdEl && Number.isFinite(btcUsd)) {
    kpiBtcUsdEl.textContent = formatUsd(btcUsd);
    latestBtcUsd = btcUsd;
  }
  if (equityLatestEl && Number.isFinite(balance)) {
    equityLatestEl.textContent = `equity: ${balance.toFixed(2)}`;
  }

  pushEquityPoint(balance);
}

function updatePortfolioSummary(summary) {
  if (!summary || typeof summary !== "object") {
    return;
  }

  const equity = asFiniteNumber(summary.equity);
  if (equityLatestEl && Number.isFinite(equity)) {
    equityLatestEl.textContent = `equity: ${equity.toFixed(2)}`;
  }
  pushEquityPoint(equity);
}

function updateForecast(snapshot) {
  if (!snapshot || typeof snapshot !== "object") {
    return;
  }

  const current = asFiniteNumber(snapshot.current_btc_usd);
  const target = asFiniteNumber(snapshot.forecast_btc_usd);
  const deltaPct = asFiniteNumber(snapshot.delta_pct);
  const ts = asFiniteNumber(snapshot.ts);

  if (forecastCurrentEl) {
    forecastCurrentEl.textContent = formatUsd(current);
  }
  if (forecastTargetEl) {
    forecastTargetEl.textContent = formatUsd(target);
  }
  if (forecastDeltaEl) {
    forecastDeltaEl.textContent = formatPct(deltaPct);
    forecastDeltaEl.classList.remove("positive", "negative");
    if (Number.isFinite(deltaPct) && deltaPct > 0.0) {
      forecastDeltaEl.classList.add("positive");
    }
    if (Number.isFinite(deltaPct) && deltaPct < 0.0) {
      forecastDeltaEl.classList.add("negative");
    }
  }
  if (forecastUpdatedEl) {
    forecastUpdatedEl.textContent = `updated ${formatTs(ts)}`;
  }
}

function updatePriceSnapshot(snapshot) {
  const coinbase = asFiniteNumber(snapshot.coinbase_btc_usd);
  const binance = asFiniteNumber(snapshot.binance_btc_usdt);
  const kraken = asFiniteNumber(snapshot.kraken_btc_usd);

  const samples = [coinbase, binance, kraken].filter((value) => Number.isFinite(value));
  if (samples.length === 0) {
    return;
  }

  const mid = samples.sort((a, b) => a - b)[Math.floor(samples.length / 2)];
  latestBtcUsd = mid;
  if (kpiBtcUsdEl) {
    kpiBtcUsdEl.textContent = formatUsd(mid);
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

function logClassForEvent(eventName) {
  if (eventName === "paper_fill") {
    return "fill";
  }
  if (eventName === "paper_intent") {
    return "intent";
  }
  if (eventName === "risk_reject") {
    return "reject";
  }
  if (eventName === "settings_update") {
    return "settings";
  }
  if (eventName === "pause_state") {
    return "pause";
  }
  return "default";
}

function pushExecutionLog(entry) {
  if (!logsEl || !entry || typeof entry !== "object") {
    return;
  }

  const eventName = typeof entry.event === "string" ? entry.event : "event";
  const headline = typeof entry.headline === "string" ? entry.headline : "Update";
  const detail = typeof entry.detail === "string" ? entry.detail : "";
  const ts = asFiniteNumber(entry.ts);
  const key = `${eventName}|${headline}|${detail}|${ts}`;
  if (seenLogKeys.has(key)) {
    return;
  }
  seenLogKeys.add(key);

  const item = document.createElement("article");
  item.className = `chat-item ${logClassForEvent(eventName)}`;

  const head = document.createElement("div");
  head.className = "chat-head";
  const eventSpan = document.createElement("span");
  eventSpan.textContent = headline;
  const tsSpan = document.createElement("span");
  tsSpan.textContent = formatTs(ts);
  head.append(eventSpan, tsSpan);

  const body = document.createElement("p");
  body.className = "chat-body";
  body.textContent = detail;

  item.append(head, body);
  logsEl.prepend(item);

  while (logsEl.children.length > maxChatItems) {
    const last = logsEl.lastElementChild;
    if (!last) {
      break;
    }
    logsEl.removeChild(last);
  }
}

function routeTelemetry(rawEvent) {
  let parsed = null;
  try {
    parsed = JSON.parse(rawEvent);
  } catch {
    return;
  }

  if (!parsed || typeof parsed !== "object") {
    return;
  }

  const eventType = parsed.event_type;
  if (eventType === "feed_health") {
    updateFeedHealth(parsed);
    return;
  }
  if (eventType === "portfolio_snapshot") {
    updatePortfolioSummary(parsed);
    return;
  }
  if (eventType === "price_snapshot") {
    updatePriceSnapshot(parsed);
    return;
  }
  if (eventType === "strategy_stats") {
    updateStrategyStats(parsed);
    return;
  }
  if (eventType === "btc_forecast") {
    updateForecast(parsed);
    return;
  }
  if (eventType === "settings_updated") {
    updateSettingsForm(parsed);
    pushExecutionLog({
      ts: Date.now(),
      event: "settings_update",
      headline: "Settings Updated",
      detail: "Runtime controls changed",
    });
    return;
  }
  if (eventType === "execution_log") {
    pushExecutionLog(parsed);
    return;
  }

  if (eventType === "paper_intent" || eventType === "paper_fill" || eventType === "risk_reject") {
    pushExecutionLog({
      ts: Date.now(),
      event: eventType,
      headline: eventType.replace("_", " "),
      detail: JSON.stringify(parsed),
    });
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

async function fetchSettings() {
  if (settingsPollInFlight) {
    return;
  }
  settingsPollInFlight = true;
  try {
    const response = await fetch("/settings");
    if (!response.ok) {
      return;
    }
    const payload = await response.json();
    updateSettingsForm(payload);
  } catch {
  } finally {
    settingsPollInFlight = false;
  }
}

async function fetchFeedHealth() {
  if (feedHealthPollInFlight) {
    return;
  }
  feedHealthPollInFlight = true;
  try {
    const response = await fetch("/feed/health");
    if (!response.ok) {
      return;
    }
    const payload = await response.json();
    updateFeedHealth(payload);
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
    updatePortfolioSummary(payload);
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
    updatePriceSnapshot(payload);
  } catch {
  } finally {
    priceSnapshotPollInFlight = false;
  }
}

async function fetchStrategyStats() {
  if (statsPollInFlight) {
    return;
  }
  statsPollInFlight = true;
  try {
    const response = await fetch("/strategy/stats");
    if (!response.ok) {
      return;
    }
    const payload = await response.json();
    updateStrategyStats(payload);
  } catch {
  } finally {
    statsPollInFlight = false;
  }
}

async function fetchForecast() {
  if (forecastPollInFlight) {
    return;
  }
  forecastPollInFlight = true;
  try {
    const response = await fetch("/forecast/btc-15m");
    if (!response.ok) {
      return;
    }
    const payload = await response.json();
    updateForecast(payload);
  } catch {
  } finally {
    forecastPollInFlight = false;
  }
}

async function fetchExecutionLogs() {
  if (logsPollInFlight) {
    return;
  }
  logsPollInFlight = true;
  try {
    const response = await fetch("/logs/execution");
    if (!response.ok) {
      return;
    }
    const payload = await response.json();
    const logs = Array.isArray(payload.logs) ? payload.logs : [];
    for (const logEntry of logs) {
      pushExecutionLog(logEntry);
    }
  } catch {
  } finally {
    logsPollInFlight = false;
  }
}

if (settingsFormEl) {
  settingsFormEl.addEventListener("submit", applySettings);
}

fetchSettings();
fetchStrategyStats();
fetchForecast();
fetchFeedHealth();
fetchPortfolioSummary();
fetchPriceSnapshot();
fetchExecutionLogs();

window.setInterval(fetchSettings, fetchSettingsIntervalMs);
window.setInterval(fetchStrategyStats, fetchStatsIntervalMs);
window.setInterval(fetchForecast, fetchForecastIntervalMs);
window.setInterval(fetchFeedHealth, fetchFeedHealthIntervalMs);
window.setInterval(fetchPortfolioSummary, fetchPortfolioIntervalMs);
window.setInterval(fetchPriceSnapshot, fetchPriceSnapshotIntervalMs);
window.setInterval(fetchExecutionLogs, fetchLogsIntervalMs);

connect();
