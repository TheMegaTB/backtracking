extern crate rand;
extern crate term_painter;

use term_painter::*;
use term_painter::Color::*;

use rand::{Rng, ThreadRng};
use std::slice::Iter;

type Field = i32;
type Probability = f32;
type Coordinate = (Field, Field);

type Maze = Vec<Vec<Field>>;
type MazeFrame = Vec<Vec<Cell>>;

const SIZE_X: usize = 25;
const SIZE_Y: usize = 25;

#[derive(Debug, PartialEq)]
enum Cell {
    Blocked,
    Free,
    Exit,
    Start
}

#[derive(Debug)]
enum Direction {
    Top,
    Left,
    Bottom,
    Right
}

impl Direction {
    fn to_byte(self) -> Field {
        // 0b00000001 = Top path is blocked
        // 0b00000010 = Left path is blocked
        // 0b00000100 = Bottom path is blocked
        // 0b00001000 = Right path is blocked
        // 0b10000000 = Exit
        // 0b01000000 = Start
        // 0b00100000 = Blocked
        match self {
            Direction::Top => 0b00000001,
            Direction::Left => 0b00000010,
            Direction::Bottom => 0b00000100,
            Direction::Right => 0b00001000
        }
    }

    fn to_rel_coord(&self) -> Coordinate {
        match *self {
            Direction::Top => (0, -1),
            Direction::Left => (-1, 0),
            Direction::Bottom => (0, 1),
            Direction::Right => (1, 0)
        }
    }

    fn to_number(&self) -> u8 {
        match *self {
            Direction::Top => 0,
            Direction::Left => 1,
            Direction::Bottom => 2,
            Direction::Right => 3
        }
    }

    fn from_number(n: u8) -> Direction {
        match n {
            0 => {Direction::Top},
            1 => {Direction::Left},
            2 => {Direction::Bottom},
            3 => {Direction::Right}
            _ => Direction::Top
        }
    }

    pub fn iterator() -> Iter<'static, Direction> {
        static DIRECTIONS: [Direction;  4] = [Direction::Top, Direction::Left, Direction::Bottom, Direction::Right];
        DIRECTIONS.into_iter()
    }
}

fn add_coords(a: Coordinate, b: Coordinate) -> Coordinate {
    (a.0 + b.0, a.1 + b.1)
}

fn read_bit(x: Field, shift: u8) -> bool {
    ((x & (1 << shift)) >> shift) == 1
}

fn print_blocked(m: &Maze) {
    print_blocked_with_marked(m, &mut Vec::new(), &mut Vec::new());
}

fn print_blocked_with_marked(m: &Maze, path: &mut Vec<Coordinate>, visited: &mut Vec<Coordinate>) {
    print!("\n\n\n");
    for y in 0..SIZE_Y {
        for x in 0..SIZE_X {
            let field = m[x][y];
            if read_bit(field, 6) { print!("{}", Yellow.bg(White).paint("  S  ")) } // Start
            else if read_bit(field, 7) { print!("{}", White.bg(Green).paint("  E  ")) } // Exit
            else if path.contains(&(x as i32, y as i32)) {
                if visited.contains(&(x as i32, y as i32)) {
                    print!("{}", Red.bg(Yellow).paint("  X  "))
                } else {
                    print!("{}", Red.bg(White).paint("  X  "))
                }
            }
            else if visited.contains(&(x as i32, y as i32)) { print!("{}", Yellow.bg(Yellow).paint("  0  ")); }
            else if read_bit(field, 5) { print!("{}", Black.bg(Black).paint("  +  ")); } // Blocked
            else { print!("{}", White.bg(White).paint("  0  ")); } // Else e.g. free
        }
        print!("\n");
    }
}

fn frame_to_maze(frame: MazeFrame) -> Maze {
    let mut maze: Maze = (0..SIZE_X).map(|_| {
        (0..SIZE_Y).map(|_| {
            0b00000000
        }).collect::<Vec<_>>()
    }).collect::<Vec<_>>();

    for x in 0..SIZE_X {
        for y in 0..SIZE_Y {
            match frame[x][y] {
                Cell::Blocked => {
                    maze[x][y] |= 0b00100000;
                    if x+1 < SIZE_X { maze[x+1][y] |= Direction::Left.to_byte()   }
                    if x >= 1       { maze[x-1][y] |= Direction::Right.to_byte()  }
                    if y+1 < SIZE_Y { maze[x][y+1] |= Direction::Top.to_byte()    }
                    if y >= 1       { maze[x][y-1] |= Direction::Bottom.to_byte() }
                },
                Cell::Free => {},
                Cell::Exit => {
                    maze[x][y] += 0b10000000;
                },
                Cell::Start => {
                    maze[x][y] += 0b01000000;
                }
            };
        }
    }

    maze
}

fn get_field<'a>(arr: &'a mut MazeFrame, coord: Coordinate) -> &'a mut Cell {
    &mut arr[coord.0 as usize][coord.1 as usize]
}

fn bound_check(next_field: Coordinate) -> bool {
    (next_field.0 > 0 && next_field.1 > 0) && (SIZE_X > (next_field.0).abs() as usize && SIZE_Y > (next_field.1).abs() as usize)
}

fn calculate_fill_percentage(m: &mut MazeFrame) -> f32 {
    let fields = SIZE_X * SIZE_Y;
    let mut used_fields = 0;
    for x in 0..SIZE_X {
        for y in 0..SIZE_Y {
            if m[x][y] == Cell::Free {
                used_fields += 1;
            }
        }
    }
    used_fields as f32 / fields as f32
}

fn generate(m: &mut MazeFrame, d: &Direction, field: Coordinate, mut rng: ThreadRng, p_fork: (Probability, Probability), is_fork: bool, prev_neighbours: u8) {
    *get_field(m, field) = Cell::Free;

    let neighbour_count = Direction::iterator().fold(0, |acc, dir| {
        let t_field = add_coords(field, dir.to_rel_coord());
        if bound_check(t_field) && m[t_field.0 as usize][t_field.1 as usize] == Cell::Free { acc + 1 } else { acc }
    });

    if !is_fork && prev_neighbours < 2 {
        if rng.gen::<f32>() < p_fork.0 { // Fork left
            let new_dir = Direction::from_number((d.to_number()+1) % 4);
            let next_field = add_coords(field, new_dir.to_rel_coord());
            if bound_check(next_field) { generate(m, &Direction::from_number((d.to_number()+1) % 4), next_field, rng.clone(), p_fork, true, neighbour_count); }
        }
        if rng.clone().gen::<f32>() < p_fork.1 { // Fork right
            let new_dir = Direction::from_number((d.to_number()+3) % 4);
            let next_field = add_coords(field, new_dir.to_rel_coord());
            if bound_check(next_field) { generate(m, &Direction::from_number((d.to_number()+3) % 4), next_field, rng.clone(), p_fork, true, neighbour_count); }
        }
    }

    if !(rng.gen::<f32>() < calculate_fill_percentage(m).powf(2.0)) {
        let next_field = add_coords(field, d.to_rel_coord());
        if bound_check(next_field) { generate(m, d, next_field, rng, p_fork, false, neighbour_count); }
    }
}

fn solve_maze(m: &Maze, field: Coordinate, visited: &mut Vec<Coordinate>, cur_path: &mut Vec<Coordinate>) -> bool {
    if visited.contains(&field) || !bound_check(field) { return false }
    visited.push(field);
    cur_path.push(field);

    print_blocked_with_marked(m, cur_path, visited);
    std::thread::sleep_ms(250);

    let f = m[field.0 as usize][field.1 as usize];
    if read_bit(f, 7) { return true }
    if !read_bit(f, 0) { // Top is free
        if solve_maze(m, add_coords(field, Direction::Top.to_rel_coord()), visited, &mut cur_path.clone()) { return true }
    }
    if !read_bit(f, 1) { // Left is free
        if solve_maze(m, add_coords(field, Direction::Left.to_rel_coord()), visited, &mut cur_path.clone()) { return true }
    }
    if !read_bit(f, 2) { // Bottom is free
        if solve_maze(m, add_coords(field, Direction::Bottom.to_rel_coord()), visited, &mut cur_path.clone()) { return true }
    }
    if !read_bit(f, 3) { // Right is free
        if solve_maze(m, add_coords(field, Direction::Right.to_rel_coord()), visited, &mut cur_path.clone()) { return true }
    }
    false
}

fn main() {
    println!("Hello, world!");

    let mut frame: MazeFrame = (0..SIZE_X).map(|_| {
        (0..SIZE_Y).map(|_| {
            Cell::Blocked
        }).collect::<Vec<_>>()
    }).collect::<Vec<_>>();

    // TODO: Populate maze frame
    let rng = rand::thread_rng();
    let center = ((SIZE_X / 2) as i32, (SIZE_Y / 2) as i32);
    for dir in Direction::iterator() { generate(&mut frame, dir, center, rng.clone(), (0.07, 0.05), true, 4); }
    *get_field(&mut frame, center) = Cell::Start;
    *get_field(&mut frame, (15, (SIZE_Y-1) as i32)) = Cell::Exit;

    let maze = frame_to_maze(frame);
    // print_blocked(&maze);

    println!("Result: {}", solve_maze(&maze, center, &mut Vec::new(), &mut Vec::new()));

}

// let mut i: u8 = 0b0000010;
// i += 0b00010000;
// let x = (i & 0b00010000) >> 4;
// println!("{:b}", x);
