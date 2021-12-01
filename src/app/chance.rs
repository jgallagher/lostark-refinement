#[cfg_attr(feature = "persistence", derive(serde::Deserialize, serde::Serialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[allow(clippy::enum_variant_names)]
pub(super) enum Chance {
    TwentyFive,
    ThirtyFive,
    FourtyFive,
    FiftyFive,
    SixtyFive,
    SeventyFive,
}

impl Chance {
    pub(super) fn up(&mut self) {
        *self = match self {
            Chance::TwentyFive => Chance::ThirtyFive,
            Chance::ThirtyFive => Chance::FourtyFive,
            Chance::FourtyFive => Chance::FiftyFive,
            Chance::FiftyFive => Chance::SixtyFive,
            Chance::SixtyFive => Chance::SeventyFive,
            Chance::SeventyFive => Chance::SeventyFive,
        };
    }

    pub(super) fn down(&mut self) {
        *self = match self {
            Chance::TwentyFive => Chance::TwentyFive,
            Chance::ThirtyFive => Chance::TwentyFive,
            Chance::FourtyFive => Chance::ThirtyFive,
            Chance::FiftyFive => Chance::FourtyFive,
            Chance::SixtyFive => Chance::FiftyFive,
            Chance::SeventyFive => Chance::SixtyFive,
        };
    }

    pub(super) fn as_f64(self) -> f64 {
        match self {
            Chance::TwentyFive => 0.25,
            Chance::ThirtyFive => 0.35,
            Chance::FourtyFive => 0.45,
            Chance::FiftyFive => 0.55,
            Chance::SixtyFive => 0.65,
            Chance::SeventyFive => 0.75,
        }
    }

    pub(super) fn as_str(self) -> &'static str {
        match self {
            Chance::TwentyFive => "25%",
            Chance::ThirtyFive => "35%",
            Chance::FourtyFive => "45%",
            Chance::FiftyFive => "55%",
            Chance::SixtyFive => "65%",
            Chance::SeventyFive => "75%",
        }
    }
}
