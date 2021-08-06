use crate::rect::*;
use crate::collision::*;
use crate::entity::*;

use sdl2::video::Window;
use sdl2::render::Canvas;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::keyboard::KeyboardState;
use sdl2::keyboard::Scancode;
use std::collections::HashMap;
use rand::Rng;

static jump_speed: f32 = -1.5;
static movement_speed: f32 = 0.8;
static coyote_time: f64 = 0.1;

static wall_spacing_range:std::ops::Range<f32> = 0.35..1.0;
static wall_height_range:std::ops::Range<f32> = 0.1..0.6;
static wall_gap: f32 = 0.3;
static wall_w: f32 = 0.1;

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
    next_wall: f32,
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
            next_wall: aspect_ratio,
        };

        state.player_id = state.add_entity(Entity::new_player(aspect_ratio/2.0, 0.4));
        state.add_entity(Entity::new_platform(aspect_ratio/2.0 - 0.5, PlatformHeight::Middle));
    
        return state;
    }

    pub fn draw(&self, canvas: &mut Canvas<Window>, w: u32, h: u32) {
        let mut draw_entity = |entity: &Entity, a: f32| {
            canvas.set_draw_color(entity.colour);
            let x_transformed = entity.aabb.x - self.cam_x;
            let entity_screenspace_rect = sdl2::rect::Rect::new(    
                (x_transformed / a * w as f32) as i32,
                (entity.aabb.y * h as f32) as i32,
             (entity.aabb.w / a * w as f32) as u32,
            (entity.aabb.h * h as f32) as u32,
            );
            canvas.fill_rect(entity_screenspace_rect).unwrap();
        };

            /*
        let a = w as f32 / h as f32;
        for (_, entity) in self.entities.iter() {
            canvas.set_draw_color(entity.colour);
            let x_transformed = entity.aabb.x - self.cam_x;
            let entity_screenspace_rect = sdl2::rect::Rect::new(    
                (x_transformed / a * w as f32) as i32,
                (entity.aabb.y * h as f32) as i32,
                (entity.aabb.w / a * w as f32) as u32,
            (entity.aabb.h * h as f32) as u32,
            );
            canvas.fill_rect(entity_screenspace_rect).unwrap();
        }
        */

        // poor mans bucket sort
        self.entities.iter().filter(|(_, entity)| entity.draw_order == DrawOrder::Front).for_each(|(_, entity)| draw_entity(entity, w as f32/h as f32));
        self.entities.iter().filter(|(_, entity)| entity.draw_order == DrawOrder::Back).for_each(|(_, entity)| draw_entity(entity, w as f32/h as f32));
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

        if self.cam_x >= self.next_wall {
            let height = rand::thread_rng().gen_range(wall_height_range.clone());
            self.add_entity(Entity::new_wall_segment(Rect::new(self.aspect_ratio + self.next_wall, height - 1000.0, wall_w, 1000.0)));
            self.add_entity(Entity::new_wall_segment(Rect::new(self.aspect_ratio + self.next_wall, height + wall_gap, wall_w, 1000.0)));
            self.next_wall += rand::thread_rng().gen_range(wall_spacing_range.clone());
        }

        self.frame_collisions.clear();
        self.frame_movements.clear();

        GameState::apply_gravity(&mut self.entities, self.gravity, dt as f32);
        simulate_collisions(&self.entities, &mut self.frame_collisions, dt as f32);
        compute_movement(&self.entities, &self.frame_collisions, &mut self.frame_movements, dt as f32);
        GameState::apply_movement(&mut self.entities, &self.frame_movements);
        GameState::cease_falling(&mut self.entities, &self.frame_collisions);

        if self.player_is_grounded() {
            self.last_grounded = self.time;
        }

        self.cull_entities();
    }

    pub fn cease_falling(entities: &mut HashMap<u32, Entity>, collisions: &Vec<CollisionEvent>) {
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
        let player = self.entities.get_mut(&self.player_id).unwrap();
        if player.vy < 0.0 {
            player.vy /= 2.0;
        }
    }

    pub fn apply_movement(entities: &mut HashMap<u32, Entity>, movements: &Vec<(u32, f32, f32)>) {
        for (entity_id, dx, dy) in movements {
            let e = entities.get_mut(entity_id).unwrap();
            e.aabb.x += dx;
            e.aabb.y += dy;
        }
    }

    pub fn cull_entities(&mut self) {
        let scr_rect = Rect::new(self.cam_x, 0.0, self.aspect_ratio, 1.0);
        let player_id = self.player_id;
        self.entities.retain(|entity_key, entity| rect_intersection(entity.aabb, scr_rect) || *entity_key == player_id);

        println!("{} entities remain", self.entities.len());
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