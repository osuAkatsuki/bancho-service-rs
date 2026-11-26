pub enum NewsKind {
    Ranked,
    Loved,
}

impl NewsKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            NewsKind::Ranked => "ranked",
            NewsKind::Loved => "loved",
        }
    }
}
