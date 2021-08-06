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



