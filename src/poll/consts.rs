pub enum NumberEmojis {
    One,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
}

impl NumberEmojis {
    pub fn as_str(&self) -> &'static str {
        match self {
            NumberEmojis::One => "1️⃣",
            NumberEmojis::Two => "2️⃣",
            NumberEmojis::Three => "3️⃣",
            NumberEmojis::Four => "4️⃣",
            NumberEmojis::Five => "5️⃣",
            NumberEmojis::Six => "6️⃣",
            NumberEmojis::Seven => "7️⃣",
        }
    }
}

pub const NUMBERS: &[NumberEmojis] = &[
    NumberEmojis::One,
    NumberEmojis::Two,
    NumberEmojis::Three,
    NumberEmojis::Four,
    NumberEmojis::Five,
    NumberEmojis::Six,
    NumberEmojis::Seven,
];

pub const FINISHED: &str = "✅";
