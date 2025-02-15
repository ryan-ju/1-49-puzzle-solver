use colored::{Color, Colorize};
use core::fmt;
use std::collections::HashMap;
use std::{rc::Rc, sync::LazyLock};

pub fn run() {
    println!("Hello, world!");
}

pub const BOARD: [&str; 17] = [
    "x0x1x0x2x0x2x0x2x",
    "11111112222222220",
    "x3x1x4x2x4x2x5x2x",
    "03111444445555555",
    "x3x1x4x6x4x5x5x7x",
    "33314446444555570",
    "x3x1x6x6x6x5x7x7x",
    "03333666667777777",
    "x3x8x6x6x6x9x7xax",
    "333888869999777a0",
    "xbx8x8x6x8x9x9xax",
    "0bccc888889999aaa",
    "xbxcx8xcx8xdx9xax",
    "bbbcccccdddd999a0",
    "xbxcxbxcxdxdxaxax",
    "0bbbbbcccddddaaaa",
    "xbx0xbx0xdx0xax0x",
];

pub const SIZE: usize = 17;

pub static PIECE_NAMES: [char; 14] = [
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd',
];

pub static PIECE_COLORS: [Color; 14] = [
    Color::Red,
    Color::Green,
    Color::Yellow,
    Color::Blue,
    Color::Magenta,
    Color::Cyan,
    Color::White,
    Color::BrightRed,
    Color::BrightGreen,
    Color::BrightYellow,
    Color::BrightBlue,
    Color::BrightMagenta,
    Color::BrightCyan,
    Color::BrightWhite,
];

pub type Coordinate = (usize, usize);

#[derive(Debug)]
pub enum Rotation {
    // Rotate clockwise
    R0,
    R90,
    R180,
    R270,
}

impl Rotation {
    // Maps the input coordinates to the output coordinates.
    // Input and output are both (x, y, width, height)
    fn transform(&self, input: (usize, usize, usize, usize)) -> (usize, usize, usize, usize) {
        let (x, y, w, h) = input;
        match self {
            Rotation::R0 => (x, y, w, h),
            Rotation::R90 => (h - y - 1, x, h, w),
            Rotation::R180 => (w - x - 1, h - y - 1, w, h),
            Rotation::R270 => (y, w - x - 1, h, w),
        }
    }

    fn transform_size(&self, input: (usize, usize)) -> (usize, usize) {
        let (w, h) = input;
        match self {
            Rotation::R0 => (w, h),
            Rotation::R90 => (h, w),
            Rotation::R180 => (w, h),
            Rotation::R270 => (h, w),
        }
    }
}

pub static ROTATIONS: [Rotation; 4] = [Rotation::R0, Rotation::R90, Rotation::R180, Rotation::R270];

// Flip horizontally
pub fn flip(input: (usize, usize, usize)) -> (usize, usize) {
    let (x, y, w) = input;
    (w - x - 1, y)
}

#[derive(Debug)]
pub struct Sprite {
    // true if the cell is non-empty
    value: Vec<Vec<bool>>,
    // Array coordinates that are non-empty.  This should improve performance.
    value_coordinates: Vec<Coordinate>,
    // The index of the first non-empty cell in the first row
    top_left: usize,
    rotation: &'static Rotation,
    flipped: bool, // Flip is applied after rotation
}

pub fn eq_sprites(a: &[Vec<bool>], b: &[Vec<bool>]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    a.iter()
        .zip(b.iter())
        .all(|(a, b)| a.len() == b.len() && a.iter().zip(b.iter()).all(|(a, b)| *a == *b))
}

#[derive(Debug)]
pub struct Piece {
    pub name: char,
    pub color: Color,
    // Variants are the pieces with different orientations.  Note the variants are unique in shape.
    pub variants: Vec<Sprite>,
}

impl fmt::Display for Piece {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for (i, variant) in self.variants.iter().enumerate() {
            writeln!(f, "Variant: {}", i)?;
            writeln!(
                f,
                "Rotation: {:?}, flipped: {}, top-left: {}",
                variant.rotation, variant.flipped, variant.top_left
            )?;
            for row in &variant.value {
                for cell in row {
                    write!(
                        f,
                        "{}",
                        if *cell {
                            self.name.to_string().color(self.color)
                        } else {
                            ".".color(self.color).black()
                        }
                    )?;
                }
                writeln!(f)?;
            }
            writeln!(f)?;
            writeln!(f)?;
        }

        Ok(())
    }
}

fn create_new_variant(
    value: &[Vec<bool>],
    rotation: &Rotation,
    flipped: bool,
) -> (Vec<Vec<bool>>, Vec<Coordinate>, usize) {
    let w = value[0].len();
    let h = value.len();

    let mut top_left: usize = usize::MAX;
    let (w_new, h_new) = rotation.transform_size((w, h));
    let mut value_new: Vec<Vec<bool>> = vec![vec![false; w_new]; h_new];
    let mut value_coordinates: Vec<Coordinate> = vec![];
    for i in 0..w {
        for (j, row) in value.iter().enumerate() {
            let (i_new, j_new, _, _) = rotation.transform((i, j, w, h));
            let (i_new, j_new) = if flipped {
                flip((i_new, j_new, w_new))
            } else {
                (i_new, j_new)
            };
            let v = row[i];
            value_new[j_new][i_new] = v;
            if v && j_new == 0 && i_new < top_left {
                top_left = i_new;
            }
            if v {
                value_coordinates.push((i_new, j_new));
            }
        }
    }

    (value_new, value_coordinates, top_left)
}

pub fn extract_piece_from_board(idx: char, no_variants: bool) -> Piece {
    struct Bound {
        left: usize,
        right: usize, // Exclusive
        top: usize,
        bottom: usize, // Exclusive
    }
    let bound: Bound = BOARD.iter().enumerate().fold(
        Bound {
            left: usize::MAX,
            right: 0,
            top: usize::MAX,
            bottom: 0,
        },
        |mut acc: Bound, (i, line)| {
            let left = line.chars().take_while(|&c| c != idx).count();
            if left < acc.left {
                acc.left = left;
            }
            let right = SIZE - line.chars().rev().take_while(|&c| c != idx).count();
            if right > acc.right {
                acc.right = right;
            }
            if left != SIZE {
                if acc.top == usize::MAX {
                    acc.top = i;
                }
                acc.bottom = i + 1;
            }
            acc
        },
    );

    // Extract the sprite
    let mut value: Vec<Vec<bool>> = vec![];
    let mut value_coordinates: Vec<Coordinate> = vec![];
    for (j, row) in BOARD.iter().enumerate().take(bound.bottom).skip(bound.top) {
        value.push(
            row[bound.left..bound.right]
                .chars()
                .map(|c| c == idx)
                .collect(),
        );
        for (i, c) in row[bound.left..bound.right].chars().enumerate() {
            if c == idx {
                value_coordinates.push((i, j));
            }
        }
    }

    let color = PIECE_COLORS[PIECE_NAMES.iter().position(|&c| c == idx).unwrap()];

    if no_variants {
        return Piece {
            name: idx,
            color,
            variants: vec![Sprite {
                value,
                value_coordinates,
                top_left: 0,
                rotation: &ROTATIONS[0],
                flipped: false,
            }],
        };
    }

    // Create the variants
    let mut variants: Vec<Sprite> = vec![];
    for rotation in ROTATIONS.iter() {
        'outer: for flipped in [false, true] {
            let (value_new, value_coordinates, top_left) =
                create_new_variant(&value, rotation, flipped);

            for variant in &variants {
                if eq_sprites(&value_new, &variant.value) {
                    continue 'outer;
                }
            }

            variants.push(Sprite {
                value: value_new,
                value_coordinates,
                top_left,
                rotation,
                flipped,
            });
        }
    }

    Piece {
        name: idx,
        color,
        variants,
    }
}

// The board's teeth piece.  Obviously there's only one variant.
pub static PIECE_ZERO: LazyLock<Piece> = LazyLock::new(|| extract_piece_from_board('0', true));

// This excludes PIECE_ZERO.
pub static PIECES_OTHER: LazyLock<Vec<Piece>> = LazyLock::new(|| {
    PIECE_NAMES
        .iter()
        .skip(1)
        .map(|idx| extract_piece_from_board(*idx, false))
        .collect()
});

pub static PIECE_MAP: LazyLock<HashMap<char, &'static Piece>> = LazyLock::new(|| {
    let mut map: HashMap<char, &'static Piece> = HashMap::new();
    map.insert(PIECE_ZERO.name, &PIECE_ZERO);
    for piece in PIECES_OTHER.iter() {
        map.insert(piece.name, piece);
    }
    map
});

pub struct BoardPiece {
    pub name: char,
    pub sprite: &'static Sprite,
    // The position of the top-left non-empty corner of the piece on the board
    pub anchor: Coordinate,
}

fn number_to_coordinate(number: u8) -> Coordinate {
    let n = (number - 1) as usize;
    ((n % 7 + 1) * 2, (n / 7 + 1) * 2)
}

pub struct BoardState {
    pub pieces: Vec<Rc<BoardPiece>>,
    pub state: Vec<Vec<char>>,
    // The next empty position.  The next piece's top-left corner will be placed here.
    pub anchor: Coordinate,
    pub pieces_to_place: Vec<char>,
}

impl fmt::Display for BoardState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for row in &self.state {
            for cell in row {
                write!(
                    f,
                    "{} ",
                    if *cell == 'x' || *cell == '.' {
                        cell.to_string().black()
                    } else {
                        cell.to_string().color(PIECE_MAP[cell].color)
                    }
                )?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

impl BoardState {
    pub fn new(target: u8) -> Vec<Self> {
        // Initialise the state
        let mut state = vec![vec!['x'; 17]; 17];
        for (row, value) in BOARD.iter().enumerate() {
            for (col, c) in value.chars().enumerate() {
                if c == '0' {
                    state[row][col] = '0';
                } else if c != 'x' {
                    state[row][col] = '.';
                }
            }
        }
        let pieces: Vec<Rc<BoardPiece>> = vec![];
        let target_coordinate = number_to_coordinate(target);
        let target_piece: &'static Piece = PIECE_MAP[&'d'];
        let mut result: Vec<Self> = vec![];

        let anchors: Vec<Coordinate> = vec![
            // The top-left corner of the target piece, off set from the target coordinate
            (target_coordinate.0 + 1, target_coordinate.1 - 2),
            (target_coordinate.0 - 1, target_coordinate.1 - 2),
        ];

        for (i, anchor) in anchors.iter().enumerate() {
            if let Ok(board_state) = (BoardState {
                pieces: pieces.clone(),
                state: state.clone(),
                anchor: (anchor.0, anchor.1),
                // Note this excludes the 0th and the last piece (i.e., the target piece)
                pieces_to_place: (1..=12).map(|x| PIECE_NAMES[x]).collect(),
            }
            .place_piece(target_piece, i))
            {
                result.push(board_state);
            }
        }

        result
    }

    pub fn place_piece(
        &self,
        piece: &'static Piece,
        variant_index: usize,
    ) -> Result<BoardState, ()> {
        let sprite: &Sprite = &piece.variants[variant_index];

        let mut state_new = self.state.clone();

        if self.detect_overlap(&mut state_new, piece.name, sprite) {
            return Err(());
        }

        let mut pieces_new = self.pieces.clone();
        pieces_new.push(Rc::new(BoardPiece {
            name: piece.name,
            sprite,
            anchor: self.anchor,
        }));

        let mut pieces_to_place_new = self.pieces_to_place.clone();
        pieces_to_place_new.retain(|&x| x != piece.name);

        if pieces_to_place_new.is_empty() {
            return Ok(BoardState {
                pieces: pieces_new,
                state: state_new,
                // Anchor is not important, as the puzzle is already solved
                anchor: self.anchor,
                pieces_to_place: pieces_to_place_new,
            });
        }

        // Find the next anchor.  Note if it's the target piece, we need to start from the top-left corner of the board.
        let mut anchor_new = (0, 0);
        loop {
            anchor_new.0 += 1;
            if anchor_new.0 >= SIZE {
                anchor_new.0 = 0;
                anchor_new.1 += 1;
                if anchor_new.1 >= SIZE {
                    // We have reached the end of the board, which is impossible and is a bug
                    panic!("The board is full, but the puzzle is not solved");
                }
            }
            if state_new[anchor_new.1][anchor_new.0] == '.' {
                break;
            }
        }

        Ok(BoardState {
            pieces: pieces_new,
            state: state_new,
            anchor: anchor_new,
            pieces_to_place: pieces_to_place_new,
        })
    }

    #[cfg(not(feature = "fast"))]
    fn detect_overlap(
        &self,
        state: &mut Vec<Vec<char>>,
        piece_name: char,
        sprite: &Sprite,
    ) -> bool {
        for j in 0..sprite.value.len() {
            for i in 0..sprite.value[j].len() {
                if sprite.value[j][i] {
                    let new_x = self.anchor.0 + i;
                    if new_x < sprite.top_left {
                        return true;
                    }
                    let x = self.anchor.0 + i - sprite.top_left;
                    let y = self.anchor.1 + j;
                    if x >= SIZE || y >= SIZE || self.state[y][x] != '.' {
                        return true;
                    }
                    state[y][x] = piece_name;
                }
            }
        }
        false
    }

    #[cfg(feature = "fast")]
    fn detect_overlap(&self, state: &mut [Vec<char>], piece_name: char, sprite: &Sprite) -> bool {
        // This should run faster, as we don't check the empty cells in the sprite
        for (i, j) in &sprite.value_coordinates {
            let new_x = self.anchor.0 + i;
            if new_x < sprite.top_left {
                return true;
            }
            let x = self.anchor.0 + i - sprite.top_left;
            let y = self.anchor.1 + j;
            if x >= SIZE || y >= SIZE || self.state[y][x] != '.' {
                return true;
            }
            state[y][x] = piece_name;
        }
        false
    }
}

// To log after this many states are computed
const LOG_FREQUENCY: u64 = 100000;

pub fn solve(target: u8) {
    let mut states = BoardState::new(target);

    let mut counter: u64 = 0;

    // Depth-first search algorithm
    while !states.is_empty() {
        let state: &BoardState = &(states.pop().unwrap());
        if state.pieces_to_place.is_empty() {
            println!("Solution found:");
            println!("{}", state);
            return;
        }

        for piece_index in state.pieces_to_place.iter() {
            let piece = PIECE_MAP[piece_index];
            for i in 0..piece.variants.len() {
                if let Ok(new_state) = state.place_piece(piece, i) {
                    states.push(new_state);
                    counter += 1;

                    if counter % LOG_FREQUENCY == 0 {
                        println!(
                            "# states searched: {}, stack size: {}",
                            counter,
                            states.len()
                        );
                    }
                }
            }
        }
    }

    println!("No solution found");
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_number_to_coordinate() {
        assert_eq!(number_to_coordinate(1), (2, 2));
        assert_eq!(number_to_coordinate(10), (6, 4));
        assert_eq!(number_to_coordinate(49), (14, 14));
    }

    #[test]
    fn test_print_piece() {
        print!("Piece: {}", PIECES_OTHER[4])
    }

    #[test]
    fn test_create_new_variant() {
        let value: &Vec<Vec<bool>> = &PIECE_MAP[&'d'].variants[0].value;
        let (value_new, _, _) = create_new_variant(&value, &Rotation::R90, false);

        assert_eq!(eq_sprites(value, &value_new), true);
    }

    #[test]
    fn test_extract_piece_from_board() {
        let piece = extract_piece_from_board('d', false);
        print!("Piece: {}", piece);
    }
}
