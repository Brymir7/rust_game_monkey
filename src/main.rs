use ::rand::{
    distributions::{Standard, Uniform},
    prelude::*,
};
use std::fmt::Write;
const GRAVITY: f32 = 10.0;
const FLOOR_Y: f32 = 480.0;

use macroquad::prelude::*;
fn window_config() -> Conf {
    Conf {
        window_title: String::from("Monke Dinosauros"),
        fullscreen: false,
        window_resizable: true,
        window_height: FLOOR_Y as i32 + 20,
        window_width: 1024,
        sample_count: 2,
        ..Default::default()
    }
}

#[macroquad::main(window_config)]
async fn main() {
    let mut game = Game::new();
    loop {
        if is_key_pressed(KeyCode::Escape) {
            break;
        }
        if game.update(get_frame_time()) == GameResult::GameOver {
            break;
        };
        game.draw();
        next_frame().await;
    }
    print!("Score: {}", game.score);
}

fn meters_to_pixels(meters: f32) -> f32 {
    meters * 100.0
}

struct Player {
    position: Vec2,
    velocity: Vec2,
    jump_strength: f32,
}
#[derive(Eq, PartialEq)]
enum EnemyKind {
    Ground,
    Bird,
}
struct Enemy {
    position: Vec2,
    velocity: Vec2,
    kind: EnemyKind,
}
struct PowerUp {
    position: Vec2,
    velocity_y: f32,
    duration: f32,
}
struct Resources {
    texture_player: Texture2D,
    texture_powerup: Texture2D,
    texture_enemy: Texture2D,
    texture_bird: Texture2D,
}
impl Resources {
    fn new() -> Resources {
        let image_data =
            std::fs::read(format!("./images/monkey.png")).expect("failed to open image");
        let texture_player = Texture2D::from_file_with_format(&image_data, None);
        texture_player.set_filter(FilterMode::Nearest);
        let image_data =
            std::fs::read(format!("./images/banana.png")).expect("failed to open image");
        let texture_powerup = Texture2D::from_file_with_format(&image_data, None);
        texture_powerup.set_filter(FilterMode::Nearest);
        let image_data =
            std::fs::read(format!("./images/tree1.png")).expect("failed to open image");
        let texture_obstacle = Texture2D::from_file_with_format(&image_data, None);
        texture_obstacle.set_filter(FilterMode::Nearest);
        let image_data = std::fs::read(format!("./images/bird.png")).expect("failed to open image");
        let texture_bird = Texture2D::from_file_with_format(&image_data, None);
        texture_bird.set_filter(FilterMode::Nearest);
        Resources {
            texture_player: texture_player,
            texture_powerup: texture_powerup,
            texture_enemy: texture_obstacle,
            texture_bird: texture_bird,
        }
    }
}
#[derive(PartialEq, Eq)]
enum PowerUpState {
    PowerUpExists,
    PowerUpGone,
    PowerUpConsumed,
}
impl PowerUp {
    fn new(x: f32, y: f32) -> PowerUp {
        PowerUp {
            position: Vec2::new(x, y),
            velocity_y: 0.0,
            duration: 10.0,
        }
    }
    fn update(&mut self, player: &Player, deltatime: f32, resources: &Resources) -> PowerUpState {
        self.velocity_y += meters_to_pixels(GRAVITY) * 0.5 * deltatime;
        self.position.y += self.velocity_y * deltatime;
        if self.position.y > FLOOR_Y {
            self.position.y = FLOOR_Y;
            self.duration -= deltatime;
        }
        if self.duration < 0.0 {
            return PowerUpState::PowerUpGone;
        }
        if player.position.x + resources.texture_player.width() > self.position.x
            && player.position.x < self.position.x + resources.texture_powerup.width()
            && player.position.y - resources.texture_player.height() <= self.position.y
            && player.position.y >= self.position.y - resources.texture_powerup.height()
        {
            return PowerUpState::PowerUpConsumed;
        }

        return PowerUpState::PowerUpExists;
    }
    fn draw(&self, resources: &Resources) {
        draw_texture(
            resources.texture_powerup,
            self.position.x,
            self.position.y - resources.texture_powerup.height(),
            WHITE,
        );
    }
}
struct Game {
    player: Player,
    enemies: Vec<Enemy>,
    powerups: Vec<PowerUp>,
    rng: SmallRng,
    time_until_spawn: f32,
    background: Texture2D,
    score: f32,
    resources: Resources,
}
impl Distribution<EnemyKind> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> EnemyKind {
        match rng.gen_range(0..3) {
            0 => EnemyKind::Ground,
            1 => EnemyKind::Bird,
            2 => EnemyKind::Ground,
            _ => unreachable!(),
        }
    }
}

impl Player {
    fn new() -> Player {
        Player {
            position: Vec2::new(30.0, FLOOR_Y),
            velocity: Vec2::ZERO,
            jump_strength: 7.0,
        }
    }
    fn update(&mut self, deltatime: f32) {
        // Jump
        if is_key_down(KeyCode::W) {
            if self.position.y == FLOOR_Y {
                self.velocity.y = meters_to_pixels(-self.jump_strength);
            }
        } else if self.velocity.y < 0.0 {
            self.velocity.y = 0.0;
        }
        if is_key_down(KeyCode::D) {
            self.velocity.x += meters_to_pixels(6.0) * deltatime;
        }
        if is_key_down(KeyCode::A) {
            self.velocity.x += -meters_to_pixels(6.0) * deltatime;
        }
        if is_key_down(KeyCode::S) {
            self.velocity.y += meters_to_pixels(GRAVITY) * deltatime;
        }

        //Gravity
        self.velocity.y += meters_to_pixels(GRAVITY) * deltatime;
        //Friction
        self.velocity.x *= 0.98;
        // Update position
        self.position += self.velocity * deltatime;

        // Don't fall through ground
        if self.position.y > FLOOR_Y {
            self.position.y = FLOOR_Y;
            self.velocity.y = 0.0;
        }
        if self.position.x < 0.0 {
            self.position.x = 0.0;
        }
        if self.position.x > 1024.0 {
            self.position.x = 0.0;
        }
    }
    fn draw(&self, resources: &Resources) {
        draw_texture(
            resources.texture_player,
            self.position.x,
            self.position.y - resources.texture_player.height(),
            WHITE,
        );
    }
}
enum EnemyUpdateResult {
    EnemyAlive,
    EnemyGone,
    EnemyCollidePlayer,
}

#[derive(PartialEq, Eq)]
enum GameResult {
    GameGoing,
    GameOver,
}
impl Enemy {
    fn new(kind: EnemyKind) -> Enemy {
        Enemy {
            position: Vec2::new(
                screen_width() + 20.0,
                match kind {
                    EnemyKind::Ground => FLOOR_Y,
                    EnemyKind::Bird => FLOOR_Y - 100.0,
                },
            ),
            velocity: (Vec2::ZERO),
            kind: kind,
        }
    }
    fn update(
        &mut self,
        deltatime: f32,
        player: &Player,
        resources: &Resources,
    ) -> EnemyUpdateResult {
        self.velocity.x = meters_to_pixels(150.0) * deltatime;
        self.position.x -= self.velocity.x * deltatime;
        if player.position.x + resources.texture_player.width() > self.position.x
            && player.position.x
                < self.position.x
                    + match self.kind {
                        EnemyKind::Ground => resources.texture_enemy.width(),
                        EnemyKind::Bird => resources.texture_bird.width(),
                    }
            && player.position.y <= self.position.y
            && player.position.y
                >= self.position.y
                    - match self.kind {
                        EnemyKind::Ground => resources.texture_enemy.height(),
                        EnemyKind::Bird => resources.texture_bird.height(),
                    }
        {
            return EnemyUpdateResult::EnemyCollidePlayer;
        } else if self.position.x < 0.0 {
            return EnemyUpdateResult::EnemyGone;
        }
        return EnemyUpdateResult::EnemyAlive;
    }
    fn draw(&self, resources: &Resources) {
        draw_texture(
            match self.kind {
                EnemyKind::Bird => resources.texture_bird,
                EnemyKind::Ground => resources.texture_enemy,
            },
            self.position.x,
            self.position.y
                - match self.kind {
                    EnemyKind::Bird => resources.texture_bird.height(),
                    EnemyKind::Ground => resources.texture_enemy.height(),
                },
            WHITE,
        );
    }
}

impl Game {
    fn new() -> Game {
        let image_data =
            std::fs::read(format!("./images/background.png")).expect("failed to open image");
        let texture = Texture2D::from_file_with_format(&image_data, None);
        texture.set_filter(FilterMode::Nearest);
        Game {
            player: Player::new(),
            enemies: vec![],
            powerups: vec![],
            rng: SmallRng::from_rng(thread_rng()).expect("failed to seed rng"),
            time_until_spawn: 0.0,
            background: texture,
            score: 0.0,
            resources: Resources::new(),
        }
    }
    fn update(&mut self, deltatime: f32) -> GameResult {
        self.time_until_spawn -= deltatime;
        self.score += 20.0 * deltatime;
        let mut game_state = GameResult::GameGoing;
        let mut power_up_state = PowerUpState::PowerUpGone;
        if self.time_until_spawn < 0.0 {
            self.time_until_spawn = self.rng.sample(Uniform::from(1.5..2.0));
            self.enemies.push(Enemy::new(random()));
        }
        self.player.update(deltatime);
        self.enemies.retain_mut(|enemy| {
            match enemy.update(deltatime, &self.player, &self.resources) {
                EnemyUpdateResult::EnemyAlive => true,
                EnemyUpdateResult::EnemyGone => false,
                EnemyUpdateResult::EnemyCollidePlayer => {
                    game_state = GameResult::GameOver;
                    return false;
                }
            }
        });
        self.powerups.retain_mut(|powerup| {
            match powerup.update(&self.player, deltatime, &self.resources) {
                PowerUpState::PowerUpExists => true,
                PowerUpState::PowerUpGone => false,
                PowerUpState::PowerUpConsumed => {
                    power_up_state = PowerUpState::PowerUpConsumed;
                    false
                }
            }
        });
        if power_up_state == PowerUpState::PowerUpConsumed {
            self.player.jump_strength *= 1.1;
        }
        if self.score as i32 % 500 == 0 && self.powerups.len() < 1 {
            self.powerups.push(PowerUp::new(
                self.rng.sample(Uniform::from(
                    0.0..screen_width() - self.resources.texture_powerup.width(),
                )),
                0.0,
            )); // need to know how much to offset widht by to include image size
        }
        return game_state;
    }
    fn draw(&self) {
        clear_background(BLACK);
        let mut score = String::new();
        let amount_of_trees_in_screen: i32 = (screen_width() / self.background.width()) as i32;
        for tree in 0..amount_of_trees_in_screen {
            draw_texture(
                self.background,
                self.background.width() * tree as f32,
                FLOOR_Y - self.background.height(),
                WHITE,
            );
        }

        draw_line(
            0.0,
            FLOOR_Y + 10.0,
            screen_width(),
            FLOOR_Y + 10.0,
            20.0,
            DARKGREEN,
        );
        self.player.draw(&self.resources);
        for enemy in &self.enemies {
            enemy.draw(&self.resources);
        }
        for powerup in &self.powerups {
            powerup.draw(&self.resources);
        }
        write!(score, "Score: {}", self.score).expect("Failed to Write!");
        draw_text(&score, 0.0, 32.0, 32.0, WHITE);
    }
}

//Ebenen oben zumm drauf klettern / dort vogel gegener
