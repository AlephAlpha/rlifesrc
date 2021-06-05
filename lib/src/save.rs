#![cfg(feature = "serde")]
//! Saves the world.

use crate::{
    cells::{Coord, State},
    config::Config,
    error::Error,
    rules::Rule,
    search::Algorithm,
    traits::Search,
    world::World,
};
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// A representation of reasons for setting a cell which can be easily serialized.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum ReasonSer {
    /// Known before the search starts,
    Known,

    /// Decides the state of a cell by choice.
    Decide,

    /// Deduced from the rule when constitifying another cell.
    Rule(Coord),

    /// Deduced from symmetry.
    Sym(Coord),

    /// Deduced from conflicts.
    Deduce,

    /// Deduced from a learnt clause.
    Clause(Vec<Coord>),

    /// Tries another state of a cell when the original state
    /// leads to a conflict.
    ///
    /// Remembers the number of remaining states to try.
    ///
    /// Only used in Generations rules.
    TryAnother(usize),
}

/// A representation of setting a cell which can be easily serialized.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SetCellSer {
    /// The coordinates of the set cell.
    pub(crate) coord: Coord,

    /// The state.
    pub(crate) state: State,

    /// The reason for setting a cell.
    pub(crate) reason: ReasonSer,
}

/// A representation of the world which can be easily serialized.
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(default)]
pub struct WorldSer {
    /// World configuration.
    pub config: Config,

    /// Number of conflicts during the search.
    pub conflicts: u64,

    /// A stack to records the cells whose values are set during the search.
    ///
    /// The cells in this table always have known states.
    pub set_stack: Vec<SetCellSer>,

    /// The position of the next cell to be examined in the [`set_stack`](#structfield.set_stack).
    ///
    /// Be careful when modifying this value.
    /// If you have changed other things in the saved file, please set this value to `0`,
    /// otherwise rlifesrc might gives the wrong result.
    pub check_index: u32,

    /// Time used in searching. This field is handled by the frontend.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timing: Option<Duration>,
}

impl WorldSer {
    /// Restores the world from the [`WorldSer`].
    pub fn deser<'a, R: Rule, A: Algorithm<'a, R>>(
        &self,
        world: &mut World<'a, R, A>,
    ) -> Result<(), Error> {
        for &SetCellSer {
            coord,
            state,
            ref reason,
        } in self.set_stack.iter()
        {
            let cell = world.find_cell(coord).ok_or(Error::SetCellError(coord))?;
            if let Some(old_state) = cell.state.get() {
                if old_state != state {
                    return Err(Error::SetCellError(coord));
                }
            } else if state.0 >= world.rule.gen() {
                return Err(Error::InvalidState(coord, state));
            } else {
                let reason = A::deser_reason(world, reason)?;
                let _ = world.set_cell(cell, state, reason);
            }
        }
        world.conflicts = self.conflicts;
        if self.check_index < self.set_stack.len() as u32 {
            world.check_index = self.check_index;
        }
        Ok(())
    }

    /// Restores the world from the [`WorldSer`].
    pub fn world(&self) -> Result<Box<dyn Search>, Error> {
        let mut world = self.config.world()?;
        world.deser(self)?;
        Ok(world)
    }
}

impl<'a, R: Rule, A: Algorithm<'a, R>> World<'a, R, A> {
    /// Saves the world as a [`WorldSer`].
    pub fn ser(&self) -> WorldSer {
        WorldSer {
            config: self.config.clone(),
            conflicts: self.conflicts,
            set_stack: self.set_stack.iter().map(|s| s.ser()).collect(),
            check_index: self.check_index,
            timing: None,
        }
    }
}
