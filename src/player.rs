use std::ops::Deref;
use colored::Color;
use uuid::Uuid;
use crate::tile::RenderStyle;

#[derive(Debug, Clone)]
pub(crate) enum MeepleColor {
    Red,
    Green,
    Blue,
    Black,
    Yellow,
}

impl MeepleColor {
    pub(crate) fn render_color(&self, style: &RenderStyle) -> Color {
        match (self, style) {
            (MeepleColor::Red, RenderStyle::Ascii | RenderStyle::Ansi) => Color::Red,
            (MeepleColor::Green, RenderStyle::Ascii | RenderStyle::Ansi) => Color::Green,
            (MeepleColor::Blue, RenderStyle::Ascii | RenderStyle::Ansi) => Color::Blue,
            (MeepleColor::Black, RenderStyle::Ascii | RenderStyle::Ansi) => Color::Black,
            (MeepleColor::Yellow, RenderStyle::Ascii | RenderStyle::Ansi) => Color::BrightYellow,

            (MeepleColor::Red, RenderStyle::TrueColor) => Color::TrueColor {
                r: 194,
                g: 0,
                b: 25,
            },
            (MeepleColor::Green, RenderStyle::TrueColor) => Color::TrueColor {
                r: 16,
                g: 126,
                b: 50,
            },
            (MeepleColor::Blue, RenderStyle::TrueColor) => Color::TrueColor {
                r: 10,
                g: 79,
                b: 147,
            },
            (MeepleColor::Black, RenderStyle::TrueColor) => Color::TrueColor {
                r: 43,
                g: 42,
                b: 44,
            },
            (MeepleColor::Yellow, RenderStyle::TrueColor) => Color::TrueColor {
                r: 215,
                g: 184,
                b: 18,
            },
        }
    }
}

pub type PlayerId = Uuid;

const MEEPLE_COUNT: usize = 7;

#[derive(Debug, Clone)]
pub struct Player {
    pub(crate) id: PlayerId,
    pub(crate) name: Option<String>,
    pub(crate) meeple: Vec<Meeple>,
    pub(crate) meeple_color: MeepleColor
}

impl Player {
    fn new(color: MeepleColor) -> Self {
        let meeple = Vec::with_capacity(MEEPLE_COUNT);

        let mut player = Self {
            id: PlayerId::new_v4(),
            name: None,
            meeple,
            meeple_color: color.clone(),
        };

        for _ in 0..MEEPLE_COUNT {
            player.meeple.push(Meeple::new(&player, color.clone()));
        }

        player
    }

    pub(crate) fn black() -> Self {
        Self::new(MeepleColor::Black)
    }

    pub(crate) fn green() -> Self {
        Self::new(MeepleColor::Green)
    }

    pub(crate) fn red() -> Self {
        Self::new(MeepleColor::Red)
    }

    pub(crate) fn blue() -> Self {
        Self::new(MeepleColor::Blue)
    }

    pub(crate) fn yellow() -> Self {
        Self::new(MeepleColor::Yellow)
    }
}

#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
pub(crate) struct RegionIndex(usize);

impl RegionIndex {
    pub(crate) fn new(v: usize) -> Self {
        Self(v)
    }
}

impl Deref for RegionIndex {
    type Target = usize;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Clone)]
pub struct Meeple {
    pub(crate) color: MeepleColor,
    pub(crate) player_id: PlayerId,
}

impl Meeple {
    fn new(player: &Player, color: MeepleColor) -> Self {
        Self {
            player_id: player.id,
            color,
        }
    }

    pub(crate) fn dummy() -> Self {
        Self {
            player_id: Uuid::nil(),
            color: MeepleColor::Black,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_player_has_meeple_all_with_no_placement() {
        let player = Player::new(MeepleColor::Red);

        assert_eq!(player.meeple.len(), MEEPLE_COUNT);
    }
}
