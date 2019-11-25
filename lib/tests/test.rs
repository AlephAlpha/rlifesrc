use rlifesrc_lib::{Config, Status};

#[test]
fn default() {
    let mut search = Config::default().set_world().unwrap();
    assert_eq!(search.search(None), Status::Found);
}

#[test]
fn not_found() {
    let config = Config::new(5, 5, 3);
    let mut search = config.set_world().unwrap();
    assert_eq!(search.search(None), Status::None);
}

#[test]
fn max_cell_count() {
    let config = Config::new(5, 5, 1);
    let mut search = config.set_world().unwrap();
    assert_eq!(search.search(None), Status::Found);
    search.set_max_cell_count(Some(3));
    assert_eq!(search.search(None), Status::None);
}

#[test]
fn p3_spaceship() {
    let config = Config::new(16, 5, 3).set_translate(0, 1);
    let mut search = config.set_world().unwrap();
    assert_eq!(search.search(None), Status::Found);
    assert_eq!(
        search.display_gen(0),
        String::from(
            "........O.......\n\
             .OO.OOO.OOO.....\n\
             .OO....O..OO.OO.\n\
             O..O.OO...O..OO.\n\
             ............O..O\n"
        )
    );
}

#[test]
fn lwss() {
    let config = Config::new(6, 6, 4).set_translate(0, 2);
    let mut search = config.set_world().unwrap();
    assert_eq!(search.search(None), Status::Found);
}
