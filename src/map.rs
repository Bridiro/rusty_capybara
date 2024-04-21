#[allow(dead_code)]
pub mod map {
    use std::collections::HashMap;
    use std::io::{self, BufRead};

    type Position = (i32, i32);

    #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
    enum Direction {
        Up,
        Down,
        Left,
        Right,
    }

    impl Direction {
        fn right(&self) -> Direction {
            match *self {
                Direction::Up => Direction::Right,
                Direction::Down => Direction::Left,
                Direction::Left => Direction::Up,
                Direction::Right => Direction::Down,
            }
        }

        fn left(&self) -> Direction {
            match *self {
                Direction::Up => Direction::Left,
                Direction::Down => Direction::Right,
                Direction::Left => Direction::Down,
                Direction::Right => Direction::Up,
            }
        }

        fn back(&self) -> Direction {
            match *self {
                Direction::Up => Direction::Down,
                Direction::Down => Direction::Up,
                Direction::Left => Direction::Right,
                Direction::Right => Direction::Left,
            }
        }
    }

    #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
    enum Kind {
        Start,
        Unknown,
        Empty,
        Checkpoint,
        Blue,
        Victim,
        Ramp,
        Black,
    }

    struct Cell {
        pos: Position,
        kind: Kind,
        neighbors: HashMap<Direction, Position>,
    }

    impl Cell {
        fn new(pos: Position, kind: Kind) -> Cell {
            Cell {
                pos: pos,
                kind: kind,
                neighbors: HashMap::new(),
            }
        }

        fn add_neighbor(&mut self, direction: Direction, neighbor_pos: Position) {
            if self.neighbors.contains_key(&direction) {
                return;
            }
            self.neighbors.insert(direction, neighbor_pos);
        }
    }

    pub struct Maze {
        dir: Direction,
        pos: Position,
        last_checkpoint: Position,
        cells: HashMap<Position, Cell>,
        path: Vec<Position>,
    }

    impl Maze {
        pub fn new() -> Maze {
            let mut cells = HashMap::new();
            cells.insert((0, 0), Cell::new((0, 0), Kind::Start));
            Maze {
                dir: Direction::Up,
                pos: (0, 0),
                last_checkpoint: (0, 0),
                cells,
                path: vec![],
            }
        }

        fn coordinate_to_direction(&self, pos: Position) -> Direction {
            if pos.0 == self.pos.0 && pos.1 == self.pos.1 - 1 {
                return Direction::Up;
            } else if pos.0 == self.pos.0 && pos.1 == self.pos.1 + 1 {
                return Direction::Down;
            } else if pos.0 == self.pos.0 - 1 && pos.1 == self.pos.1 {
                return Direction::Left;
            } else if pos.0 == self.pos.0 + 1 && pos.1 == self.pos.1 {
                return Direction::Right;
            }
            panic!("Invalid position!");
        }

        fn get_direction(&mut self) -> Option<Direction> {
            if !self.path.is_empty() {
                let coordinate = self.path.remove(0);
                return Some(self.coordinate_to_direction(coordinate));
            }
            let right_direction = self.dir.right();
            let straight_direction = self.dir;
            let left_direction = self.dir.left();
            let back_direction = self.dir.back();

            if let Some(cell) = self.cells.get(&self.pos) {
                if let Some(right_neighbor_pos) = cell.neighbors.get(&right_direction) {
                    if let Some(right_neighbor_cell) = self.cells.get(right_neighbor_pos) {
                        if right_neighbor_cell.kind == Kind::Unknown {
                            return Some(right_direction);
                        }
                    }
                }
                if let Some(straight_neighbor_pos) = cell.neighbors.get(&straight_direction) {
                    if let Some(straight_neighbor_cell) = self.cells.get(straight_neighbor_pos) {
                        if straight_neighbor_cell.kind == Kind::Unknown {
                            return Some(straight_direction);
                        }
                    }
                }
                if let Some(left_neighbor_pos) = cell.neighbors.get(&left_direction) {
                    if let Some(left_neighbor_cell) = self.cells.get(left_neighbor_pos) {
                        if left_neighbor_cell.kind == Kind::Unknown {
                            return Some(left_direction);
                        }
                    }
                }
                if let Some(back_neighbor_pos) = cell.neighbors.get(&back_direction) {
                    if let Some(back_neighbor_cell) = self.cells.get(back_neighbor_pos) {
                        if back_neighbor_cell.kind == Kind::Unknown {
                            return Some(back_direction);
                        }
                    }
                }
            }

            if let Some(path) = self.bfs(Kind::Unknown) {
                self.path = path;
                let coordinate = self.path.remove(0);
                Some(self.coordinate_to_direction(coordinate))
            } else {
                None
            }
        }

        fn bfs(&mut self, tar: Kind) -> Option<Vec<Position>> {
            let mut queue = vec![self.pos];
            let mut visited = vec![self.pos];
            let mut parent = HashMap::new();
            let mut found = false;
            let mut target = (0, 0);
            while !queue.is_empty() {
                let current = queue.remove(0);
                if let Some(cell) = self.cells.get(&current) {
                    if cell.kind == tar {
                        found = true;
                        target = current;
                        break;
                    }
                    for neighbor in cell.neighbors.values() {
                        if !visited.contains(neighbor) {
                            if let Some(neighbor_cell) = self.cells.get(neighbor) {
                                if neighbor_cell.kind != Kind::Black {
                                    queue.push(*neighbor);
                                    visited.push(*neighbor);
                                    parent.insert(*neighbor, current);
                                }
                            }
                        }
                    }
                }
            }

            let mut path = vec![];
            if found {
                let mut current = target;
                while current != self.pos {
                    path.push(current);
                    current = parent[&current];
                }
                path.reverse();
            }

            if path.len() > 0 {
                println!("path: {:?} to: {:?}", path, tar);
                return Some(path);
            } else {
                if let Some(path) = self.bfs(Kind::Start) {
                    println!("path: {:?} to: {:?}", path, tar);
                    if path.len() > 0 {
                        return Some(path);
                    } else {
                        return None;
                    }
                } else {
                    return None;
                }
            }
        }

        fn move_one(&mut self) -> Option<Direction> {
            if let Some(direction) = self.get_direction() {
                self.dir = direction;

                match direction {
                    Direction::Up => self.pos.1 -= 1,
                    Direction::Down => self.pos.1 += 1,
                    Direction::Left => self.pos.0 -= 1,
                    Direction::Right => self.pos.0 += 1,
                }
                if let Some(cell) = self.cells.get_mut(&self.pos) {
                    if cell.kind == Kind::Unknown {
                        cell.kind = Kind::Empty;
                    }
                }
                Some(direction)
            } else {
                None
            }
        }

        fn add_cell(&mut self, direction: Direction) {
            let cell_pos = match direction {
                Direction::Up => (self.pos.0, self.pos.1 - 1),
                Direction::Down => (self.pos.0, self.pos.1 + 1),
                Direction::Left => (self.pos.0 - 1, self.pos.1),
                Direction::Right => (self.pos.0 + 1, self.pos.1),
            };

            if let Some(cell) = self.cells.get_mut(&cell_pos) {
                if !cell.neighbors.contains_key(&direction.back()) {
                    cell.add_neighbor(direction.back(), self.pos);
                    let current_cell = self.cells.get_mut(&self.pos).unwrap();
                    current_cell.add_neighbor(direction, cell_pos);
                }
            } else {
                if cell_pos.0 < 0 {
                    self.cells.iter_mut().for_each(|(_, cell)| {
                        cell.pos.0 += 1;
                    });
                } else if cell_pos.1 < 0 {
                    self.cells.iter_mut().for_each(|(_, cell)| {
                        cell.pos.1 += 1;
                    });
                }
                let mut new_cell = Cell::new(cell_pos, Kind::Unknown);
                self.cells
                    .get_mut(&self.pos)
                    .unwrap()
                    .add_neighbor(direction, cell_pos);
                new_cell.add_neighbor(direction.back(), self.pos);
                self.cells.insert(cell_pos, new_cell);
            }
        }

        pub fn add_checkpoint(&mut self) {
            self.cells.get_mut(&self.pos).unwrap().kind = Kind::Checkpoint;
            self.last_checkpoint = self.pos;
        }

        pub fn add_victim(&mut self) {
            self.cells.get_mut(&self.pos).unwrap().kind = Kind::Victim;
        }

        pub fn add_ramp(&mut self) {
            self.cells.get_mut(&self.pos).unwrap().kind = Kind::Ramp;
        }

        pub fn add_blue(&mut self) {
            self.cells.get_mut(&self.pos).unwrap().kind = Kind::Blue;
        }

        pub fn add_black(&mut self) {
            self.cells.get_mut(&self.pos).unwrap().kind = Kind::Black;
            match self.dir {
                Direction::Up => self.pos.1 += 1,
                Direction::Down => self.pos.1 -= 1,
                Direction::Left => self.pos.0 += 1,
                Direction::Right => self.pos.0 -= 1,
            }
        }

        pub fn lack_of_progress(&mut self) {
            self.pos = self.last_checkpoint;
        }

        pub fn print_maze(&self) {
            let mut min_x = std::i32::MAX;
            let mut max_x = std::i32::MIN;
            let mut min_y = std::i32::MAX;
            let mut max_y = std::i32::MIN;

            for (pos, _) in &self.cells {
                min_x = min_x.min(pos.0);
                max_x = max_x.max(pos.0);
                min_y = min_y.min(pos.1);
                max_y = max_y.max(pos.1);
            }

            for y in (min_y - 1)..=(max_y + 1) {
                for x in (min_x - 1)..=(max_x + 1) {
                    let pos = (x, y);
                    let cell = self.cells.get(&pos);

                    let symbol = match cell {
                        Some(cell) => match cell.kind {
                            Kind::Start => 'S',
                            Kind::Unknown => '?',
                            Kind::Empty => ' ',
                            Kind::Checkpoint => 'C',
                            Kind::Blue => 'B',
                            Kind::Victim => 'V',
                            Kind::Ramp => 'R',
                            Kind::Black => '█',
                        },
                        None => '.',
                    };

                    if pos == self.pos {
                        let arrow = match self.dir {
                            Direction::Up => '^',
                            Direction::Down => 'v',
                            Direction::Left => '<',
                            Direction::Right => '>',
                        };
                        print!("{} ", arrow);
                    } else if self.path.contains(&(x, y)) {
                        print!("* ");
                    } else {
                        print!("{} ", symbol);
                    }
                }
                println!();
            }
        }

        pub fn test_mapping() {
            let mut maze = Maze::new();
            maze.print_maze();

            loop {
                println!("Enter the new cells around you (U/D/L/R):");
                let stdin = io::stdin();
                let input = stdin.lock().lines().next().unwrap().unwrap();
                let input = input.trim().to_uppercase();

                for c in input.chars() {
                    match c {
                        'U' => {
                            maze.add_cell(Direction::Up);
                        }
                        'D' => {
                            maze.add_cell(Direction::Down);
                        }
                        'L' => {
                            maze.add_cell(Direction::Left);
                        }
                        'R' => {
                            maze.add_cell(Direction::Right);
                        }
                        _ => {
                            println!("Invalid input! Please enter U/D/L/R.");
                            break;
                        }
                    }
                }

                if let Some(dir) = maze.move_one() {
                    println!("Moved {:?}", dir);
                } else {
                    println!("Labirinth solved!");
                    break;
                }
                maze.print_maze();

                println!("Is the next cell black? (Y/N):");
                let stdin = io::stdin();
                let input = stdin.lock().lines().next().unwrap().unwrap();
                let input = input.trim().to_uppercase();

                match input.as_str() {
                    "Y" => maze.add_black(),
                    "N" => (),
                    _ => {
                        println!("Invalid input! Please enter Y/N.");
                        break;
                    }
                }

                maze.print_maze();
            }
            maze.print_maze();
        }
    }
}
