use std::ops::Deref;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub(crate) enum MeepleColor {
    Red,
    Green,
    Blue,
    Black,
}

pub type PlayerId = Uuid;

const MEEPLE_COUNT: usize = 7;

#[derive(Debug)]
pub struct Player {
    pub(crate) id: PlayerId,
    name: Option<String>,
    pub(crate) meeple: Vec<Meeple>,
}

impl Player {
    fn new(color: MeepleColor) -> Self {
        let meeple = Vec::with_capacity(MEEPLE_COUNT);

        let mut player = Self {
            id: PlayerId::new_v4(),
            name: None,
            meeple,
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
