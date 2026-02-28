pub fn module_ready() -> bool {
    true
}

pub fn index_html() -> &'static str {
    include_str!("../static/index.html")
}

pub fn styles_css() -> &'static str {
    include_str!("../static/styles.css")
}

pub fn app_js() -> &'static str {
    include_str!("../static/app.js")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ui_bundle_contains_index_html() {
        let html = index_html();

        assert!(html.contains("<!doctype html>"));
        assert!(html.contains("/static/styles.css"));
        assert!(html.contains("/static/app.js"));
    }

    #[test]
    fn ui_shell_contains_paper_live_panels() {
        let html = index_html();
        assert!(html.contains("Feed Health"));
        assert!(html.contains("Strategy Settings"));
        assert!(html.contains("BTC 15m Forecast"));
        assert!(html.contains("Equity Curve"));
        assert!(html.contains("Execution Logs"));
    }

    #[test]
    fn app_js_renders_feed_health_from_mode_and_source_counts() {
        let js = app_js();

        assert!(js.contains("source_counts"));
        assert!(js.contains("top source"));
    }

    #[test]
    fn app_js_renders_portfolio_summary_and_chart() {
        let js = app_js();

        assert!(js.contains("/portfolio/summary"));
        assert!(js.contains("equityPoints"));
        assert!(js.contains("renderEquityChart"));
    }

    #[test]
    fn app_js_polls_feed_health_periodically() {
        let js = app_js();

        assert!(js.contains("window.setInterval"));
        assert!(js.contains("fetchFeedHealthIntervalMs"));
    }

    #[test]
    fn app_js_routes_price_snapshot_and_polls_endpoint() {
        let js = app_js();

        assert!(js.contains("price_snapshot"));
        assert!(js.contains("/prices/snapshot"));
    }

    #[test]
    fn ui_shell_contains_top_kpis_requested_by_user() {
        let html = index_html();

        assert!(html.contains("Balance"));
        assert!(html.contains("Total P&amp;L"));
        assert!(html.contains("Exec Latency"));
        assert!(html.contains("Win Rate"));
        assert!(html.contains("BTC/USD"));
    }

    #[test]
    fn ui_shell_contains_settings_dashboard_and_chat_logs_regions() {
        let html = index_html();

        assert!(html.contains("Strategy Settings"));
        assert!(html.contains("BTC 15m Forecast"));
        assert!(html.contains("Execution Logs"));
    }

    #[test]
    fn app_js_patches_settings_and_routes_new_telemetry() {
        let js = app_js();

        assert!(js.contains("/settings"));
        assert!(js.contains("PATCH"));
        assert!(js.contains("/strategy/stats"));
        assert!(js.contains("/forecast/btc-15m"));
        assert!(js.contains("settings_updated"));
        assert!(js.contains("strategy_stats"));
        assert!(js.contains("btc_forecast"));
        assert!(js.contains("execution_log"));
    }
}
