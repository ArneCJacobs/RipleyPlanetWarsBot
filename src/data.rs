use serde::{Deserialize, Serialize};
use std::fmt;

pub type PlanetName = String;
pub type ExpeditionId = u64;
pub type PlayerId = u8;
pub type PlanetId = usize;
pub const ME_ID: PlayerId = 1;

#[derive(Deserialize, Clone, Default)]
pub struct Input {
    pub planets: Vec<Planet>,
    pub expeditions: Vec<Expedition>,
}

impl fmt::Debug for Input {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut planets = self.planets.clone();
        planets.sort_by_key(|p| p.name.clone());
        let mut expeditions = self.expeditions.clone();
        expeditions.sort_by_key(|e| e.id);

        f.write_str("{\n")?;
        f.write_str("\t\"planets\": [\n")?;
        for planet in planets {
            f.write_fmt(format_args!("\t\t{:?},\n", planet))?;
        }
        f.write_str("\t],\n")?;
        f.write_str("\t\"expeditions\": [\n")?;
        for expedition in expeditions {
            f.write_fmt(format_args!("\t\t{:?},\n", expedition))?;
        }
        f.write_str("\t]\n}")?;
        Ok(())
    }
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
        ((self.x - other.x).powi(2) + (self.y - other.y).powi(2)).sqrt()
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct Expedition {
    pub id: ExpeditionId,
    pub ship_count: i64,
    pub origin: PlanetName,
    pub destination: PlanetName,
    pub owner: PlayerId,
    pub turns_remaining: i64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Output {
    pub moves: Vec<Move>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Move {
    pub origin: PlanetName,
    pub destination: PlanetName,
    pub ship_count: i64,
}

impl Move {
    pub fn new(origin: PlanetName, destination: PlanetName, ship_count: i64) -> Self {
        Move {
            origin,
            destination,
            ship_count,
        }
    }
}
