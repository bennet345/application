use vulkano::{buffer::BufferContents, pipeline::graphics::vertex_input::Vertex};
use crate::polynomial;

#[derive(Copy, Clone)]
struct Pair {
    quantity: f32,
    price: f32,
}

impl Default for Pair {
    fn default() -> Self {
        Self {
            quantity: 0.0,
            price: 0.0,
        }
    }
}

#[derive(BufferContents, Vertex, Clone)]
#[repr(C)]
pub struct GraphInput {
    #[format(R32G32B32A32_SFLOAT)] pub transform: [[f32; 4]; 4],
    #[format(R32G32B32_SFLOAT)] pub supply: [f32; 3],
    #[format(R32G32B32_SFLOAT)] pub demand: [f32; 3],
    #[format(R32_SFLOAT)] pub outside: f32,
    #[format(R32_SFLOAT)] pub tax: f32,
    #[format(R32_SFLOAT)] pub reduction: f32,
    #[format(R32G32_SFLOAT)] pub natural: [f32; 2],
    #[format(R32G32_SFLOAT)] pub optimal: [f32; 2],
    #[format(R32_SFLOAT)] pub slide: f32,
}

#[derive(Debug)]
pub struct Surplus {
    pub producer: f32,
    pub consumer: f32,
    pub government: f32,
    pub outside: f32,
    pub loss: f32,
}

pub struct Graph {
    pub supply: polynomial::Polynomial,
    pub demand: polynomial::Polynomial,
    pub outside: f32,
    pub tax: f32,
    pub reduction: f32,
    pub slide: f32,
}

impl Graph {
    fn natural_equilibrium(&self) -> Pair {
        let polynomial = self.demand - self.tax - self.supply;
        for mut quantity in polynomial.solutions() {
            if quantity <= 0.0 { continue; }
            quantity *= 1.0 - self.reduction;
            return Pair {
                quantity,
                price: (self.demand - self.tax).y(quantity),
            };
        }
        Default::default()
    }

    fn optimal_equilibrium(&self) -> Pair {
        let polynomial = self.demand + self.outside - self.supply;
        for quantity in polynomial.solutions() {
            if quantity <= 0.0 { continue; }
            return Pair {
                quantity,
                price: self.supply.y(quantity),
            };
        }
        Default::default()
    }

    pub fn input(&self, transform: crate::Transform) -> GraphInput {
        let natural = self.natural_equilibrium();
        let optimal = self.optimal_equilibrium();

        GraphInput {
            transform: transform.matrix().to_cols_array_2d(),
            supply: [self.supply.a, self.supply.b, self.supply.c],
            demand: [self.demand.a, self.demand.b, self.demand.c],
            outside: self.outside,
            tax: self.tax,
            reduction: self.reduction,
            natural: [natural.quantity, natural.price],
            optimal: [optimal.quantity, optimal.price],
            slide: self.slide,
        }
    }

    pub fn surplus(&self) -> Surplus {
        let equilibrium = self.natural_equilibrium();

        let consumer = (self.demand - self.tax - equilibrium.price).integral(equilibrium.quantity);
        let producer = -(self.supply - equilibrium.price).integral(equilibrium.quantity);
        let government = self.tax * equilibrium.quantity;
        let outside = self.outside * equilibrium.quantity;

        Surplus {
            consumer, producer, government, outside, 
            loss: self.maximum_surplus() - consumer - producer - outside - government,
        } 
    }

    fn maximum_surplus(&self) -> f32 {
        let equilibrium = self.optimal_equilibrium();
        (self.demand + self.outside - self.supply).integral(equilibrium.quantity) 
    }
}

pub mod vs_supply_demand {
    vulkano_shaders::shader! {
        ty: "vertex",
        src: r"
            #version 450

            layout(location = 0) in vec3 position;
            layout(location = 1) in vec2 uv;
            layout(location = 2) in mat4 transform;
            layout(location = 6) in vec3 supply;
            layout(location = 7) in vec3 demand;
            layout(location = 8) in float outside;
            layout(location = 9) in float tax;
            layout(location = 10) in float reduction; 
            layout(location = 11) in vec2 natural; 
            layout(location = 12) in vec2 optimal; 
            layout(location = 13) in float slide; 

            layout(location = 0) out vec2 _uv;
            layout(location = 1) out vec3 _supply;
            layout(location = 2) out vec3 _demand;
            layout(location = 3) out float _outside;
            layout(location = 4) out float _tax;
            layout(location = 5) out float _reduction;
            layout(location = 6) out vec2 _natural;
            layout(location = 7) out vec2 _optimal;
            layout(location = 8) out float _slide;

            layout(set = 0, binding = 0) uniform Data {
                mat4 world;
                mat4 view;
                mat4 proj;
            } uniforms;

            void main() {
                _uv = uv;
                _supply = supply;
                _demand = demand;
                _outside = outside;
                _tax = tax;
                _reduction = reduction;
                _natural = natural;
                _optimal = optimal;
                _slide = slide;
                mat4 worldview = uniforms.view * uniforms.world;
                gl_Position = uniforms.proj * worldview * transform * vec4(position, 1.0);
            }
        ",
    }
}

pub mod fs_supply_demand {
    vulkano_shaders::shader! {
        ty: "fragment",
        src: r"
            #version 450

            layout(location = 0) in vec2 _uv;
            layout(location = 1) in vec3 _supply;
            layout(location = 2) in vec3 _demand;
            layout(location = 3) in float _outside;
            layout(location = 4) in float _tax;
            layout(location = 5) in float _reduction;
            layout(location = 6) in vec2 _natural;
            layout(location = 7) in vec2 _optimal;
            layout(location = 8) in float _slide;

            layout(location = 0) out vec4 f_color;

            void main() {
                vec2 uv = vec2(_uv[0] * 1.5, _uv[1] * 2.25 - 0.2 - _slide);


                float demand = _demand[0] * pow(uv[0], 2.0) + _demand[1] * uv[0] + _demand[2] - _tax;
                float demand_optimal = _demand[0] * pow(uv[0], 2.0) + _demand[1] * uv[0] + _demand[2] + _outside;
                float supply = _supply[0] * pow(uv[0], 2.0) + _supply[1] * uv[0] + _supply[2];

                float surplus = 
                    (_demand[0] - _supply[0]) / 3.0 * pow(uv[0], 3.0) +
                    (_demand[1] - _supply[1]) / 2.0 * pow(uv[0], 2.0) +
                    (_demand[2] - _supply[2] + _outside) * uv[0];

                if (abs(supply - uv[1]) < 0.01 || abs(demand_optimal - uv[1]) < 0.01) {
                    f_color = vec4(0.0, 0.0, 0.0, 1.0);
                } else if (abs(demand - uv[1]) < 0.01) {
                    f_color = vec4(1.0, 0.5, 0.0, 1.0);
                } else if (_reduction != 0.0 && abs(_natural[0] - uv[0]) < 0.01 && uv[1] > supply) {
                    f_color = vec4(1.0, 0.5, 0.0, 1.0);
                } else if (uv[0] < _natural[0] && uv[1] > _natural[1] && uv[1] < demand) {
                    f_color = vec4(0.0, 0.75, 1.0, 1.0);
                } else if (uv[0] < _natural[0] && uv[1] < _natural[1] && uv[1] > supply) {
                    f_color = vec4(0.9, 0.95, 0.0, 1.0);
                } else if (uv[0] < _natural[0] && uv[1] > demand && uv[1] < demand_optimal) {
                    if (uv[1] - demand < _outside) {
                        f_color = vec4(0.1, 0.6, 0.1, 1.0);
                    } else if (uv[1] - demand < _outside + _tax) {
                        f_color = vec4(0.05, 0.3, 0.05, 1.0);
                    } 
                } else if (
                    uv[0] > _natural[0] && uv[0] < _optimal[0] && uv[1] > supply && uv[1] < demand_optimal ||
                    uv[0] < _natural[0] && uv[0] > _optimal[0] && uv[1] < supply && uv[1] > demand_optimal
                ) {
                    f_color = vec4(0.9, 0.2, 0.25, 1.0);
                } else if (abs(uv[1] + _slide) < 0.01) {
                    f_color = vec4(0.0, 0.0, 0.0, 1.0);
                } else if (abs(surplus - (uv[1] + _slide)) < 0.01) {
                    f_color = vec4(1.0, 1.0, 1.0, 1.0);
                } else {
                    discard;
                }
            }
        ",
    }
}
