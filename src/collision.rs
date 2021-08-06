use crate::rect::*;
use crate::entity::*;

use std::collections::HashMap;


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CollisionDirection {
    Above,
    Left,
    Right,
    Below,
}
pub struct CollisionEvent {
    pub subject: u32,
    pub object: u32,
    pub dir: CollisionDirection,
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

        if x_movt != 0.0 || y_movt != 0.0 {
            movements.push((*entity_key, x_movt, y_movt));
        }
    }
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
        CollisionDirection::Above
    } else if subject_old.top() >= object.bot() && subject_desired.top() <= object.bot() {
        CollisionDirection::Below
    } else {
        CollisionDirection::Below
    }
}