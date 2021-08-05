use sdl2::video::Window;
use sdl2::render::Canvas;
use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::keyboard::KeyboardState;
use sdl2::keyboard::Scancode;
use std::collections::HashMap;
use rand::Rng;
use std::time::{SystemTime, UNIX_EPOCH};


static jump_speed: f32 = -1.5;
static movement_speed: f32 = 0.8;
static coyote_time: f64 = 0.1;


#[derive(Debug, Clone, Copy)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl Rect {
    pub fn new(x: f32, y: f32, w: f32, h: f32) -> Rect {
        Rect {x: x, y: y, w: w, h: h}
    }

    pub fn dilate(&self, d: f32) -> Rect {
        return Rect::new(self.x - d, self.y - d, self.w + 2.0*d, self.h + 2.0*d);
    }

    pub fn left(self) -> f32 {
        self.x
    }
    pub fn right(self) -> f32 {
        self.x + self.w
    }
    pub fn top(self) -> f32 {
        self.y
    }
    pub fn bot(self) -> f32 {
        self.y + self.h
    }
}

#[test]
fn test_intersection() {
    assert_eq!(rect_intersection(Rect::new(0.0, 0.0, 1.0, 1.0,), Rect::new(0.5, 0.0, 1.0, 1.0)), true);
    assert_eq!(rect_intersection(Rect::new(0.0, 0.0, 1.0, 1.0,), Rect::new(-0.5, 0.0, 1.0, 1.0)), true);
    assert_eq!(rect_intersection(Rect::new(0.0, 0.0, 1.0, 1.0,), Rect::new(0.0, 0.5, 1.0, 1.0)), true);
    assert_eq!(rect_intersection(Rect::new(0.0, 0.0, 1.0, 1.0,), Rect::new(0.0, -0.5, 1.0, 1.0)), true);

    assert_eq!(rect_intersection(Rect::new(0.0, 0.0, 1.0, 1.0,), Rect::new(0.5, -0.05, 0.1, 0.1)), true);
}

pub fn rect_intersection(a: Rect, b: Rect) -> bool {
    //let epsilon = 0.001f32;
    let epsilon = 0.0;
    let a_d = a.dilate(-epsilon);
    let b_d = b.dilate(-epsilon);
    
    // not sure if overkill
    let a_in_b_x = (a_d.left() >= b_d.left() && a_d.left() <= b_d.right()) || (a_d.right() >= b_d.left() && a_d.right() <= b_d.right());
    let b_in_a_x = (b_d.left() >= a_d.left() && b_d.left() <= a_d.right()) || (b_d.right() >= a_d.left() && b_d.right() <= a_d.right());
    
    let a_in_b_y = (a_d.top() >= b_d.top() && a_d.top() <= b_d.bot()) || (a_d.bot() >= b_d.top() && a_d.bot() <= b_d.bot());
    let b_in_a_y = (b_d.top() >= a_d.top() && b_d.top() <= a_d.bot()) || (b_d.bot() >= a_d.top() && b_d.bot() <= a_d.bot());

    let x_overlap = a_in_b_x || b_in_a_x;
    let y_overlap = a_in_b_y || b_in_a_y;

    return x_overlap && y_overlap;
}

#[test]
fn test_rcd() {
    {
        let sold = Rect::new(0.0, 0.0, 1.0, 1.0);
        let snew = Rect::new(0.2, 0.0, 1.0, 1.0);
        let obj = Rect::new(1.1, 0.0, 1.0, 1.0);

        assert_eq!(rect_collision_direction(sold, snew, obj), CollisionDirection::Left);
    }
    {
        let sold = Rect::new(0.0, 0.0, 1.0, 1.0);
        let snew = Rect::new(0.0, 0.2, 1.0, 1.0);
        let obj = Rect::new(0.0, 1.1, 1.0, 1.0);

        assert_eq!(rect_collision_direction(sold, snew, obj), CollisionDirection::Above);
    }
    {
        let sold = Rect::new(1.1, 0.0, 1.0, 1.0);
        let snew = Rect::new(0.9, 0.0, 1.0, 1.0);
        let obj = Rect::new(0.0, 0.0, 1.0, 1.0);

        assert_eq!(rect_collision_direction(sold, snew, obj), CollisionDirection::Right);
    }
    {
        let sold = Rect::new(0.0, 1.1, 1.0, 1.0);
        let snew = Rect::new(0.9, 0.9, 1.0, 1.0);
        let obj = Rect::new(0.0, 0.0, 1.0, 1.0);

        assert_eq!(rect_collision_direction(sold, snew, obj), CollisionDirection::Below);
    }
}

pub fn rect_collision_direction(subject_old: Rect, subject_desired: Rect, object: Rect) -> CollisionDirection {
    if subject_old.right() <= object.left() && subject_desired.right() >= object.left() {
        CollisionDirection::Left
    } else if subject_old.left() >= object.right() && subject_desired.left() <= object.right() {
        CollisionDirection::Right
    } else if subject_old.bot() <= object.top() && subject_desired.bot() >= object.top() {
        println!("above!");
        CollisionDirection::Above
    } else if subject_old.top() >= object.bot() && subject_desired.top() <= object.bot() {
        println!("below collision");
        CollisionDirection::Below
    } else {
        println!("bad collision");
        CollisionDirection::Below
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Entity {
    pub aabb: Rect,
    pub deadly: bool,
    pub obeys_gravity: bool,
    pub colour: Color,
    pub vx: f32,
    pub vy: f32,
}

pub enum PlatformHeight {
    Top,
    Middle,
    Bottom,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CollisionDirection {
    Above,
    Left,
    Right,
    Below,
}

pub struct CollisionEvent {
    subject: u32,
    object: u32,
    dir: CollisionDirection,
}

impl Entity {
    pub fn new_player(x: f32, y: f32) -> Entity {
        Entity {
            aabb: Rect::new(x, y, 0.05, 0.05),
            deadly: false,
            obeys_gravity: true,
            colour: Color::RGB(255, 255, 255),
            vx: 0.0, vy: 0.0,
        }
    }

    pub fn new_platform(player_x: f32, which: PlatformHeight) -> Entity {
        Entity {
            aabb: match which {
                PlatformHeight::Top => Rect::new(player_x + 0.6, 0.2, 0.1, 0.05),
                PlatformHeight::Middle => Rect::new(player_x + 0.5, 0.5, 0.1, 0.05),
                PlatformHeight::Bottom => Rect::new(player_x + 0.4, 0.8, 0.1, 0.05),
            },
            deadly: false,
            obeys_gravity: false,
            colour: match which {
                PlatformHeight::Top => Color::RGB(255,0,0),
                PlatformHeight::Middle => Color::RGB(0, 255, 0),
                PlatformHeight::Bottom => Color::RGB(0, 0, 255),
            },
            vx: 0.0, vy: 0.0,
        }
    }

    pub fn new_wall_segment(r: Rect) -> Entity {
        Entity {
            aabb: r,
            deadly: true,
            obeys_gravity: false,
            colour: Color::RGB(150, 150, 150),
            vx: 0.0, vy: 0.0,
        }
    }
}

pub struct GameState {
    aspect_ratio: f32,
    gravity: f32,
    cam_x: f32,
    cam_vx: f32,
    entities: HashMap<u32, Entity>,
    player_id: u32,
    frame_collisions: Vec<CollisionEvent>,
    frame_movements: Vec<(u32, f32, f32)>,
    time: f64,
    last_grounded: f64,
}

impl GameState {
    pub fn new(aspect_ratio: f32, gravity: f32, cam_vx: f32) -> GameState {
        let mut state = GameState {
            aspect_ratio: aspect_ratio,
            gravity: gravity,
            cam_x: 0.0,
            cam_vx: cam_vx,
            entities: HashMap::new(),
            frame_collisions: Vec::new(),
            frame_movements: Vec::new(),
            player_id: 0,
            time: 0.0,
            last_grounded: 0.0,
        };

        state.player_id = state.add_entity(Entity::new_player(aspect_ratio/2.0, 0.4));
        state.add_entity(Entity::new_platform(aspect_ratio/2.0 - 0.5, PlatformHeight::Middle));
    
        return state;
    }

    pub fn draw(&self, canvas: &mut Canvas<Window>, w: u32, h: u32) {
        for (_, entity) in self.entities.iter() {
            canvas.set_draw_color(entity.colour);
            let a = w as f32 / h as f32;
            let x_transformed = entity.aabb.x - self.cam_x;
            let entity_screenspace_rect = sdl2::rect::Rect::new(    
                (x_transformed / a * w as f32) as i32,
                (entity.aabb.y * h as f32) as i32,
             (entity.aabb.w / a * w as f32) as u32,
            (entity.aabb.h * h as f32) as u32,
            );
            canvas.fill_rect(entity_screenspace_rect).unwrap();
        }
    }

    pub fn add_entity(&mut self, entity: Entity) -> u32 {
        let key = rand::thread_rng().gen();
        self.entities.insert(key, entity);
        return key;
    }

    pub fn update_held_keys(&mut self, keys: &KeyboardState) {
        if keys.is_scancode_pressed(Scancode::A) {
            self.entities.get_mut(&self.player_id).unwrap().vx = -movement_speed;
        } else if keys.is_scancode_pressed(Scancode::D) {
            self.entities.get_mut(&self.player_id).unwrap().vx = movement_speed;
        } else {
            self.entities.get_mut(&self.player_id).unwrap().vx = 0.0;
        }
    }

    pub fn update(&mut self, dt: f64) {
        self.time += dt;
        self.cam_x += self.cam_vx * dt as f32;

        self.frame_collisions.clear();
        self.frame_movements.clear();

        GameState::apply_gravity(&mut self.entities, self.gravity, dt as f32);
        GameState::simulate_collisions(&self.entities, &mut self.frame_collisions, dt as f32);
        GameState::compute_movement(&self.entities, &self.frame_collisions, &mut self.frame_movements, dt as f32);
        GameState::apply_movement(&mut self.entities, &self.frame_movements);
        GameState::cease_falling(&mut self.entities, &self.frame_collisions);

        if self.player_is_grounded() {
            self.last_grounded = self.time;
        }
    }

    pub fn cease_falling(entities: &mut HashMap<u32, Entity>, collisions: &Vec<CollisionEvent>) {
        // maybe ask the collision buffer instead, if collision down vy = 0
        // if collision up vy = 0

        for (entity_key, entity) in entities {
            if entity.obeys_gravity && collisions.iter().any(|col| 
                    col.subject == *entity_key && 
                    (col.dir == CollisionDirection::Above || 
                    col.dir == CollisionDirection::Below)) {
                entity.vy = 0.0;
            }
        }
    }

    pub fn apply_gravity(entities: &mut HashMap<u32, Entity>, gravity: f32, dt: f32) {
        for (_, entity) in entities {
            if entity.obeys_gravity {
                entity.vy += dt * gravity;
            }
        }   
    }

    pub fn player_is_grounded(&self) -> bool {
        self.frame_collisions.iter().any(|col| col.subject == self.player_id && col.dir == CollisionDirection::Above)
    }

    pub fn try_jump(&mut self) {
        // how to check if grounded? 
        if self.time - self.last_grounded < coyote_time {
            self.entities.get_mut(&self.player_id).unwrap().vy = jump_speed;
        }
    }

    pub fn release_jump(&mut self) {
        // how to check if grounded? 
        let player = self.entities.get_mut(&self.player_id).unwrap();
        if player.vy < 0.0 {
            player.vy /= 2.0;
        }
    }

    // maybe comput movt and spit it out to a buffer
    // then apply it

    pub fn compute_movement(entities: &HashMap<u32, Entity>, collisions: &Vec<CollisionEvent>, movements: &mut Vec<(u32, f32, f32)>, dt: f32) {
        for (entity_key, entity) in entities.iter() {

            let max_x = collisions
                .iter()
                .filter(|col| col.subject == *entity_key)
                .filter(|col | col.dir == CollisionDirection::Left)
                .map(|col| -entity.aabb.x -entity.aabb.w + entities.get(&col.object).unwrap().aabb.x)
                .fold(f32::INFINITY, |a, b| a.min(b));

            let max_y = collisions
                .iter()
                .filter(|col| col.subject == *entity_key)
                .filter(|col | col.dir == CollisionDirection::Above)
                .map(|col| -entity.aabb.y -entity.aabb.h + entities.get(&col.object).unwrap().aabb.y)
                .fold(f32::INFINITY, |a, b| a.min(b));
            
            let min_x =  collisions
                .iter()
                .filter(|col| col.subject == *entity_key)
                .filter(|col | col.dir == CollisionDirection::Right)
                .map(|col| -entity.aabb.x + entities.get(&col.subject).unwrap().aabb.w + entities.get(&col.object).unwrap().aabb.x)
                .fold(-f32::INFINITY, |a, b| a.max(b));

            let min_y =  collisions
                .iter()
                .filter(|col| col.subject == *entity_key)
                .filter(|col | col.dir == CollisionDirection::Below)
                .map(|col| -entity.aabb.y + entities.get(&col.subject).unwrap().aabb.h + entities.get(&col.object).unwrap().aabb.y)
                .fold(-f32::INFINITY, |a, b| a.max(b));

            let x_movt = match entity.vx*dt {
                vx if vx > 0.0 => {vx.min(max_x)},
                vx if vx < 0.0 => {vx.max(min_x)},
                _ => 0.0,
            };

            let y_movt = match entity.vy*dt {
                vy if vy > 0.0 => {vy.min(max_y)},
                vy if vy < 0.0 => {vy.max(min_y)},
                _ => 0.0,
            };

            println!("entity y {} h {} vy {} y movt {}", entity.aabb.y, entity.aabb.h, entity.vy*dt, y_movt);
            println!("x ({},{}) y ({}, {})", min_x, max_x, min_y, max_y);

            if x_movt != 0.0 || y_movt != 0.0 {
                println!("pushing movt {:?} {:?} {:?}", entity_key, x_movt, y_movt);
                movements.push((*entity_key, x_movt, y_movt));
            }
        }
    }

    pub fn apply_movement(entities: &mut HashMap<u32, Entity>, movements: &Vec<(u32, f32, f32)>) {
        for (entity_id, dx, dy) in movements {
            let e = entities.get_mut(entity_id).unwrap();
            e.aabb.x += dx;
            e.aabb.y += dy;
        }
    }

    // chucks them into the vec
    pub fn simulate_collisions(entities: &HashMap<u32, Entity>, collisions: &mut Vec<CollisionEvent>, t: f32) {
        for (subject_key, subject) in entities {
            for (object_key, object) in entities {
                if *subject_key == *object_key {continue};

                let dx = subject.vx * t;
                let dy = subject.vy * t;
                let subject_rect_old = subject.aabb;
                let subject_rect_desired = Rect {
                    x: subject_rect_old.x + dx,
                    y: subject_rect_old.y + dy,
                    w: subject_rect_old.w,
                    h: subject_rect_old.h,
                };
                let object_rect = object.aabb;

                if rect_intersection(subject_rect_desired, object_rect) {
                    let collision_dir = rect_collision_direction(subject_rect_old, subject_rect_desired, object_rect);
                    collisions.push(CollisionEvent {
                        dir: collision_dir,
                        subject: *subject_key,
                        object: *object_key,
                    });
                }
            }
        }
    }

    pub fn handle_input(&mut self, e: Event) {
        match e {
            //Event::KeyDown{keycode: Some(Keycode::A), ..} => {}
            Event::KeyDown{keycode: Some(Keycode::Space), ..} => {self.try_jump()}
            Event::KeyUp{keycode: Some(Keycode::Space), ..} => {self.release_jump()}
            Event::KeyDown{keycode: Some(Keycode::J), ..} => { self.add_entity(Entity::new_platform(
                self.entities.get(&self.player_id).unwrap().aabb.x, 
                PlatformHeight::Bottom));}
            Event::KeyDown{keycode: Some(Keycode::K), ..} => { self.add_entity(Entity::new_platform(
                self.entities.get(&self.player_id).unwrap().aabb.x, 
                PlatformHeight::Middle));}
            Event::KeyDown{keycode: Some(Keycode::L), ..} => { self.add_entity(Entity::new_platform(
                self.entities.get(&self.player_id).unwrap().aabb.x, 
                PlatformHeight::Top));}
            _ => {}
        }
    }
}