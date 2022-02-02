//! Basic Euclidean vector types.

use std::ops;

const COS: [i32; 4] = [1, 0, -1, 0];
const SIN: [i32; 4] = [0, 1, 0, -1];

/// A 2D vector to represent a coordinate, translation, etc.
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, Default)]
#[cfg_attr(test, derive(Debug))]
pub struct Vector2D { pub x: i32, pub y: i32 }

impl ops::Add<Vector2D> for Vector2D {
    type Output = Vector2D;
    fn add(self, rhs: Vector2D) -> Vector2D {
        Vector2D { x: self.x + rhs.x, y: self.y + rhs.y }
    }
}

impl ops::Sub<Vector2D> for Vector2D {
    type Output = Vector2D;
    fn sub(self, rhs: Vector2D) -> Vector2D {
        Vector2D { x: self.x - rhs.x, y: self.y - rhs.y }
    }
}

impl Vector2D {
    /// Rotates the vector by (rotation * 90) degrees.
    /// (counter-clockwise in the standard coordinate system where the y-axis is upwards.)
    pub fn rotate(&self, rotation: i32) -> Vector2D {
        let t = ((rotation % 4 + 4) % 4) as usize;
        Vector2D { 
            x: self.x * COS[t] - self.y * SIN[t],
            y: self.x * SIN[t] + self.y * COS[t],
        }
    }
}
