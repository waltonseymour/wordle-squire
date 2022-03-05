use rand::seq::SliceRandom;
use rand::thread_rng;
use std::fs::File;
use std::io::{self, BufRead};

#[derive(Copy, Clone, Debug, PartialEq)]
enum GuessState {
    Missing,
    WrongPlace,
    Correct,
}

type GuessResult = [GuessState; 5];

fn evaluate_guess(solution: &str, guess: &str) -> GuessResult {
    let mut result: GuessResult = [GuessState::Missing; 5];

    let mut seen = [false; 5];

    for (i, c) in guess.chars().enumerate() {
        if solution.chars().nth(i).unwrap() == c {
            result[i] = GuessState::Correct;
            seen[i] = true;
        }
    }

    for (i, c) in guess.chars().enumerate() {
        if result[i] != GuessState::Correct && solution.contains(c) {
            let maybe_index = solution
                .chars()
                .enumerate()
                .position(|(j, x)| x == c && !seen[j]);

            if maybe_index.is_some() {
                let index = maybe_index.unwrap();
                result[i] = GuessState::WrongPlace;
                seen[index] = true;
            }
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

#[cfg(test)]
mod tests {
    use super::GuessState::*;
    use super::*;

    #[test]
    fn test_evaluate_guess() {
        let solution = "water";
        let guess = "enter";

        let result = evaluate_guess(solution, guess);

        let expected = [Missing, Missing, Correct, Correct, Correct];

        assert_eq!(result[0], expected[0]);
        assert_eq!(result[1], expected[1]);
        assert_eq!(result[2], expected[2]);
        assert_eq!(result[3], expected[3]);
        assert_eq!(result[4], expected[4]);

        // test 2
        let solution = "water";
        let guess = "axaer";

        let result = evaluate_guess(solution, guess);

        let expected = [WrongPlace, Missing, Missing, Correct, Correct];

        assert_eq!(result[0], expected[0]);
        assert_eq!(result[1], expected[1]);
        assert_eq!(result[2], expected[2]);
        assert_eq!(result[3], expected[3]);
        assert_eq!(result[4], expected[4]);

        // test 3
        let solution = "attic";
        let guess = "matah";

        let result = evaluate_guess(solution, guess);

        let expected = [Missing, WrongPlace, Correct, Missing, Missing];

        assert_eq!(result[0], expected[0]);
        assert_eq!(result[1], expected[1]);
        assert_eq!(result[2], expected[2]);
        assert_eq!(result[3], expected[3]);
        assert_eq!(result[4], expected[4]);
    }
}
