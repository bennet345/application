#[derive(Clone)]
pub struct Game {
    pub size: [i32; 3],
    pub food: Vec<Food>,
    pub snake: Snake,
    pub transform: crate::Transform,
    pub progress: f32,
}

impl Game {
    pub fn run(&mut self, iteration: i32) {
        self.forward();
        for i in (0..self.food.len()).rev() {
            if iteration - self.food[i].time > 25000 {
                self.food.remove(i);
            }
        }
        self.food.push(Food {
            position: self.unoccupied(),
            time: iteration,
        }); 
    }

    pub fn forward(&mut self) {
        let new_position = (self.snake.parts[self.snake.parts.len() - 1] + self.snake.direction) % self.size;
        if let Some(_) = self.snake.parts.iter().skip(1).find(|element| element.0 == new_position.0) { 
            self.snake.parts.drain(..self.snake.parts.len() - 2); 
        } else if let Some((i, _)) = self.food.iter().enumerate().find(|element| element.1.position == new_position.0) { 
            self.food.remove(i);
            if self.snake.parts[0].0 == new_position.0 { 
                self.snake.parts.drain(..self.snake.parts.len() - 2); 
            }
        } else {
            self.snake.parts.pop_front();
        }
        self.snake.parts.push_back(new_position);
    }

    pub fn unoccupied(&self) -> [i32; 3] {
        loop {
            let position = [
                (rand::random::<f64>() * self.size[0] as f64) as i32,
                (rand::random::<f64>() * self.size[1] as f64) as i32,
                (rand::random::<f64>() * self.size[2] as f64) as i32,
            ];
            if let Some(_) = self.food.iter().find(|element| element.position == position) { continue; }
            if self.snake.parts.contains(&Position(position)) { continue; }
            return position;
        }
    }

    pub fn extend_progress(&self, progress: f32, attached: Position, attaching: Position) -> crate::Transform {
        let delta = attaching + -attached;
        let scale = glam::Vec3::new(
            if delta[0] != 0 { progress } else { 1.0 } * self.transform.scale[0] / self.size[0] as f32,
            if delta[1] != 0 { progress } else { 1.0 } * self.transform.scale[1] / self.size[1] as f32,
            if delta[2] != 0 { progress } else { 1.0 } * self.transform.scale[2] / self.size[2] as f32,
        );
        let translation = self.transform.translation + glam::Vec3::new(
            (attached[0] as f32 + 0.5 * delta[0] as f32 * (1.0 + progress)) * self.transform.scale[0] / self.size[0] as f32,
            (attached[1] as f32 + 0.5 * delta[1] as f32 * (1.0 + progress)) * self.transform.scale[1] / self.size[1] as f32,
            (attached[2] as f32 + 0.5 * delta[2] as f32 * (1.0 + progress)) * self.transform.scale[2] / self.size[2] as f32,
        );

        crate::Transform {
            scale,            
            translation,
            ..Default::default()
        }
    }

    pub fn cubes(&self) -> Vec<crate::CubeInput> {
        let mut output = vec![];
        let amount = self.snake.parts.len();

        let color = [1.0, 0.0, 0.0];

        output.push(crate::CubeInput {
            color,
            transform: self.extend_progress(1.0 - self.progress, self.snake.parts[1], self.snake.parts[0]).array_matrix(),
        });

        for part in self.snake.parts.iter().skip(1).rev().skip(1).rev() {
            output.push(crate::CubeInput {
                color,
                transform: crate::Transform {
                    translation: self.transform.translation + glam::Vec3::new(
                        part[0] as f32 * self.transform.scale[0] / self.size[0] as f32, 
                        part[1] as f32 * self.transform.scale[1] / self.size[1] as f32, 
                        part[2] as f32 * self.transform.scale[2] / self.size[2] as f32,
                    ),
                    scale: glam::Vec3::new(
                        self.transform.scale[0] / self.size[0] as f32,
                        self.transform.scale[1] / self.size[1] as f32,
                        self.transform.scale[2] / self.size[2] as f32,
                    ),
                    ..self.transform
                }.array_matrix(),
            });
        }

        output.push(crate::CubeInput {
            color,
            transform: self.extend_progress(self.progress, self.snake.parts[amount - 2], self.snake.parts[amount - 1]).array_matrix(),
        });

        for food in &self.food {
            output.push(crate::CubeInput {
                color: [0.0, 1.0, 0.0],
                transform: crate::Transform {
                    translation: self.transform.translation + glam::Vec3::new(
                        food.position[0] as f32 * self.transform.scale[0] / self.size[0] as f32, 
                        food.position[1] as f32 * self.transform.scale[1] / self.size[1] as f32, 
                        food.position[2] as f32 * self.transform.scale[2] / self.size[2] as f32,
                    ),
                    scale: glam::Vec3::new(
                        self.transform.scale[0] / self.size[0] as f32,
                        self.transform.scale[1] / self.size[1] as f32,
                        self.transform.scale[2] / self.size[2] as f32,
                    ),
                    ..Default::default()
                }.array_matrix(),
            });
        }

        for corner in [
            [false, false, false],
            [true, false, true],
            [true, true, false],
            [false, true, true],
        ] {
            let mut position = self.transform.translation;
            let mut vector = self.transform.scale;
            for i in 0..3 {
                if corner[i] { position[i] += self.transform.scale[i]; };
                if corner[i] { vector[i] *= -1.0; };
            }
            output.push((&crate::Vector {
                position,
                vector: glam::Vec3::new(vector[0], 0.0, 0.0),
            }).into());
            output.push((&crate::Vector {
                position,
                vector: glam::Vec3::new(0.0, vector[1], 0.0),
            }).into());
            output.push((&crate::Vector {
                position,
                vector: glam::Vec3::new(0.0, 0.0, vector[2]),
            }).into());
        }

        output
    }
}

#[derive(Clone, PartialEq)]
pub struct Food {
    pub time: i32,
    pub position: [i32; 3],
}

#[derive(Clone)]
pub struct Snake {
    pub parts: std::collections::VecDeque<Position>,
    pub direction: Direction,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Position([i32; 3]);
type Direction = Position;

impl Position {
    fn vectorize(&self, transform: &crate::Transform) -> glam::Vec3 {
        transform.rotation * glam::Vec3::new(self.0[0] as f32, self.0[1] as f32, self.0[2] as f32) 
    }
}

impl std::ops::Add for Position {
    type Output = Position;

    fn add(self, other: Self) -> Self::Output {
        Self([
             self.0[0] + other.0[0],
             self.0[1] + other.0[1],
             self.0[2] + other.0[2],
        ])
    }
}

impl std::ops::Neg for Position {
    type Output = Position;

    fn neg(self) -> Self::Output {
        Self([
             -self.0[0],
             -self.0[1],
             -self.0[2],
        ])
    }
}

impl std::ops::Index<usize> for Position {
    type Output = i32;
    
    fn index(&self, position: usize) -> &Self::Output {
        &self.0[position]
    }
}

impl std::ops::Rem<[i32; 3]> for Position {
    type Output = Position;

    fn rem(self, value: [i32; 3]) -> Self::Output {
        Self([
            self.0[0].rem_euclid(value[0]),
            self.0[1].rem_euclid(value[1]),
            self.0[2].rem_euclid(value[2]),
        ])
    }
}

impl Snake {
    pub fn new() -> Self {
        Self {
            parts: std::collections::VecDeque::from_iter([
                Position([0, 0, 0]),
                Position([1, 0, 0]),
                Position([2, 0, 0]),
            ]),
            direction: Position([1, 0, 0]),
        }
    }

    pub fn set_direction(&mut self, new_direction: Direction) {
        let delta = self.parts[self.parts.len() - 1] + -self.parts[self.parts.len() - 2];
        if new_direction == -delta { return; }
        self.direction = new_direction;
    }

    pub fn vectors(&self, transform: &crate::Transform) -> [(glam::Vec3, Position); 6] {
        [
            (Position([1, 0, 0]).vectorize(transform), Position([1, 0, 0])),
            (Position([-1, 0, 0]).vectorize(transform), Position([-1, 0, 0])),
            (Position([0, 1, 0]).vectorize(transform), Position([0, 1, 0])),
            (Position([0, -1, 0]).vectorize(transform), Position([0, -1, 0])),
            (Position([0, 0, 1]).vectorize(transform), Position([0, 0, 1])),
            (Position([0, 0, -1]).vectorize(transform), Position([0, 0, -1])),
        ]
    }
}
