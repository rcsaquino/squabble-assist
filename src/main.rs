use rsautogui::{keyboard, screen};
use serde_json;
use std::fs;
use std::{collections::HashMap, thread, time};

fn main() {
    // Main game loop
    let mut done = false;
    while !done {
        println!("Resetting possible_answers...");
        thread::sleep(time::Duration::from_millis(500));
        let possible_answers: Vec<&str> =
            serde_json::from_str(include_str!("sowpods_min.json")).unwrap();
        thread::sleep(time::Duration::from_millis(750));
        done = guess(possible_answers, 1);
    }
    // End process
    println!("Process complete!")
}

fn remove_from_db(word: &str) {
    let data = &fs::read_to_string("./src/sowpods_min.json").unwrap()[..];
    let mut sowpods: Vec<&str> = serde_json::from_str(data).unwrap();
    sowpods.retain(|x| *x != word);
    fs::write(
        "./src/sowpods_min.json",
        serde_json::to_string(&sowpods).unwrap(),
    )
    .expect("Unable to write file");
}

fn guess(mut possible_answers: Vec<&str>, mut round: u32) -> bool {
    while possible_answers.len() > 0 {
        // If round 1, initially supply "tares" as the answer. Else, get best answer.
        let answer = if round == 1 {
            "tares"
        } else {
            get_best_answer(possible_answers.clone())
        };

        // Type out the answer
        type_answer(answer);
        // Leave a short buffer to allow animations to occur
        thread::sleep(time::Duration::from_millis(500));
        // Check if win/lose conditions met. If so, return
        if get_pixel(790, 826) == screen::Rgba([130, 53, 245, 255]) {
            println!("End of game.");
            return true;
        }
        // Check answer with round as argument as check_answer fn needs round to identify which pixel to look at.
        let score = check_answer(round);
        // Print out the results for debugging.
        println!("Round {} - {} - {:?}", round, answer, score);
        // Filter out possible_answers depending on the answer score
        possible_answers =
            remove_possible_answers(possible_answers.clone(), answer.to_string(), score);

        // Check if word is invalid. If so, backspace and continue.
        if score == [3; 5]
            // Also verify if word is really invalid or round just restarted.
            && get_pixel(835, 416) != screen::Rgba([167, 113, 248, 255])
        {
            println!("===== Invalid word. Removing from db =====> {}", answer);
            remove_from_db(answer);
            for _ in 0..5 {
                keyboard::key_tap(keyboard::Key::Backspace);
                thread::sleep(time::Duration::from_millis(50));
            }
        }
        // Else increment round by 1
        else {
            round += 1;
        }
        // If answer is correct or if round > 6, reset.
        if score == [2; 5] || round > 6 {
            println!("Correct answer || Round > 6");
            return false;
        }
    }
    // If there are no more possible guesses, spam "tares" depending on the number of rounds left.
    if possible_answers.len() == 0 {
        for _ in 0..(6 - round) {
            type_answer("tares");
            thread::sleep(time::Duration::from_millis(150));
        }
        println!("Possible answers == 0");
        return false;
    }
    println!("Unhandled condition!");
    return false;
}

fn get_pixel(x: u32, y: u32) -> screen::Rgba<u8> {
    let img = screen::screenshot(1920, 1080);
    *img.get_pixel(x, y)
}

fn get_best_answer(possible_answers: Vec<&str>) -> &str {
    let mut word_hash: HashMap<&str, i32> = HashMap::new();
    for answer in possible_answers.iter() {
        for answer2 in &possible_answers {
            for a_char_index in 0..5 {
                let a_char = answer.chars().nth(a_char_index).unwrap();
                if a_char == answer2.chars().nth(a_char_index).unwrap() {
                    *word_hash.entry(&answer2).or_insert(0) += 5;
                } else if answer2.contains(a_char) {
                    *word_hash.entry(&answer2).or_insert(0) += 4;
                }
            }
        }
    }

    let mut word_hash_vec: Vec<(&&str, &i32)> = word_hash.iter().collect();
    word_hash_vec.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    word_hash_vec[0].0
}

fn remove_possible_answers(
    mut possible_answers: Vec<&str>,
    answer: String,
    score: [u8; 5],
) -> Vec<&str> {
    let mut present: Vec<char> = Vec::new();
    let mut absent: Vec<char> = Vec::new();
    for i in 0..5 {
        match score[i] {
            2 => {
                present.push(answer.chars().nth(i).unwrap());
                possible_answers.retain(|pans| {
                    pans.chars().nth(i).unwrap() == answer.chars().nth(i).unwrap()
                        && pans.contains(answer.chars().nth(i).unwrap())
                });
            }
            1 => {
                present.push(answer.chars().nth(i).unwrap());
                possible_answers.retain(|pans| {
                    pans.chars().nth(i).unwrap() != answer.chars().nth(i).unwrap()
                        && pans.contains(answer.chars().nth(i).unwrap())
                });
            }
            0 => absent.push(answer.chars().nth(i).unwrap()),
            3 => continue,
            9 => continue,
            _ => panic!("Error: Invalid Score!"),
        }
    }
    for c in absent {
        if !present.contains(&c) {
            possible_answers.retain(|ans| !ans.contains(c));
        }
    }

    possible_answers.retain(|pans| pans != &answer);

    return possible_answers;
}

fn type_answer(answer: &str) {
    keyboard::typewrite(answer);
    keyboard::key_tap(keyboard::Key::Return)
}

fn check_answer(round: u32) -> [u8; 5] {
    let img = screen::screenshot(1920, 1080);
    let mut score: [u8; 5] = [0; 5];
    let round_y: u32 = 416 + (73 * (round - 1));
    for x in 0..5 {
        let letter_x: u32 = 835 + (73 * x);
        let temp: u8 = match *img.get_pixel(letter_x, round_y) {
            // Green
            screen::Rgba([46, 216, 60, 255]) => 2,
            // Yellow
            screen::Rgba([214, 190, 0, 255]) => 1,
            // Gray
            screen::Rgba([155, 93, 247, 255]) => 0,
            // None
            screen::Rgba([167, 113, 248, 255]) => 3,
            // Else
            _ => 9,
        };
        score[x as usize] = temp;
    }
    return score;
}
