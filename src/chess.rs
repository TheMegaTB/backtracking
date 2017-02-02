extern crate rand;

const SIZE: usize = 6;

fn in_bounds(field: (i32, i32)) -> bool {
    field.0 >= 0 && field.0 < SIZE as i32 && field.1 >= 0 && field.1 < SIZE as i32
}

fn print_board(board: &mut [[i32; SIZE]; SIZE]) {
    println!("");
    println!("");
    for x in board.iter() {
        for y in x.iter() {
            print!("{:^4}", y);
        }
        print!("\n");
    }
}

fn solve_problem(board: &mut [[i32; SIZE]; SIZE], current_field: (i32, i32)) -> bool {
    // print_board(board);
    let current_number = board[current_field.0 as usize][current_field.1 as usize];
    // println!("{} @ {:?}", current_number, current_field);
    if current_number == (SIZE*SIZE) as i32 - 1 {
        return true
    }
    let targets = [
        (current_field.0 - 2, current_field.1 - 1), // Top Left
        (current_field.0 - 2, current_field.1 + 1), // Top Right
        (current_field.0 - 1, current_field.1 + 2), // Right Top
        (current_field.0 + 1, current_field.1 + 2), // Right Bottom
        (current_field.0 + 2, current_field.1 + 1), // Bottom Right
        (current_field.0 + 2, current_field.1 - 1), // Bottom Left
        (current_field.0 + 1, current_field.1 - 2), // Left Bottom
        (current_field.0 - 1, current_field.1 - 2)  // Left Top
    ];
    for target in targets.into_iter() {
        if in_bounds(*target) && board[target.0 as usize][target.1 as usize] == -1 {
            board[target.0 as usize][target.1 as usize] = current_number + 1;
            if solve_problem(board, *target) {
                return true;
            } else {
                board[target.0 as usize][target.1 as usize] = -1;
            }
        }
    }
    board[current_field.0 as usize][current_field.1 as usize] -= 1;
    return false
}

fn main() {
    println!("Hello, world!");

    let mut minus_one = 0;
    minus_one -= 1;

    let mut board = [[minus_one; SIZE]; SIZE];
    board[0][0] = 0;

    println!("Result: {}", solve_problem(&mut board, (0, 0)));
    print_board(&mut board);

}

// let mut i: u8 = 0b0000010;
// i += 0b00010000;
// let x = (i & 0b00010000) >> 4;
// println!("{:b}", x);
