use serde::{Deserialize, Serialize};

const RAW_STATUE_DATA: &str = include_str!("../StatueEffectRawData.json");

#[derive(Serialize, Deserialize, Default)]
pub struct StatueConfig {
    pub statues: Vec<Statue>,
}

#[derive(Serialize, Deserialize, Copy, Clone)]
pub struct Statue {
    #[serde(rename = "statueId")]
    pub statue_id: usize,
    #[serde(rename = "monsId")]
    pub mons_id: usize,
    pub rarity: usize,
    pub width: u32,
    pub height: u32,
    #[serde(rename = "type1Id")]
    pub type_1_id: i8,
    #[serde(rename = "type2Id")]
    pub type_2_id: i8,
    #[serde(rename = "pokeTypeEffect")]
    pub poke_type_effect: [u16; 2],
}

#[derive(Serialize, Deserialize)]
pub struct StatueEffectRawData {
    table: Vec<Statue>,
}

pub fn get_statue_data() -> Vec<Statue> {
    let data = serde_json::from_str::<StatueEffectRawData>(RAW_STATUE_DATA)
        .expect("Failed to parse raw data");
    data.table
}

impl StatueConfig {
    pub fn get_spawn_count_bonus(&self) -> u32 {
        let mut tiles_used = 0;
        self.statues
            .iter()
            .for_each(|s| tiles_used += s.width * s.height);
        if tiles_used < 1 {
            0
        } else if tiles_used < 0x10 {
            5
        } else if tiles_used < 0x2e {
            0xf
        } else if tiles_used <= 0x3c {
            0x14
        } else {
            0x1e
        }
    }

    pub fn get_bonus_rates(&self) -> [u16; 18] {
        let mut bonuses = [0; 18];

        self.statues.iter().for_each(|s| {
            if s.type_1_id != -1 {
                bonuses[s.type_1_id as usize] += s.poke_type_effect[0];
            }
            if s.type_2_id != -1 {
                bonuses[s.type_2_id as usize] += s.poke_type_effect[1];
            }
        });

        bonuses
    }

    pub fn add_statue(&mut self, statue: Statue) {
        if self.statues.len() < 18 {
            self.statues.push(statue);
        }
    }

    pub fn remove_statue(&mut self, index: usize) {
        self.statues.remove(index);
    }
}
