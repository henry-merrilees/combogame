#![feature(map_many_mut)]
use std::collections::HashMap;

const INITIAL_DICE_SUM: usize = 18;
const BOARD: [Tile; 16] = {
    use Tile::*;
    [
        Go, Wreck, Improve, Sabotage, Duel, Swap, Improve, Wreck, Duel, Sabotage, Improve, Wreck,
        Duel, Swap, Improve, Sabotage,
    ]
};
fn main() {
    let mut players = HashMap::new();

    players.insert(
        "Henry".to_string(),
        Player::new(Die {
            sides: vec![1, 2, 3, 4, 5, 6],
        }),
    );
    players.insert(
        "Kai".to_string(),
        Player::new(Die {
            sides: vec![1, 2, 3, 4, 5, 6],
        }),
    );

    let mut game = Game {
        board: BOARD,
        players,
    };
    loop {
        game.turn();
    }
}

#[derive(Debug)]
struct Die {
    sides: Vec<usize>,
}

impl Die {
    fn roll(&self) -> usize {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let side = rng.gen_range(0..self.sides.len());
        self.sides[side]
    }
}

#[derive(Debug)]
struct Player {
    location: usize,
    die: Die,
}

impl Player {
    fn new(die: Die) -> Player {
        Player { location: 0, die }
    }
}

#[derive(Debug)]
enum Tile {
    Go,
    Wreck,
    Improve,
    Sabotage,
    Swap,
    Duel,
}

#[derive(Debug)]
struct Game {
    board: [Tile; 16],
    players: HashMap<String, Player>,
}

impl Game {
    fn new() -> Game {
        let mut players = HashMap::new();
        loop {
            let mut name = String::new();
            println!("Enter player name: ");
            std::io::stdin().read_line(&mut name).unwrap();
            name = name.trim().to_string();

            let mut dice: Vec<usize>;
            loop {
                println!("Enter dice values: ");
                let mut dice_str = String::new();
                std::io::stdin().read_line(&mut dice_str).unwrap();
                dice = dice_str
                    .split_whitespace()
                    .filter_map(|s| {
                        s.chars()
                            .filter(|c| c.is_numeric())
                            .collect::<String>()
                            .parse()
                            .ok()
                    })
                    .collect();
                dice.sort();

                let dice_sum = dice.iter().sum::<usize>();
                let dice_len = dice.len();

                let sum_check = dice_sum == INITIAL_DICE_SUM;
                let len_check = dice.len() == 6;

                match (sum_check, len_check) {
                    (true, true) => {}
                    (false, true) => {
                        println!("Dice values must sum to {INITIAL_DICE_SUM}, not {dice_sum}");
                        continue;
                    }
                    (true, false) => {
                        println!("Must enter 6 dice values, not {dice_len}");
                        continue;
                    }
                    (false, false) => {
                        println!("Must enter 6 dice values, not {dice_len} and they must sum to {INITIAL_DICE_SUM}, not {dice_sum}");
                        continue;
                    }
                }
                break;
            }

            let die = Die { sides: dice };
            let player = Player::new(die);
            players.insert(name, player);
            println!("Add another player? (y/N)");
            let mut cont = String::new();
            std::io::stdin().read_line(&mut cont).unwrap();
            if cont.trim().to_lowercase() != "y" {
                break;
            }
        }
        Game {
            board: BOARD,
            players,
        }
    }

    fn turn(&mut self) {
        let player_names = self.players.keys().cloned().collect::<Vec<String>>();
        for name in player_names {
            let tile = {
                let player = self.players.get_mut(&name).unwrap();

                let roll = player.die.roll();
                let new_location = (player.location + roll) % 16;
                for _ in 0..(new_location / 16) {
                    println!("{} landed/passed Go", name);
                    let options: Vec<String> =
                        player.die.sides.iter().map(usize::to_string).collect();

                    let selection_index = dialoguer::Select::new()
                        .with_prompt("Select a die face to increase by two")
                        .items(&options)
                        .default(0)
                        .interact()
                        .unwrap();

                    player.die.sides[selection_index] =
                        player.die.sides[selection_index].saturating_add(2);
                    player.die.sides.sort();
                }

                player.location = new_location % 16;

                &self.board[player.location]
            };
            match tile {
                Tile::Go => {
                    println!("{} landed on Go", name);
                }
                Tile::Wreck => {
                    let player = self.players.get_mut(&name).unwrap();
                    println!("{} landed on Wreck", name);
                    // choose one of your own die faces to decrease by two
                    let options: Vec<String> =
                        player.die.sides.iter().map(usize::to_string).collect();

                    let selection_index = dialoguer::Select::new()
                        .with_prompt("Select a die face to decrease by two")
                        .items(&options)
                        .default(0)
                        .interact()
                        .unwrap();

                    player.die.sides[selection_index] =
                        player.die.sides[selection_index].saturating_sub(2);
                    player.die.sides.sort();
                }
                Tile::Improve => {
                    let player = self.players.get_mut(&name).unwrap();
                    println!("{} landed on Improve", name);
                    // choose one of your own die faces to increase by one
                    let options: Vec<String> =
                        player.die.sides.iter().map(usize::to_string).collect();

                    let selection_index = dialoguer::Select::new()
                        .with_prompt("Select a die face to increase by one")
                        .items(&options)
                        .default(0)
                        .interact()
                        .unwrap();

                    // TODO index not value
                    player.die.sides[selection_index] =
                        player.die.sides[selection_index].saturating_add(1);
                    player.die.sides.sort();
                }
                Tile::Sabotage => {
                    // choose an opponent's die face to decrease by one
                    let opponent_name = {
                        let player_names = self.players.keys().cloned().collect::<Vec<String>>();
                        let opponent_names = player_names
                            .iter()
                            .filter(|&n| n != &name)
                            .cloned()
                            .collect::<Vec<String>>();
                        let selection = dialoguer::Select::new()
                            .with_prompt("Select an opponent")
                            .items(&opponent_names)
                            .default(0)
                            .interact()
                            .unwrap();
                        player_names[selection].clone()
                    };

                    let opponent = self.players.get_mut(&opponent_name).unwrap();

                    let options: Vec<String> =
                        opponent.die.sides.iter().map(usize::to_string).collect();

                    let selection_index = dialoguer::Select::new()
                        .with_prompt("Select a die face to decrease by one")
                        .items(&options)
                        .default(0)
                        .interact()
                        .unwrap();

                    // TODO index not value
                    opponent.die.sides[selection_index] =
                        opponent.die.sides[selection_index].saturating_sub(1);
                    opponent.die.sides.sort();
                }
                Tile::Swap => {
                    // choose an opponent to swap a die face with
                    let opponent_name = {
                        let player_names = self.players.keys().cloned().collect::<Vec<String>>();
                        let opponent_names = player_names
                            .iter()
                            .filter(|&n| n != &name)
                            .cloned()
                            .collect::<Vec<String>>();

                        let selection = dialoguer::Select::new()
                            .with_prompt("Select an opponent")
                            .items(&opponent_names)
                            .default(0)
                            .interact()
                            .unwrap();
                        opponent_names[selection].clone() //Check
                    };
                    assert_ne!(name, opponent_name);

                    let player = self.players.get(&name).unwrap();
                    let opponent = self.players.get(&opponent_name).unwrap();

                    let player_options: Vec<String> =
                        player.die.sides.iter().map(usize::to_string).collect();

                    let player_selection_index = dialoguer::Select::new()
                        .with_prompt("Select a face from your die to swap")
                        .items(&player_options)
                        .default(0)
                        .interact()
                        .unwrap();

                    let opponent_options: Vec<String> =
                        opponent.die.sides.iter().map(usize::to_string).collect();

                    let opponent_selection_index = dialoguer::Select::new()
                        .with_prompt("Select a die face from the opponent's die to swap")
                        .items(&opponent_options)
                        .default(0)
                        .interact()
                        .unwrap();

                    let [player_die, opponent_die] =
                        self.players.get_many_mut([&name, &opponent_name]).unwrap();

                    // TODO index not value
                    std::mem::swap(
                        &mut player_die.die.sides[player_selection_index],
                        &mut opponent_die.die.sides[opponent_selection_index],
                    );
                    player_die.die.sides.sort();
                    opponent_die.die.sides.sort();
                }
                Tile::Duel => {
                    println!("{} landed on Duel", name);
                }
            }
        }
    }
}
