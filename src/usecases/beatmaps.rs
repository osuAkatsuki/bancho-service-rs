const MIRRORS_BEATMAPSET_URL: &[(&str, &str)] = &[
    ("osu! (official servers)", "https://osu.ppy.sh/beatmapsets"),
    ("osu.direct (Chimu)", "https://osu.direct/beatmapsets"),
    (
        "Akatsuki (direct download)",
        "https://beatmaps.akatsuki.gg/api/d",
    ),
];
pub fn generate_mirror_links(beatmap_set_id: i32, song_name: &str) -> Vec<String> {
    MIRRORS_BEATMAPSET_URL
        .iter()
        .map(|(mirror_name, mirror_url)| {
            format!("{mirror_name}: [{mirror_url}/{beatmap_set_id} {song_name}]")
        })
        .collect()
}
