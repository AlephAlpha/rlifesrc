//! A macro to generate the corresponding Generations rule of a rule.
#![macro_use]

/// Implements `Rule` trait for a rule and the corresponding Generations rule.
#[doc(hidden)]
macro_rules! impl_rule {
    {
        $(#[$doc_desc:meta])*
        pub struct NbhdDesc($desc_type:ty);

        $(#[$doc:meta])*
        pub struct $rule:ident {
            Parser: $parser:ident,
            impl_table: $impl_table:ty $(,)?
        }

        $(#[$doc_gen:meta])*
        pub struct $rule_gen:ident {
            Parser: $parser_gen:ident,
        }

        fn new_desc {
            ALIVE => $alive_desc:expr,
            DEAD => $dead_desc:expr,
        }

        fn update_desc(
            $cell:ident,
            $state:ident,
            $new:ident,
            $change_num:ident $(,)?
        ) $update_desc_body:block

        fn consistify<$a:lifetime>(
            $world:ident,
            $cell_cons:ident,
            $flags:ident $(,)?
        ) $consistify_body:block

        fn consistify_gen<$a_gen:lifetime>(
            $world_gen:ident,
            $cell_cons_gen:ident,
            $flags_gen:ident $(,)?
        ) $consistify_gen_body:block
    } => {
        $(#[$doc_desc])*
        #[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
        pub struct NbhdDesc($desc_type);

        $(#[$doc])*
        pub struct $rule {
            /// Whether the rule contains `B0`.
            b0: bool,
            /// Whether the rule contains `S8`.
            s8: bool,
            /// An array of actions for all neighborhood descriptors.
            impl_table: $impl_table,
        }

        /// A parser for the rule.
        impl $parser for $rule {
            fn from_bs(b: Vec<u8>, s: Vec<u8>) -> Self {
                Self::new(b, s)
            }
        }

        impl FromStr for $rule {
            type Err = Error;

            fn from_str(input: &str) -> Result<Self, Self::Err> {
                let rule: $rule = $parser::parse_rule(input)
                    .map_err(Error::ParseRuleError)?;
                if rule.has_b0_s8() {
                    Err(Error::B0S8Error)
                } else {
                    Ok(rule)
                }
            }
        }

        impl Rule for $rule {
            type Desc = NbhdDesc;

            const IS_GEN: bool = false;

            fn has_b0(&self) -> bool {
                self.b0
            }

            fn has_b0_s8(&self) -> bool {
                self.b0 && self.s8
            }

            fn gen(&self) -> usize {
                2
            }

            fn new_desc(state: State, succ_state: State) -> Self::Desc {
                let nbhd_state = match state {
                    ALIVE => $alive_desc,
                    _ => $dead_desc,
                };
                let succ_state = match succ_state {
                    ALIVE => 0b01,
                    _ => 0b10,
                };
                let state = match state {
                    ALIVE => 0b01,
                    _ => 0b10,
                };
                NbhdDesc(nbhd_state << 4 | succ_state << 2 | state)
            }

            fn update_desc(
                $cell: CellRef<Self>,
                $state: Option<State>,
                $new: bool,
            ) {
                $update_desc_body
                let change_num = match $state {
                    Some(ALIVE) => 0b01,
                    Some(_) => 0b10,
                    _ => 0,
                };
                if let Some(pred) = $cell.pred {
                    let mut desc = pred.desc.get();
                    desc.0 ^= change_num << 2;
                    pred.desc.set(desc);
                }
                let mut desc = $cell.desc.get();
                desc.0 ^= change_num;
                $cell.desc.set(desc);
            }

            fn consistify<$a>($world: &mut World<$a, Self>, $cell_cons: CellRef<$a, Self>) -> bool {
                let $flags = $world.rule.impl_table[$cell_cons.desc.get().0 as usize];
                if $flags.is_empty() {
                    return true;
                }
                if $flags.contains(ImplFlags::CONFLICT) {
                    return false;
                }
                if $flags.intersects(ImplFlags::SUCC) {
                    let state = if $flags.contains(ImplFlags::SUCC_DEAD) {
                        DEAD
                    } else {
                        ALIVE
                    };
                    let succ = $cell_cons.succ.unwrap();
                    return $world.set_cell(succ, state, Reason::Deduce);
                }
                if $flags.intersects(ImplFlags::SELF) {
                    let state = if $flags.contains(ImplFlags::SELF_DEAD) {
                        DEAD
                    } else {
                        ALIVE
                    };
                    if !$world.set_cell($cell_cons, state, Reason::Deduce) {
                        return false;
                    }
                }
                if $flags.intersects(ImplFlags::NBHD) {
                    $consistify_body
                }
                true
            }
        }

        /// The neighborhood descriptor.
        ///
        /// Including a descriptor for the corresponding non-Generations rule,
        /// and the states of the successor.
        #[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
        pub struct NbhdDescGen ($desc_type, Option<State>);

        $(#[$doc_gen])*
        pub struct $rule_gen {
            /// Whether the rule contains `B0`.
            b0: bool,
            /// Whether the rule contains `S8`.
            s8: bool,
            /// Number of states.
            gen: usize,
            /// An array of actions for all neighborhood descriptors.
            impl_table: $impl_table,
        }

        impl $rule_gen {
            /// Constructs a new rule from the `b` and `s` data
            /// and the number of states.
            pub fn new(b: Vec<u8>, s: Vec<u8>, gen: usize) -> Self {
                let life = $rule::new(b, s);
                let impl_table = life.impl_table;
                Self {
                    b0: life.b0,
                    s8: life.s8,
                    gen,
                    impl_table,
                }
            }

            /// Converts to the corresponding non-Generations rule.
            pub fn non_gen(self) -> $rule {
                $rule {
                    b0: self.b0,
                    s8: self.s8,
                    impl_table: self.impl_table,
                }
            }
        }

        /// A parser for the rule.
        impl $parser_gen for $rule_gen {
            fn from_bsg(b: Vec<u8>, s: Vec<u8>, gen: usize) -> Self {
                Self::new(b, s, gen)
            }
        }

        impl FromStr for $rule_gen {
            type Err = Error;

            fn from_str(input: &str) -> Result<Self, Self::Err> {
                let rule: $rule_gen = $parser_gen::parse_rule(input)
                    .map_err(Error::ParseRuleError)?;
                if rule.has_b0_s8() {
                    Err(Error::B0S8Error)
                } else {
                    Ok(rule)
                }
            }
        }

        /// NOTE: This implementation does work when the number of states is 2.
        impl Rule for $rule_gen {
            type Desc = NbhdDescGen;

            const IS_GEN: bool = true;

            fn has_b0(&self) -> bool {
                self.b0
            }

            fn has_b0_s8(&self) -> bool {
                self.b0 && self.s8
            }

            fn gen(&self) -> usize {
                self.gen
            }

            fn new_desc(state: State, succ_state: State) -> Self::Desc {
                let desc = $rule::new_desc(state, succ_state);
                NbhdDescGen(desc.0, Some(succ_state))
            }

            fn update_desc(
                $cell: CellRef<Self>,
                $state: Option<State>,
                $new: bool,
            ) {
                $update_desc_body
                let $change_num = match $state {
                    Some(ALIVE) => 0b01,
                    Some(_) => 0b10,
                    _ => 0,
                };
                if let Some(pred) = $cell.pred {
                    let mut desc = pred.desc.get();
                    desc.0 ^= $change_num << 2;
                    desc.1 = if $new { $state } else { None };
                    pred.desc.set(desc);
                }
                let mut desc = $cell.desc.get();
                desc.0 ^= $change_num;
                $cell.desc.set(desc);
            }

            fn consistify<$a_gen>(
                $world_gen: &mut World<$a_gen, Self>,
                $cell_cons_gen: CellRef<$a_gen, Self>,
            ) -> bool {
                let desc = $cell_cons_gen.desc.get();
                let $flags_gen = $world_gen.rule.impl_table[desc.0 as usize];
                let gen = $world_gen.rule.gen;
                match $cell_cons_gen.state.get() {
                    Some(DEAD) => {
                        if let Some(State(j)) = desc.1 {
                            if j >= 2 {
                                return false;
                            }
                        }
                        if $flags_gen.intersects(ImplFlags::SUCC) {
                            let state = if $flags_gen.contains(ImplFlags::SUCC_DEAD) {
                                DEAD
                            } else {
                                ALIVE
                            };
                            let succ = $cell_cons_gen.succ.unwrap();
                            return $world_gen.set_cell(succ, state, Reason::Deduce);
                        }
                    }
                    Some(ALIVE) => {
                        if let Some(State(j)) = desc.1 {
                            if j == 0 || j > 2 {
                                return false;
                            }
                        }
                        if $flags_gen.intersects(ImplFlags::SUCC) {
                            let state = if $flags_gen.contains(ImplFlags::SUCC_DEAD) {
                                State(2)
                            } else {
                                ALIVE
                            };
                            let succ = $cell_cons_gen.succ.unwrap();
                            return $world_gen.set_cell(succ, state, Reason::Deduce);
                        }
                    }
                    Some(State(i)) => {
                        assert!(i >= 2);
                        if let Some(State(j)) = desc.1 {
                            return j == (i + 1) % gen;
                        } else {
                            let succ = $cell_cons_gen.succ.unwrap();
                            return $world_gen.set_cell(succ, State((i + 1) % gen), Reason::Deduce);
                        }
                    }
                    None => match desc.1 {
                        Some(DEAD) => {
                            if $flags_gen.contains(ImplFlags::SELF_ALIVE) {
                                return $world_gen.set_cell(
                                    $cell_cons_gen,
                                    State(gen - 1),
                                    Reason::Deduce
                                );
                            } else {
                                return true;
                            }
                        }
                        Some(ALIVE) => {
                            if $flags_gen.intersects(ImplFlags::SELF) {
                                let state = if $flags_gen.contains(ImplFlags::SELF_DEAD) {
                                    DEAD
                                } else {
                                    ALIVE
                                };
                                if !$world_gen.set_cell($cell_cons_gen, state, Reason::Deduce) {
                                    return false;
                                }
                            }
                        }
                        Some(State(j)) => {
                            return $world_gen.set_cell(
                                $cell_cons_gen,
                                State(j - 1),
                                Reason::Deduce
                            );
                        }
                        None => return true,
                    },
                }

                if $flags_gen.is_empty() {
                    return true;
                }

                if $flags_gen.contains(ImplFlags::CONFLICT) {
                    return false;
                }

                $consistify_gen_body

                true
            }
        }
    };
}
