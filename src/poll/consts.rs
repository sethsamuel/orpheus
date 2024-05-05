#[derive(Eq, Hash, PartialEq, Debug, Clone, Copy)]
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

#[derive(Debug)]
pub struct TryFromError;
impl TryFrom<&str> for NumberEmojis {
    type Error = TryFromError;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "1️⃣" => Ok(NumberEmojis::One),
            "2️⃣" => Ok(NumberEmojis::Two),
            "3️⃣" => Ok(NumberEmojis::Three),
            "4️⃣" => Ok(NumberEmojis::Four),
            "5️⃣" => Ok(NumberEmojis::Five),
            "6️⃣" => Ok(NumberEmojis::Six),
            "7️⃣" => Ok(NumberEmojis::Seven),
            _ => Err(TryFromError),
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
