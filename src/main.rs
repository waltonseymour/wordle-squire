use actix_cors::Cors;
use actix_web::middleware::Logger;
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

    // frequency of letters in guess
    let letter_freq = state
        .guess
        .chars()
        .fold(std::collections::HashMap::new(), |mut acc, x| {
            *acc.entry(x).or_insert(0) += 1;
            acc
        });

    // chars that must be present in the word somewhere
    let mut must_contain = std::collections::HashMap::new();

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
                // cannot have c at i
                cannot_contain[i].insert(c);
                // must exist elsewhere
                *must_contain.entry(c).or_insert(0) += 1;
            }
            GuessState::Correct => {
                *must_contain.entry(c).or_insert(0) += 1;
            }
        }
    }

    for (i, c) in word.chars().enumerate() {
        // contains invalid letter
        if cannot_contain[i].contains(&c) {
            return false;
        }
    }

    for (c, count) in must_contain {
        if word.matches(c).count() < count {
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

fn filter_solutions(possible_solutions: HashSet<String>, result: GuessResult) -> HashSet<String> {
    let mut new_set = HashSet::new();

    for word in possible_solutions {
        if word_matches_state(&word, &result) {
            new_set.insert(word);
        }
    }

    new_set
}

#[post("/words")]
async fn get_words(
    results: web::Json<Vec<GuessResult>>,
    library: web::Data<Library>,

    word_freq: web::Data<HashMap<String, f64>>,
) -> impl Responder {
    let words = &library.words;
    let mut possible_solutions: HashSet<String> =
        std::collections::HashSet::from_iter(words.iter().cloned());

    for result in results.0 {
        possible_solutions = filter_solutions(possible_solutions, result);
    }

    let mut possible_solutions: Vec<String> = possible_solutions.into_iter().collect();

    possible_solutions.sort_by(|a, b| {
        word_freq
            .get(b)
            .unwrap()
            .partial_cmp(word_freq.get(a).unwrap())
            .unwrap()
    });

    HttpResponse::Ok().json(possible_solutions)
}

#[post("/solutions")]
async fn get_solutions(
    results: web::Json<Vec<GuessResult>>,
    library: web::Data<Library>,
) -> impl Responder {
    let solutions = &library.solutions;
    let mut possible_solutions: HashSet<String> =
        std::collections::HashSet::from_iter(solutions.iter().cloned());

    for result in results.0 {
        possible_solutions = filter_solutions(possible_solutions, result);
    }

    HttpResponse::Ok().json(possible_solutions)
}
#[get("/health")]
async fn health() -> impl Responder {
    HttpResponse::Ok().body("OK")
}

struct Library {
    words: Vec<String>,
    solutions: Vec<String>,
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

    let word_freq = read_word_freq();

    let library_data = web::Data::new(Library { words, solutions });

    let word_freq_data = web::Data::new(word_freq);

    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    HttpServer::new(move || {
        App::new()
            .wrap(Cors::permissive())
            .wrap(Logger::default())
            .app_data(library_data.clone())
            .app_data(word_freq_data.clone())
            .service(health)
            .service(get_solutions)
            .service(get_words)
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
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

        // Case 2
        let solution = "apple";
        let guess = "maker";

        let result = evaluate_guess(solution, guess);
        let is_match = word_matches_state("twues", &result);
        // we know it must contain a
        assert_eq!(is_match, false);

        // Case 3
        let solution = "enter";
        let guess = "leech";

        let result = evaluate_guess(solution, guess);

        let is_match = word_matches_state("maker", &result);
        // we know there must be 2 e's
        assert_eq!(is_match, false);

        // Case 4
        let result = GuessResult {
            guess: "enter".to_string(),
            result: [WrongPlace, Missing, Missing, Correct, Missing],
        };

        let is_match = word_matches_state("asked", &result);
        // we know there must be 2 e's
        assert_eq!(is_match, false);

        // Case 5
        let result = GuessResult {
            guess: "enter".to_string(),
            result: [Missing, Missing, Correct, Missing, Missing],
        };

        let is_match = word_matches_state("total", &result);
        // entirely possible for 2 t's
        assert_eq!(is_match, true);

        // Case 5
        let result = GuessResult {
            guess: "enter".to_string(),
            result: [WrongPlace, Missing, WrongPlace, Missing, Missing],
        };

        let is_match = word_matches_state("elate", &result);
        // cannot be 2 e's
        assert_eq!(is_match, false);
    }
}
