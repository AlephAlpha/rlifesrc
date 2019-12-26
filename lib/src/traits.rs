//! A trait for `World`.
use crate::{
    cells::{Coord, State, ALIVE, DEAD},
    config::Config,
    error::Error,
    rules::Rule,
    search::Status,
    world::World,
};
use std::fmt::Write;

#[cfg(feature = "serialize")]
use crate::save::WorldSer;

/// A trait for `World`.
///
/// So that we can switch between different rule types using trait objects.
pub trait Search {
    /// The search function.
    ///
    /// Returns `Found` if a result is found,
    /// `None` if such pattern does not exist,
    /// `Searching` if the number of steps exceeds `max_step`
    /// and no results are found.
    fn search(&mut self, max_step: Option<u64>) -> Status;

    /// Gets the state of a cell. Returns `Err(())` if there is no such cell.
    fn get_cell_state(&self, coord: Coord) -> Result<Option<State>, Error>;

    /// World configuration.
    fn config(&self) -> &Config;

    /// Whether the rule is a Generations rule.
    fn is_gen_rule(&self) -> bool;

    /// Number of known living cells in some generation.
    fn cell_count_gen(&self, t: isize) -> usize;

    /// Minumum number of known living cells in all generation.
    fn cell_count(&self) -> usize;

    /// Number of conflicts during the search.
    fn conflicts(&self) -> u64;

    /// Set the max cell counts.
    ///
    /// Currently this is the only parameter that you can change
    /// during the search.
    fn set_max_cell_count(&mut self, max_cell_count: Option<usize>);

    #[cfg(feature = "serialize")]
    /// Saves the world as a `WorldSer`,
    /// which can be easily serialized.
    fn ser(&self) -> WorldSer;

    /// Displays the whole world in some generation,
    /// in a mix of [Plaintext](https://conwaylife.com/wiki/Plaintext) and
    /// [RLE](https://conwaylife.com/wiki/Rle) format.
    ///
    /// * **Dead** cells are represented by `.`;
    /// * **Living** cells are represented by `o` for rules with 2 states,
    ///   `A` for rules with more states;
    /// * **Dying** cells are represented by uppercase letters starting from `B`;
    /// * **Unknown** cells are represented by `?`.
    fn rle_gen(&self, t: isize) -> String {
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
                let state = self.get_cell_state((x, y, t)).unwrap();
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
    fn plaintext_gen(&self, t: isize) -> String {
        let mut str = String::new();
        for y in 0..self.config().height {
            for x in 0..self.config().width {
                let state = self.get_cell_state((x, y, t)).unwrap();
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

/// The `Search` trait is implemented for every `World`.
impl<'a, R: Rule> Search for World<'a, R> {
    fn search(&mut self, max_step: Option<u64>) -> Status {
        self.search(max_step)
    }

    fn get_cell_state(&self, coord: Coord) -> Result<Option<State>, Error> {
        self.get_cell_state(coord)
    }

    fn config(&self) -> &Config {
        &self.config
    }

    fn is_gen_rule(&self) -> bool {
        R::IS_GEN
    }

    fn cell_count_gen(&self, t: isize) -> usize {
        self.cell_count[t as usize]
    }

    fn cell_count(&self) -> usize {
        *self.cell_count.iter().min().unwrap()
    }

    fn conflicts(&self) -> u64 {
        self.conflicts
    }

    fn set_max_cell_count(&mut self, max_cell_count: Option<usize>) {
        self.set_max_cell_count(max_cell_count)
    }

    #[cfg(feature = "serialize")]
    fn ser(&self) -> WorldSer {
        self.ser()
    }
}
