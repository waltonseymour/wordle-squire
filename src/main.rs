use rand::seq::SliceRandom;
use rand::thread_rng;
use serde_json::{value, Map, Result, Value};
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufRead};

#[derive(Copy, Clone, Debug, PartialEq)]
enum GuessState {
    Missing,
    WrongPlace,
    Correct,
}

#[derive(Debug)]
struct GuessResult {
    result: [GuessState; 5],
    guess: String,
}

fn word_matches_state(word: &str, state: &GuessResult) -> bool {
    // chars that cannot be at the given index
    let mut cannot_contain = [
        std::collections::HashSet::new(),
        std::collections::HashSet::new(),
        std::collections::HashSet::new(),
        std::collections::HashSet::new(),
        std::collections::HashSet::new(),
    ];

    let letter_freq = state
        .guess
        .chars()
        .fold(std::collections::HashMap::new(), |mut acc, x| {
            *acc.entry(x).or_insert(0) += 1;
            acc
        });

    // chars that must be present in the word somewhere
    let mut must_contain = std::collections::HashSet::new();

    for (i, c) in state.guess.chars().enumerate() {
        match state.result[i] {
            GuessState::Missing => {
                if *letter_freq.get(&c).unwrap_or(&0) == 1 {
                    // cannot contain c anywhere
                    for set in &mut cannot_contain {
                        set.insert(c);
                    }
                } else {
                    // cannot contain c anywhere other than where it is correct
                    for (j, k) in state.guess.chars().enumerate() {
                        if k == c && state.result[j] == GuessState::Correct {
                            continue;
                        }

                        cannot_contain[j].insert(c);
                    }
                }
            }
            GuessState::WrongPlace => {
                if *letter_freq.get(&c).unwrap_or(&0) == 1 {
                    // cannot have c at i
                    cannot_contain[i].insert(c);
                    // must exist elsewhere
                    must_contain.insert(c);
                }
            }
            _ => continue,
        }
    }

    for (i, c) in word.chars().enumerate() {
        // contains invalid letter
        if cannot_contain[i].contains(&c) {
            return false;
        }

        // does not contain correct letter
        if state.result[i] == GuessState::Correct && state.guess.chars().nth(i).unwrap() != c {
            return false;
        }
    }

    for c in must_contain {
        // does not contain necessary letter
        if !word.contains(c) {
            return false;
        }
    }

    true
}

fn evaluate_guess(solution: &str, guess: &str) -> GuessResult {
    let mut result = [GuessState::Missing; 5];

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

    GuessResult {
        result,
        guess: guess.to_owned(),
    }
}

/**
 * read_word_freq will return the mapped values of words to their relative frequency in google scholar (higher is more frequent)
 */
fn read_word_freq() -> HashMap<String, f64> {
    let file = File::open("freq_map.json").expect("could not open file");
    let parsed: Value = serde_json::from_reader(file).expect("could not read json");

    let map = parsed.as_object().unwrap().clone();

    let mut hm = HashMap::new();
    for (k, v) in map {
        hm.insert(k, v.as_f64().unwrap());
    }
    hm
}

fn main() {
    let file = File::open("words.csv").expect("could not read file");

    let mut words: Vec<String> = vec![];

    for line in io::BufReader::new(file).lines() {
        words.push(line.unwrap());
    }

    let word_freq = read_word_freq();

    let mut rng = thread_rng();

    let solution = words.choose(&mut rng).unwrap();
    println!("{}", solution);

    let mut guess = String::new();
    std::io::stdin().read_line(&mut guess).unwrap();

    let guess = guess.trim();

    let result = evaluate_guess(solution, &guess);
    println!("{:?}", result);

    let mut matching_words: Vec<_> = words
        .iter()
        .filter(|x| word_matches_state(x, &result))
        .collect();

    matching_words.sort_by(|a, b| word_freq.get(*b).partial_cmp(&word_freq.get(*a)).unwrap());

    for word in matching_words {
        println!("{}", word);
    }
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

        assert_eq!(result.result[0], expected[0]);
        assert_eq!(result.result[1], expected[1]);
        assert_eq!(result.result[2], expected[2]);
        assert_eq!(result.result[3], expected[3]);
        assert_eq!(result.result[4], expected[4]);

        // test 2
        let solution = "water";
        let guess = "axaer";

        let result = evaluate_guess(solution, guess);

        let expected = [WrongPlace, Missing, Missing, Correct, Correct];

        assert_eq!(result.result[0], expected[0]);
        assert_eq!(result.result[1], expected[1]);
        assert_eq!(result.result[2], expected[2]);
        assert_eq!(result.result[3], expected[3]);
        assert_eq!(result.result[4], expected[4]);

        // test 3
        let solution = "attic";
        let guess = "matah";

        let result = evaluate_guess(solution, guess);

        let expected = [Missing, WrongPlace, Correct, Missing, Missing];

        assert_eq!(result.result[0], expected[0]);
        assert_eq!(result.result[1], expected[1]);
        assert_eq!(result.result[2], expected[2]);
        assert_eq!(result.result[3], expected[3]);
        assert_eq!(result.result[4], expected[4]);
    }

    #[test]
    fn test_word_matches_state() {
        let solution = "water";
        let guess = "enter";

        let result = evaluate_guess(solution, guess);

        let is_match = word_matches_state("eater", &result);
        // we know there is only 1 e
        assert_eq!(is_match, false);

        let is_match = word_matches_state("pater", &result);
        assert_eq!(is_match, true);

        let solution = "apple";
        let guess = "maker";

        let result = evaluate_guess(solution, guess);
        let is_match = word_matches_state("twues", &result);
        // we know it must contain a
        assert_eq!(is_match, false);
    }
}
