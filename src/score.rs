use std::collections::HashMap;
use colored::Colorize;
use crate::player::{Player, PlayerId};
use crate::tile::RenderStyle;

#[derive(Debug, Default)]
pub struct Score(HashMap<PlayerId, u32>);

impl Score {
    // @todo make a proper pretty table
    pub(crate) fn render(&self, players: &HashMap<PlayerId, Player>, render_style: &RenderStyle) -> String {

        let mut out = String::new();

        for (player_id, score) in &self.0 {

            let player = players.get(player_id).expect("should exist");

            out += format!("{} = {} \n", &player.name.clone().unwrap_or_else(|| "unnamed".parse().unwrap()).color(player.meeple_color.render_color(render_style)), score).as_str();
        }

        out

    }
}

impl Score {
    pub(crate) fn new() -> Self {
        Score(Default::default())
    }

    pub(crate) fn add_delta(&mut self, delta: &Self) {
        for (player_id, score) in delta.0.iter() {
            self.add_score(*player_id, *score);
        }
    }

    pub(crate) fn add_score(&mut self, player_id: PlayerId, score: u32) {
        *self.0.entry(player_id).or_insert(0) += score;
    }
}

