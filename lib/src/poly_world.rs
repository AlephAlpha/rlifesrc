//! A polymorphic world.

use crate::{
    cells::{Coord, State},
    config::Config,
    rules::{Life, LifeGen, NtLife, NtLifeGen},
    search::{Backjump, LifeSrc, Status},
    world::World,
};
use from_variants::FromVariants;

#[cfg(feature = "serde")]
use crate::{error::Error, save::WorldSer};

#[cfg(doc)]
use crate::cells::ALIVE;

/// A polymorphic [`World`].
#[non_exhaustive]
#[derive(FromVariants)]
pub enum PolyWorld {
    /// A [`World`] with [`Life`] rule and [`LifeSrc`] algorithm.
    Life(World<Life, LifeSrc>),
    /// A [`World`] with [`LifeGen`] rule and [`LifeSrc`] algorithm.
    LifeGen(World<LifeGen, LifeSrc>),
    /// A [`World`] with [`NtLife`] rule and [`LifeSrc`] algorithm.
    NtLife(World<NtLife, LifeSrc>),
    /// A [`World`] with [`NtLifeGen`] rule and [`LifeSrc`] algorithm.
    NtLifeGen(World<NtLifeGen, LifeSrc>),
    /// A [`World`] with [`Life`] rule and [`Backjump`] algorithm.
    LifeBackjump(World<Life, Backjump<Life>>),
    /// A [`World`] with [`NtLife`] rule and [`Backjump`] algorithm.
    NtLifeBackjump(World<NtLife, Backjump<NtLife>>),
}

macro_rules! dispatch {
    ($self: expr, $world: ident => $action: expr) => {
        match $self {
            PolyWorld::Life($world) => $action,
            PolyWorld::LifeGen($world) => $action,
            PolyWorld::NtLife($world) => $action,
            PolyWorld::NtLifeGen($world) => $action,
            PolyWorld::LifeBackjump($world) => $action,
            PolyWorld::NtLifeBackjump($world) => $action,
        }
    };
}

impl PolyWorld {
    /// The search function.
    ///
    /// Returns [`Status::Found`] if a result is found,
    /// [`Status::None`] if such pattern does not exist,
    /// [`Status::Searching`] if the number of steps exceeds `max_step`
    /// and no results are found.
    #[inline]
    pub fn search(&mut self, max_step: Option<u64>) -> Status {
        dispatch!(self, world => world.search(max_step))
    }

    /// Gets the state of a cell. Returns `Err(())` if there is no such cell.
    #[inline]
    pub fn get_cell_state(&self, coord: Coord) -> Option<State> {
        dispatch!(self, world => world.get_cell_state(coord))
    }

    /// World configuration.
    #[inline]
    pub const fn config(&self) -> &Config {
        dispatch!(self, world => world.config())
    }

    /// Whether the rule is a Generations rule.
    #[inline]
    pub const fn is_gen_rule(&self) -> bool {
        dispatch!(self, world => world.is_gen_rule())
    }

    /// Whether the rule contains `B0`.
    ///
    /// In other words, whether a cell would become [`ALIVE`] in the next
    /// generation, if all its neighbors in this generation are dead.
    #[inline]
    pub fn is_b0_rule(&self) -> bool {
        dispatch!(self, world => world.is_b0_rule())
    }

    /// Number of known living cells in some generation.
    ///
    /// For Generations rules, dying cells are not counted.
    #[inline]
    pub fn cell_count_gen(&self, t: i32) -> u32 {
        dispatch!(self, world => world.cell_count_gen(t))
    }

    /// Minimum number of known living cells in all generation.
    ///
    /// For Generations rules, dying cells are not counted.
    #[inline]
    pub fn cell_count(&self) -> u32 {
        dispatch!(self, world => world.cell_count())
    }

    /// Number of conflicts during the search.
    #[inline]
    pub const fn conflicts(&self) -> u64 {
        dispatch!(self, world => world.conflicts())
    }

    /// Set the max cell counts.
    ///
    /// Currently this is the only parameter that you can change
    /// during the search.
    #[inline]
    pub fn set_max_cell_count(&mut self, max_cell_count: Option<u32>) {
        dispatch!(self, world => world.set_max_cell_count(max_cell_count));
    }

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
    #[inline]
    pub fn rle_gen(&self, t: i32) -> String {
        dispatch!(self, world => world.rle_gen(t))
    }

    /// Displays the whole world in some generation in
    /// [Plaintext](https://conwaylife.com/wiki/Plaintext) format.
    ///
    /// Do not use this for Generations rules.
    ///
    /// * **Dead** cells are represented by `.`;
    /// * **Living** and **Dying** cells are represented by `o`;
    /// * **Unknown** cells are represented by `?`.
    #[inline]
    pub fn plaintext_gen(&self, t: i32) -> String {
        dispatch!(self, world => world.plaintext_gen(t))
    }

    /// Saves the world as a [`WorldSer`],
    /// which can be easily serialized.
    #[cfg(feature = "serde")]
    #[cfg_attr(any(docs_rs, github_io), doc(cfg(feature = "serde")))]
    #[inline]
    pub fn ser(&self) -> WorldSer {
        dispatch!(self, world => world.ser())
    }

    /// Restores the world from the [`WorldSer`].
    #[cfg(feature = "serde")]
    #[cfg_attr(any(docs_rs, github_io), doc(cfg(feature = "serde")))]
    #[inline]
    pub fn deser(&mut self, ser: &WorldSer) -> Result<(), Error> {
        dispatch!(self, world => world.deser(ser))
    }
}
