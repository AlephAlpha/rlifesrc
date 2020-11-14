# [rlifesrc](https://github.com/AlephAlpha/rlifesrc)

__Rust Life Search__, or __rlifesrc__, is a Game of Life pattern searcher written in Rust.

The program is based on David Bell's [lifesrc](https://github.com/DavidKinder/Xlife/tree/master/Xlife35/source/lifesearch) and Jason Summers's [WinLifeSearch](https://github.com/jsummers/winlifesearch/), using [an algorithm invented by Dean Hickerson](https://github.com/DavidKinder/Xlife/blob/master/Xlife35/source/lifesearch/ORIGIN).

Compared to WinLifeSearch, rlifesrc is still slower, and lacks many important features. But it supports non-totalistic Life-like and Generations rules. Supports for Generations rules are experimental.

## Usage

This algorithm is suitable for long and thin or flat and wide patterns, but it can also search for other patterns.

First set up height, width, period, translation and other parameters in the `Setting` tab. Then click `Apply settings` to apply these parameters. Then click `Start` to start searching.

When a result is found, you can click `Start` again to search for the next result, or click `Reset` to reset the world.

It may takes a very long time to find a result. You can click `Save` to save the current search status in a JSON file, and click `Load` to load a saved status.

The result is printed in a mix of [Plaintext](https://conwaylife.com/wiki/Plaintext) and [RLE](https://conwaylife.com/wiki/Rle) format. Specifically:

* **Dead** cells are represented by `.`;
* **Living** cells are represented by `o` for rules with 2 states,
  `A` for rules with more states;
* **Dying** cells are represented by uppercase letters starting from `B`;
* **Unknown** cells are represented by `?`;
* Each line is ended with `$`;
* The whole pattern is ended with `!`.

Currently it cannot properly display Generations rules with more than 25 states.

You can click the `+`/`-` sign next to `Generation` to increase/decrease the displayed generation.

`Cells` means the number of known living cells in the current generation. For Generations rules, dying cells are not counted.

## Settings

### Rule

Rule of the cellular automaton.

Supports Life-like, isotropic non-totalistic, hexagonal, MAP rules, and their corresponding Generations rules.

### Width

Width of the pattern.

### Height

Height of the pattern.

### Period

Period of the pattern.

### dx

Horizontal translation.

### dy

Vertical translation.

### Transformation

Transformation of the pattern.

After the last generation in a period, the pattern will return to the first generation, applying this transformation first, and then the translation defined by `dx` and `dy`.

8 different transformations correspond to the 10 elements of the dihedral group _D_<sub>8</sub>. Here:

* `Id` means the identity transformation.
* `R` means rotations around the center of the world. The number after it is the counterclockwise rotation angle in degrees.
* `F` means reflections (flips). The symbol after it is the axis of reflection.

For example, if you want to find a vertical spaceship with [glide symmetric](https://conwaylife.com/wiki/Types_of_spaceships#Glide_symmetric_spaceship), you can set the transformation to `F|`.

Some transformations require that the world is square.

### Symmetry

Symmetry of the pattern.

10 different symmetries correspond to the 10 subgroups of the dihedral group _D_<sub>8</sub>. The notations are stolen from Oscar Cunningham's [Logic Life Search](https://github.com/OscarCunningham/logic-life-search). Please see the [Life Wiki](https://conwaylife.com/wiki/Symmetry) for details.

Some symmetries require that the world is square.

### Max cell count

Upper bound of numbers of minimum living cells in all generations.

If this value is set to 0, it means there is no limitation.

### Search order

The order to find a new unknown cell.

It will always search all generations of one cell before going to another cell.

`Automatic` means that it will start from the shorter side, i.e., start from the columns if there are more columns than rows, from the rows if there are more rows than columns. It will never choose the diagonal search order.

Diagonal search order requires  that the world is square.

### Choice of state for unknown cells

How to choose a state for unknown cells.

`Random` might work better for oscillators.

### Non empty front

Force the first row or column to be nonempty

Here "front" means the first row or column to be searched, according to the search order.

### Reduce max cell count

Reduce the `Max cell count` when a result is found.

The new `Max cell count` will be set to the cell count of the current result minus one.
