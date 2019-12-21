//! A macro to generate the corresponding Generations rule of a rule.
#![macro_use]

/// Generates the corresponding Generations rule of a rule.
macro_rules! generations {
    {
        $(#[$doc:meta])*
        pub struct $gen_rule:ident {
            Rule: $rule:ident,
            Parser: $parser:ident,
            impl_table: $impl_table:ty $(,)?
        }

        fn update_desc(
            $cell:ident,
            $old_state:ident,
            $state:ident,
            $change_num:ident $(,)?
        ) $update_desc_body:block

        fn consistify<$a:lifetime>(
            $world:ident,
            $cell_1:ident,
            $flags:ident $(,)?
        ) $consistify_body:block
    } => {
        /// The neighborhood descriptor.
        ///
        /// Including a descriptor for the corresponding non-Generations rule,
        /// and the states of the successor.
        #[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
        pub struct NbhdDescGen {
            desc: NbhdDesc,
            succ_state: Option<State>,
        }

        $(#[$doc])*
        pub struct $gen_rule {
            /// Whether the rule contains `B0`.
            b0: bool,
            /// Number of states.
            gen: usize,
            /// An array of actions for all neighborhood descriptors.
            impl_table: $impl_table,
        }

        impl $gen_rule {
            /// Constructs a new rule from the `b` and `s` data
            /// and the number of states.
            pub fn new(b: Vec<u8>, s: Vec<u8>, gen: usize) -> Self {
                let b0 = b.contains(&0);
                let life = $rule::new(b, s);
                let impl_table = life.impl_table;
                Self {
                    b0,
                    gen,
                    impl_table,
                }
            }

            /// Converts to the corresponding non-Generations rule.
            pub fn non_gen(self) -> $rule {
                $rule {
                    b0: self.b0,
                    impl_table: self.impl_table,
                }
            }
        }

        /// A parser for the rule.
        impl $parser for $gen_rule {
            fn from_bsg(b: Vec<u8>, s: Vec<u8>, gen: usize) -> Self {
                Self::new(b, s, gen)
            }
        }

        impl FromStr for $gen_rule {
            type Err = ParseRuleError;

            fn from_str(input: &str) -> Result<Self, Self::Err> {
                $parser::parse_rule(input)
            }
        }

        /// NOTE: This implementation does work when the number of states is 2.
        impl Rule for $gen_rule {
            type Desc = NbhdDescGen;

            const IS_GEN: bool = true;

            fn has_b0(&self) -> bool {
                self.b0
            }

            fn gen(&self) -> usize {
                self.gen
            }

            fn new_desc(state: State, succ_state: State) -> Self::Desc {
                let desc = $rule::new_desc(state, succ_state);
                NbhdDescGen {
                    desc,
                    succ_state: Some(succ_state),
                }
            }

            fn update_desc(
                $cell: CellRef<Self>,
                $old_state: Option<State>,
                $state: Option<State>,
            ) {
                $update_desc_body
                let $change_num = match ($state, $old_state) {
                    (Some(ALIVE), Some(ALIVE)) => 0,
                    (Some(_), Some(ALIVE)) | (Some(ALIVE), Some(_)) => 0b11,
                    (Some(ALIVE), None) | (None, Some(ALIVE)) => 0b01,
                    (Some(_), None) | (None, Some(_)) => 0b10,
                    _ => 0,
                };
                if let Some(pred) = $cell.pred {
                    let mut desc = pred.desc.get();
                    desc.desc.0 ^= $change_num << 2;
                    desc.succ_state = $state;
                    pred.desc.set(desc);
                }
                let mut desc = $cell.desc.get();
                desc.desc.0 ^= $change_num;
                $cell.desc.set(desc);
            }

            fn consistify<$a>($world: &mut World<$a, Self>, $cell_1: CellRef<$a, Self>) -> bool {
                let desc = $cell_1.desc.get();
                let $flags = $world.rule.impl_table[desc.desc.0];
                let gen = $world.rule.gen;
                match $cell_1.state.get() {
                    Some(DEAD) => {
                        if let Some(State(j)) = desc.succ_state {
                            if j >= 2 {
                                return false;
                            }
                        }
                        if $flags.intersects(ImplFlags::SUCC) {
                            let state = if $flags.contains(ImplFlags::SUCC_DEAD) {
                                DEAD
                            } else {
                                ALIVE
                            };
                            let succ = $cell_1.succ.unwrap();
                            return $world.set_cell(succ, state, Reason::Deduce);
                        }
                    }
                    Some(ALIVE) => {
                        if let Some(State(j)) = desc.succ_state {
                            if j == 0 || j > 2 {
                                return false;
                            }
                        }
                        if $flags.intersects(ImplFlags::SUCC) {
                            let state = if $flags.contains(ImplFlags::SUCC_DEAD) {
                                State(2)
                            } else {
                                ALIVE
                            };
                            let succ = $cell_1.succ.unwrap();
                            return $world.set_cell(succ, state, Reason::Deduce);
                        }
                    }
                    Some(State(i)) => {
                        assert!(i >= 2);
                        if let Some(State(j)) = desc.succ_state {
                            return j == (i + 1) % gen;
                        } else {
                            let succ = $cell_1.succ.unwrap();
                            return $world.set_cell(succ, State((i + 1) % gen), Reason::Deduce);
                        }
                    }
                    None => match desc.succ_state {
                        Some(DEAD) => {
                            if $flags.contains(ImplFlags::SELF_ALIVE) {
                                return $world.set_cell($cell_1, State(gen - 1), Reason::Deduce);
                            } else {
                                return true;
                            }
                        }
                        Some(ALIVE) => {
                            if $flags.intersects(ImplFlags::SELF) {
                                let state = if $flags.contains(ImplFlags::SELF_DEAD) {
                                    DEAD
                                } else {
                                    ALIVE
                                };
                                if !$world.set_cell($cell_1, state, Reason::Deduce) {
                                    return false;
                                }
                            }
                        }
                        Some(State(j)) => {
                            return $world.set_cell($cell_1, State(j - 1), Reason::Deduce);
                        }
                        None => return true,
                    },
                }

                if $flags.contains(ImplFlags::CONFLICT) {
                    return false;
                }

                $consistify_body

                true
            }
        }
    };
}
