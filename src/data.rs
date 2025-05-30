use serde::{Deserialize, Serialize};

pub type PlanetName = String;
pub type ExpeditionId = u64;
pub type PlayerId = u8;
pub type PlanetId = usize;
pub const ME_ID: PlayerId = 1;

#[derive(Deserialize, Debug, Clone, Default)]
pub struct Input {
    pub planets: Vec<Planet>,
    pub expeditions: Vec<Expedition>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Planet {
    pub ship_count: i64,
    pub x: f32,
    pub y: f32,
    pub owner: Option<PlayerId>, 
    pub name: PlanetName,
    #[serde(default = "default_index")]
    pub index: usize,
}

fn default_index() -> usize {
    99999
}


impl Planet {
    pub fn distance(&self, other: &Planet) -> f32 {
        (
            (self.x - other.x).powi(2) +
            (self.y - other.y).powi(2)
        ).sqrt()
    }
}


#[derive(Deserialize, Debug, Clone)]
pub struct PlanetLocation {
    pub x: f32,
    pub y: f32,
}

impl PlanetLocation {
    pub fn distance(&self, other: &PlanetLocation) -> f32 {
        (
            (self.x - other.x).powi(2) +
            (self.y - other.y).powi(2)
        ).sqrt()
    }
}

impl From<&Planet> for PlanetLocation {
    fn from(planet: &Planet) -> Self {
        let Planet{ x, y, ..} = *planet;
        PlanetLocation { x, y } 
    }
}

impl From<&mut Planet> for PlanetLocation {
    fn from(planet: &mut Planet) -> Self {
        let Planet{ x, y, ..} = *planet;
        PlanetLocation { x, y } 
    }
}



#[derive(Deserialize, Debug, Clone)]
pub struct Expedition {
    pub id: ExpeditionId,
    pub ship_count: i64,
    pub origin: PlanetName,
    pub destination: PlanetName,
    pub owner: PlayerId,
    pub turns_remaining: i64
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Output {
    pub  moves: Vec<Move>,
}


#[derive(Serialize, Deserialize, Debug)]
pub struct Move {
    pub origin: PlanetName,
    pub destination: PlanetName,
    pub ship_count: i64,
}

impl Move {
    pub fn new(
        origin: PlanetName,
        destination: PlanetName,
        ship_count: i64,
    ) -> Self {
        Move {
            origin,
            destination,
            ship_count,
        }
    }
}
