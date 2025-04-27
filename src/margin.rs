use embedded_graphics::{
    prelude::{Point, Size},
    primitives::Rectangle,
};

pub struct Margin {
    top: u32,
    right: u32,
    bottom: u32,
    left: u32,
}

impl Margin {
    /// Create a new margin
    pub fn new(top: u32, right: u32, bottom: u32, left: u32) -> Self {
        Self {
            top,
            right,
            bottom,
            left,
        }
    }

    /// Create a new margin with the same value for all sides
    pub fn all(value: u32) -> Self {
        Self {
            top: value,
            right: value,
            bottom: value,
            left: value,
        }
    }

    /// Create a new margin with the same value for the horizontal and vertical sides
    pub fn symmetric(vertical: u32, horizontal: u32) -> Self {
        Self {
            top: vertical,
            right: horizontal,
            bottom: vertical,
            left: horizontal,
        }
    }

    /// Create a new margin with the same value for the top and bottom sides
    pub fn vertical(value: u32) -> Self {
        Self {
            top: value,
            right: 0,
            bottom: value,
            left: 0,
        }
    }

    /// Create a new margin with the same value for the left and right sides
    pub fn horizontal(value: u32) -> Self {
        Self {
            top: 0,
            right: value,
            bottom: 0,
            left: value,
        }
    }
}

pub trait MarginExt {
    fn margin(self, margin: Margin) -> Self;
}

impl MarginExt for Rectangle {
    fn margin(self, margin: Margin) -> Self {
        let top_left = Point::new(
            self.top_left.x - margin.left as i32,
            self.top_left.y - margin.top as i32,
        );
        let size = Size::new(
            self.size.width + margin.left + margin.right,
            self.size.height + margin.top + margin.bottom,
        );

        Rectangle::new(top_left, size)
    }
}
