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
        assert!(html.contains("Paper Fills"));
        assert!(html.contains("Money Made"));
        assert!(html.contains("Equity Curve"));
    }

    #[test]
    fn ui_shell_contains_live_prices_panel() {
        let html = index_html();
        assert!(html.contains("Live Prices"));
        assert!(html.contains("Coinbase BTC/USD"));
        assert!(html.contains("Binance BTC/USDT"));
        assert!(html.contains("Kraken XBT/USD"));
        assert!(html.contains("Polymarket YES"));
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
}
