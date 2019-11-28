use rlifesrc_lib::{Config, Status, Symmetry, Transform};

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
    let config = Config::new(5, 5, 1).set_max_cell_count(Some(5));
    let mut search = config.set_world().unwrap();
    assert_eq!(search.search(None), Status::Found);
    search.set_max_cell_count(Some(3));
    assert_eq!(search.search(None), Status::None);
}

#[test]
fn reduce_max() {
    let config = Config::new(5, 5, 1)
        .set_max_cell_count(Some(5))
        .set_reduce_max(true);
    let mut search = config.set_world().unwrap();
    assert_eq!(search.search(None), Status::Found);
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

#[test]
fn lwss_flip() {
    let config = Config::new(5, 5, 2)
        .set_translate(0, 1)
        .set_transform(Transform::FlipCol);
    let mut search = config.set_world().unwrap();
    assert_eq!(search.search(None), Status::Found);
}

#[test]
fn turtle() {
    let config = Config::new(12, 13, 3)
        .set_translate(0, 1)
        .set_symmetry(Symmetry::D2Col);
    let mut search = config.set_world().unwrap();
    assert_eq!(search.search(None), Status::Found);
}
