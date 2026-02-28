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
}
