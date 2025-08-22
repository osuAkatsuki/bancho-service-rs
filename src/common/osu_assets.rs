pub fn get_beatmap_cover_url(beatmap_set_id: i32) -> String {
    format!("https://assets.ppy.sh/beatmaps/{beatmap_set_id}/covers/cover.jpg")
}
