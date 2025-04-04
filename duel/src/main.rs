use rand::Rng;
use std::{io, thread, time, env};
use std::sync::{Arc, Mutex};
use crossterm::{execute, terminal::{Clear, ClearType}, cursor};
use std::io::Write;

struct Player {
    name: String,
    vitality: u32,
    speed: u64,
    strength: u32,
    score: u32,
}

impl Player {
    fn new(name: &str, vitality: u32, speed: u64, strength: u32) -> Self {
        Self {
            name: name.to_string(),
            vitality,
            speed,
            strength,
            score: 0,
        }
    }

    fn play_turn(&mut self, objectives: &Vec<i32>) -> u32 {
        println!("Au tour de {} (Vitalité={}, Vitesse={}, Force={})", self.name, self.vitality, self.speed, self.strength);
        println!("→ Objectifs : {:?}", objectives);
        println!("Appuyer sur ENTREE pour démarrer le tour..");
        let _ = io::stdin().read_line(&mut String::new());
        
        let mut total_score = 0;
        
        for &target in objectives.iter() {
            let misses = Arc::new(Mutex::new(0));
            let misses_clone = Arc::clone(&misses);
            let counter = Arc::new(Mutex::new(-1)); // Initialiser le compteur à -1 pour rendre le 0 possible selon la logique qui suit
            let counter_clone = Arc::clone(&counter);
            let speed = self.speed;
            
            let running = Arc::new(Mutex::new(true));
            let running_clone = Arc::clone(&running);
            
            let handle = thread::spawn(move || {
                while *running_clone.lock().unwrap() {
                    let mut counter = counter_clone.lock().unwrap();
                    *counter = *counter + 1;
                    if *counter > 100{
                        *counter = 0;
                        *misses_clone.lock().unwrap() += 1;
                    }
                    
                    execute!(
                        io::stdout(),
                        Clear(ClearType::CurrentLine),
                        cursor::MoveToColumn(0)
                    ).unwrap();
                    print!("\r→ Objectif {} : Miss = {} | Compteur = {}", target, *misses_clone.lock().unwrap(), *counter);
                    io::stdout().flush().unwrap();
                    thread::sleep(time::Duration::from_millis(speed));
                }
            });
            
            let _ = io::stdin().read_line(&mut String::new());
            *running.lock().unwrap() = false;
            handle.join().unwrap();

            // Ajuster et afficher le compteur pour qu'il ne soit pas négatif si le joueur est trop rapide pour le thread
            let mut result = *counter.lock().unwrap();
            if result < 0 {
                result = 0;
                println!("\r→ Objectif {} : Miss = 0 | Compteur = 0", target);
            }

            let miss_count = *misses.lock().unwrap();
            let diff = (target - result).abs();
            let score = match diff {
                0 => (100 + self.strength) / (miss_count + 1),
                1..=5 => (80 + self.strength) / (miss_count + 1),
                6..=10 => (60 + self.strength) / (miss_count + 1),
                11..=20 => (40 + self.strength) / (miss_count + 1),
                21..=50 => (20 + self.strength) / (miss_count + 1),
                _ => (0 + self.strength) / (miss_count + 1),
            };
            total_score += score;
        }
        
        self.score = (total_score as f64 / objectives.len() as f64).ceil() as u32; // scrore moyen arrondi à l'entier supérieur
        println!("\n# Fin du tour #\n→ Score moyen {}\n\n", self.score);
        self.score
    }
}

fn apply_poison(player: &mut Player) {
    let mut choice = String::new();
    loop {
        io::stdin().read_line(&mut choice).unwrap();
        match choice.trim() {
            "1" => {
                player.speed = player.speed.saturating_sub(5);
                println!("Vitesse de {} réduite à {}", player.name, player.speed);
                break;
            },
            "2" => {
                player.strength = player.strength.saturating_sub(5);
                println!("Force de {} réduite à {}", player.name, player.strength);
                break;
            },
            _ => println!("Choix invalide. Veuillez entrer 1 ou 2."),
        }
    }
}
fn main() {
    let args: Vec<String> = env::args().collect();
    let name1 = args.iter().position(|r| r == "--name1").map(|i| args[i + 1].clone()).unwrap_or("Joueur 1".to_string());
    let name2 = args.iter().position(|r| r == "--name2").map(|i| args[i + 1].clone()).unwrap_or("Joueur 2".to_string());
    let vitality: u32 = args.iter().position(|r| r == "--vitality").map(|i| args[i + 1].parse().unwrap_or(50)).unwrap_or(50);
    let num_objectives: usize = args.iter().position(|r| r == "--objectifs").map(|i| args[i + 1].parse().unwrap_or(5)).unwrap_or(5);
    let speed: u64 = args.iter().position(|r| r == "--speed").map(|i| args[i + 1].parse().unwrap_or(50)).unwrap_or(50);
    let strength: u32 = args.iter().position(|r| r == "--strength").map(|i| args[i + 1].parse().unwrap_or(50)).unwrap_or(50);

    let mut player1 = Player::new(&name1, vitality, speed, strength);
    let mut player2 = Player::new(&name2, vitality, speed, strength);
    
    let mut rng = rand::rng();
    
    let mut manche = 1;

    println!("##### Démarrage de la partie #####");

    while (player1.vitality > 0) && (player2.vitality > 0){
        let mut objectives: Vec<i32> = (0..num_objectives).map(|_| rng.random_range(0..=100)).collect();
        println!("## Manche {} ##", manche);
        let score1 = player1.play_turn(&objectives);
        objectives = (0..num_objectives).map(|_| rng.random_range(0..=100)).collect();
        let score2 = player2.play_turn(&objectives);

        if score1 == score2 {
            println!("ÉGALITÉ ! Personne ne perd de vitalité.");
        } else if score1 > score2 {
            let diff = score1 - score2;
            player2.vitality = player2.vitality.saturating_sub(diff);
            println!("{} gagne la manche ! {} perd {} points de vitalité.", player1.name, player2.name, diff);
            println!("{} vous devez choisir quel poison appliquer à {} :", player1.name, player2.name);
            println!("→ 1: -5 vitesse");
            println!("→ 2: -5 force");
            apply_poison(&mut player2);
        } else {
            let diff = score2 - score1;
            player1.vitality = player1.vitality.saturating_sub(diff);
            println!("{} gagne la manche ! {} perd {} points de vitalité.", player2.name, player1.name, diff);
            println!("{} vous devez choisir quel poison appliquer à {} :", player2.name, player1.name);
            println!("→ 1: -5 vitesse");
            println!("→ 2: -5 force");
            apply_poison(&mut player1);
        }
        println!("## FIN Manche {} ##", manche);
        manche+=1;
    }
    
    
    println!("Fin de la partie: {} {} PV, {} {} PV", player1.name, player1.vitality, player2.name, player2.vitality);
}
