pub fn workspace_bootstrap() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::workspace_bootstrap;

    #[test]
    fn workspace_builds() {
        assert!(workspace_bootstrap());
    }
}
