use derive_more::Display;
use rand::seq::SliceRandom;
use rand::thread_rng;
use serde::ser::{SerializeSeq, Serializer};
use serde::{self, Deserialize, Serialize};
use std::fmt;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
#[derive(Debug, Display, PartialEq)]
#[display(fmt = "direction blocked")]
pub struct DirectionBlocked;

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
#[derive(Debug, Clone)]
pub struct Maze {
    player: Position,
    exit: Position,
    size: usize,
    map: Vec<Tile>,
}

#[derive(Debug, Clone)]
struct MazeGenerationTile {
    position: Position,
    link: Position,
    tile_type: Option<TileType>,
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
#[derive(Debug, PartialEq, Serialize)]
pub struct NeighbouringTileTypes {
    left: TileType,
    right: TileType,
    up: TileType,
    down: TileType,
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
#[derive(Debug, Copy, Clone, Serialize)]
pub struct Position {
    pub x: usize,
    pub y: usize,
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Tile {
    tile_type: TileType,
    visibility: TileVisibility,
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
#[derive(Debug, Copy, Clone, Serialize, PartialEq)]
pub enum TileVisibility {
    Hidden,
    Revealed,
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
#[derive(Debug, Copy, Clone, Serialize, PartialEq)]
pub enum TileType {
    Blocked,
    Open,
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
#[derive(Debug, Copy, Clone, Deserialize, Serialize, PartialEq)]
pub enum Direction {
    #[serde(rename = "up")]
    Up,
    #[serde(rename = "down")]
    Down,
    #[serde(rename = "left")]
    Left,
    #[serde(rename = "right")]
    Right,
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
impl Maze {
    pub fn new(size: usize) -> Self {
        let random_map = Maze::generate_random_map(size);

        let mut maze = Maze {
            player: Position { x: 0, y: 0 },
            exit: Position {
                x: size - 1,
                y: size - 1,
            },
            size,
            map: random_map,
        };

        maze.reveal_around_player();
        maze
    }

    fn generate_random_map(size: usize) -> Vec<Tile> {
        fn find(
            size: usize,
            map: &[MazeGenerationTile],
            p: Position,
            q: Position,
        ) -> (Position, Position) {
            let cell_p = map[size * p.y + p.x].link;
            let cell_q = map[size * q.y + q.x].link;

            if p != cell_p || q != cell_q {
                find(size, map, cell_p, cell_q)
            } else {
                (cell_p, cell_q)
            }
        }

        assert_eq!(size % 2, 1, "Random maze only allows odd numbers");

        let mut gen_map = Vec::with_capacity(size * size);

        for i in 0..size {
            for j in 0..size {
                let pos = Position { x: j, y: i };
                gen_map.push(MazeGenerationTile {
                    position: pos,
                    link: pos,
                    tile_type: match (j & 1 == 0, i & 1 == 0) {
                        (true, true) => Some(TileType::Open),
                        (false, false) => Some(TileType::Blocked),
                        (false, true) | (true, false) => None,
                    },
                });
            }
        }

        let mut neither_map = gen_map
            .iter()
            .cloned()
            .filter(|x| match x.tile_type {
                None => true,
                _ => false,
            })
            .collect::<Vec<_>>();

        neither_map.shuffle(&mut thread_rng());

        for i in neither_map {
            let pos = i.position;

            let (p, q) = find(
                size,
                &gen_map,
                if pos.y & 1 == 0 {
                    Position {
                        x: pos.x + 1,
                        y: pos.y,
                    }
                } else {
                    Position {
                        x: pos.x,
                        y: pos.y - 1,
                    }
                },
                if pos.y & 1 == 0 {
                    Position {
                        x: pos.x - 1,
                        y: pos.y,
                    }
                } else {
                    Position {
                        x: pos.x,
                        y: pos.y + 1,
                    }
                },
            );

            if p != q {
                gen_map[size * pos.y + pos.x].tile_type = Some(TileType::Open);
                gen_map[size * p.y + p.x].link = q;
            } else {
                gen_map[size * pos.y + pos.x].tile_type = Some(TileType::Blocked);
            }
        }

        gen_map
            .iter()
            .map(|x| Tile {
                tile_type: x.tile_type.unwrap(),
                visibility: TileVisibility::Hidden,
            })
            .collect::<Vec<_>>()
    }

    fn to_index(&self, x: usize, y: usize) -> usize {
        self.size * y + x
    }

    fn reveal(&mut self, x: usize, y: usize) {
        let i = self.to_index(x, y);
        self.map[i].reveal();
    }

    fn tile_at(&self, x: usize, y: usize) -> Tile {
        self.map[self.to_index(x, y)]
    }

    fn tile_type_at(&self, x: i32, y: i32) -> TileType {
        if x < 0 || y < 0 || x >= self.size as i32 || y >= self.size as i32 {
            TileType::Blocked
        } else {
            self.tile_at(x as usize, y as usize).tile_type
        }
    }

    #[cfg(target_arch = "wasm32")]
    pub fn move_player(&mut self, direction: Direction) -> Result<(), JsValue> {
        self.internal_move_player(direction)
            .map_err(|_| JsValue::from_str("direction blocked"))
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn move_player(&mut self, direction: Direction) -> Result<(), DirectionBlocked> {
        self.internal_move_player(direction)
    }

    fn internal_move_player(&mut self, direction: Direction) -> Result<(), DirectionBlocked> {
        use Direction::*;

        let (x, y) = match direction {
            Up => (self.player.x as i32, self.player.y as i32 - 1),
            Down => (self.player.x as i32, self.player.y as i32 + 1),
            Left => (self.player.x as i32 - 1, self.player.y as i32),
            Right => (self.player.x as i32 + 1, self.player.y as i32),
        };

        if x < 0
            || y < 0
            || (x as usize) >= self.size
            || (y as usize) >= self.size
            || self.tile_at(x as usize, y as usize).tile_type == TileType::Blocked
        {
            return Err(DirectionBlocked);
        }

        self.player = Position {
            x: x as usize,
            y: y as usize,
        };

        self.reveal_around_player();
        Ok(())
    }

    pub fn neighbouring_tile_types(&self) -> NeighbouringTileTypes {
        let player_x = self.player.x as i32;
        let player_y = self.player.y as i32;

        NeighbouringTileTypes {
            left: self.tile_type_at(player_x - 1, player_y),
            right: self.tile_type_at(player_x + 1, player_y),
            up: self.tile_type_at(player_x, player_y - 1),
            down: self.tile_type_at(player_x, player_y + 1),
        }
    }

    pub fn player(&self) -> Position {
        self.player
    }

    fn reveal_around_player(&mut self) {
        self.reveal(self.player.x, self.player.y);
        if self.player.x > 0 {
            self.reveal(self.player.x - 1, self.player.y);
        }
        if self.player.y > 0 {
            self.reveal(self.player.x, self.player.y - 1);
        }
        if self.player.x < self.size - 1 {
            self.reveal(self.player.x + 1, self.player.y);
        }
        if self.player.y < self.size - 1 {
            self.reveal(self.player.x, self.player.y + 1);
        }
    }
}

impl PartialEq for Position {
    fn eq(&self, other: &Self) -> bool {
        self.x == other.x && self.y == other.y
    }
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
impl Tile {
    pub fn open() -> Self {
        Tile {
            tile_type: TileType::Open,
            visibility: TileVisibility::Hidden,
        }
    }

    pub fn blocked() -> Self {
        Tile {
            tile_type: TileType::Blocked,
            visibility: TileVisibility::Hidden,
        }
    }

    pub fn reveal(&mut self) {
        self.visibility = TileVisibility::Revealed
    }

    pub fn is_revealed(self) -> bool {
        self.visibility == TileVisibility::Revealed
    }
}

impl Serialize for Maze {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // To serialise a single row without copying all the elements into a new array.
        struct Row<'a> {
            row_index: usize,
            player: &'a Position,
            exit: &'a Position,
            elements: &'a [Tile],
        }

        impl<'a> Serialize for Row<'a> {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                let mut seq = serializer.serialize_seq(Some(self.elements.len()))?;
                for x in 0..self.elements.len() {
                    if self.player.x == x && self.player.y == self.row_index {
                        seq.serialize_element("player")?;
                    } else if self.exit.x == x && self.exit.y == self.row_index {
                        seq.serialize_element("exit")?;
                    } else {
                        seq.serialize_element(&self.elements[x])?;
                    }
                }
                seq.end()
            }
        }

        let mut seq = serializer.serialize_seq(Some(self.size))?;
        for y in 0..self.size {
            seq.serialize_element(&Row {
                row_index: y,
                player: &self.player,
                exit: &self.exit,
                elements: &self.map[(y * self.size)..(y * self.size) + self.size],
            })?;
        }
        seq.end()
    }
}

impl Serialize for Tile {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if self.is_revealed() {
            match self.tile_type {
                TileType::Open => serializer.serialize_str("open"),
                TileType::Blocked => serializer.serialize_str("blocked"),
            }
        } else {
            serializer.serialize_str("hidden")
        }
    }
}

impl fmt::Display for Direction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Direction::*;
        write!(
            f,
            "{}",
            match self {
                Up => "up",
                Down => "down",
                Left => "left",
                Right => "right",
            }
        )
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use serde_json;

    pub fn maze_from_slice_with_player_at(x: usize, y: usize, map: &[Tile]) -> Maze {
        let size = (map.len() as f64).sqrt() as usize;
        assert_eq!(map.len(), size * size);
        let mut maze = Maze {
            player: Position { x, y },
            exit: Position {
                x: size - 1,
                y: size - 1,
            },
            size,
            map: Vec::from(map),
        };

        maze.reveal_around_player();
        maze
    }

    #[test]
    /// A new map gets created with a backing array of cells of size equal to a square of `size`
    /// sides.
    fn creating_maze_with_size() {
        for size in (1..100).filter(|x| x & 1 != 0) {
            let maze = Maze::new(size);
            assert_eq!(maze.map.len(), size * size);
        }
    }

    #[test]
    /// Start and exit tile are not blocked in a random maze
    fn random_maze_start_and_exit_not_blocked() {
        for size in (1..100).filter(|x| x & 1 != 0) {
            let maze = Maze::new(size);

            let start_tile_type = maze.map[maze.to_index(0, 0)].tile_type;
            let end_tile_type = maze.map[maze.to_index(size - 1, size - 1)].tile_type;

            assert_eq!(start_tile_type, TileType::Open);
            assert_eq!(end_tile_type, TileType::Open);
        }
    }

    #[test]
    /// The map maze should serialize to a 2d array instead of its internal representation.
    fn mazemap_serializes_to_a_2d_array() {
        fn set(maze: &mut Maze, x: usize, y: usize, cell: Tile) {
            let i = maze.to_index(x, y);
            maze.map[i] = cell;
        };

        let test_cases = [
            (3, r#"[["player","hidden","blocked"],["hidden","blocked","blocked"],["blocked","blocked","exit"]]"#),
            (2, r#"[["player","hidden"],["hidden","exit"]]"#),
        ];
        for &(size, expected) in test_cases.into_iter() {
            let mut blocked = Tile::blocked();
            blocked.reveal();
            let mut open = Tile::open();
            open.reveal();
            let mut maze = maze_from_slice_with_player_at(0, 0, &vec![blocked; size * size]);

            set(&mut maze, 0, 0, open);
            set(&mut maze, 1, 0, Tile::blocked());
            set(&mut maze, 0, 1, Tile::open());

            let serialized = serde_json::to_string(&maze).unwrap();
            assert_eq!(serialized.as_str(), expected);
        }
    }
}

#[cfg(test)]
mod neighbouring_tile_types {
    use super::tests::maze_from_slice_with_player_at;
    use super::*;

    fn neighbouring_tile_types_test_setup(
        size: usize,
        player_position: Position,
    ) -> NeighbouringTileTypes {
        let maze = maze_from_slice_with_player_at(
            player_position.x,
            player_position.y,
            &vec![Tile::open(); size * size],
        );
        maze.neighbouring_tile_types()
    }

    #[test]
    /// Checks that negative coordinates given to the neighbouring_tile_types function actually return blocked
    fn when_in_upper_left_corner_up_and_left_are_blocked() {
        let tile_types_actual = neighbouring_tile_types_test_setup(2, Position { x: 0, y: 0 });
        let tile_types_should_be = NeighbouringTileTypes {
            left: TileType::Blocked,
            right: TileType::Open,
            up: TileType::Blocked,
            down: TileType::Open,
        };
        assert_eq!(tile_types_actual, tile_types_should_be);
    }

    #[test]
    /// Checks that the neighbouring_tile_types function returns simply the map given no borders
    fn when_in_middle_all_open() {
        let tile_types_actual = neighbouring_tile_types_test_setup(3, Position { x: 1, y: 1 });
        let tile_types_should_be = NeighbouringTileTypes {
            left: TileType::Open,
            right: TileType::Open,
            up: TileType::Open,
            down: TileType::Open,
        };
        assert_eq!(tile_types_actual, tile_types_should_be);
    }

    #[test]
    /// Checks that coordinates given to the neighbouring_tile_types function that exceeds the size actually return blocked
    fn when_in_bottom_right_corner_down_and_right_are_blocked() {
        let tile_types_actual = neighbouring_tile_types_test_setup(100, Position { x: 99, y: 99 });
        let tile_types_should_be = NeighbouringTileTypes {
            left: TileType::Open,
            right: TileType::Blocked,
            up: TileType::Open,
            down: TileType::Blocked,
        };
        assert_eq!(tile_types_actual, tile_types_should_be);
    }
}

#[cfg(test)]
mod move_player {
    use super::{Direction, DirectionBlocked, Maze, Position, Tile};
    use lazy_static::lazy_static;

    pub fn maze_from_slice_with_player_at(x: usize, y: usize, map: &[Tile]) -> Maze {
        let size = (map.len() as f64).sqrt() as usize;
        assert_eq!(map.len(), size * size);
        Maze {
            player: Position { x, y },
            exit: Position {
                x: size - 1,
                y: size - 1,
            },
            size,
            map: Vec::from(map),
        }
    }

    mod when_player_moves_in {
        use super::*;
        lazy_static! {
            static ref BLOCKED_MAP: [Tile; 9] = [
                Tile::blocked(),
                Tile::blocked(),
                Tile::blocked(),
                Tile::blocked(),
                Tile::open(),
                Tile::blocked(),
                Tile::blocked(),
                Tile::blocked(),
                Tile::blocked(),
            ];
        }

        #[test]
        fn open_direction_up_player_is_moved() {
            let direction = Direction::Up;
            let (x, y) = (1, 1);
            let map = &[Tile::open(); 3 * 3];

            let mut maze = maze_from_slice_with_player_at(x, y, map);
            maze.move_player(direction).unwrap();

            assert_eq!(maze.player.x, 1);
            assert_eq!(maze.player.y, 0);
        }
        #[test]
        fn open_direction_down_player_is_moved() {
            let direction = Direction::Down;
            let (x, y) = (1, 1);
            let map = &[Tile::open(); 3 * 3];

            let mut maze = maze_from_slice_with_player_at(x, y, map);
            maze.move_player(direction).unwrap();

            assert_eq!(maze.player.x, 1);
            assert_eq!(maze.player.y, 2);
        }
        #[test]
        fn open_direction_left_player_is_moved() {
            let direction = Direction::Left;
            let (x, y) = (1, 1);
            let map = &[Tile::open(); 3 * 3];

            let mut maze = maze_from_slice_with_player_at(x, y, map);
            maze.move_player(direction).unwrap();

            assert_eq!(maze.player.x, 0);
            assert_eq!(maze.player.y, 1);
        }
        #[test]
        fn open_direction_right_player_is_moved() {
            let direction = Direction::Right;
            let (x, y) = (1, 1);
            let map = &[Tile::open(); 3 * 3];

            let mut maze = maze_from_slice_with_player_at(x, y, map);
            maze.move_player(direction).unwrap();

            assert_eq!(maze.player.x, 2);
            assert_eq!(maze.player.y, 1);
        }

        #[test]
        fn blocked_direction_up_player_is_moved() {
            let direction = Direction::Up;
            let (x, y) = (1, 1);
            let map = &BLOCKED_MAP[..];

            let mut maze = maze_from_slice_with_player_at(x, y, map);
            let err = maze.move_player(direction);

            assert_eq!(err, Err(DirectionBlocked));
            assert_eq!(maze.player.x, 1);
            assert_eq!(maze.player.y, 1);
        }
        #[test]
        fn blocked_direction_down_player_is_moved() {
            let direction = Direction::Down;
            let (x, y) = (1, 1);
            let map = &BLOCKED_MAP[..];

            let mut maze = maze_from_slice_with_player_at(x, y, map);
            let err = maze.move_player(direction);

            assert_eq!(err, Err(DirectionBlocked));
            assert_eq!(maze.player.x, 1);
            assert_eq!(maze.player.y, 1);
        }
        #[test]
        fn blocked_direction_left_player_is_moved() {
            let direction = Direction::Left;
            let (x, y) = (1, 1);
            let map = &BLOCKED_MAP[..];

            let mut maze = maze_from_slice_with_player_at(x, y, map);
            let err = maze.move_player(direction);

            assert_eq!(err, Err(DirectionBlocked));
            assert_eq!(maze.player.x, 1);
            assert_eq!(maze.player.y, 1);
        }
        #[test]
        fn blocked_direction_right_player_is_moved() {
            let direction = Direction::Right;
            let (x, y) = (1, 1);
            let map = &BLOCKED_MAP[..];

            let mut maze = maze_from_slice_with_player_at(x, y, map);
            let err = maze.move_player(direction);

            assert_eq!(err, Err(DirectionBlocked));
            assert_eq!(maze.player.x, 1);
            assert_eq!(maze.player.y, 1);
        }

        #[test]
        fn edge_direction_up_player_is_moved() {
            let direction = Direction::Up;
            let (x, y) = (0, 0);
            let map = &[Tile::open()];

            let mut maze = maze_from_slice_with_player_at(x, y, map);
            let err = maze.move_player(direction);

            assert_eq!(err, Err(DirectionBlocked));
            assert_eq!(maze.player.x, 0);
            assert_eq!(maze.player.y, 0);
        }
        #[test]
        fn edge_direction_down_player_is_moved() {
            let direction = Direction::Down;
            let (x, y) = (0, 0);
            let map = &[Tile::open()];

            let mut maze = maze_from_slice_with_player_at(x, y, map);
            let err = maze.move_player(direction);

            assert_eq!(err, Err(DirectionBlocked));
            assert_eq!(maze.player.x, 0);
            assert_eq!(maze.player.y, 0);
        }
        #[test]
        fn edge_direction_left_player_is_moved() {
            let direction = Direction::Left;
            let (x, y) = (0, 0);
            let map = &[Tile::open()];

            let mut maze = maze_from_slice_with_player_at(x, y, map);
            let err = maze.move_player(direction);

            assert_eq!(err, Err(DirectionBlocked));
            assert_eq!(maze.player.x, 0);
            assert_eq!(maze.player.y, 0);
        }
        #[test]
        fn edge_direction_right_player_is_moved() {
            let direction = Direction::Right;
            let (x, y) = (0, 0);
            let map = &[Tile::open()];

            let mut maze = maze_from_slice_with_player_at(x, y, map);
            let err = maze.move_player(direction);

            assert_eq!(err, Err(DirectionBlocked));
            assert_eq!(maze.player.x, 0);
            assert_eq!(maze.player.y, 0);
        }
    }
}
