use crate::settings::AppSettings;

pub fn get_profile_link(user_id: i64) -> String {
    let frontend_base = &AppSettings::get().frontend_base_url;
    format!("{frontend_base}/u/{user_id}")
}
