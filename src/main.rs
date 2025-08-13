// Ladybug Runner - an endless side-scrolling runner built with macroquad.
// The code is intentionally compact; this pass adds English documentation and
// clarifying inline comments without altering gameplay logic.
use macroquad::prelude::*;

/// Downward acceleration applied every frame (pixels / s^2).
const GRAVITY: f32 = 1800.0;
/// Initial vertical velocity when jumping (negative => upward).
const JUMP_VEL: f32 = -800.0;
/// Ground band height.
const GROUND_H: f32 = 80.0;
/// Player visual width.
const PLAYER_W: f32 = 60.0;
/// Player visual height.
const PLAYER_H: f32 = 60.0;
/// Starting horizontal world/obstacle speed.
const BASE_SPEED: f32 = 30.0;
/// Base acceleration applied to speed over time while playing.
const SPEED_GROWTH: f32 = 8.0;      // base acceleration
/// Cap so difficulty does not explode.
const MAX_SPEED: f32 = 140.0;       // difficulty ceiling
/// Minimum seconds between spawns (randomized).
const SPAWN_MIN: f32 = 0.9;
/// Maximum seconds between spawns (randomized).
const SPAWN_MAX: f32 = 1.8;

/// High-level game state machine.
#[derive(Clone, Copy, PartialEq)]
enum State {
    /// Start screen / title.
    Menu,
    /// Active gameplay.
    Playing,
    /// Reached after a collision; can restart or return to menu.
    GameOver,
}

/// Player avatar (simple rectangle with reduced internal hitbox).
struct Player {
    /// Top-left position in screen space.
    pos: Vec2,
    /// Current velocity (only Y is used currently).
    vel: Vec2,
    /// Whether the player currently rests on the ground.
    on_ground: bool,
}
impl Player {
    /// Create a player positioned a bit from the left side and above the ground.
    fn new(screen_w: f32, ground_y: f32) -> Self {
        Self {
            pos: vec2(screen_w * 0.2, ground_y - PLAYER_H),
            vel: vec2(0.0, 0.0),
            on_ground: true,
        }
    }
    /// Returns a slightly shrunken collision rectangle (for fairness).
    fn rect(&self) -> Rect {
        Rect {
            x: self.pos.x + 8.0,
            y: self.pos.y + 8.0,
            w: PLAYER_W - 16.0,
            h: PLAYER_H - 16.0,
        } // “hitbox” levemente menor
    }
    /// Integrate motion & handle jump / ground collision.
    fn update(&mut self, dt: f32, ground_y: f32, jump_pressed: bool) {
        if jump_pressed && self.on_ground {
            self.vel.y = JUMP_VEL;
            self.on_ground = false;
        }
        self.vel.y += GRAVITY * dt;
        self.pos.y += self.vel.y * dt;

        // Clamp to ground and reset vertical motion if landing.
        if self.pos.y + PLAYER_H >= ground_y {
            self.pos.y = ground_y - PLAYER_H;
            self.vel.y = 0.0;
            self.on_ground = true;
        }
    }
    /// Draw the player using primitive shapes (acts as placeholder art).
    fn draw(&self) {
        // Body
        draw_rectangle(self.pos.x, self.pos.y, PLAYER_W, PLAYER_H, RED);
        // Simple "antennae" detail.
        draw_circle(self.pos.x + 12.0, self.pos.y + 12.0, 6.0, BLACK);
        draw_circle(self.pos.x + 48.0, self.pos.y + 12.0, 6.0, BLACK);
    }
}

struct Obstacle {
    pos: Vec2,
    size: Vec2,
    speed: f32,
}
impl Obstacle {
    /// Create an obstacle with randomized width/height anchored on ground.
    fn new(x: f32, ground_y: f32, speed: f32) -> Self {
        // Random simple dimensions.
        let w = rand::gen_range(40.0, 70.0);
        let h = rand::gen_range(50.0, 120.0);
        Self {
            pos: vec2(x, ground_y - h),
            size: vec2(w, h),
            speed,
        }
    }
    /// Collision rectangle (slightly inset for fairness / readability).
    fn rect(&self) -> Rect {
        Rect {
            x: self.pos.x + 4.0,
            y: self.pos.y + 4.0,
            w: self.size.x - 8.0,
            h: self.size.y - 8.0,
        }
    }
    /// Move left according to current speed.
    fn update(&mut self, dt: f32) {
        self.pos.x -= self.speed * dt;
    }
    /// Render using a rectangle plus small decorative dots.
    fn draw(&self) {
        draw_rectangle(self.pos.x, self.pos.y, self.size.x, self.size.y, DARKGREEN);
        // Dots to give some texture / style.
        for i in 0..3 {
            draw_circle(
                self.pos.x + 10.0 + 12.0 * i as f32,
                self.pos.y + 10.0,
                3.0,
                BLACK,
            );
        }
    }
    /// Whether the obstacle has completely left the screen on the left side.
    fn offscreen(&self) -> bool {
        self.pos.x + self.size.x < -10.0
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    // Runtime state variables.
    let mut state = State::Menu;
    let mut score: f32 = 0.0;
    let mut hi_score: f32 = 0.0;
    let mut spawn_t: f32 = 0.0;
    let mut next_spawn: f32 = rand::gen_range(SPAWN_MIN, SPAWN_MAX);
    let mut speed = BASE_SPEED;
    let mut obstacles: Vec<Obstacle> = Vec::new();
    // Player is allocated only when starting the game to ensure fresh state.
    let mut player: Option<Player> = None;

    loop {
        let dt = get_frame_time();
        let sw = screen_width();
        let sh = screen_height();
        let ground_y = sh - GROUND_H;

        clear_background(Color::from_rgba(240, 245, 250, 255));

        // Simple layered background for parallax suggestion.
        draw_rectangle(0.0, ground_y - 120.0, sw, 20.0, LIGHTGRAY);
        draw_rectangle(0.0, ground_y - 60.0, sw, 15.0, GRAY);
        draw_rectangle(
            0.0,
            ground_y,
            sw,
            GROUND_H,
            Color::from_rgba(210, 230, 210, 255),
        );

        // Unified input (desktop + mobile / touch).
        let jump_pressed = is_key_pressed(KeyCode::Space)
            || is_mouse_button_pressed(MouseButton::Left)
            || !touches().is_empty(); // any touch on screen

        // Finite state machine dispatch.
        match state {
            State::Menu => {
                // Title / instructions.
                draw_text("Ladybug Runner", 32.0, 80.0, 48.0, BLACK);
                draw_text("Toque ou SPACE para jogar", 32.0, 130.0, 28.0, DARKGRAY);
                draw_text(
                    &format!("Recorde: {}", hi_score as i32),
                    32.0,
                    170.0,
                    24.0,
                    DARKGRAY,
                );

                if jump_pressed {
                    // Reset transient state for a fresh run.
                    score = 0.0;
                    speed = BASE_SPEED;
                    obstacles.clear();
                    spawn_t = 0.0;
                    next_spawn = rand::gen_range(SPAWN_MIN, SPAWN_MAX);
                    player = Some(Player::new(sw, ground_y));
                    state = State::Playing;
                }
            }
            State::Playing => {
                let player_ref = player.as_mut().expect("Player inexistente");

                // Update player physics.
                player_ref.update(dt, ground_y, jump_pressed);

                // Difficulty scaling (capped).
                speed = (speed + SPEED_GROWTH * dt).min(MAX_SPEED);

                // Obstacle spawning timer.
                spawn_t += dt;
                if spawn_t >= next_spawn {
                    obstacles.push(Obstacle::new(sw + 40.0, ground_y, speed));
                    spawn_t = 0.0;
                    next_spawn = rand::gen_range(SPAWN_MIN.max(0.5), SPAWN_MAX);
                }

                // Update & render obstacles; check collisions.
                let mut alive = true;
                for o in obstacles.iter_mut() {
                    o.update(dt);
                    o.draw();
                    if o.rect().overlaps(&player_ref.rect()) {
                        alive = false;
                    }
                }
                obstacles.retain(|o| !o.offscreen());

                // Score accrues with speed & time.
                score += speed * dt * 0.1;

                // Draw player last for layering.
                player_ref.draw();

                // Heads-up display.
                draw_text(
                    &format!("Score: {}", score as i32),
                    24.0,
                    32.0,
                    32.0,
                    BLACK,
                );
                draw_text(
                    &format!("Hi: {}", hi_score as i32),
                    24.0,
                    64.0,
                    24.0,
                    DARKGRAY,
                );

                if !alive {
                    if score > hi_score { hi_score = score; }
                    state = State::GameOver;
                }
            }
            State::GameOver => {
                draw_text("Game Over!", 32.0, 80.0, 48.0, MAROON);
                draw_text(&format!("Score: {}", score as i32), 32.0, 130.0, 32.0, BLACK);
                draw_text(
                    &format!("Recorde: {}", hi_score as i32),
                    32.0,
                    170.0,
                    28.0,
                    DARKGRAY,
                );
                draw_text("Toque/SPACE para reiniciar | M para Menu", 32.0, 210.0, 24.0, DARKGRAY);
                if is_key_pressed(KeyCode::M) { state = State::Menu; }
                else if jump_pressed {
                    // Direct restart keeping the hi-score.
                    score = 0.0;
                    speed = BASE_SPEED;
                    obstacles.clear();
                    spawn_t = 0.0;
                    next_spawn = rand::gen_range(SPAWN_MIN, SPAWN_MAX);
                    player = Some(Player::new(sw, ground_y));
                    state = State::Playing;
                }
            }
        }

        next_frame().await;
    }
}

fn window_conf() -> Conf {
    Conf {
        window_title: "Ladybug Runner".to_string(),
        high_dpi: true,
        fullscreen: false, // ignored on mobile
        sample_count: 4,
        ..Default::default()
    }
}

