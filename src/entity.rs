use crate::rect::*;

use sdl2::pixels::Color;


pub enum PlatformHeight {
    Top,
    Middle,
    Bottom,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DrawOrder {
    Front,
    Back,
}

#[derive(Debug, Clone, Copy)]
pub struct Entity {
    pub aabb: Rect,
    pub deadly: bool,
    pub obeys_gravity: bool,
    pub colour: Color,
    pub vx: f32,
    pub vy: f32,
    pub draw_order: DrawOrder,
}

impl Entity {
    pub fn new_player(x: f32, y: f32) -> Entity {
        Entity {
            aabb: Rect::new(x, y, 0.05, 0.05),
            deadly: false,
            obeys_gravity: true,
            colour: Color::RGB(255, 255, 255),
            vx: 0.0, vy: 0.0,
            draw_order: DrawOrder::Front,
        }
    }

    pub fn new_platform(player_x: f32, which: PlatformHeight) -> Entity {
        Entity {
            aabb: match which {
                PlatformHeight::Top => Rect::new(player_x + 0.5, 0.4, 0.1, 0.05),
                PlatformHeight::Middle => Rect::new(player_x + 0.4, 0.6, 0.1, 0.05),
                PlatformHeight::Bottom => Rect::new(player_x + 0.3, 0.8, 0.1, 0.05),
            },
            deadly: false,
            obeys_gravity: false,
            colour: match which {
                PlatformHeight::Top => Color::RGB(255,0,0),
                PlatformHeight::Middle => Color::RGB(0, 255, 0),
                PlatformHeight::Bottom => Color::RGB(0, 0, 255),
            },
            vx: 0.0, vy: 0.0,
            draw_order: DrawOrder::Front,
        }
    }

    pub fn new_wall_segment(r: Rect) -> Entity {
        Entity {
            aabb: r,
            deadly: true,
            obeys_gravity: false,
            colour: Color::RGB(150, 150, 150),
            vx: 0.0, vy: 0.0,
            draw_order: DrawOrder::Back,
        }
    }
}