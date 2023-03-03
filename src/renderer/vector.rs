use std::ops::*;

use serde::{Serialize, Deserialize};

pub trait VectorCast<To> {
    fn cast(value: Self) -> Vector2<To>;
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Default, Clone, Copy, Serialize, Deserialize)]
pub struct Vector2<T>(pub T, pub T);

impl<T: Copy> Vector2<T> {
    pub fn new(x: T, y: T) -> Self {
        Self(x, y)
    } 

    #[inline]
    pub fn x(&self) -> T {
        self.0
    }

    #[inline]
    pub fn y(&self) -> T {
        self.1
    }
}

impl VectorCast<i32> for Vector2<f64> {
    fn cast(value: Self) -> Vector2<i32> {
        Vector2(value.0 as i32, value.1 as i32)
    }
}

impl<T: Copy> From<T> for Vector2<T> {
    fn from(value: T) -> Self {
        Self(value, value)
    }
}

impl<T> From<(T, T)> for Vector2<T> {
    fn from(value: (T, T)) -> Self {
        Self(value.0, value.1)
    }
}

impl<T: Add<Output = T>> Add for Vector2<T> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0, self.1 + rhs.1)
    }
}

impl<T: Sub<Output = T>> Sub for Vector2<T> {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0, self.1 - rhs.1)
    }
}

impl<T: Mul<Output = T>> Mul for Vector2<T> {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self(self.0 * rhs.0, self.1 * rhs.1)
    }
}

impl<T: Div<Output = T>> Div for Vector2<T> {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        Self(self.0 / rhs.0, self.1 / rhs.1)
    }
}
