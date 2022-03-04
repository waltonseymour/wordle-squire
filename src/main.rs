use rand::seq::SliceRandom;
use rand::thread_rng;
use std::fs::File;
use std::io::{self, BufRead};

#[derive(Copy, Clone, Debug)]
enum GuessState {
    Missing,
    WrongPlace,
    Correct,
}

type GuessResult = [GuessState; 5];

fn evaluate_guess(solution: &str, guess: &str) -> GuessResult {
    let mut result: GuessResult = [GuessState::Missing; 5];
    for (i, c) in guess.chars().enumerate() {
        if solution.contains(c) && solution.chars().nth(i).unwrap() == c {
            result[i] = GuessState::Correct;
        } else if solution.contains(c) {
            result[i] = GuessState::WrongPlace;
        }
    }

    result
}

fn main() {
    let file = File::open("words.csv").expect("could not read file");

    let mut words: Vec<String> = vec![];

    for line in io::BufReader::new(file).lines() {
        words.push(line.unwrap());
    }

    let mut rng = thread_rng();

    let solution = words.choose(&mut rng).unwrap();
    println!("{}", solution);

    let mut buffer = String::new();
    std::io::stdin().read_line(&mut buffer).unwrap();

    let result = evaluate_guess(solution, &buffer);
    println!("{:?}", result);
}
