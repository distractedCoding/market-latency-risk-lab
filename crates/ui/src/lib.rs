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
    }

    #[test]
    fn app_js_renders_feed_health_from_mode_and_source_counts() {
        let js = app_js();

        assert!(js.contains("source_counts"));
        assert!(js.contains("top source"));
    }

    #[test]
    fn app_js_polls_feed_health_periodically() {
        let js = app_js();

        assert!(js.contains("window.setInterval"));
        assert!(js.contains("fetchFeedHealthIntervalMs"));
    }
}
