#![allow(unused)]
mod flag_util;
mod personal_info;
mod personal_info_bdsp;
mod personal_table;
mod resource_util;

use crate::personal_info::PersonalInfo;
use lazy_static::lazy_static;
use resource_util::load_string_list;
use serde::Deserialize;
use std::fs::File;
use std::io::Read;
use clap::{Parser, ArgEnum};

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

#[derive(Copy, Clone)]
pub struct XorShift {
    state: [u32; 4],
}

impl XorShift {
    pub fn from_state(state: [u32; 4]) -> Self {
        Self { state }
    }

    pub fn get_state(&self) -> [u32; 4] {
        self.state
    }

    pub fn next(&mut self) -> u32 {
        let s0 = self.state[0];
        self.state[0] = self.state[1];
        self.state[1] = self.state[2];
        self.state[2] = self.state[3];

        let tmp = s0 ^ s0 << 11;
        let tmp = tmp ^ tmp >> 8 ^ self.state[2] ^ self.state[2] >> 19;

        self.state[3] = tmp;

        (tmp % 0xffffffff).wrapping_add(0x80000000)
    }

    pub fn advance(&mut self, advances: usize) {
        for _ in 0..advances {
            self.next();
        }
    }

    pub fn advance_to_state(&mut self, state: [u32; 4]) -> Option<usize> {
        let mut advances = 0;

        // 10,000 is an arbitary limit to avoid an infinite loop
        while self.get_state() != state {
            self.next();
            advances += 1;

            if advances > 10_000 {
                return None;
            }
        }

        Some(advances)
    }

    fn get_mask(num: u32) -> u32 {
        let mut result = num - 1;

        for i in 0..5 {
            let shift = 1 << i;
            result |= result >> shift;
        }

        result
    }

    pub fn rand_max(&mut self, max: u32) -> u32 {
        let mask = Self::get_mask(max);
        let mut rand = self.next() & mask;

        while max <= rand {
            rand = self.next() & mask;
        }

        rand
    }

    pub fn rand_range(&mut self, min: u32, max: u32) -> u32 {
        let s0 = self.state[0];
        self.state[0] = self.state[1];
        self.state[1] = self.state[2];
        self.state[2] = self.state[3];

        let tmp = s0 ^ s0 << 11;
        let tmp = tmp ^ tmp >> 8 ^ self.state[2] ^ self.state[2] >> 19;

        self.state[3] = tmp;

        let diff = max - min;

        (tmp % diff).wrapping_add(min)
    }

    pub fn rand_range_float(&mut self, min: f32, max: f32) -> f32 {
        let t = ((self.next() & 0x7FFFFF) as f32 * f32::from_be_bytes([0x34, 00, 00, 00]));
        t * min + (1.0 - t) * max
    }
}

struct BDSPGenerator {
    curr_rng: XorShift,
}

impl BDSPGenerator {
    pub fn new(rng: XorShift) -> Self {
        Self { curr_rng: rng }
    }

    pub fn is_shiny(&self) -> bool {
        let mut clone = self.curr_rng.clone();
        let pid = clone.next();
        let shiny_rand = clone.next();

        (pid & 0xFFF0 ^ pid >> 0x10 ^ shiny_rand >> 0x10 ^ shiny_rand & 0xFFF0) < 0x10
    }

    pub fn find_shiny(&mut self) -> usize {
        let mut advances = 0;
        while !self.is_shiny() {
            self.curr_rng.next();
            advances += 1;
        }
        advances
    }
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
    sheet_1: Vec<TamagoWazaIgnoreEntry>
}

#[derive(Deserialize)]
struct TamagoWazaIgnoreEntry {
    #[serde(rename = "MonsNo")]
    monsno: u16,
    #[serde(rename = "Waza")]
    waza: Vec<u16>
}

#[derive(Parser)]
struct Cli {
    #[clap(arg_enum)]
    version: Version,
    #[clap(arg_enum)]
    room: RoomType,
    #[clap(short = 'f', long, default_value = "6")]
    story_flag: u8,
    #[clap(short = 's', long)]
    shiny_only: bool,
    advances: u32,
    s0: String,
    s1: String,
    s2: String,
    s3: String
}

#[derive(ArgEnum, PartialEq, Copy, Clone)]
enum Version {
    BD = 2,
    SP
}

#[derive(ArgEnum, PartialEq, Copy, Clone)]
enum RoomType {
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
    TyphloCavern
}

fn main() {
    let cli: Cli = Cli::parse();

    println!("Advances: {}", cli.advances);
    let s0 = cli.s0.trim_start_matches("0x");
    let s0 = u32::from_str_radix(s0, 16).expect("Failed to parse s0 to u32");
    let s1 = cli.s1.trim_start_matches("0x");
    let s1 = u32::from_str_radix(s1, 16).expect("Failed to parse s1 to u32");
    let s2 = cli.s2.trim_start_matches("0x");
    let s2 = u32::from_str_radix(s2, 16).expect("Failed to parse s2 to u32");
    let s3 = cli.s3.trim_start_matches("0x");
    let s3 = u32::from_str_radix(s3, 16).expect("Failed to parse s3 to u32");
    println!("s0: {:#08X}", s0);
    println!("s1: {:#08X}", s1);
    println!("s2: {:#08X}", s2);
    println!("s3: {:#08X}", s3);
    println!();

    let special_pokemon = serde_json::from_str::<UgSpecialPokemon>(UG_SPECIAL_POKEMON).unwrap();

    let special_pokemon = special_pokemon
        .sheet_sheet_1
        .into_iter()
        .filter_map(|s| if s.id == cli.room as u8 { Some(s) } else { None })
        .collect::<Vec<Sheet1>>();

    let mut special_pokemon_rates = special_pokemon
        .iter()
        .map(|s| PokeRate {
            monsno: s.monsno,
            rate: { if cli.version == Version::BD {s.d_special_rate} else { s.p_special_rate } },
        })
        .collect::<Vec<PokeRate>>();
    special_pokemon_rates.sort_by(|pr, pr2| pr2.rate.cmp(&pr.rate));
    let special_rates_sum = special_pokemon_rates
        .iter()
        .map(|pr| pr.rate as f32)
        .sum::<f32>();

    let ug_pokemon_data = serde_json::from_str::<UgPokemonData>(UG_POKEMON_DATA).unwrap();

    let mut ug_rand_mark = serde_json::from_str::<UgRandMarkSheet>(UG_RAND_MARK).unwrap();

    let ug_encount_str = match ug_rand_mark.table.iter().find(|t| t.id == cli.room as u8).unwrap().file_name.trim_start_matches("UgEncount_") {
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
        _ => UG_ENCOUNT_20
    };
    let mut ug_encount = serde_json::from_str::<UgEncountSheet>(ug_encount_str).unwrap();

    let opposite_version = match cli.version {
        Version::BD => Version::SP,
        Version::SP => Version::BD
    };

    let enabled_pokemon = ug_encount
        .table
        .into_iter()
        .filter(|e| e.version != opposite_version as u8 && e.zukan_flag <= cli.story_flag)
        .collect::<Vec<UgEncount>>();

    let mut mons_data_indexs = Vec::new();
    for encount in enabled_pokemon.iter() {
        let pokemon_data = ug_pokemon_data
            .table
            .iter()
            .find(|p| p.monsno == encount.monsno)
            .unwrap();
        mons_data_indexs.push(TypeAndSize {
            r#type: pokemon_data.type_1_id,
            size: pokemon_data.size,
            value: {
                let pow = f32::powi(10.0, pokemon_data.size as i32);
                pow as u16 + pokemon_data.type_1_id as u16
            },
        });
        if pokemon_data.type_2_id != -1 {
            mons_data_indexs.push(TypeAndSize {
                r#type: pokemon_data.type_2_id,
                size: pokemon_data.size,
                value: {
                    let pow = f32::powi(10.0, pokemon_data.size as i32);
                    pow as u16 + pokemon_data.type_2_id as u16
                },
            });
        }
    }

    let mut type_rates = ug_rand_mark.table.iter().find(|t| t.id == cli.room as u8).unwrap()
        .typerate
        .iter()
        .enumerate()
        .filter_map(|(i, rate)| {
            if mons_data_indexs.iter().any(|ts| ts.r#type == i as i8) {
                Some(TypeRate {
                    r#type: i as i8,
                    rate: *rate,
                })
            } else {
                None
            }
        })
        .collect::<Vec<TypeRate>>();

    type_rates.sort_by(|ts, ts2| ts2.rate.cmp(&ts.rate));

    let type_rates_sum = type_rates.iter().map(|tr| tr.rate).sum::<u16>();

    let rand_mark_data = ug_rand_mark.table.iter().find(|t| t.id == cli.room as u8).unwrap().clone();
    let mut smax = rand_mark_data.smax;
    let mut mmax = rand_mark_data.mmax;
    let mut lmax = rand_mark_data.lmax;
    let mut llmax = rand_mark_data.llmax;

    let mut sizes =
        Vec::with_capacity(smax as usize + mmax as usize + llmax as usize + lmax as usize);

    while smax > 0 {
        sizes.push(0);
        smax -= 1;
    }

    while mmax > 0 {
        sizes.push(1);
        mmax -= 1;
    }

    while lmax > 0 {
        sizes.push(2);
        lmax -= 1;
    }

    while llmax > 0 {
        sizes.push(3);
        llmax -= 1;
    }

    let mut egg_move_table = serde_json::from_str::<TamagoWazaTable>(TAMAGO_WAZA_TABLE).unwrap();
    let mut egg_move_ignore_table = serde_json::from_str::<TamagoWazaIgnoreTable>(TAMAGO_WAZA_IGNORE_TABLE).unwrap();

    let mut rng = XorShift::from_state([s0, s1, s2, s3]);
    let secret_base_used_tiles = 0;
    for advances in 0..=cli.advances {
        let mut contains_shiny = false;
        let mut spawn_count = rand_mark_data.min;
        let mut log = format!("------------------------------\nAdvances: {}\n", advances);
        let mut clone = rng.clone();
        let rare_check = clone.rand_range(0, 100);
        let mut rare_mons_no = 0;
        if rare_check < 50 {
            let mut rare_es_rand: f32 = clone.rand_range_float(0.0, special_rates_sum as f32);
            for special_pokemon_rate in special_pokemon_rates.iter() {
                if rare_es_rand < special_pokemon_rate.rate as f32 {
                    rare_mons_no = special_pokemon_rate.monsno;
                    break;
                }
                rare_es_rand -= special_pokemon_rate.rate as f32;
            }
        }

        let min_max_rand = clone.rand_range(0, 100);
        if 50u32.saturating_sub(secret_base_used_tiles) <= min_max_rand {
            spawn_count = rand_mark_data.max;
        }

        if rare_check < 50 {
            spawn_count -= 1;
        }

        let mut poke_slots: Vec<TypeAndSize> = Vec::with_capacity(spawn_count as usize);

        for _ in 0..spawn_count {
            let mut r#type = 0;
            let mut type_rand = clone.rand_range_float(0.0, type_rates_sum as f32);
            for type_rate in type_rates.iter() {
                if type_rand < type_rate.rate as f32 {
                    r#type = type_rate.r#type;
                    break;
                }
                type_rand -= type_rate.rate as f32;
            }

            let pokemon_with_type = mons_data_indexs
                .iter()
                .filter(|ts| ts.r#type == r#type)
                .map(|ts| *ts)
                .collect::<Vec<TypeAndSize>>();
            let mut exist_size_list = Vec::new();
            for ts in pokemon_with_type.iter() {
                if !exist_size_list.contains(&ts.size) {
                    exist_size_list.push(ts.size);
                }
            }

            if sizes.iter().all(|s| exist_size_list.contains(s)) {
                sizes = sizes
                    .into_iter()
                    .filter(|s| !exist_size_list.contains(s))
                    .collect();
            } else {
                sizes = sizes
                    .into_iter()
                    .filter(|s| exist_size_list.contains(s))
                    .collect();
            }

            let size = if sizes.len() != 0 {
                let size_rand = clone.rand_range(0, sizes.len() as u32);
                let size = sizes[size_rand as usize];
                sizes.remove(size_rand as usize);
                size
            } else {
                let size_rand = clone.rand_range(0, exist_size_list.len() as u32);
                exist_size_list[size_rand as usize]
            };

            poke_slots.push(TypeAndSize {
                r#type,
                size,
                value: {
                    let pow = f32::powi(10.0, size as i32);
                    pow as u16 + r#type as u16
                },
            });
        }

        for poke_slot in poke_slots.iter() {
            let temp_list = mons_data_indexs
                .iter()
                .filter_map(|p| {
                    if p.value == poke_slot.value {
                        Some(*p)
                    } else {
                        None
                    }
                })
                .collect::<Vec<TypeAndSize>>();

            let mut filtered_list = Vec::new();

            for pokemon in enabled_pokemon.iter() {
                let pokemon_data = ug_pokemon_data
                    .table
                    .iter()
                    .find(|p| p.monsno == pokemon.monsno)
                    .unwrap();
                if temp_list.iter().any(|ts| {
                    (ts.r#type == pokemon_data.type_1_id || ts.r#type == pokemon_data.type_2_id)
                        && pokemon_data.size == ts.size
                }) {
                    filtered_list.push(*pokemon);
                }
            }

            let mut poke_rates: Vec<PokeRate> = Vec::new();

            for filtered in filtered_list {
                let pokemon_data = ug_pokemon_data
                    .table
                    .iter()
                    .find(|p| p.monsno == filtered.monsno)
                    .unwrap();
                poke_rates.push(PokeRate {
                    monsno: pokemon_data.monsno,
                    rate: pokemon_data.flag_rate[cli.story_flag as usize - 1] as u16,
                });
            }

            poke_rates.sort_by(|pr, pr2| pr2.rate.cmp(&pr.rate));

            let poke_rates_sum = poke_rates.iter().map(|pr| pr.rate).sum::<u16>();

            let mut species = 0;
            let mut slot_rand = clone.rand_range_float(0.0, poke_rates_sum as f32);
            for poke_rate in poke_rates.iter() {
                if slot_rand < poke_rate.rate as f32 {
                    species = poke_rate.monsno;
                    break;
                }
                slot_rand -= poke_rate.rate as f32
            }
            log = format!("{}Species: {}\n", log, SPECIES_EN[species as usize]);

            let personal_info = personal_table::BDSP.get_form_entry(species as usize, 0);

            let gender_ratio = personal_info.get_gender();

            clone.next(); //level

            let ec = clone.next(); //EC
            let curr_shiny_rand = clone.next(); //Shiny Rand
            let curr_pid = clone.next(); //PID Called twice if diglett is on!

            let is_shiny = (curr_shiny_rand & 0xFFF0
                ^ curr_shiny_rand >> 0x10
                ^ curr_pid >> 0x10
                ^ curr_pid & 0xFFF0)
                < 0x10;
            if is_shiny {
                contains_shiny = true;
            }

            log = format!("{}PID: {curr_pid:08X} EC: {ec:08X} Shiny: {}\n", log, is_shiny);

            let mut ivs = [0; 6];

            ivs[0] = clone.next() % 32; //IV 1
            ivs[1] = clone.next() % 32; //IV 2
            ivs[2] = clone.next() % 32; //IV 3
            ivs[3] = clone.next() % 32; //IV 4
            ivs[4] = clone.next() % 32; //IV 5
            ivs[5] = clone.next() % 32; //IV 6
            let ability_rand = clone.next() % 2;
            let ability = match ability_rand { //ability
                0 => personal_info.get_ability_1(),
                1 => personal_info.get_ability_2(),
                _ => 0,
            };
            let gender = if gender_ratio != 255 && gender_ratio != 254 && gender_ratio != 0 {
                let gender_rand = clone.next() % 253;
                ((gender_rand as usize) + 1 < gender_ratio) as usize
            } else {
                gender_ratio % 253
            };

            log = format!("{}IVs: {:?} Ability: {} Gender: {}\n", log, ivs, ABILITIES_EN[ability], GENDER_SYMBOLS[gender as usize]);

            let nature = clone.next() % 25; //nature
            clone.next(); //height 1
            clone.next(); //height 2
            clone.next(); //weight 1
            clone.next(); //weight 2

            let item_rand = clone.rand_range(0, 100); //item
            let item = if item_rand < 60 {
                personal_info.get_item_1()
            } else if item_rand < 80 {
                personal_info.get_item_2()
            } else {
                personal_info.get_item_3()
            };

            let hatch_species = personal_info
                .get_hatch_species();

            let mut egg_move_no = None;

            if let Some(entry) = egg_move_table
                .data
                .iter()
                .find(|e| e.no == hatch_species as u16)
            {
                let mut egg_move_table = entry.waza_no.clone();
                if let Some(ignore_entry) = egg_move_ignore_table.sheet_1.iter().find(|e| e.monsno == entry.no) {
                    let egg_move_ignore_table = ignore_entry.waza.clone();
                    egg_move_table = egg_move_table.into_iter().filter(|i| !egg_move_ignore_table.contains(i) || *i == 0).collect::<Vec<u16>>(); // i == 0 check just in case
                }
                if !egg_move_table.is_empty() {
                    let egg_move_rand = clone.rand_range(0, egg_move_table.len() as u32) as usize;
                    egg_move_no = Some(egg_move_table[egg_move_rand]);
                }
            }

            log = format!("{}Nature: {} Item: {}{}\n\n", log, NATURES_EN[nature as usize].trim(), ITEMS_EN[item].trim(), if let Some(no) = egg_move_no { format!(" Egg Move: {}", MOVES_EN[no as usize].trim()) } else {"".to_string()});

        }

        if rare_check < 50 {
            log = format!("{}Rare Species: {}\n", log, SPECIES_EN[rare_mons_no as usize].trim());
            let personal_info = personal_table::BDSP.get_form_entry(rare_mons_no as usize, 0);

            let gender_ratio = personal_info.get_gender();

            clone.next(); //level

            let ec = clone.next(); //EC
            let curr_shiny_rand = clone.next(); //Shiny Rand
            let curr_pid = clone.next(); //PID Called twice if diglett is on!

            let is_shiny = (curr_shiny_rand & 0xFFF0
                ^ curr_shiny_rand >> 0x10
                ^ curr_pid >> 0x10
                ^ curr_pid & 0xFFF0)
                < 0x10;
            if is_shiny {
                contains_shiny = true;
            }

            log = format!("{}PID: {curr_pid:08X} EC: {ec:08X} Shiny: {}\n", log, is_shiny);

            let mut ivs = [0; 6];

            ivs[0] = clone.next() % 32; //IV 1
            ivs[1] = clone.next() % 32; //IV 2
            ivs[2] = clone.next() % 32; //IV 3
            ivs[3] = clone.next() % 32; //IV 4
            ivs[4] = clone.next() % 32; //IV 5
            ivs[5] = clone.next() % 32; //IV 6
            let ability_rand = clone.next() % 2;
            let ability = match ability_rand { //ability
                0 => personal_info.get_ability_1(),
                1 => personal_info.get_ability_2(),
                _ => 0,
            };
            let gender = if gender_ratio != 255 && gender_ratio != 254 && gender_ratio != 0 {
                let gender_rand = clone.next() % 253;
                ((gender_rand as usize) + 1 < gender_ratio) as usize
            } else {
                gender_ratio % 253
            };

            log = format!("{}IVs: {:?} Ability: {} Gender: {}\n", log, ivs, ABILITIES_EN[ability].trim(), GENDER_SYMBOLS[gender as usize]);

            let nature = clone.next() % 25; //nature
            clone.next(); //height 1
            clone.next(); //height 2
            clone.next(); //weight 1
            clone.next(); //weight 2
            let item_rand = clone.rand_range(0, 100); //item
            let item = if item_rand < 60 {
                personal_info.get_item_1()
            } else if item_rand < 80 {
                personal_info.get_item_2()
            } else {
                personal_info.get_item_3()
            };
            let hatch_species = personal_info
                .get_hatch_species();

            let mut egg_move_no = None;

            if let Some(entry) = egg_move_table
                .data
                .iter()
                .find(|e| e.no == hatch_species as u16)
            {
                let mut egg_move_table = entry.waza_no.clone();
                if let Some(ignore_entry) = egg_move_ignore_table.sheet_1.iter().find(|e| e.monsno == entry.no) {
                    let egg_move_ignore_table = ignore_entry.waza.clone();
                    egg_move_table = egg_move_table.into_iter().filter(|i| !egg_move_ignore_table.contains(i) || *i == 0).collect::<Vec<u16>>(); // i == 0 check just in case
                }
                if !egg_move_table.is_empty() {
                    let egg_move_rand = clone.rand_range(0, egg_move_table.len() as u32) as usize;
                    egg_move_no = Some(egg_move_table[egg_move_rand]);
                }
            }

            log = format!("{}Nature: {} Item: {}{}\n\n", log, NATURES_EN[nature as usize].trim(), ITEMS_EN[item].trim(), if let Some(no) = egg_move_no { format!(" Egg Move: {}", MOVES_EN[no as usize].trim()) } else {"".to_string()});
        }

        if contains_shiny || !cli.shiny_only {
            print!("{}", log);
        }
        rng.next();
    }
}
