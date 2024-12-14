use rand::Rng;

pub struct SpeedFactorDistribution {
    pub min: f32,
    pub max: f32
}

impl SpeedFactorDistribution {
    pub fn next(&self) -> f32 {
        rand::thread_rng().gen::<f32>() * (self.max - self.min) + self.min
    }
}
