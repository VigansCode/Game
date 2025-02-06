use std::f64::consts::SQRT_2;
use rand::Rng;
use std::collections::HashMap;

// Game Configuration
const GRID_SIZE: i32 = 20;
const MOVE_DISTANCE: i32 = 1;
const WOLF_ATTACK_RANGE: i32 = 2;
const WOLF_PACK_BONUS: i32 = 5;
const SHEEP_HERD_BONUS: i32 = 5;

// Utility Functions
fn calculate_distance(entity1: &Animal, entity2: &Animal) -> f64 {
    ((entity2.x - entity1.x).pow(2) + (entity2.y - entity1.y).pow(2)) as f64
}

// Base Animal Class
struct Animal {
    x: i32,
    y: i32,
    animal_type: String,
    health: i32,
    is_alive: bool,
    votes: i32,
    last_action: Option<String>,
    id: String,
}

impl Animal {
    fn new(x: i32, y: i32, animal_type: String) -> Self {
        Self {
            x,
            y,
            animal_type,
            health: 100,
            is_alive: true,
            votes: 0,
            last_action: None,
            id: rand::thread_rng().gen::<u64>().to_string(),
        }
    }

    fn move(&mut self, dx: i32, dy: i32, game_state: &GameState) -> bool {
        let new_x = self.x + dx;
        let new_y = self.y + dy;

        if new_x >= 0 && new_x < GRID_SIZE && new_y >= 0 && new_y < GRID_SIZE {
            let collision = game_state.animals.iter().any(|animal| {
                animal.id != self.id && animal.is_alive && animal.x == new_x && animal.y == new_y
            });

            if !collision {
                self.x = new_x;
                self.y = new_y;
                return true;
            }
        }
        false
    }

    fn take_damage(&mut self, amount: i32) {
        self.health = std::cmp::max(0, self.health - amount);
        if self.health <= 0 {
            self.is_alive = false;
        }
    }

    fn heal(&mut self, amount: i32) {
        self.health = std::cmp::min(100, self.health + amount);
    }

    fn get_nearby_allies(&self, game_state: &GameState, range: i32) -> Vec<&Animal> {
        game_state
            .animals
            .iter()
            .filter(|animal| {
                animal.id != self.id
                    && animal.animal_type == self.animal_type
                    && animal.is_alive
                    && calculate_distance(self, animal) <= range as f64
            })
            .collect()
    }
}

// Wolf Class
struct Wolf {
    animal: Animal,
    attack_power: i32,
    stamina: i32,
}

impl Wolf {
    fn new(x: i32, y: i32) -> Self {
        Self {
            animal: Animal::new(x, y, "wolf".to_string()),
            attack_power: 20,
            stamina: 100,
        }
    }

    fn attack(&mut self, target: &mut Animal, game_state: &GameState) -> Option<String> {
        if !self.animal.is_alive || !target.is_alive || self.stamina < 20 {
            return None;
        }

        let distance = calculate_distance(&self.animal, target);
        if distance > WOLF_ATTACK_RANGE as f64 {
            return Some(format!("Wolf {} is too far to attack!", self.animal.id));
        }

        let nearby_wolves = self.animal.get_nearby_allies(game_state, 2);
        let pack_bonus = nearby_wolves.len() as i32 * WOLF_PACK_BONUS;
        let total_damage = self.attack_power + pack_bonus;

        target.take_damage(total_damage);
        self.stamina -= 20;
        self.animal.last_action = Some("attack".to_string());

        Some(format!(
            "Wolf {} attacked {} {} for {} damage! Target health: {}",
            self.animal.id, target.animal_type, target.id, total_damage, target.health
        ))
    }

    fn rest(&mut self) -> String {
        self.stamina = std::cmp::min(100, self.stamina + 30);
        self.animal.last_action = Some("rest".to_string());
        format!("Wolf {} rested. Stamina restored to {}", self.animal.id, self.stamina)
    }
}

// Sheep Class
struct Sheep {
    animal: Animal,
    defense_power: i32,
    energy: i32,
}

impl Sheep {
    fn new(x: i32, y: i32) -> Self {
        Self {
            animal: Animal::new(x, y, "sheep".to_string()),
            defense_power: 10,
            energy: 100,
        }
    }

    fn defend(&mut self, game_state: &GameState) -> Option<String> {
        if self.energy < 15 {
            return None;
        }

        let nearby_sheep = self.animal.get_nearby_allies(game_state, 2);
        let herd_bonus = nearby_sheep.len() as i32 * SHEEP_HERD_BONUS;
        let total_defense = self.defense_power + herd_bonus;

        self.animal.heal(total_defense);
        self.energy -= 15;
        self.animal.last_action = Some("defend".to_string());

        Some(format!(
            "Sheep {} defended with herd bonus! Healed for {}. Current health: {}",
            self.animal.id, total_defense, self.animal.health
        ))
    }

    fn flee(&mut self, game_state: &GameState) -> Option<String> {
        if self.energy < 10 {
            return None;
        }

        let wolves: Vec<&Animal> = game_state
            .animals
            .iter()
            .filter(|a| a.animal_type == "wolf" && a.is_alive)
            .collect();

        if wolves.is_empty() {
            return None;
        }

        let nearest_wolf = wolves
            .iter()
            .min_by(|a, b| {
                calculate_distance(&self.animal, a)
                    .partial_cmp(&calculate_distance(&self.animal, b))
                    .unwrap()
            })
            .unwrap();

        let dx = self.animal.x - nearest_wolf.x;
        let dy = self.animal.y - nearest_wolf.y;
        let magnitude = ((dx * dx + dy * dy) as f64).sqrt();

        if magnitude != 0.0 {
            let normalized_dx = (dx as f64 / magnitude).round() as i32;
            let normalized_dy = (dy as f64 / magnitude).round() as i32;

            if self.animal.move(normalized_dx, normalized_dy, game_state) {
                self.energy -= 10;
                self.animal.last_action = Some("flee".to_string());
                return Some(format!("Sheep {} fled from wolf!", self.animal.id));
            }
        }

        None
    }

    fn rest(&mut self) -> String {
        self.energy = std::cmp::min(100, self.energy + 25);
        self.animal.last_action = Some("rest".to_string());
        format!(
            "Sheep {} rested. Energy restored to {}",
            self.animal.id, self.energy
        )
    }
}

// Game State Management
struct GameState {
    animals: Vec<Animal>,
    voting_open: bool,
    round_number: i32,
    turn_order: Vec<String>,
    battle_log: Vec<Vec<String>>,
    voting_time_remaining: i32,
}

impl GameState {
    fn new() -> Self {
        Self {
            animals: Vec::new(),
            voting_open: false,
            round_number: 1,
            turn_order: Vec::new(),
            battle_log: Vec::new(),
            voting_time_remaining: 0,
        }
    }

    fn add_animal(&mut self, animal: Animal) {
        self.animals.push(animal);
        self.update_turn_order();
    }

    fn update_turn_order(&mut self) {
        self.turn_order = self
            .animals
            .iter()
            .filter(|animal| animal.is_alive)
            .map(|animal| animal.id.clone())
            .collect();
        self.turn_order.sort_by(|_, _| rand::thread_rng().gen::<bool>().cmp(&false));
    }

    fn start_voting(&mut self, voting_duration: i32) {
        self.voting_open = true;
        self.voting_time_remaining = voting_duration;
        for animal in &mut self.animals {
            animal.votes = 0;
        }
    }

    fn cast_vote(&mut self, target_animal_id: &str) -> bool {
        if !self.voting_open {
            return false;
        }

        if let Some(target) = self.animals.iter_mut().find(|a| a.id == target_animal_id && a.is_alive) {
            target.votes += 1;
            return true;
        }
        false
    }

    fn end_voting(&mut self) -> (Animal, Vec<(String, String, i32)>) {
        self.voting_open = false;
        let living_animals: Vec<&Animal> = self.animals.iter().filter(|a| a.is_alive).collect();
        let winner = living_animals
            .iter()
            .max_by_key(|a| a.votes)
            .unwrap()
            .clone();

        winner.heal(20);

        let vote_counts = living_animals
            .iter()
            .map(|a| (a.id.clone(), a.animal_type.clone(), a.votes))
            .collect();

        (winner.clone(), vote_counts)
    }

    fn check_game_over(&self) -> Option<String> {
        let wolves = self.animals.iter().filter(|a| a.animal_type == "wolf" && a.is_alive).count();
        let sheep = self.animals.iter().filter(|a| a.animal_type == "sheep" && a.is_alive).count();

        if wolves == 0 {
            Some("Sheep Win!".to_string())
        } else if sheep == 0 {
            Some("Wolves Win!".to_string())
        } else {
            None
        }
    }

    fn get_game_status(&self) -> HashMap<String, String> {
        let mut status = HashMap::new();
        status.insert("round".to_string(), self.round_number.to_string());
        status.insert("voting_open".to_string(), self.voting_open.to_string());
        status.insert(
            "voting_time_remaining".to_string(),
            self.voting_time_remaining.to_string(),
        );
        for animal in &self.animals {
            status.insert(
                format!("animal_{}_id", animal.id),
                animal.id.clone(),
            );
            status.insert(
                format!("animal_{}_type", animal.id),
                animal.animal_type.clone(),
            );
            status.insert(
                format!("animal_{}_health", animal.id),
                animal.health.to_string(),
            );
            status.insert(
                format!("animal_{}_is_alive", animal.id),
                animal.is_alive.to_string(),
            );
            status.insert(
                format!("animal_{}_position_x", animal.id),
                animal.x.to_string(),
            );
            status.insert(
                format!("animal_{}_position_y", animal.id),
                animal.y.to_string(),
            );
            status.insert(
                format!("animal_{}_last_action", animal.id),
                animal.last_action.clone().unwrap_or("None".to_string()),
            );
            status.insert(
                format!("animal_{}_votes", animal.id),
                animal.votes.to_string(),
            );
        }
        status
    }
}

// Battle System
struct BattleSystem {
    game_state: GameState,
}

impl BattleSystem {
    fn new(game_state: GameState) -> Self {
        Self { game_state }
    }

    fn initialize_battle(&mut self, num_wolves: i32, num_sheep: i32) {
        self.game_state.animals.clear();

        // Add wolves
        for _ in 0..num_wolves {
            self.game_state.add_animal(Animal::new(
                rand::thread_rng().gen_range(0..GRID_SIZE),
                rand::thread_rng().gen_range(0..GRID_SIZE),
                "wolf".to_string(),
            ));
        }

        // Add sheep
        for _ in 0..num_sheep {
            self.game_state.add_animal(Animal::new(
                rand::thread_rng().gen_range(0..GRID_SIZE),
                rand::thread_rng().gen_range(0..GRID_SIZE),
                "sheep".to_string(),
            ));
        }
    }

    fn execute_round(&mut self) -> Vec<String> {
        let mut messages = Vec::new();
        self.game_state.update_turn_order();

        for animal_id in &self.game_state.turn_order {
            if let Some(animal) = self.game_state.animals.iter_mut().find(|a| a.id == *animal_id && a.is_alive) {
                if animal.animal_type == "wolf" {
                    // Wolf AI
                    let targets: Vec<&Animal> = self
                        .game_state
                        .animals
                        .iter()
                        .filter(|a| a.animal_type == "sheep" && a.is_alive)
                        .collect();

                    if !targets.is_empty() {
                        let nearest_sheep = targets
                            .iter()
                            .min_by(|a, b| {
                                calculate_distance(animal, a)
                                    .partial_cmp(&calculate_distance(animal, b))
                                    .unwrap()
                            })
                            .unwrap();

                        let distance = calculate_distance(animal, nearest_sheep);

                        if distance <= WOLF_ATTACK_RANGE as f64 {
                            if let Some(message) = Wolf::attack(&mut Wolf::new(animal.x, animal.y), &mut nearest_sheep.clone(), &self.game_state) {
                                messages.push(message);
                            }
                        } else {
                            let dx = (nearest_sheep.x - animal.x).signum();
                            let dy = (nearest_sheep.y - animal.y).signum();
                            if animal.move(dx, dy, &self.game_state) {
                                messages.push(format!("Wolf {} moved toward sheep", animal.id));
                            }
                        }
                    }
                } else {
                    // Sheep AI
                    let nearby_wolves: Vec<&Animal> = self
                        .game_state
                        .animals
                        .iter()
                        .filter(|a| {
                            a.animal_type == "wolf"
                                && a.is_alive
                                && calculate_distance(animal, a) <= (WOLF_ATTACK_RANGE + 1) as f64
                        })
                        .collect();

                    if !nearby_wolves.is_empty() {
                        if let Some(message) = Sheep::flee(&mut Sheep::new(animal.x, animal.y), &self.game_state) {
                            messages.push(message);
                        } else if let Some(message) = Sheep::defend(&mut Sheep::new(animal.x, animal.y), &self.game_state) {
                            messages.push(message);
                        }
                    } else {
                        messages.push(Sheep::rest(&mut Sheep::new(animal.x, animal.y)));
                    }
                }
            }
        }

        self.game_state.round_number += 1;
        self.game_state.battle_log.push(messages.clone());
        messages
    }
}

fn main() {
    let mut game_state = GameState::new();
    let mut battle_system = BattleSystem::new(game_state);
    battle_system.initialize_battle(3, 5);

    for _ in 0..10 {
        let messages = battle_system.execute_round();
        for message in messages {
            println!("{}", message);
        }
    }
}
