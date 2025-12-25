use crate::cell::{Cell, CellType};
use std::collections::HashMap;
use std::sync::LazyLock;

pub static REACTIONS: &[Reaction] = &[Reaction::new(
    CellType::Sand,
    CellType::Water,
    Some(CellType::WetSand),
    None,
)];

pub struct Reaction {
    pub reactants: (CellType, CellType),
    pub products: (Option<CellType>, Option<CellType>),
    pub probability: f32,
    pub condition: Option<fn(&Cell, &Cell) -> bool>,
}

impl Reaction {
    pub const fn new(
        a: CellType,
        b: CellType,
        out_a: Option<CellType>,
        out_b: Option<CellType>,
    ) -> Self {
        Self {
            reactants: (a, b),
            products: (out_a, out_b),
            probability: 1.0,
            condition: None,
        }
    }

    pub const fn with_probability(mut self, p: f32) -> Self {
        self.probability = p;
        self
    }

    pub const fn with_condition(mut self, f: fn(&Cell, &Cell) -> bool) -> Self {
        self.condition = Some(f);
        self
    }
}

pub static REACTION_TABLE: LazyLock<ReactionTable> = LazyLock::new(ReactionTable::new);

pub struct ReactionTable(HashMap<(CellType, CellType), Vec<&'static Reaction>>);

impl Default for ReactionTable {
    fn default() -> Self {
        let mut table: HashMap<(CellType, CellType), Vec<&'static Reaction>> = HashMap::new();

        for reaction in REACTIONS {
            table.entry(reaction.reactants).or_default().push(reaction);
        }

        Self(table)
    }
}

impl ReactionTable {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_reactions(&self, a: CellType, b: CellType) -> Option<&[&'static Reaction]> {
        self.0.get(&(a, b)).map(|v| v.as_slice())
    }
}
