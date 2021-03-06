//! A trait for [`World`].
use crate::{
    cells::{Coord, State, ALIVE, DEAD},
    config::Config,
    rules::Rule,
    search::{Reason, Status},
    world::World,
};
use std::fmt::Write;

#[cfg(feature = "serde")]
use crate::{error::Error, save::WorldSer};

/// A trait for [`World`].
///
/// So that we can switch between different rule types using trait objects.
pub trait Search {
    /// The search function.
    ///
    /// Returns [`Status::Found`] if a result is found,
    /// [`Status::None`] if such pattern does not exist,
    /// [`Status::Searching`] if the number of steps exceeds `max_step`
    /// and no results are found.
    fn search(&mut self, max_step: Option<u64>) -> Status;

    /// Gets the state of a cell. Returns `Err(())` if there is no such cell.
    fn get_cell_state(&self, coord: Coord) -> Option<State>;

    /// World configuration.
    fn config(&self) -> &Config;

    /// Whether the rule is a Generations rule.
    fn is_gen_rule(&self) -> bool;

    /// Whether the rule contains `B0`.
    ///
    /// In other words, whether a cell would become [`ALIVE`] in the next
    /// generation, if all its neighbors in this generation are dead.
    fn is_b0_rule(&self) -> bool;

    /// Number of known living cells in some generation.
    ///
    /// For Generations rules, dying cells are not counted.
    fn cell_count_gen(&self, t: i32) -> u32;

    /// Minumum number of known living cells in all generation.
    ///
    /// For Generations rules, dying cells are not counted.
    fn cell_count(&self) -> u32;

    /// Number of conflicts during the search.
    fn conflicts(&self) -> u64;

    /// Set the max cell counts.
    ///
    /// Currently this is the only parameter that you can change
    /// during the search.
    fn set_max_cell_count(&mut self, max_cell_count: Option<u32>);

    #[cfg(feature = "serde")]
    /// Saves the world as a [`WorldSer`],
    /// which can be easily serialized.
    fn ser(&self) -> WorldSer;

    #[cfg(feature = "serde")]
    /// Restores the world from the [`WorldSer`].
    fn deser(&mut self, ser: &WorldSer) -> Result<(), Error>;

    /// Displays the whole world in some generation,
    /// in a mix of [Plaintext](https://conwaylife.com/wiki/Plaintext) and
    /// [RLE](https://conwaylife.com/wiki/Rle) format.
    ///
    /// * **Dead** cells are represented by `.`;
    /// * **Living** cells are represented by `o` for rules with 2 states,
    ///   `A` for rules with more states;
    /// * **Dying** cells are represented by uppercase letters starting from `B`;
    /// * **Unknown** cells are represented by `?`;
    /// * Each line is ended with `$`;
    /// * The whole pattern is ended with `!`.
    fn rle_gen(&self, t: i32) -> String {
        let mut str = String::new();
        writeln!(
            str,
            "x = {}, y = {}, rule = {}",
            self.config().width,
            self.config().height,
            self.config().rule_string
        )
        .unwrap();
        for y in 0..self.config().height {
            for x in 0..self.config().width {
                let state = self.get_cell_state((x, y, t));
                match state {
                    Some(DEAD) => str.push('.'),
                    Some(ALIVE) => {
                        if self.is_gen_rule() {
                            str.push('A')
                        } else {
                            str.push('o')
                        }
                    }
                    Some(State(i)) => str.push((b'A' + i as u8 - 1) as char),
                    _ => str.push('?'),
                };
            }
            if y == self.config().height - 1 {
                str.push('!')
            } else {
                str.push('$')
            };
            str.push('\n');
        }
        str
    }

    /// Displays the whole world in some generation in
    /// [Plaintext](https://conwaylife.com/wiki/Plaintext) format.
    ///
    /// * **Dead** cells are represented by `.`;
    /// * **Living** and **Dying** cells are represented by `o`;
    /// * **Unknown** cells are represented by `?`.
    fn plaintext_gen(&self, t: i32) -> String {
        let mut str = String::new();
        for y in 0..self.config().height {
            for x in 0..self.config().width {
                let state = self.get_cell_state((x, y, t));
                match state {
                    Some(DEAD) => str.push('.'),
                    Some(_) => str.push('o'),
                    None => str.push('?'),
                };
            }
            str.push('\n');
        }
        str
    }
}

/// The [`Search`] trait is implemented for every [`World`].
impl<'a, R: Rule, RE: Reason<'a, R>> Search for World<'a, R, RE> {
    #[inline]
    fn search(&mut self, max_step: Option<u64>) -> Status {
        self.search(max_step)
    }

    #[inline]
    fn get_cell_state(&self, coord: Coord) -> Option<State> {
        self.get_cell_state(coord)
    }

    #[inline]
    fn config(&self) -> &Config {
        &self.config
    }

    #[inline]
    fn is_gen_rule(&self) -> bool {
        R::IS_GEN
    }

    #[inline]
    fn is_b0_rule(&self) -> bool {
        self.rule.has_b0()
    }

    #[inline]
    fn cell_count_gen(&self, t: i32) -> u32 {
        self.cell_count[t as usize]
    }

    #[inline]
    fn cell_count(&self) -> u32 {
        self.cell_count()
    }

    #[inline]
    fn conflicts(&self) -> u64 {
        self.conflicts
    }

    #[inline]
    fn set_max_cell_count(&mut self, max_cell_count: Option<u32>) {
        self.set_max_cell_count(max_cell_count)
    }

    #[cfg(feature = "serde")]
    #[inline]
    fn ser(&self) -> WorldSer {
        self.ser()
    }

    #[cfg(feature = "serde")]
    #[inline]
    fn deser(&mut self, ser: &WorldSer) -> Result<(), Error> {
        ser.deser(self)
    }
}
