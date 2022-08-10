mod filter;
mod flag_util;
pub mod personal_info;
pub mod personal_info_bdsp;
pub mod personal_table;
pub mod resource_util;
mod run_results;
pub mod xorshift;

pub use filter::*;
pub use run_results::*;
use serde::Deserialize;
use std::collections::HashSet;

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

pub fn get_available_egg_moves(species: u16) -> Vec<u16> {
    let egg_move_table = serde_json::from_str::<TamagoWazaTable>(TAMAGO_WAZA_TABLE).unwrap();
    let egg_move_ignore_table =
        serde_json::from_str::<TamagoWazaIgnoreTable>(TAMAGO_WAZA_IGNORE_TABLE).unwrap();
    let hatch_species = personal_table::BDSP
        .get_form_entry(species as usize, 0)
        .get_hatch_species();

    if let Some(entry) = egg_move_table
        .data
        .iter()
        .find(|e| e.no == hatch_species as u16)
    {
        let mut egg_move_table = entry.waza_no.clone();
        if let Some(ignore_entry) = egg_move_ignore_table
            .sheet_1
            .iter()
            .find(|e| e.monsno == entry.no)
        {
            let egg_move_ignore_table = ignore_entry.waza.clone();
            egg_move_table = egg_move_table
                .into_iter()
                .filter(|i| !egg_move_ignore_table.contains(i) || *i == 0)
                .collect::<Vec<u16>>(); // i == 0 check just in case
        }
        egg_move_table.sort();
        egg_move_table
    } else {
        vec![]
    }
}

pub fn available_pokemon(version: Version, story_flag: u8, room: RoomType) -> Vec<u16> {
    let mut available = HashSet::new();

    let opposite_version = match version {
        Version::BD => Version::SP,
        Version::SP => Version::BD,
    };

    let special_pokemon = serde_json::from_str::<UgSpecialPokemon>(UG_SPECIAL_POKEMON).unwrap();

    let special_pokemon = special_pokemon
        .sheet_sheet_1
        .into_iter()
        .filter(|s| s.id == room as u8)
        .collect::<Vec<Sheet1>>();

    for pokemon in special_pokemon {
        match version {
            Version::BD => {
                if pokemon.d_special_rate > 0 {
                    available.insert(pokemon.monsno);
                }
            }
            Version::SP => {
                if pokemon.p_special_rate > 0 {
                    available.insert(pokemon.monsno);
                }
            }
        }
    }

    let ug_rand_mark = serde_json::from_str::<UgRandMarkSheet>(UG_RAND_MARK).unwrap();

    let ug_encount_str = match ug_rand_mark
        .table
        .iter()
        .find(|t| t.id == room as u8)
        .unwrap()
        .file_name
        .trim_start_matches("UgEncount_")
    {
        "02" => UG_ENCOUNT_02,
        "03" => UG_ENCOUNT_03,
        "04" => UG_ENCOUNT_04,
        "05" => UG_ENCOUNT_05,
        "06" => UG_ENCOUNT_06,
        "07" => UG_ENCOUNT_07,
        "08" => UG_ENCOUNT_08,
        "09" => UG_ENCOUNT_09,
        "10" => UG_ENCOUNT_10,
        "11" => UG_ENCOUNT_11,
        "12" => UG_ENCOUNT_12,
        _ => UG_ENCOUNT_20,
    };

    let ug_encount = serde_json::from_str::<UgEncountSheet>(ug_encount_str).unwrap();
    for pokemon in ug_encount.table {
        if pokemon.version != opposite_version as u8 && pokemon.zukan_flag <= story_flag {
            available.insert(pokemon.monsno);
        }
    }

    let mut available = available.into_iter().collect::<Vec<u16>>();

    available.sort();

    available
}
