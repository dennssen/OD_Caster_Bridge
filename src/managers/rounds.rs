use std::collections::HashMap;
use indexmap::IndexMap;
use crate::ws::state::Round;

#[derive(Clone)]
pub enum RoundOverride {
    Delete,
    Replace(Round)
}

pub struct RoundManager {
    pub best_of: i32,
    archived_rounds: IndexMap<usize, Round>,
    overrides: HashMap<usize, RoundOverride>,
    pre_converted_rounds: IndexMap<usize, Round>,
}

impl RoundManager {
    pub fn new() -> Self {
        Self {
            best_of: 0,
            archived_rounds: IndexMap::new(),
            overrides: HashMap::new(),
            pre_converted_rounds: IndexMap::new(),
        }
    }

    pub fn add_round(&mut self, round: Round) {
        let index = self.archived_rounds.len();
        self.archived_rounds.insert(index, round);
    }

    pub fn add_override(&mut self, index: usize, round_override: RoundOverride) {
        self.overrides.insert(index, round_override);
    }

    pub fn clear_overrides(&mut self) {
        self.overrides.clear();
    }

    pub fn archive_rounds(&mut self) {
        let offset = self.archived_rounds.len();
        for (i, round) in self.pre_converted_rounds.iter() {
            self.archived_rounds.insert(offset + i, *round);
        }
    }

    pub fn clear_archives(&mut self) {
        self.archived_rounds.clear();
    }

    fn extend_archived_rounds(&self, rounds: &IndexMap<usize, Round>) -> IndexMap<usize, Round> {
        let mut extended: IndexMap<usize, Round> = self.archived_rounds.clone();

        let offset = extended.len();
        for (i, (_, round)) in rounds.iter().enumerate() {
            extended.insert(offset + i, *round);
        }

        extended
    }

    pub fn save_rounds(&mut self, rounds: &IndexMap<usize, Round>) {
        self.pre_converted_rounds = rounds.clone();
    }

    pub fn convert_rounds(&self, rounds: &IndexMap<usize, Round>) -> IndexMap<usize, Round> {
        let mut converted_rounds: IndexMap<usize, Round> = IndexMap::new();

        let all_rounds: IndexMap<usize, Round> = self.extend_archived_rounds(rounds);

        for (i, round) in all_rounds.iter() {
            if let Some(round_override) = self.overrides.get(i) {
                match round_override {
                    RoundOverride::Delete => {
                        continue
                    }
                    RoundOverride::Replace(override_round) => {
                        converted_rounds.insert(*i, *override_round);
                    }
                }
            } else {
                converted_rounds.insert(*i, *round);
            }
        }

        converted_rounds
    }

    pub fn get_total_rounds_amount(&self, rounds: &IndexMap<usize, Round>) -> usize {
        self.archived_rounds.iter().count() + rounds.iter().count()
    }

    pub fn has_wiped(&self, new_rounds: &IndexMap<usize, Round>) -> bool {
        new_rounds.len() < self.pre_converted_rounds.len()
    }

    pub fn should_archive(&self, rounds: &IndexMap<usize, Round>) -> bool {
        if rounds.len() == 0 {
            return false;
        }

        !self.has_a_winner(rounds)
    }

    pub fn has_a_winner(&self, rounds: &IndexMap<usize, Round>) -> bool {
        let win_condition = (self.best_of + 1) / 2;
        if rounds.len() < win_condition as usize {
            return false;
        }

        let mut home_wins: i32 = 0;
        let mut away_wins: i32 = 0;

        for (_, round) in rounds {
            if round.home > round.away {
                home_wins += 1;
            } else if round.away > round.home {
                away_wins += 1;
            }
        }

        home_wins >= win_condition || away_wins >= win_condition
    }
}