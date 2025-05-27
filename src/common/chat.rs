pub fn safe_username(username: &str) -> String {
    username.to_lowercase().trim().replace(' ', "_")
}
