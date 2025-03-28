use rand::Rng;
use std::{io, thread, time, env};
use std::sync::{Arc, Mutex};
use crossterm::{execute, terminal::{Clear, ClearType}, cursor};
use std::io::Write;

struct Player {
    name: String,
    vitality: i32,
    speed: u64,
    strength: i32,
    score: i32,
}

impl Player {
    fn new(name: &str, vitality: i32, speed: u64, strength: i32) -> Self {
        Self {
            name: name.to_string(),
            vitality,
            speed,
            strength,
            score: 0,
        }
    }

    fn play_turn(&mut self, objectives: &Vec<i32>) -> i32 {
        println!("Au tour de {} (Vitalité={}, Vitesse={}, Force={})", self.name, self.vitality, self.speed, self.strength);
        println!("→ Objectifs : {:?}", objectives);
        println!("Appuyer sur ENTREE pour démarrer le tour..");
        let _ = io::stdin().read_line(&mut String::new());
        
        let mut total_score = 0;
        
        for &target in objectives.iter() {
            let mut misses = 0;
            let counter = Arc::new(Mutex::new(0));
            let counter_clone = Arc::clone(&counter);
            let speed = self.speed;
            
            let running = Arc::new(Mutex::new(true));
            let running_clone = Arc::clone(&running);
            
            let handle = thread::spawn(move || {
                while *running_clone.lock().unwrap() {
                    thread::sleep(time::Duration::from_millis(speed));
                    let mut counter = counter_clone.lock().unwrap();
                    *counter = *counter + 1;
                    if *counter > 100{
                        *counter = 0;
                        misses+=1;
                    }
                    
                    execute!(
                        io::stdout(),
                        Clear(ClearType::CurrentLine),
                        cursor::MoveToColumn(0)
                    ).unwrap();
                    print!("Compteur: {}", *counter);
                    io::stdout().flush().unwrap();
                }
            });
            
            let _ = io::stdin().read_line(&mut String::new());
            *running.lock().unwrap() = false;
            handle.join().unwrap();
            
            let result = *counter.lock().unwrap();
            let diff = (target - result).abs();
            let score = match diff {
                0 => (100 + self.strength) / (misses + 1),
                1..=5 => (80 + self.strength) / (misses + 1),
                6..=10 => (60 + self.strength) / (misses + 1),
                11..=20 => (40 + self.strength) / (misses + 1),
                21..=50 => (20 + self.strength) / (misses + 1),
                _ => (0 + self.strength) / (misses + 1),
            };
            println!("\n→ Objectif {} : Miss = {} | Compteur = {} // Score = {}", target, misses, result, score);
            total_score += score;
        }
        
        self.score = (total_score as f64 / objectives.len() as f64).ceil() as i32;
        println!("# Fin du tour #\n→ Score moyen {}", self.score);
        self.score
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let name1 = args.iter().position(|r| r == "--name1").map(|i| args[i + 1].clone()).unwrap_or("Michel".to_string());
    let name2 = args.iter().position(|r| r == "--name2").map(|i| args[i + 1].clone()).unwrap_or("Jacque".to_string());
    let vitality: i32 = args.iter().position(|r| r == "--vitality").map(|i| args[i + 1].parse().unwrap_or(50)).unwrap_or(50);
    let num_objectives: usize = args.iter().position(|r| r == "--objectifs").map(|i| args[i + 1].parse().unwrap_or(5)).unwrap_or(5);
    
    let mut player1 = Player::new(&name1, vitality, 50, 50);
    let mut player2 = Player::new(&name2, vitality, 50, 50);
    
    let mut rng = rand::rng();
    
    let mut manche = 1;

    println!("##### Démarrage de la partie #####");

    while (player1.vitality > 0) && (player2.vitality > 0){
        let mut objectives: Vec<i32> = (0..num_objectives).map(|_| rng.random_range(0..=100)).collect();
        println!("## Manche {} ##", manche);
        let score1 = player1.play_turn(&objectives);
        objectives = (0..num_objectives).map(|_| rng.random_range(0..=100)).collect();
        let score2 = player2.play_turn(&objectives);
        
        if score1 > score2 {
            let diff = score1 - score2;
            println!("{} gagne la manche! {} perd {} points de vitalité.", player1.name, player2.name, diff);
            player2.vitality -= diff
        } else {
            let diff = score2 - score1;
            println!("{} gagne la manche! {} perd {} points de vitalité.", player2.name, player1.name, diff);
            player1.vitality -= diff
        }
        println!("## FIN Manche {} ##", manche);
        manche+=1;
    }
    
    
    println!("Fin de la partie: {} {} PV, {} {} PV", player1.name, player1.vitality, player2.name, player2.vitality);
}
