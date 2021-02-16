//! Parsing command-line arguments.

use clap::{
    App,
    AppSettings::{AllowNegativeNumbers, ColoredHelp},
    Arg, Error, ErrorKind, Result as ClapResult,
};
use rlifesrc_lib::{rules::NtLifeGen, Config, NewState, Search, SearchOrder, Symmetry, Transform};
use std::{
    fs::File,
    io::{BufReader, Read},
    path::Path,
};

fn is_positive(s: &str) -> bool {
    s.chars().all(|c| c.is_ascii_digit()) && s != "0" && !s.starts_with('-')
}

/// A struct to store the parse results.
pub(crate) struct Args {
    pub(crate) search: Box<dyn Search>,
    pub(crate) all: bool,
    #[cfg(feature = "tui")]
    pub(crate) reset: bool,
    #[cfg(feature = "tui")]
    pub(crate) no_tui: bool,
}

impl Args {
    /// Parses the command-line arguments.
    pub(crate) fn parse() -> ClapResult<Self> {
        let mut app = App::new(env!("CARGO_PKG_NAME"))
            .version(env!("CARGO_PKG_VERSION"))
            .author(env!("CARGO_PKG_AUTHORS"))
            .about(env!("CARGO_PKG_DESCRIPTION"))
            .long_about(
                "Searching for patterns in Conway's Game of Life\n\
                 \n\
                 The program is based on David Bell's lifesrc and Jason Summers's \n\
                 WinLifeSearch, using an algorithm invented by Dean Hickerson.\n\
                 \n\
                 For a detailed help, please visit:\n\
                 https://github.com/AlephAlpha/rlifesrc/blob/master/tui/README.md (In Chinese)\n\
                 https://github.com/AlephAlpha/rlifesrc/blob/master/tui/README_en.md (In English)\n",
            )
            .settings(&[AllowNegativeNumbers, ColoredHelp])
            .arg(
                Arg::with_name("CONFIG")
                    .help("Read config from a file")
                    .long_help(
                        "Read config from a file\n\
                         Supported formats: JSON, YAML, TOML.\n\
                         When a config file is provided, all the other flags and options, \
                         except --all (-a), --reset-time, --no-tui (-n), are ignored.\n",
                    )
                    .short("C")
                    .long("config")
                    .takes_value(true),
            )
            .arg(
                Arg::with_name("X")
                    .help("Width of the pattern")
                    .required_unless("CONFIG")
                    .index(1)
                    .validator(|x| {
                        if is_positive(&x) {
                            Ok(())
                        } else {
                            Err(String::from("width must be a positive integer"))
                        }
                    }),
            )
            .arg(
                Arg::with_name("Y")
                    .help("Height of the pattern")
                    .required_unless("CONFIG")
                    .index(2)
                    .validator(|y| {
                        if is_positive(&y) {
                            Ok(())
                        } else {
                            Err(String::from("height must be a positive integer"))
                        }
                    }),
            )
            .arg(
                Arg::with_name("P")
                    .help("Period of the pattern")
                    .default_value("1")
                    .index(3)
                    .validator(|p| {
                        if is_positive(&p) {
                            Ok(())
                        } else {
                            Err(String::from("period must be a positive integer"))
                        }
                    }),
            )
            .arg(
                Arg::with_name("DX")
                    .help("Horizontal translation")
                    .default_value("0")
                    .index(4)
                    .validator(|d| d.parse::<i32>().map(|_| ()).map_err(|e| e.to_string())),
            )
            .arg(
                Arg::with_name("DY")
                    .help("Vertical translation")
                    .default_value("0")
                    .index(5)
                    .validator(|d| d.parse::<i32>().map(|_| ()).map_err(|e| e.to_string())),
            )
            .arg(
                Arg::with_name("DIAG")
                    .help("Diagonal width")
                    .long_help(
                        "Diagonal width\n\
                         If the diagonal width is n > 0, the cells at position (x, y) \
                         where abs(x - y) >= n are assumed to be dead.\n\
                         If this value is set to 0, it would be ignored.\n",
                    )
                    .short("d")
                    .long("diag")
                    .takes_value(true)
                    .default_value("0")
                    .validator(|d| {
                        if is_positive(&d) || d == "0" {
                            Ok(())
                        } else {
                            Err(String::from("diagonal width must be a positive integer"))
                        }
                    }),
            )
            .arg(
                Arg::with_name("TRANSFORM")
                    .help("Transformation of the pattern")
                    .long_help(
                        "Transformation of the pattern\n\
                         After the last generation in a period, the pattern will return to \
                         the first generation, applying this transformation first, \
                         and then the translation defined by DX and DY.\n\
                         You may need to add quotation marks for some of the transformations.\n\
                         \"Id\" is the identical transformation.\n\
                         \"R\" means counterclockwise rotation.\n\
                         \"F\" means flipping (reflection) across an axis.\n",
                    )
                    .short("t")
                    .long("transform")
                    .takes_value(true)
                    .possible_values(&["Id", "R90", "R180", "R270", "F|", "F-", "F\\", "F/"])
                    .default_value("Id"),
            )
            .arg(
                Arg::with_name("SYMMETRY")
                    .help("Symmetry of the pattern")
                    .long_help(
                        "Symmetry of the pattern\n\
                         You may need to add quotation marks for some of the symmetries.\n\
                         The usages of these symmetries are the same as Oscar Cunningham's \
                         Logic Life Search.\n\
                         See [https://conwaylife.com/wiki/Symmetry] \n",
                    )
                    .short("s")
                    .long("symmetry")
                    .takes_value(true)
                    .possible_values(&[
                        "C1", "C2", "C4", "D2|", "D2-", "D2\\", "D2/", "D4+", "D4X", "D8",
                    ])
                    .default_value("C1"),
            )
            .arg(
                Arg::with_name("RULE")
                    .help("Rule of the cellular automaton")
                    .long_help(
                        "Rule of the cellular automaton\n\
                         Supports Life-like, isotropic non-totalistic, hexagonal, MAP rules, \
                         and their corresponding Generations rules.\n",
                    )
                    .short("r")
                    .long("rule")
                    .takes_value(true)
                    .default_value("B3/S23")
                    .validator(|d| {
                        d.parse::<NtLifeGen>()
                            .map(|_| ())
                            .map_err(|e| e.to_string())
                    }),
            )
            .arg(
                Arg::with_name("ORDER")
                    .help("Search order")
                    .long_help(
                        "Search order\n\
                         Row first or column first.\n",
                    )
                    .short("o")
                    .long("order")
                    .takes_value(true)
                    .possible_values(&[
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
                Arg::with_name("CHOOSE")
                    .help("How to choose a state for unknown cells\n")
                    .short("c")
                    .long("choose")
                    .takes_value(true)
                    .possible_values(&["dead", "alive", "random", "d", "a", "r"])
                    .default_value("alive"),
            )
            .arg(
                Arg::with_name("MAX")
                    .help("Upper bound of numbers of minimum living cells in all generations")
                    .long_help(
                        "Upper bound of numbers of minimum living cells in all generations\n\
                         If this value is set to 0, it means there is no limitation.\n",
                    )
                    .short("m")
                    .long("max")
                    .takes_value(true)
                    .default_value("0")
                    .validator(|d| d.parse::<u32>().map(|_| ()).map_err(|e| e.to_string())),
            )
            .arg(
                Arg::with_name("REDUCE")
                    .help("Reduce the max cell count when a result is found")
                    .long_help(
                        "Reduce the max cell count when a result is found\n\
                         The new max cell count will be set to the cell count of \
                         the current result minus one.\n",
                    )
                    .short("R")
                    .long("reduce")
                    .takes_value(false),
            )
            .arg(
                Arg::with_name("SUBPERIOD")
                    .help("Allow patterns with subperiod")
                    .long_help("Allow patterns whose fundamental period are smaller than the given period")
                    .short("p")
                    .long("subperiod")
                    .takes_value(false),
            )
            .arg(
                Arg::with_name("SKIPSUBSYM")
                    .help("Skip patterns invariant under more transformations than the given symmetry")
                    .long_help(
                        "Skip patterns which are invariant under more transformations than \
                         required by the given symmetry.\n\
                         In another word, skip patterns whose symmetry group properly contains \
                         the given symmetry group.\n",
                    )
                    .short("S")
                    .long("skip-subsym")
                    .takes_value(false),
            )
            .arg(
                Arg::with_name("BACKJUMP")
                    .help("(Experimental) Enable backjumping")
                    .long_help(
                        "(Experimental) Enable backjumping\n\
                        The current implementation of backjumping is very slow, only \
                        useful for large (e.g., 64x64) still lifes.",
                    )
                    .long("backjump")
                    .takes_value(false),
            );

        #[cfg(feature = "tui")]
        {
            app = app
                .arg(
                    Arg::with_name("ALL")
                        .help("Prints all possible results instead of only the first one")
                        .long_help(
                            "Prints all possible results instead of only the first one\n\
                             Only useful when --no-tui is set.\n",
                        )
                        .short("a")
                        .long("all")
                        .requires("NOTUI"),
                )
                .arg(
                    Arg::with_name("RESET")
                        .help("Resets the time when starting a new search")
                        .long("reset-time")
                        .conflicts_with("NOTUI"),
                )
                .arg(
                    Arg::with_name("NOTUI")
                        .help("Starts searching immediately, without entering the TUI")
                        .short("n")
                        .long("no-tui"),
                );
        }

        #[cfg(not(feature = "tui"))]
        {
            app = app.arg(
                Arg::with_name("ALL")
                    .help("Searches for all possible pattern")
                    .long_help("Searches for all possible pattern")
                    .short("a")
                    .long("all"),
            );
        }

        let matches = app.get_matches_safe()?;

        let config;

        if let Some(config_file) = matches.value_of("CONFIG") {
            let path = Path::new(config_file);
            let file = File::open(path)
                .map_err(|e| Error::with_description(&e.to_string(), ErrorKind::Io))?;
            let mut reader = BufReader::new(file);
            match path.extension().and_then(|s| s.to_str()) {
                Some("json") => {
                    config = serde_json::from_reader(reader).map_err(|e| {
                        Error::with_description(
                            &format! {"Invalid config file: {}",e},
                            ErrorKind::Io,
                        )
                    })?;
                }
                Some("yaml") | Some("yml") => {
                    config = serde_yaml::from_reader(reader).map_err(|e| {
                        Error::with_description(
                            &format! {"Invalid config file: {}",e},
                            ErrorKind::Io,
                        )
                    })?;
                }
                Some("toml") => {
                    let mut buf = Vec::new();
                    reader
                        .read_to_end(&mut buf)
                        .map_err(|e| Error::with_description(&e.to_string(), ErrorKind::Io))?;
                    config = toml::from_slice(&buf).map_err(|e| {
                        Error::with_description(
                            &format! {"Invalid config file: {}",e},
                            ErrorKind::Io,
                        )
                    })?;
                }
                _ => {
                    return Err(Error::with_description(
                        "Unsupported config file format",
                        ErrorKind::InvalidValue,
                    ))
                }
            }
        } else {
            let width = matches.value_of("X").unwrap().parse().unwrap();
            let height = matches.value_of("Y").unwrap().parse().unwrap();
            let period = matches.value_of("P").unwrap().parse().unwrap();

            let dx = matches.value_of("DX").unwrap().parse().unwrap();
            let dy = matches.value_of("DY").unwrap().parse().unwrap();

            let transform: Transform = matches.value_of("TRANSFORM").unwrap().parse().unwrap();
            let symmetry: Symmetry = matches.value_of("SYMMETRY").unwrap().parse().unwrap();

            let search_order = match matches.value_of("ORDER").unwrap() {
                "row" | "r" => Some(SearchOrder::RowFirst),
                "column" | "c" => Some(SearchOrder::ColumnFirst),
                "diagonal" | "d" => Some(SearchOrder::Diagonal),
                _ => None,
            };
            let new_state = match matches.value_of("CHOOSE").unwrap() {
                "dead" | "d" => NewState::ChooseDead,
                "alive" | "a" => NewState::ChooseAlive,
                "random" | "r" => NewState::Random,
                _ => NewState::ChooseAlive,
            };
            let max_cell_count = matches.value_of("MAX").unwrap().parse().unwrap();
            let max_cell_count = match max_cell_count {
                0 => None,
                i => Some(i),
            };
            let diagonal_width = matches.value_of("DIAG").unwrap().parse().unwrap();
            let diagonal_width = match diagonal_width {
                0 => None,
                i => Some(i),
            };
            let reduce_max = matches.is_present("REDUCE");
            let skip_subperiod = !matches.is_present("SUBPERIOD");
            let skip_subsymmetry = matches.is_present("SKIPSUBSYM");
            let backjump = matches.is_present("BACKJUMP");

            let rule_string = matches.value_of("RULE").unwrap().to_string();

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

        let all = matches.is_present("ALL");
        #[cfg(feature = "tui")]
        let reset = matches.is_present("RESET");
        #[cfg(feature = "tui")]
        let no_tui = matches.is_present("NOTUI");

        let search = config.world().map_err(|e| {
            Error::with_description(&format! {"Invalid config: {}",e}, ErrorKind::InvalidValue)
        })?;

        if search.is_gen_rule() && config.backjump {
            return Err(Error::with_description(
                "Backjumping is not yet supported for Generations rules.",
                ErrorKind::InvalidValue,
            ));
        }

        Ok(Args {
            search,
            all,
            #[cfg(feature = "tui")]
            reset,
            #[cfg(feature = "tui")]
            no_tui,
        })
    }
}
