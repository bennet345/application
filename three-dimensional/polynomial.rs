#[derive(Copy, Clone)]
pub struct Polynomial {
    pub a: f32, 
    pub b: f32, 
    pub c: f32,
}

impl Polynomial {
    pub fn new(a: f32, b: f32, c: f32) -> Self {
        Self { a, b, c, }
    }

    pub fn y(&self, x: f32) -> f32 {
        self.a * x.powi(2) + self.b * x + self.c
    }

    pub fn solutions(&self) -> Vec<f32> {
        let root = self.b.powi(2) - 4.0 * self.a * self.c;

        match root {
            x if x > 0.0 => vec![
                (-self.b + root.sqrt()) / 2.0 / self.a, 
                (-self.b - root.sqrt()) / 2.0 / self.a,
            ],
            0.0 => vec![-self.b / 2.0 / self.a],
            _ => vec![],
        }
    }

    pub fn integral(&self, x: f32) -> f32 {
        self.a / 3.0 * x.powi(3) +
        self.b / 2.0 * x.powi(2) +
        self.c       * x
    } 
}

impl std::ops::Add for Polynomial {
    type Output = Self;
    fn add(self, other: Self) -> Self::Output {
        Self {
            a: self.a + other.a,
            b: self.b + other.b,
            c: self.c + other.c,
        }
    }
}

impl std::ops::Add<f32> for Polynomial {
    type Output = Self;
    fn add(self, other: f32) -> Self::Output {
        Self {
            a: self.a,
            b: self.b,
            c: self.c + other,
        }
    }
}

impl std::ops::Sub<f32> for Polynomial {
    type Output = Self;
    fn sub(self, other: f32) -> Self::Output {
        Self {
            a: self.a,
            b: self.b,
            c: self.c - other,
        }
    }
}

impl std::ops::Sub for Polynomial {
    type Output = Self;
    fn sub(self, other: Self) -> Self::Output {
        Self {
            a: self.a - other.a,
            b: self.b - other.b,
            c: self.c - other.c,
        }
    }
}

impl std::ops::Mul<f32> for Polynomial {
    type Output = Self;
    fn mul(self, other: f32) -> Self::Output {
        Self {
            a: self.a * other,
            b: self.b * other,
            c: self.c * other,
        }
    }
}

impl Default for Polynomial {
    fn default() -> Self {
        Polynomial {
            a: 0.0, b: 0.0, c: 0.0,
        }
    }
}
