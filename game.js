// Game Configuration
const GAME_CONFIG = {
    GRID_SIZE: 20,
    MOVE_DISTANCE: 1,
    WOLF_ATTACK_RANGE: 2,
    WOLF_PACK_BONUS: 5,
    SHEEP_HERD_BONUS: 5,
};

// Utility Functions
const calculateDistance = (entity1, entity2) => {
    return Math.sqrt(
        Math.pow(entity2.x - entity1.x, 2) + 
        Math.pow(entity2.y - entity1.y, 2)
    );
};

// Base Animal Class
class Animal {
    constructor(x, y, type) {
        this.x = x;
        this.y = y;
        this.type = type;
        this.health = 100;
        this.isAlive = true;
        this.votes = 0;
        this.lastAction = null;
        this.id = Math.random().toString(36).substr(2, 9);
    }

    move(dx, dy, gameState) {
        const newX = this.x + dx;
        const newY = this.y + dy;
        
        if (newX >= 0 && newX < GAME_CONFIG.GRID_SIZE && 
            newY >= 0 && newY < GAME_CONFIG.GRID_SIZE) {
            
            const collision = gameState.animals.some(animal => 
                animal !== this && 
                animal.isAlive &&
                animal.x === newX && 
                animal.y === newY
            );

            if (!collision) {
                this.x = newX;
                this.y = newY;
                return true;
            }
        }
        return false;
    }

    takeDamage(amount) {
        this.health = Math.max(0, this.health - amount);
        if (this.health <= 0) {
            this.isAlive = false;
        }
    }

    heal(amount) {
        this.health = Math.min(100, this.health + amount);
    }

    getNearbyAllies(gameState, range = 2) {
        return gameState.animals.filter(animal => 
            animal !== this &&
            animal.type === this.type &&
            animal.isAlive &&
            calculateDistance(this, animal) <= range
        );
    }
}

// Wolf Class
class Wolf extends Animal {
    constructor(x, y) {
        super(x, y, 'wolf');
        this.attackPower = 20;
        this.stamina = 100;
    }

    attack(target, gameState) {
        if (!this.isAlive || !target.isAlive || this.stamina < 20) {
            return null;
        }

        const distance = calculateDistance(this, target);
        if (distance > GAME_CONFIG.WOLF_ATTACK_RANGE) {
            return `Wolf ${this.id} is too far to attack!`;
        }

        const nearbyWolves = this.getNearbyAllies(gameState);
        const packBonus = nearbyWolves.length * GAME_CONFIG.WOLF_PACK_BONUS;
        const totalDamage = this.attackPower + packBonus;
        
        target.takeDamage(totalDamage);
        this.stamina -= 20;
        this.lastAction = 'attack';
        
        return `Wolf ${this.id} attacked ${target.type} ${target.id} for ${totalDamage} damage! Target health: ${target.health}`;
    }

    rest() {
        this.stamina = Math.min(100, this.stamina + 30);
        this.lastAction = 'rest';
        return `Wolf ${this.id} rested. Stamina restored to ${this.stamina}`;
    }
}

// Sheep Class
class Sheep extends Animal {
    constructor(x, y) {
        super(x, y, 'sheep');
        this.defensePower = 10;
        this.energy = 100;
    }

    defend(gameState) {
        if (this.energy < 15) {
            return null;
        }

        const nearbySheep = this.getNearbyAllies(gameState);
        const herdBonus = nearbySheep.length * GAME_CONFIG.SHEEP_HERD_BONUS;
        const totalDefense = this.defensePower + herdBonus;
        
        this.heal(totalDefense);
        this.energy -= 15;
        this.lastAction = 'defend';
        
        return `Sheep ${this.id} defended with herd bonus! Healed for ${totalDefense}. Current health: ${this.health}`;
    }

    flee(gameState) {
        if (this.energy < 10) {
            return null;
        }

        const wolves = gameState.animals.filter(a => a.type === 'wolf' && a.isAlive);
        if (wolves.length === 0) return null;

        const nearestWolf = wolves.reduce((nearest, wolf) => {
            const distance = calculateDistance(this, wolf);
            return distance < calculateDistance(this, nearest) ? wolf : nearest;
        });

        const dx = this.x - nearestWolf.x;
        const dy = this.y - nearestWolf.y;
        const magnitude = Math.sqrt(dx * dx + dy * dy);
        
        if (magnitude !== 0) {
            const normalizedDx = Math.round(dx / magnitude);
            const normalizedDy = Math.round(dy / magnitude);
            
            if (this.move(normalizedDx, normalizedDy, gameState)) {
                this.energy -= 10;
                this.lastAction = 'flee';
                return `Sheep ${this.id} fled from wolf!`;
            }
        }

        return null;
    }

    rest() {
        this.energy = Math.min(100, this.energy + 25);
        this.lastAction = 'rest';
        return `Sheep ${this.id} rested. Energy restored to ${this.energy}`;
    }
}

// Game State Management
class GameState {
    constructor() {
        this.animals = [];
        this.votingOpen = false;
        this.roundNumber = 1;
        this.turnOrder = [];
        this.battleLog = [];
        this.votingTimeRemaining = 0;
    }

    addAnimal(animal) {
        this.animals.push(animal);
        this.updateTurnOrder();
    }

    updateTurnOrder() {
        this.turnOrder = [...this.animals]
            .filter(animal => animal.isAlive)
            .sort(() => Math.random() - 0.5);
    }

    startVoting(votingDuration = 30) {
        this.votingOpen = true;
        this.votingTimeRemaining = votingDuration;
        this.animals.forEach(animal => animal.votes = 0);
    }

    castVote(targetAnimalId) {
        if (!this.votingOpen) return false;
        
        const target = this.animals.find(a => a.id === targetAnimalId);
        if (target && target.isAlive) {
            target.votes++;
            return true;
        }
        return false;
    }

    endVoting() {
        this.votingOpen = false;
        const livingAnimals = this.animals.filter(a => a.isAlive);
        const winner = livingAnimals.reduce((prev, current) => 
            (prev.votes > current.votes) ? prev : current
        );
        
        winner.heal(20);
        
        return {
            winner,
            voteCounts: livingAnimals.map(a => ({
                id: a.id,
                type: a.type,
                votes: a.votes
            }))
        };
    }

    checkGameOver() {
        const wolves = this.animals.filter(a => a.type === 'wolf' && a.isAlive);
        const sheep = this.animals.filter(a => a.type === 'sheep' && a.isAlive);
        
        if (wolves.length === 0) return 'Sheep Win!';
        if (sheep.length === 0) return 'Wolves Win!';
        return null;
    }

    getGameStatus() {
        return {
            round: this.roundNumber,
            votingOpen: this.votingOpen,
            votingTimeRemaining: this.votingTimeRemaining,
            animals: this.animals.map(a => ({
                id: a.id,
                type: a.type,
                health: a.health,
                isAlive: a.isAlive,
                position: { x: a.x, y: a.y },
                lastAction: a.lastAction,
                votes: a.votes
            }))
        };
    }
}

// Battle System
class BattleSystem {
    constructor(gameState) {
        this.gameState = gameState;
    }

    initializeBattle(numWolves, numSheep) {
        this.gameState.animals = [];
        
        // Add wolves
        for (let i = 0; i < numWolves; i++) {
            this.gameState.addAnimal(new Wolf(
                Math.floor(Math.random() * GAME_CONFIG.GRID_SIZE),
                Math.floor(Math.random() * GAME_CONFIG.GRID_SIZE)
            ));
        }
        
        // Add sheep
        for (let i = 0; i < numSheep; i++) {
            this.gameState.addAnimal(new Sheep(
                Math.floor(Math.random() * GAME_CONFIG.GRID_SIZE),
                Math.floor(Math.random() * GAME_CONFIG.GRID_SIZE)
            ));
        }
    }

    executeRound() {
        const messages = [];
        this.gameState.updateTurnOrder();

        // Execute turns in random order
        for (const animal of this.gameState.turnOrder) {
            if (!animal.isAlive) continue;

            if (animal.type === 'wolf') {
                // Wolf AI
                const targets = this.gameState.animals.filter(a => 
                    a.type === 'sheep' && a.isAlive
                );

                if (targets.length > 0) {
                    const nearestSheep = targets.reduce((nearest, sheep) => {
                        const distance = calculateDistance(animal, sheep);
                        return distance < calculateDistance(animal, nearest) ? sheep : nearest;
                    });

                    const distance = calculateDistance(animal, nearestSheep);

                    if (distance <= GAME_CONFIG.WOLF_ATTACK_RANGE) {
                        const attackMessage = animal.attack(nearestSheep, this.gameState);
                        if (attackMessage) messages.push(attackMessage);
                    } else {
                        const dx = Math.sign(nearestSheep.x - animal.x);
                        const dy = Math.sign(nearestSheep.y - animal.y);
                        if (animal.move(dx, dy, this.gameState)) {
                            messages.push(`Wolf ${animal.id} moved toward sheep`);
                        }
                    }
                }
            } else {
                // Sheep AI
                const nearbyWolves = this.gameState.animals.filter(a => 
                    a.type === 'wolf' && 
                    a.isAlive && 
                    calculateDistance(animal, a) <= GAME_CONFIG.WOLF_ATTACK_RANGE + 1
                );

                if (nearbyWolves.length > 0) {
                    const fleeMessage = animal.flee(this.gameState);
                    if (fleeMessage) {
                        messages.push(fleeMessage);
                    } else {
                        const defendMessage = animal.defend(this.gameState);
                        if (defendMessage) messages.push(defendMessage);
                    }
                } else {
                    messages.push(animal.rest());
                }
            }
        }

        this.gameState.roundNumber++;
        this.gameState.battleLog.push(messages);
        return messages;
    }
}

// Initialize game and UI setup
document.addEventListener('DOMContentLoaded', () => {
    const gameState = new GameState();
    const battleSystem = new BattleSystem(gameState);
    let gameBoard;

    function initializeGameBoard() {
        const board = document.getElementById('gameBoard');
        board.style.gridTemplateColumns = `repeat(${GAME_CONFIG.GRID_SIZE}, 30px)`;
        board.innerHTML = '';

        for (let y = 0; y < GAME_CONFIG.GRID_SIZE; y++) {
            for (let x = 0; x < GAME_CONFIG.GRID_SIZE; x++) {
                const cell = document.createElement('div');
                cell.className = 'cell';
                cell.dataset.x = x;
                cell.dataset.y = y;
                board.appendChild(cell);
            }
        }
        gameBoard = board;
    }

    function updateGameBoard() {
        // Clear all cells
        document.querySelectorAll('.cell').forEach(cell => {
            cell.textContent = '';
            cell.className = 'cell';
        });

        // Update cells with animals
        gameState.animals.forEach(animal => {
            if (animal.isAlive) {
                const cell = gameBoard.children[animal.y * GAME_CONFIG.GRID_SIZE + animal.x];
                cell.textContent = animal.type === 'wolf' ? '🐺' : '🐑';
                cell.classList.add(animal.type);
            }
        });
    }

    function updateBattleLog(messages) {
        const battleLog = document.getElementById('battleLog');
        messages.forEach(message => {
            const messageElement = document.createElement('div');
            messageElement.textContent = message;
            battleLog.appendChild(messageElement);
        });
        battleLog.scrollTop = battleLog.scrollHeight;
    }

    function updateAnimalStats() {
        const statsContainer = document.getElementById('animalStats');
        statsContainer.innerHTML = '';

        gameState.animals.forEach(animal => {
            if (animal.isAlive) {
                const card = document.createElement('div');
                card.className = 'animal-card';
                card.innerHTML = `
                    <strong>${animal.type} ${animal.id}</strong><br>
                    Health: ${animal.health}<br>
                    ${animal.type === 'wolf' ? `Stamina: ${animal.stamina}` : `Energy: ${animal.energy}`}<br>
                    Last Action: ${animal.lastAction || 'None'}<br>
                    Votes: ${animal.votes}
                `;
                statsContainer.appendChild(card);
            }
        });
    }

    function showVotingOptions() {
        const votingSection = document.getElementById('votingSection');
        const votingOptions = document.getElementById('votingOptions');
        votingSection.style.display = 'block';
        votingOptions.innerHTML = '';

        gameState.animals
            .filter(animal => animal.isAlive)
            .forEach(animal => {
                const button = document.createElement('button');
                button.textContent = `Vote for ${animal.type} ${animal.id}`;
                button.onclick = () => {
                    gameState.castVote(animal.id);
                    updateAnimalStats();
                };
                votingOptions.appendChild(button);
            });
    }

    // Button Event Listeners
    document.getElementById('startGame').addEventListener('
