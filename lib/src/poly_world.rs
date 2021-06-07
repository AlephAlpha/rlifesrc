//! A polymorphic world.

use proxy_enum::proxy;

/// A polymorphic world.
///
/// This extra level of mod is needed because of the [`proxy_enum::proxy`] macro.
#[proxy(PolyWorld)]
mod proxy {
    use crate::{
        cells::{Coord, State},
        config::Config,
        rules::{Life, LifeGen, NtLife, NtLifeGen},
        search::{Backjump, LifeSrc, Status},
        world::World,
    };

    #[cfg(feature = "serde")]
    #[cfg_attr(any(docs_rs, github_io), doc(cfg(feature = "serde")))]
    use crate::{error::Error, save::WorldSer};

    /// A polymorphic [`World`].
    #[non_exhaustive]
    pub enum PolyWorld {
        /// A [`World`] with [`Life`] rule and [`LifeSrc`] algorithm.
        Life(World<'static, Life, LifeSrc>),
        /// A [`World`] with [`LifeGen`] rule and [`LifeSrc`] algorithm.
        LifeGen(World<'static, LifeGen, LifeSrc>),
        /// A [`World`] with [`NtLife`] rule and [`LifeSrc`] algorithm.
        NtLife(World<'static, NtLife, LifeSrc>),
        /// A [`World`] with [`NtLifeGen`] rule and [`LifeSrc`] algorithm.
        NtLifeGen(World<'static, NtLifeGen, LifeSrc>),
        /// A [`World`] with [`Life`] rule and [`Backjump`] algorithm.
        LifeBackjump(World<'static, Life, Backjump<'static, Life>>),
        /// A [`World`] with [`NtLife`] rule and [`Backjump`] algorithm.
        NtLifeBackjump(World<'static, NtLife, Backjump<'static, NtLife>>),
    }

    impl PolyWorld {
        /// The search function.
        ///
        /// Returns [`Status::Found`] if a result is found,
        /// [`Status::None`] if such pattern does not exist,
        /// [`Status::Searching`] if the number of steps exceeds `max_step`
        /// and no results are found.
        #[implement]
        pub fn search(&mut self, max_step: Option<u64>) -> Status {}

        /// Gets the state of a cell. Returns `Err(())` if there is no such cell.
        #[implement]
        pub fn get_cell_state(&self, coord: Coord) -> Option<State> {}

        /// World configuration.
        #[implement]
        pub fn config(&self) -> &Config {}

        /// Whether the rule is a Generations rule.
        #[implement]
        pub fn is_gen_rule(&self) -> bool {}

        /// Whether the rule contains `B0`.
        ///
        /// In other words, whether a cell would become [`ALIVE`] in the next
        /// generation, if all its neighbors in this generation are dead.
        #[implement]
        pub fn is_b0_rule(&self) -> bool {}

        /// Number of known living cells in some generation.
        ///
        /// For Generations rules, dying cells are not counted.
        #[implement]
        pub fn cell_count_gen(&self, t: i32) -> u32 {}

        /// Minimum number of known living cells in all generation.
        ///
        /// For Generations rules, dying cells are not counted.
        #[implement]
        pub fn cell_count(&self) -> u32 {}

        /// Number of conflicts during the search.
        #[implement]
        pub fn conflicts(&self) -> u64 {}

        /// Set the max cell counts.
        ///
        /// Currently this is the only parameter that you can change
        /// during the search.
        #[implement]
        pub fn set_max_cell_count(&mut self, max_cell_count: Option<u32>) {}

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
        #[implement]
        pub fn rle_gen(&self, t: i32) -> String {}

        /// Displays the whole world in some generation in
        /// [Plaintext](https://conwaylife.com/wiki/Plaintext) format.
        ///
        /// Do not use this for Generations rules.
        ///
        /// * **Dead** cells are represented by `.`;
        /// * **Living** and **Dying** cells are represented by `o`;
        /// * **Unknown** cells are represented by `?`.
        #[implement]
        pub fn plaintext_gen(&self, t: i32) -> String {}

        /// Saves the world as a [`WorldSer`],
        /// which can be easily serialized.
        #[cfg(feature = "serde")]
        #[cfg_attr(any(docs_rs, github_io), doc(cfg(feature = "serde")))]
        #[implement]
        pub fn ser(&self) -> WorldSer {}

        /// Restores the world from the [`WorldSer`].
        #[cfg(feature = "serde")]
        #[cfg_attr(any(docs_rs, github_io), doc(cfg(feature = "serde")))]
        #[implement]
        pub fn deser(&mut self, ser: &WorldSer) -> Result<(), Error> {}
    }
}

pub use proxy::PolyWorld;
