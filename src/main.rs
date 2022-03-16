use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{self, BufRead};

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
enum GuessState {
    Missing,
    WrongPlace,
    Correct,
}

#[derive(Debug, Serialize, Deserialize)]
struct GuessResult {
    result: [GuessState; 5],
    guess: String,
}

fn word_matches_state(word: &str, state: &GuessResult) -> bool {
    for (i, c) in word.chars().enumerate() {
        // does not contain correct letter
        if state.result[i] == GuessState::Correct && state.guess.chars().nth(i).unwrap() != c {
            return false;
        }
    }

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
 * entropy_score will return the frequency-weighted entropy value of the guess for the
 */
fn entropy_score(
    possible_words: &Vec<String>,
    possible_solutions: &Vec<String>,
    word_freq: &HashMap<String, f64>,
    guess: &str,
) -> f64 {
    let mut score = 0.;

    for solution in possible_solutions {
        let result = evaluate_guess(solution, guess);

        let mut num_matches = 0;

        // compute how many words are possible matches after guess
        for word in possible_words {
            if word_matches_state(word, &result) {
                num_matches += 1;
            }
        }

        // words removed
        score += possible_words.len() as f64 - num_matches as f64;
    }

    // avg num of words removed
    score / possible_solutions.len() as f64
}

/**
 * simulate_guess will simulate every possible guess with every remaining possible solution and determine the best guess
 */
fn simulate_guess(
    possible_words: &Vec<String>,
    possible_solutions: &Vec<String>,
    word_freq: &HashMap<String, f64>,
) {
    for guess in possible_words {
        println!("{}", guess);
        let score = entropy_score(&possible_words, &possible_solutions, word_freq, &guess);

        println!("{} : {}", guess, score);
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

fn filter_solutions(possible_solutions: HashSet<String>, result: GuessResult) -> HashSet<String> {
    let mut new_set = HashSet::new();

    for word in possible_solutions {
        if word_matches_state(&word, &result) {
            new_set.insert(word);
        }
    }

    new_set
}

#[post("/solutions")]
async fn get_solutions(guess: web::Json<GuessResult>) -> impl Responder {
    println!("{:?}", guess);
    HttpResponse::Ok().body("OK")
}

#[get("/health")]
async fn health(req_body: String) -> impl Responder {
    HttpResponse::Ok().body("OK")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let file = File::open("words.csv").expect("could not read file");

    let mut words: Vec<String> = vec![];

    for line in io::BufReader::new(file).lines() {
        words.push(line.unwrap());
    }

    let file = File::open("solutions.csv").expect("could not read file");

    let mut solutions: Vec<String> = vec![];

    for line in io::BufReader::new(file).lines() {
        solutions.push(line.unwrap());
    }

    HttpServer::new(|| App::new().service(health).service(get_solutions))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await

    // let word_freq = read_word_freq();

    // let mut rng = thread_rng();

    // let solution = solutions.choose(&mut rng).unwrap();

    // let mut possible_solutions: HashSet<String> =
    //     std::collections::HashSet::from_iter(solutions.iter().cloned());

    // for _ in 0..5 {
    //     let mut guess = String::new();
    //     std::io::stdin().read_line(&mut guess).unwrap();

    //     let guess = guess.trim();
    //     let result = evaluate_guess(solution, &guess);
    //     println!("{:?}", result);

    //     possible_solutions = filter_solutions(possible_solutions, result);

    //     for word in &possible_solutions {
    //         println!("{}", word);
    //     }
    // }

    // println!("{}", solution);
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
