use sdl2::video::Window;
use sdl2::render::Canvas;
use sdl2::pixels::Color;
use std::collections::HashMap;
use std::f32::INFINITY;
use rand::Rng;

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

pub fn rect_intersection(a: Rect, b: Rect) -> bool {
    // not sure if overkill
    let a_in_b_x = (a.left() > b.left() && a.left() < b.right()) || (a.right() > b.left() && a.right() < b.right());
    let b_in_a_x = (b.left() > a.left() && b.left() < a.right()) || (b.right() > a.left() && b.right() < a.right());
    
    let a_in_b_y = (a.top() > b.top() && a.top() < b.bot()) || (a.bot() > b.top() && a.bot() < b.bot());
    let b_in_a_y = (b.top() > a.top() && b.top() < a.bot()) || (b.bot() > a.top() && b.bot() < a.bot());

    let x_overlap = a_in_b_x || b_in_a_x;
    let y_overlap = a_in_b_y || b_in_a_y;

    return x_overlap && y_overlap;
}

pub fn rect_collision_direction(subject_old: Rect, subject_desired: Rect, object: Rect) -> CollisionDirection {
    
    if subject_old.right() < object.left() && subject_desired.right() > object.left() {
        CollisionDirection::Left
    } else if subject_old.left() > object.right() && subject_desired.left() < object.right() {
        CollisionDirection::Right
    } else if subject_old.bot() < object.top() && subject_desired.bot() > object.top() {
        CollisionDirection::Above
    } else if subject_old.top() > object.bot() && subject_desired.top() < object.bot() {
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

#[derive(Debug, Clone, Copy)]
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
    entities: HashMap<u32, Entity>,
    player_id: u32,
    frame_collisions: Vec<CollisionEvent>,
}

impl GameState {
    pub fn new(aspect_ratio: f32, gravity: f32) -> GameState {
        let mut state = GameState {
            aspect_ratio: aspect_ratio,
            gravity: gravity,
            cam_x: 0.0,
            entities: HashMap::new(),
            frame_collisions: Vec::new(),
            player_id: 0,
        };

        state.player_id = state.add_entity(Entity::new_player(aspect_ratio/2.0, 0.4));
        state.add_entity(Entity::new_platform(aspect_ratio/2.0 - 0.5, PlatformHeight::Middle));
    
        return state;
    }

    pub fn draw(&self, canvas: &mut Canvas<Window>, w: u32, h: u32) {
        for (_, entity) in self.entities.iter() {
            canvas.set_draw_color(entity.colour);
            let a = w as f32 / h as f32;
            let entity_screenspace_rect = sdl2::rect::Rect::new(    
                (entity.aabb.x / a * w as f32) as i32,
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

    pub fn update(&mut self, dt: f64) {
        self.frame_collisions.clear();

        GameState::apply_gravity(&mut self.entities, self.gravity, dt as f32);
        GameState::simulate_collisions(&self.entities, &mut self.frame_collisions, dt as f32);
        GameState::apply_movement(&mut self.entities, &self.frame_collisions, dt as f32);
    }

    pub fn apply_gravity(entities: &mut HashMap<u32, Entity>, gravity: f32, dt: f32) {
        for (_, entity) in entities {
            if entity.obeys_gravity {
                entity.vy += dt * gravity;
            }
        }   
    }

    pub fn apply_movement(entities: &mut HashMap<u32, Entity>, collisions: &Vec<CollisionEvent>, dt: f32) {
        for (entity_key, entity) in entities.iter_mut() {
            if collisions.iter().all(|col| col.subject != *entity_key) {
                // not involved in a collision
                entity.aabb.x += entity.vx * dt;
                entity.aabb.y += entity.vy * dt;
            }
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
}