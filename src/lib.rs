mod filter;
mod flag_util;
pub mod personal_info;
pub mod personal_info_bdsp;
pub mod personal_table;
mod resource_util;
mod run_results;
pub mod xorshift;

pub use filter::*;
pub use run_results::*;

use lazy_static::lazy_static;
use resource_util::load_string_list;
use serde::Deserialize;

const SPECIES_EN_RAW: &str = include_str!("../resources/text/other/en/species_en.txt");
const ABILITIES_EN_RAW: &str = include_str!("../resources/text/other/en/abilities_en.txt");
const NATURES_EN_RAW: &str = include_str!("../resources/text/other/en/natures_en.txt");
const MOVES_EN_RAW: &str = include_str!("../resources/text/other/en/moves_en.txt");
const ITEMS_EN_RAW: &str = include_str!("../resources/text/items/items_en.txt");
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
pub const GENDER_SYMBOLS: [char; 3] = ['♂', '♀', '-'];

lazy_static! {
    pub static ref SPECIES_EN: Vec<&'static str> = load_string_list(SPECIES_EN_RAW);
    pub static ref ABILITIES_EN: Vec<&'static str> = load_string_list(ABILITIES_EN_RAW);
    pub static ref NATURES_EN: Vec<&'static str> = load_string_list(NATURES_EN_RAW);
    pub static ref MOVES_EN: Vec<&'static str> = load_string_list(MOVES_EN_RAW);
    pub static ref ITEMS_EN: Vec<&'static str> = load_string_list(ITEMS_EN_RAW);
}

#[derive(Deserialize, Clone)]
struct UgSpecialPokemon {
    #[serde(rename = "Sheet1")]
    sheet_sheet_1: Vec<Sheet1>,
}

#[derive(Deserialize, Copy, Clone)]
struct Sheet1 {
    id: u8,
    monsno: u16,
    version: u8,
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
    move_type: u8,
    #[serde(rename = "reactioncode")]
    reaction_code: Vec<u8>,
    move_rate: Vec<u8>,
    submove_rate: Vec<u8>,
    reaction: Vec<u8>,
    #[serde(rename = "flagrate")]
    flag_rate: Vec<u8>,
    #[serde(rename = "rateup")]
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
    size: u8,
    min: u8,
    max: u8,
    smax: u8,
    mmax: u8,
    lmax: u8,
    llmax: u8,
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
    #[serde(rename = "formNo")]
    form_no: u16,
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

#[derive(PartialEq, Copy, Clone)]
pub enum Version {
    BD = 2,
    SP,
}

#[derive(PartialEq, Copy, Clone)]
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
