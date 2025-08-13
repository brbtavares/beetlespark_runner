use macroquad::experimental::collections::storage::store;
use macroquad::prelude::*;

const GRAVITY: f32 = 1800.0;
const JUMP_VEL: f32 = -800.0;
const GROUND_H: f32 = 80.0;
const PLAYER_W: f32 = 60.0;
const PLAYER_H: f32 = 60.0;
const BASE_SPEED: f32 = 30.0;
const SPAWN_MIN: f32 = 0.9;
const SPAWN_MAX: f32 = 1.8;

#[derive(Clone, Copy, PartialEq)]
enum State {
    Menu,
    Playing,
    GameOver,
}

struct Player {
    pos: Vec2,
    vel: Vec2,
    on_ground: bool,
}
impl Player {
    fn new(screen_w: f32, ground_y: f32) -> Self {
        Self {
            pos: vec2(screen_w * 0.2, ground_y - PLAYER_H),
            vel: vec2(0.0, 0.0),
            on_ground: true,
        }
    }
    fn rect(&self) -> Rect {
        Rect {
            x: self.pos.x + 8.0,
            y: self.pos.y + 8.0,
            w: PLAYER_W - 16.0,
            h: PLAYER_H - 16.0,
        } // “hitbox” levemente menor
    }
    fn update(&mut self, dt: f32, ground_y: f32, jump_pressed: bool) {
        if jump_pressed && self.on_ground {
            self.vel.y = JUMP_VEL;
            self.on_ground = false;
        }
        self.vel.y += GRAVITY * dt;
        self.pos.y += self.vel.y * dt;

        // chão
        if self.pos.y + PLAYER_H >= ground_y {
            self.pos.y = ground_y - PLAYER_H;
            self.vel.y = 0.0;
            self.on_ground = true;
        }
    }
    fn draw(&self) {
        // placeholder: corpo
        draw_rectangle(self.pos.x, self.pos.y, PLAYER_W, PLAYER_H, RED);
        // “anteninhas”/detalhe simples
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
    fn new(x: f32, ground_y: f32, speed: f32) -> Self {
        // alturas/laguras aleatórias simples
        let w = rand::gen_range(40.0, 70.0);
        let h = rand::gen_range(50.0, 120.0);
        Self {
            pos: vec2(x, ground_y - h),
            size: vec2(w, h),
            speed,
        }
    }
    fn rect(&self) -> Rect {
        Rect {
            x: self.pos.x + 4.0,
            y: self.pos.y + 4.0,
            w: self.size.x - 8.0,
            h: self.size.y - 8.0,
        }
    }
    fn update(&mut self, dt: f32) {
        self.pos.x -= self.speed * dt;
    }
    fn draw(&self) {
        draw_rectangle(self.pos.x, self.pos.y, self.size.x, self.size.y, DARKGREEN);
        // “pontos” para dar cara de inseto/folha
        for i in 0..3 {
            draw_circle(
                self.pos.x + 10.0 + 12.0 * i as f32,
                self.pos.y + 10.0,
                3.0,
                BLACK,
            );
        }
    }
    fn offscreen(&self) -> bool {
        self.pos.x + self.size.x < -10.0
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut state = State::Menu;
    let mut score: f32 = 0.0;
    let mut hi_score: f32 = 0.0;

    let (mut spawn_t, mut next_spawn) = (0.0f32, rand::gen_range(SPAWN_MIN, SPAWN_MAX));
    let mut speed = BASE_SPEED;

    let mut obstacles: Vec<Obstacle> = vec![];

    loop {
        let dt = get_frame_time();
        let sw = screen_width();
        let sh = screen_height();
        let ground_y = sh - GROUND_H;

        clear_background(Color::from_rgba(240, 245, 250, 255));

        // fundo/parallax simples
        draw_rectangle(0.0, ground_y - 120.0, sw, 20.0, LIGHTGRAY);
        draw_rectangle(0.0, ground_y - 60.0, sw, 15.0, GRAY);
        draw_rectangle(
            0.0,
            ground_y,
            sw,
            GROUND_H,
            Color::from_rgba(210, 230, 210, 255),
        );

        // input (desktop + mobile)
        let jump_pressed = is_key_pressed(KeyCode::Space)
            || is_mouse_button_pressed(MouseButton::Left)
            || !touches().is_empty(); // toque na tela

        // estados
        match state {
            State::Menu => {
                // título
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
                    // reset
                    score = 0.0;
                    speed = BASE_SPEED;
                    obstacles.clear();
                    // cria player
                    let mut player = Player::new(sw, ground_y);
                    // guarda dentro de storage de frame (hack simples)
                    set_app_state(
                        player, obstacles, speed, spawn_t, next_spawn, state, hi_score, score,
                    );
                    state = State::Playing;
                }
            }
            State::Playing => {
                // recupera estado mutável
                let (
                    mut player,
                    mut obstacles_,
                    mut speed_,
                    mut spawn_t_,
                    mut next_spawn_,
                    _,
                    mut hi_score_,
                    mut score_,
                ) = get_app_state(sw, ground_y);

                // update player
                player.update(dt, ground_y, jump_pressed);

                // dificuldade escala com o tempo
                speed_ += 8.0 * dt;

                // spawner
                spawn_t_ += dt;
                if spawn_t_ >= next_spawn_ {
                    obstacles_.push(Obstacle::new(sw + 40.0, ground_y, speed_));
                    spawn_t_ = 0.0;
                    next_spawn_ = rand::gen_range(SPAWN_MIN.max(0.5), SPAWN_MAX);
                }

                // update/draw obstacles
                let mut alive = true;
                for o in obstacles_.iter_mut() {
                    o.update(dt);
                    o.draw();
                    if o.rect().overlaps(&player.rect()) {
                        alive = false;
                    }
                }
                obstacles_.retain(|o| !o.offscreen());

                // score
                score_ += speed_ * dt * 0.1;

                // draw player por cima
                player.draw();

                // HUD
                draw_text(
                    &format!("Score: {}", score_ as i32),
                    24.0,
                    32.0,
                    32.0,
                    BLACK,
                );
                draw_text(
                    &format!("Hi: {}", hi_score_ as i32),
                    24.0,
                    64.0,
                    24.0,
                    DARKGRAY,
                );

                if !alive {
                    if score_ > hi_score_ {
                        hi_score_ = score_;
                    }
                    set_app_state(
                        player,
                        obstacles_,
                        speed_,
                        spawn_t_,
                        next_spawn_,
                        State::GameOver,
                        hi_score_,
                        score_,
                    );
                    state = State::GameOver;
                } else {
                    set_app_state(
                        player,
                        obstacles_,
                        speed_,
                        spawn_t_,
                        next_spawn_,
                        State::Playing,
                        hi_score_,
                        score_,
                    );
                }
            }
            State::GameOver => {
                let (_, _, _, _, _, _, hi, sc) = get_app_state(sw, ground_y);
                draw_text("Game Over!", 32.0, 80.0, 48.0, MAROON);
                draw_text(&format!("Score: {}", sc as i32), 32.0, 130.0, 32.0, BLACK);
                draw_text(
                    &format!("Recorde: {}", hi as i32),
                    32.0,
                    170.0,
                    28.0,
                    DARKGRAY,
                );
                draw_text("Toque/SPACE para reiniciar", 32.0, 210.0, 24.0, DARKGRAY);
                if jump_pressed {
                    state = State::Menu;
                }
            }
        }

        next_frame().await;
    }
}

// --- armazenamento simples no “storage” do frame (para evitar usar globals estáticos) ---
#[allow(clippy::too_many_arguments)]
fn set_app_state(
    player: Player,
    obstacles: Vec<Obstacle>,
    speed: f32,
    spawn_t: f32,
    next_spawn: f32,
    _state: State,
    hi_score: f32,
    score: f32,
) {
    store(player);
    store(obstacles);
    store(speed);
    store(spawn_t);
    store(next_spawn);
    store(hi_score);
    store(score);
}
#[allow(clippy::type_complexity)]
fn get_app_state(
    _sw: f32,
    _ground_y: f32,
) -> (Player, Vec<Obstacle>, f32, f32, f32, State, f32, f32) {
    (
        storage::get::<Player>().unwrap(),
        storage::get::<Vec<Obstacle>>().unwrap(),
        storage::get::<f32>().unwrap(), // speed
        storage::get::<f32>().unwrap(), // spawn_t
        storage::get::<f32>().unwrap(), // next_spawn
        State::Playing,
        storage::get::<f32>().unwrap(), // hi
        storage::get::<f32>().unwrap(), // score
    )
}

fn window_conf() -> Conf {
    Conf {
        window_title: "Ladybug Runner".to_string(),
        high_dpi: true,
        fullscreen: false, // em mobile ignora
        sample_count: 4,
        ..Default::default()
    }
}

