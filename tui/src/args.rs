//! Parsing command-line arguments.

use clap::{
    command,
    error::{ErrorKind, Result},
    value_parser, Arg, ArgAction,
};
use rlifesrc_lib::{
    rules::NtLifeGen, Config, NewState, PolyWorld, SearchOrder, Symmetry, Transform,
};
use std::{
    fs::File,
    io::{BufReader, Read},
    path::PathBuf,
};

/// A struct to store the parse results.
pub struct Args {
    pub(crate) world: PolyWorld,
    pub(crate) all: bool,
    #[cfg(feature = "tui")]
    pub(crate) reset: bool,
    #[cfg(feature = "tui")]
    pub(crate) no_tui: bool,
}

impl Args {
    /// Parses the command-line arguments.
    pub(crate) fn parse() -> Result<Self> {
        let mut app = command!()
            .long_about(
                "Searching for patterns in Conway's Game of Life\n\
                 \n\
                 The program is based on David Bell's lifesrc and Jason Summers's \n\
                 WinLifeSearch, using an algorithm invented by Dean Hickerson.\n\
                 \n\
                 For a detailed help, please visit:\n\
                 https://github.com/AlephAlpha/rlifesrc/blob/master/tui/README.md (In Chinese)\n\
                 https://github.com/AlephAlpha/rlifesrc/blob/master/tui/README_en.md (In English)",
            )
            .arg(
                Arg::new("CONFIG")
                    .help("Read config from a file")
                    .long_help(
                        "Read config from a file\n\
                         Supported formats: JSON, YAML, TOML.\n\
                         When a config file is provided, all the other flags and options, \
                         except --all (-a), --reset-time, --no-tui (-n), are ignored.",
                    )
                    .short('C')
                    .long("config")
                    .value_parser(value_parser!(PathBuf)),
            )
            .arg(
                Arg::new("X")
                    .help("Width of the pattern")
                    .required_unless_present("CONFIG")
                    .value_parser(value_parser!(i32).range(1..)),
            )
            .arg(
                Arg::new("Y")
                    .help("Height of the pattern")
                    .required_unless_present("CONFIG")
                    .value_parser(value_parser!(i32).range(1..)),
            )
            .arg(
                Arg::new("P")
                    .help("Period of the pattern")
                    .default_value("1")
                    .value_parser(value_parser!(i32).range(1..)),
            )
            .arg(
                Arg::new("DX")
                    .help("Horizontal translation")
                    .default_value("0")
                    .allow_negative_numbers(true)
                    .value_parser(value_parser!(i32)),
            )
            .arg(
                Arg::new("DY")
                    .help("Vertical translation")
                    .default_value("0")
                    .allow_negative_numbers(true)
                    .value_parser(value_parser!(i32)),
            )
            .arg(
                Arg::new("DIAG")
                    .help("Diagonal width")
                    .long_help(
                        "Diagonal width\n\
                         If the diagonal width is n > 0, the cells at position (x, y) \
                         where abs(x - y) >= n are assumed to be dead.\n\
                         If this value is set to 0, it would be ignored.",
                    )
                    .short('d')
                    .long("diag")
                    .default_value("0")
                    .value_parser(value_parser!(i32).range(0..)),
            )
            .arg(
                Arg::new("TRANSFORM")
                    .help("Transformation of the pattern")
                    .long_help(
                        "Transformation of the pattern\n\
                         After the last generation in a period, the pattern will return to \
                         the first generation, applying this transformation first, \
                         and then the translation defined by DX and DY.\n\
                         You may need to add quotation marks for some of the transformations.\n\
                         Supported values are Id, R90, R180, R270, F|, F-, F\\, F/.\n\
                         Id is the identical transformation.\n\
                         R means counterclockwise rotation.\n\
                         F means flipping (reflection) across an axis.",
                    )
                    .short('t')
                    .long("transform")
                    .value_parser(value_parser!(Transform))
                    .default_value("Id"),
            )
            .arg(
                Arg::new("SYMMETRY")
                    .help("Symmetry of the pattern")
                    .long_help(
                        "Symmetry of the pattern\n\
                         You may need to add quotation marks for some of the symmetries.\n\
                         The usages of these symmetries are the same as Oscar Cunningham's \
                         Logic Life Search.\n\
                         Supported values are C1, C2, C4, D2|, D2-, D2\\, D2/, D4+, D4X, D8.\n\
                         See [https://conwaylife.com/wiki/Static_symmetry#Reflectional] ",
                    )
                    .short('s')
                    .long("symmetry")
                    .value_parser(value_parser!(Symmetry))
                    .default_value("C1"),
            )
            .arg(
                Arg::new("RULE")
                    .help("Rule of the cellular automaton")
                    .long_help(
                        "Rule of the cellular automaton\n\
                         Supports Life-like, isotropic non-totalistic, hexagonal, MAP rules, \
                         and their corresponding Generations rules.",
                    )
                    .short('r')
                    .long("rule")
                    .default_value("B3/S23")
                    .value_parser(|d: &str| d.parse::<NtLifeGen>().map(|_| d.to_string())),
            )
            .arg(
                Arg::new("ORDER")
                    .help("Search order")
                    .long_help(
                        "Search order\n\
                         Row first or column first.",
                    )
                    .short('o')
                    .long("order")
                    .value_parser([
                        "row",
                        "column",
                        "automatic",
                        "diagonal",
                        "r",
                        "c",
                        "a",
                        "d",
                    ])
                    .default_value("automatic"),
            )
            .arg(
                Arg::new("CHOOSE")
                    .help("How to choose a state for unknown cells")
                    .short('c')
                    .long("choose")
                    .value_parser(["dead", "alive", "random", "d", "a", "r"])
                    .default_value("alive"),
            )
            .arg(
                Arg::new("MAX")
                    .help("Upper bound of numbers of minimum living cells in all generations")
                    .long_help(
                        "Upper bound of numbers of minimum living cells in all generations\n\
                         If this value is set to 0, it means there is no limitation.",
                    )
                    .short('m')
                    .long("max")
                    .default_value("0")
                    .value_parser(value_parser!(u32)),
            )
            .arg(
                Arg::new("REDUCE")
                    .help("Reduce the max cell count when a result is found")
                    .long_help(
                        "Reduce the max cell count when a result is found\n\
                         The new max cell count will be set to the cell count of \
                         the current result minus one.",
                    )
                    .short('R')
                    .long("reduce")
                    .action(ArgAction::SetTrue),
            )
            .arg(
                Arg::new("SUBPERIOD")
                    .help("Allow patterns with subperiod")
                    .long_help("Allow patterns whose fundamental period are smaller than the given period")
                    .short('p')
                    .long("subperiod")
                    .action(ArgAction::SetTrue),
            )
            .arg(
                Arg::new("SKIPSUBSYM")
                    .help("Skip patterns invariant under more transformations than the given symmetry")
                    .long_help(
                        "Skip patterns which are invariant under more transformations than \
                         required by the given symmetry.\n\
                         In another word, skip patterns whose symmetry group properly contains \
                         the given symmetry group.",
                    )
                    .short('S')
                    .long("skip-subsym")
                    .action(ArgAction::SetTrue),
            )
            .arg(
                Arg::new("BACKJUMP")
                    .help("(Experimental) Enable backjumping")
                    .long_help(
                        "(Experimental) Enable backjumping\n\
                        The current implementation of backjumping is very slow, only \
                        useful for large (e.g., 64x64) still lifes.",
                    )
                    .long("backjump")
                    .action(ArgAction::SetTrue),
            );

        #[cfg(feature = "tui")]
        {
            app = app
                .arg(
                    Arg::new("ALL")
                        .help("Prints all possible results instead of only the first one")
                        .long_help(
                            "Prints all possible results instead of only the first one\n\
                             Only useful when --no-tui is set.",
                        )
                        .short('a')
                        .long("all")
                        .requires("NOTUI")
                        .action(ArgAction::SetTrue),
                )
                .arg(
                    Arg::new("RESET")
                        .help("Resets the time when starting a new search")
                        .long("reset-time")
                        .conflicts_with("NOTUI")
                        .action(ArgAction::SetTrue),
                )
                .arg(
                    Arg::new("NOTUI")
                        .help("Starts searching immediately, without entering the TUI")
                        .short('n')
                        .long("no-tui")
                        .action(ArgAction::SetTrue),
                );
        }

        #[cfg(not(feature = "tui"))]
        {
            app = app.arg(
                Arg::new("ALL")
                    .help("Searches for all possible pattern")
                    .long_help("Searches for all possible pattern")
                    .short('a')
                    .long("all")
                    .action(ArgAction::SetTrue),
            );
        }

        let matches = app.clone().try_get_matches()?;

        let config;

        if let Some(path) = matches.get_one::<PathBuf>("CONFIG") {
            let file = File::open(path).map_err(|e| app.error(ErrorKind::Io, e))?;
            let mut reader = BufReader::new(file);
            match path.extension().and_then(|s| s.to_str()) {
                Some("json") => {
                    config = serde_json::from_reader(reader).map_err(|e| {
                        app.error(ErrorKind::Io, format!("Invalid config file: {}", e))
                    })?;
                }
                Some("yaml" | "yml") => {
                    config = serde_yaml::from_reader(reader).map_err(|e| {
                        app.error(ErrorKind::Io, format!("Invalid config file: {}", e))
                    })?;
                }
                Some("toml") => {
                    let mut buf = String::new();
                    reader.read_to_string(&mut buf).map_err(|e| {
                        app.error(ErrorKind::Io, format!("Invalid config file: {}", e))
                    })?;
                    config = toml::from_str(&buf).map_err(|e| {
                        app.error(ErrorKind::Io, format!("Invalid config file: {}", e))
                    })?;
                }
                _ => return Err(app.error(ErrorKind::Io, "Unsupported config file format")),
            }
        } else {
            let width = *matches.get_one("X").unwrap();
            let height = *matches.get_one("Y").unwrap();
            let period = *matches.get_one("P").unwrap();

            let dx = *matches.get_one("DX").unwrap();
            let dy = *matches.get_one("DY").unwrap();

            let transform = *matches.get_one("TRANSFORM").unwrap();
            let symmetry = *matches.get_one("SYMMETRY").unwrap();

            let search_order = match matches.get_one::<String>("ORDER").unwrap().as_str() {
                "row" | "r" => Some(SearchOrder::RowFirst),
                "column" | "c" => Some(SearchOrder::ColumnFirst),
                "diagonal" | "d" => Some(SearchOrder::Diagonal),
                _ => None,
            };
            let new_state = match matches.get_one::<String>("CHOOSE").unwrap().as_str() {
                "dead" | "d" => NewState::ChooseDead,
                "alive" | "a" => NewState::ChooseAlive,
                "random" | "r" => NewState::Random,
                _ => NewState::ChooseAlive,
            };
            let max_cell_count = *matches.get_one("MAX").unwrap();
            let max_cell_count = match max_cell_count {
                0 => None,
                i => Some(i),
            };
            let diagonal_width = *matches.get_one("DIAG").unwrap();
            let diagonal_width = match diagonal_width {
                0 => None,
                i => Some(i),
            };
            let reduce_max = matches.get_flag("REDUCE");
            let skip_subperiod = !matches.get_flag("SUBPERIOD");
            let skip_subsymmetry = matches.get_flag("SKIPSUBSYM");
            let backjump = matches.get_flag("BACKJUMP");

            let rule_string = matches.get_one::<String>("RULE").unwrap().to_string();

            config = Config::new(width, height, period)
                .set_translate(dx, dy)
                .set_transform(transform)
                .set_symmetry(symmetry)
                .set_search_order(search_order)
                .set_new_state(new_state)
                .set_max_cell_count(max_cell_count)
                .set_reduce_max(reduce_max)
                .set_rule_string(rule_string)
                .set_diagonal_width(diagonal_width)
                .set_skip_subperiod(skip_subperiod)
                .set_skip_subsymmetry(skip_subsymmetry)
                .set_backjump(backjump);
        }

        let all = matches.get_flag("ALL");
        #[cfg(feature = "tui")]
        let reset = matches.get_flag("RESET");
        #[cfg(feature = "tui")]
        let no_tui = matches.get_flag("NOTUI");

        let world = config
            .world()
            .map_err(|e| app.error(ErrorKind::InvalidValue, format!("Invalid config: {}", e)))?;

        if world.is_gen_rule() && config.backjump {
            return Err(app.error(
                ErrorKind::InvalidValue,
                "Backjumping is not yet supported for Generations rules.",
            ));
        }

        Ok(Self {
            world,
            all,
            #[cfg(feature = "tui")]
            reset,
            #[cfg(feature = "tui")]
            no_tui,
        })
    }
}
