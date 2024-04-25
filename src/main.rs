#![feature(map_many_mut)]
use std::collections::HashMap;

const INITIAL_DICE_SUM: usize = 18;
const BOARD: [Tile; 16] = {
    use Tile::*;
    [
        Go,
        Wreck,
        Improve,
        Sabotage,
        SumDuel,
        Swap,
        Improve,
        Wreck,
        ProductDuel,
        Sabotage,
        Improve,
        Wreck,
        SumDuel,
        Swap,
        Improve,
        Sabotage,
    ]
};
fn main() {
    let mut game = Game::new();

    while game.turn().is_none() {
        continue;
    }
}

#[derive(Debug, Hash, Eq, PartialEq, PartialOrd, Ord, Clone)]
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

#[derive(Debug, Hash, Eq, PartialEq, PartialOrd, Ord, Clone)]
struct Player {
    location: usize,
    score: usize,
    die: Die,
}

impl Player {
    fn new(die: Die) -> Player {
        Player {
            location: 0,
            score: 0,
            die,
        }
    }
}

#[derive(Debug)]
enum Tile {
    Go,
    Wreck,
    Improve,
    Sabotage,
    Swap,
    SumDuel,
    ProductDuel,
}

#[derive(Debug)]
struct Game {
    board: [Tile; 16],
    players: HashMap<String, Player>,
}

#[derive(Debug, Hash, Eq, PartialEq, PartialOrd, Ord)]
struct GameState {
    players: Vec<Player>,
}

impl From<Game> for GameState {
    fn from(game: Game) -> GameState {
        let mut players: Vec<Player> = game.players.values().cloned().collect();
        players.sort();

        GameState { players }
    }
}

// Map from state to reward
struct StateActionTable {
    table: HashMap<GameState, f64>,
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
                println!("Enter 6 whitespace-separated dice values which sum to 18: ");
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

    fn turn(&mut self) -> Option<String> {
        println!("{:?}", self);
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
                    println!("{} landed on Wreck", name);
                    let player = self.players.get_mut(&name).unwrap();
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
                    println!("{} landed on Improve", name);
                    let player = self.players.get_mut(&name).unwrap();
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
                    println!("{} landed on Sabotage", name);
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
                    // TODO implement increment/decrement constraint. I didn't do this because I like chaos.
                    println!("{} landed on Swap", name);
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

                    let [player, opponent] =
                        self.players.get_many_mut([&name, &opponent_name]).unwrap();

                    // TODO index not value
                    std::mem::swap(
                        &mut player.die.sides[player_selection_index],
                        &mut opponent.die.sides[opponent_selection_index],
                    );
                    player.die.sides.sort();
                    opponent.die.sides.sort();
                }
                Tile::SumDuel => {
                    println!("{} landed on SumDuel", name);
                    // roll three dice and sum the results
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
                        opponent_names[selection].clone()
                    };

                    let [player, opponent] =
                        self.players.get_many_mut([&name, &opponent_name]).unwrap();

                    let mut player_sum = 0;
                    let mut opponent_sum = 0;
                    for _ in 0..3 {
                        player_sum += player.die.roll();
                        opponent_sum += opponent.die.roll();
                    }

                    if player_sum > opponent_sum {
                        println!("{} won the duel", name);
                        player.score += 1;
                    } else {
                        println!("{} won the duel", opponent_name);
                        opponent.score += 1;
                    }
                }
                Tile::ProductDuel => {
                    println!("{} landed on ProductDuel", name);
                    // roll three dice and multiply the results
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
                        opponent_names[selection].clone()
                    };

                    let [player, opponent] =
                        self.players.get_many_mut([&name, &opponent_name]).unwrap();

                    let mut player_product = 1;
                    let mut opponent_product = 1;

                    for _ in 0..3 {
                        player_product *= player.die.roll();
                        opponent_product *= opponent.die.roll();
                    }

                    if player_product > opponent_product {
                        println!("{} won the duel", name);
                        player.score += 1;
                    } else {
                        println!("{} won the duel", opponent_name);
                        opponent.score += 1;
                    }
                }
            }
        }

        let max_score = self.players.values().map(|p| p.score).max().unwrap();
        if max_score >= 5 {
            let winners = self
                .players
                .iter()
                .filter(|(_, p)| p.score == max_score)
                .map(|(name, _)| name)
                .cloned()
                .collect::<Vec<String>>();

            if winners.len() == 1 {
                println!("{} wins!", winners[0]);
                return Some(winners[0].clone());
            } else {
                println!("Multiple people are fulfilling the win condition and are tied. Move to a faceoff.");
                // remove all players except the winners
                self.players.retain(|name, _| winners.contains(name));

                println!("Sudden death!");
                return None;
            }
        }
        None
    }
}
