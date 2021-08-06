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
    pub subject_rect: Rect,
    pub object_rect: Rect,
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
                    subject_rect: subject.aabb,
                    object_rect: object.aabb,
                });
            }
        }
    }
}

fn movement_bounds(subject_key: u32, collisions: &Vec<CollisionEvent>) -> (f32, f32, f32, f32) {
    let max_dx = collisions.iter().filter(|col| col.subject == subject_key)
        .filter(|col| col.dir == CollisionDirection::Left)
        .map(|col| col.object_rect.left() - col.subject_rect.right())
        .fold(f32::INFINITY, |a, b| a.min(b));

    let max_dy = collisions.iter().filter(|col| col.subject == subject_key)
        .filter(|col| col.dir == CollisionDirection::Above)
        .map(|col| col.object_rect.top() - col.subject_rect.bot())
        .fold(f32::INFINITY, |a, b| a.min(b));
        
    let min_dx = collisions.iter().filter(|col| col.subject == subject_key)
        .filter(|col| col.dir == CollisionDirection::Right)
        .map(|col| col.object_rect.right() - col.subject_rect.left())
        .fold(-f32::INFINITY, |a, b| a.max(b));

    let min_dy = collisions.iter().filter(|col| col.subject == subject_key)
        .filter(|col| col.dir == CollisionDirection::Below)
        .map(|col| col.object_rect.bot() - col.subject_rect.top())
        .fold(-f32::INFINITY, |a, b| a.max(b));

    return (min_dx, max_dx, min_dy, max_dy);
}

fn clamp(val: f32, min: f32, max: f32) -> f32 {
    match val {
        val if val <= min => min,
        val if val >= max => max,
        _ => val
    }
}

pub fn compute_movement(entities: &HashMap<u32, Entity>, collisions: &Vec<CollisionEvent>, movements: &mut Vec<(u32, f32, f32)>, dt: f32) {
    for (entity_key, entity) in entities.iter() {
        let (min_x, max_x, min_y, max_y) = movement_bounds(*entity_key, collisions);
        let x_movt = clamp(entity.vx * dt, min_x, max_x);
        let y_movt = clamp(entity.vy * dt, min_y, max_y);

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
        println!("bad collision");
        CollisionDirection::Below
    }
}