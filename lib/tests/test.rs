use rlifesrc_lib::{Config, Error, KnownCell, SearchOrder, Status, Symmetry, Transform, ALIVE};

#[test]
fn default() -> Result<(), Error> {
    let mut search = Config::default().world()?;
    assert_eq!(search.search(None), Status::Found);
    Ok(())
}

#[test]
fn not_found() -> Result<(), Error> {
    let config = Config::new(5, 5, 3);
    let mut search = config.world()?;
    assert_eq!(search.search(None), Status::None);
    Ok(())
}

#[test]
fn square_world() -> Result<(), Error> {
    let config = Config::new(10, 9, 1).set_symmetry(Symmetry::D2Diag);
    assert!(config.world().is_err());
    let config = Config::new(10, 10, 1).set_symmetry(Symmetry::D2Diag);
    let mut search = config.world()?;
    assert_eq!(search.search(None), Status::Found);
    Ok(())
}

#[test]
fn max_cell_count() -> Result<(), Error> {
    let config = Config::new(5, 5, 1).set_max_cell_count(Some(5));
    let mut search = config.world()?;
    assert_eq!(search.search(None), Status::Found);
    search.set_max_cell_count(Some(3));
    assert_eq!(search.search(None), Status::None);
    Ok(())
}

#[test]
fn reduce_max() -> Result<(), Error> {
    let config = Config::new(5, 5, 1)
        .set_max_cell_count(Some(5))
        .set_reduce_max(true);
    let mut search = config.world()?;
    assert_eq!(search.search(None), Status::Found);
    assert_eq!(search.search(None), Status::None);
    Ok(())
}

#[test]
fn p3_spaceship() -> Result<(), Error> {
    let config = Config::new(16, 5, 3).set_translate(0, 1);
    let mut search = config.world()?;
    assert_eq!(search.search(None), Status::Found);
    assert_eq!(
        search.rle_gen(0),
        String::from(
            "x = 16, y = 5, rule = B3/S23\n\
             ........o.......$\n\
             .oo.ooo.ooo.....$\n\
             .oo....o..oo.oo.$\n\
             o..o.oo...o..oo.$\n\
             ............o..o!\n"
        )
    );
    Ok(())
}

#[test]
fn backjump() -> Result<(), Error> {
    let config = Config::new(16, 5, 3).set_translate(0, 1).set_backjump(true);
    let mut search = config.world()?;
    assert_eq!(search.search(None), Status::Found);
    assert_eq!(
        search.rle_gen(0),
        String::from(
            "x = 16, y = 5, rule = B3/S23\n\
             ........o.......$\n\
             .oo.ooo.ooo.....$\n\
             .oo....o..oo.oo.$\n\
             o..o.oo...o..oo.$\n\
             ............o..o!\n"
        )
    );
    Ok(())
}

#[test]
fn lwss() -> Result<(), Error> {
    let config = Config::new(6, 6, 4).set_translate(0, 2);
    let mut search = config.world()?;
    assert_eq!(search.search(None), Status::Found);
    Ok(())
}

#[test]
fn lwss_flip() -> Result<(), Error> {
    let config = Config::new(5, 5, 2)
        .set_translate(0, 1)
        .set_transform(Transform::FlipCol);
    let mut search = config.world()?;
    assert_eq!(search.search(None), Status::Found);
    Ok(())
}

#[test]
fn turtle() -> Result<(), Error> {
    let config = Config::new(12, 13, 3)
        .set_translate(0, 1)
        .set_symmetry(Symmetry::D2Col);
    let mut search = config.world()?;
    assert_eq!(search.search(None), Status::Found);
    Ok(())
}

#[test]
fn diagonal() -> Result<(), Error> {
    let config = Config::new(8, 8, 4)
        .set_translate(1, 1)
        .set_search_order(Some(SearchOrder::Diagonal));
    let mut search = config.world()?;
    assert_eq!(search.search(None), Status::Found);
    Ok(())
}

#[test]
fn diagonal_width() -> Result<(), Error> {
    let config = Config::new(10, 10, 4)
        .set_translate(1, 1)
        .set_search_order(Some(SearchOrder::Diagonal))
        .set_diagonal_width(4);
    let mut search = config.world()?;
    assert_eq!(search.search(None), Status::Found);
    Ok(())
}

#[test]
fn b0() -> Result<(), Error> {
    let config = Config::new(3, 3, 2).set_rule_string("B026/S1");
    let mut search = config.world()?;
    assert_eq!(search.search(None), Status::Found);
    Ok(())
}

#[test]
fn p3_2333() -> Result<(), Error> {
    let config = Config::new(4, 4, 3).set_rule_string("23/3/3");
    let mut search = config.world()?;
    assert_eq!(search.search(None), Status::Found);
    Ok(())
}

#[test]
fn snowflakes() -> Result<(), Error> {
    let config = Config::new(4, 4, 2)
        .set_translate(0, 1)
        .set_rule_string("B2ci3ai4c8/S02ae3eijkq4iz5ar6i7e");
    let mut search = config.world()?;
    assert_eq!(search.search(None), Status::Found);
    Ok(())
}

#[test]
fn transform_subperiod() -> Result<(), Error> {
    let config = Config::new(13, 13, 6)
        .set_translate(0, 6)
        .set_rule_string("B2/S")
        .set_transform(Transform::FlipCol)
        .set_skip_subperiod(false);
    let mut search = config.world()?;
    assert_eq!(search.search(None), Status::Found);

    let config = Config::new(13, 13, 6)
        .set_translate(0, 6)
        .set_rule_string("B2/S")
        .set_transform(Transform::FlipCol);
    let mut search = config.world()?;
    assert_eq!(search.search(None), Status::None);
    Ok(())
}

#[test]
fn known_cells() -> Result<(), Error> {
    let config = Config::new(3, 3, 1).set_known_cells(vec![
        KnownCell {
            coord: (0, 0, 0),
            state: ALIVE,
        },
        KnownCell {
            coord: (1, 0, 0),
            state: ALIVE,
        },
        KnownCell {
            coord: (2, 0, 0),
            state: ALIVE,
        },
    ]);
    let mut search = config.world()?;
    assert_eq!(search.search(None), Status::None);
    Ok(())
}

#[test]
#[cfg(feature = "serde")]
fn ser() -> Result<(), Error> {
    let config = Config::new(16, 5, 3).set_translate(0, 1);
    let mut search = config.world()?;
    assert_eq!(search.search(Some(100)), Status::Searching);
    let count = search.cell_count();
    let save = search.ser();
    let mut new_search = save.world()?;
    assert_eq!(new_search.cell_count(), count);
    assert_eq!(new_search.search(None), Status::Found);
    assert_eq!(
        new_search.rle_gen(0),
        String::from(
            "x = 16, y = 5, rule = B3/S23\n\
             ........o.......$\n\
             .oo.ooo.ooo.....$\n\
             .oo....o..oo.oo.$\n\
             o..o.oo...o..oo.$\n\
             ............o..o!\n"
        )
    );
    Ok(())
}
