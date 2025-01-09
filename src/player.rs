use std::ops::Deref;
use crate::tile::PlacedTile;
use uuid::Uuid;

#[derive(Debug)]
enum PlayerColor {
    Red,
    Green,
    Blue,
    Yellow,
    Black,
}

pub type PlayerId = Uuid;

const MEEPLE_COUNT: usize = 7;

#[derive(Debug)]
pub struct Player {
    id: PlayerId,
    color: PlayerColor,
    name: Option<String>,
    meeple: Vec<Meeple>,
}

impl Player {
    fn new(color: PlayerColor) -> Self {
        let meeple = Vec::with_capacity(MEEPLE_COUNT);

        let mut player = Self {
            id: PlayerId::new_v4(),
            name: None,
            color,
            meeple,
        };

        for _ in 0..MEEPLE_COUNT {
            player.meeple.push(Meeple::new(&player));
        }

        player
    }

    pub(crate) fn black() -> Self {
        Self::new(PlayerColor::Black)
    }

    pub(crate) fn green() -> Self {
        Self::new(PlayerColor::Green)
    }

    pub(crate) fn red() -> Self {
        Self::new(PlayerColor::Red)
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

#[derive(Debug)]
struct MeepleLocation {
    placed_tile: PlacedTile,
    region_index: RegionIndex,
}

#[derive(Debug)]
pub struct Meeple {
    id: Uuid,
    player_id: PlayerId,
    location: Option<MeepleLocation>,
}

impl Meeple {
    fn new(player: &Player) -> Self {
        Self {
            player_id: player.id,
            id: Uuid::new_v4(),
            location: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_player_has_meeple_all_with_no_placement() {
        let player = Player::new(PlayerColor::Red);

        assert_eq!(player.meeple.len(), MEEPLE_COUNT);

        for meeple in player.meeple {
            assert!(matches!(meeple.location, None))
        }
    }
}
