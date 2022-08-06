mod filter;
mod flag_util;
pub mod personal_info;
pub mod personal_info_bdsp;
pub mod personal_table;
pub mod resource_util;
mod run_results;
pub mod xorshift;

use std::collections::HashSet;
pub use filter::*;
pub use run_results::*;
use serde::Deserialize;

const TAMAGO_WAZA_TABLE: &str = include_str!("../TamagoWazaTable.json");
const TAMAGO_WAZA_IGNORE_TABLE: &str = include_str!("../UgTamagoWazaIgnoreTable.json");
const UG_POKEMON_DATA: &str = include_str!("../UgPokemonData.json");
const UG_RAND_MARK: &str = include_str!("../UgRandMark.json");
const UG_SPECIAL_POKEMON: &str = include_str!("../UgSpecialPokemon.json");
const UG_ENCOUNT_02: &str = include_str!("../UgEncount_02.json");
const UG_ENCOUNT_03: &str = include_str!("../UgEncount_03.json");
const UG_ENCOUNT_04: &str = include_str!("../UgEncount_04.json");
const UG_ENCOUNT_05: &str = include_str!("../UgEncount_05.json");
const UG_ENCOUNT_06: &str = include_str!("../UgEncount_06.json");
const UG_ENCOUNT_07: &str = include_str!("../UgEncount_07.json");
const UG_ENCOUNT_08: &str = include_str!("../UgEncount_08.json");
const UG_ENCOUNT_09: &str = include_str!("../UgEncount_09.json");
const UG_ENCOUNT_10: &str = include_str!("../UgEncount_10.json");
const UG_ENCOUNT_11: &str = include_str!("../UgEncount_11.json");
const UG_ENCOUNT_12: &str = include_str!("../UgEncount_12.json");
const UG_ENCOUNT_20: &str = include_str!("../UgEncount_20.json");

#[derive(Deserialize, Clone)]
struct UgSpecialPokemon {
    #[serde(rename = "Sheet1")]
    sheet_sheet_1: Vec<Sheet1>,
}

#[derive(Deserialize, Copy, Clone)]
struct Sheet1 {
    id: u8,
    monsno: u16,
    #[serde(rename = "Dspecialrate")]
    d_special_rate: u16,
    #[serde(rename = "Pspecialrate")]
    p_special_rate: u16,
}

#[derive(Deserialize)]
struct UgEncountSheet {
    table: Vec<UgEncount>,
}

#[derive(Deserialize, Copy, Clone, Debug)]
struct UgEncount {
    monsno: u16,
    version: u8,
    #[serde(rename = "zukanflag")]
    zukan_flag: u8,
}

#[derive(Deserialize)]
struct UgPokemonData {
    table: Vec<UgPokemon>,
}

#[derive(Deserialize)]
struct UgPokemon {
    monsno: u16,
    #[serde(rename = "type1ID")]
    type_1_id: i8,
    #[serde(rename = "type2ID")]
    type_2_id: i8,
    size: u8,
    #[serde(rename = "movetype")]
    #[allow(dead_code)]
    move_type: u8,
    #[serde(rename = "flagrate")]
    flag_rate: Vec<u8>,
    #[serde(rename = "rateup")]
    #[allow(dead_code)]
    rate_up: u8,
}

#[derive(Deserialize, Clone)]
struct UgRandMarkSheet {
    table: Vec<UgRandMark>,
}

#[derive(Deserialize, Clone)]
struct UgRandMark {
    id: u8,
    #[serde(rename = "FileName")]
    file_name: String,
    min: u8,
    max: u8,
    smax: u8,
    mmax: u8,
    lmax: u8,
    llmax: u8,
    #[allow(dead_code)]
    watermax: u8,
    typerate: Vec<u16>,
}

#[derive(Copy, Clone, Debug)]
struct PokeRate {
    monsno: u16,
    rate: u16,
}

#[derive(Copy, Clone, Debug)]
struct TypeAndSize {
    r#type: i8,
    size: u8,
    value: u16,
}

#[derive(Debug)]
struct TypeRate {
    r#type: i8,
    rate: u16,
}

#[derive(Deserialize)]
struct TamagoWazaTable {
    #[serde(rename = "Data")]
    data: Vec<TamagoWazaEntry>,
}

#[derive(Deserialize)]
struct TamagoWazaEntry {
    no: u16,
    #[serde(rename = "wazaNo")]
    waza_no: Vec<u16>,
}

#[derive(Deserialize)]
struct TamagoWazaIgnoreTable {
    #[serde(rename = "Sheet1")]
    sheet_1: Vec<TamagoWazaIgnoreEntry>,
}

#[derive(Deserialize)]
struct TamagoWazaIgnoreEntry {
    #[serde(rename = "MonsNo")]
    monsno: u16,
    #[serde(rename = "Waza")]
    waza: Vec<u16>,
}

#[derive(PartialEq, Eq, Copy, Clone)]
pub enum Version {
    BD = 2,
    SP,
}

#[derive(PartialEq, Eq, Copy, Clone)]
pub enum RoomType {
    SpaciousCave = 2,
    GrasslandCave,
    FountainspringCave,
    RockyCave,
    VolcanicCave,
    SwampyCave,
    DazzlingCave,
    WhiteoutCave,
    IcyCave,
    RiverbankCave,
    SandsearCave,
    StillWaterCavern,
    SunlitCavern,
    BigBluffCavern,
    StargleamCavern,
    GlacialCavern,
    BogsunkCavern,
    TyphloCavern,
}

pub fn available_pokemon(version: Version, story_flag: u8, room: Option<RoomType>) -> Vec<u16> {
    let mut available = HashSet::new();

    let opposite_version = match version {
        Version::BD => Version::SP,
        Version::SP => Version::BD,
    };

    if let Some(room) = room {
        let ug_encount_str = match room as u8 {
            2 => UG_ENCOUNT_02,
            3 => UG_ENCOUNT_03,
            4 => UG_ENCOUNT_04,
            5 => UG_ENCOUNT_05,
            6 => UG_ENCOUNT_06,
            7 => UG_ENCOUNT_07,
            8 => UG_ENCOUNT_08,
            9 => UG_ENCOUNT_09,
            10 => UG_ENCOUNT_10,
            11 => UG_ENCOUNT_11,
            _ => UG_ENCOUNT_12
        };
        let ug_encount = serde_json::from_str::<UgEncountSheet>(ug_encount_str).unwrap();
        for pokemon in ug_encount.table {
            if pokemon.version != opposite_version as u8 && pokemon.zukan_flag <= story_flag {
                available.insert(pokemon.monsno);
            }
        }

    } else {
        for i in 2..13 {
            let ug_encount_str = match i {
                2 => UG_ENCOUNT_02,
                3 => UG_ENCOUNT_03,
                4 => UG_ENCOUNT_04,
                5 => UG_ENCOUNT_05,
                6 => UG_ENCOUNT_06,
                7 => UG_ENCOUNT_07,
                8 => UG_ENCOUNT_08,
                9 => UG_ENCOUNT_09,
                10 => UG_ENCOUNT_10,
                11 => UG_ENCOUNT_11,
                _ => UG_ENCOUNT_12
            };

            let ug_encount = serde_json::from_str::<UgEncountSheet>(ug_encount_str).unwrap();
            for pokemon in ug_encount.table {
                if pokemon.version != opposite_version as u8 && pokemon.zukan_flag <= story_flag {
                    available.insert(pokemon.monsno);
                }
            }
        }
    }



    available.into_iter().collect()
}
