#![allow(unused)]

use bdsp_ug_generator::resource_util::load_string_list;
use bdsp_ug_generator::xorshift::XorShift;
use bdsp_ug_generator::{personal_table, run_results, Filter, Pokemon, RoomType, Version};
use clap::{ArgEnum, Parser};
use lazy_static::lazy_static;
use std::fmt::Write;

#[derive(Parser)]
struct Cli {
    #[clap(arg_enum)]
    version: ArgVersion,
    #[clap(arg_enum)]
    room: ArgRoomType,
    #[clap(short, long)]
    diglett: bool,
    #[clap(short = 'f', long, default_value = "6")]
    story_flag: u8,
    #[clap(short = 's', long)]
    shiny_only: bool,
    #[clap(
        long,
        default_value = "0/0/0/0/0/0",
        help = "Input format is x/x/x/x/x/x. Values can be elided for the default of 0. Ex 31//31/31/31/31"
    )]
    min_ivs: String,
    #[clap(
        long,
        default_value = "31/31/31/31/31/31",
        help = "Input format is x/x/x/x/x/x. Values can be elided for the default of 31. Ex /0////"
    )]
    max_ivs: String,
    #[clap(long, help = "Input pokemon species number")]
    species: Option<u16>,
    #[clap(long, help = "Input is a comma separated list of nature IDs")]
    nature: Option<String>,
    #[clap(long, help = "Input is 0 or 1 for ability 1 and 2")]
    ability: Option<u8>,
    #[clap(long, help = "Input is a item ID number")]
    item: Option<u16>,
    #[clap(long, help = "Input is a move ID number")]
    egg_move: Option<u16>,
    #[clap(long, help = "Options are 0, 1, 2 for male, female, genderless")]
    gender: Option<u8>,
    advances: u32,
    s0: String,
    s1: String,
    s2: String,
    s3: String,
}

#[derive(ArgEnum, Clone)]
enum ArgVersion {
    BD = 2,
    SP,
}

impl From<ArgVersion> for Version {
    fn from(av: ArgVersion) -> Self {
        match av {
            ArgVersion::BD => Version::BD,
            ArgVersion::SP => Version::SP,
        }
    }
}

#[derive(ArgEnum, Copy, Clone)]
pub enum ArgRoomType {
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

impl From<ArgRoomType> for RoomType {
    fn from(art: ArgRoomType) -> Self {
        match art {
            ArgRoomType::SpaciousCave => RoomType::SpaciousCave,
            ArgRoomType::GrasslandCave => RoomType::GrasslandCave,
            ArgRoomType::FountainspringCave => RoomType::FountainspringCave,
            ArgRoomType::RockyCave => RoomType::RockyCave,
            ArgRoomType::VolcanicCave => RoomType::VolcanicCave,
            ArgRoomType::SwampyCave => RoomType::SwampyCave,
            ArgRoomType::DazzlingCave => RoomType::DazzlingCave,
            ArgRoomType::WhiteoutCave => RoomType::WhiteoutCave,
            ArgRoomType::IcyCave => RoomType::IcyCave,
            ArgRoomType::RiverbankCave => RoomType::RiverbankCave,
            ArgRoomType::SandsearCave => RoomType::SandsearCave,
            ArgRoomType::StillWaterCavern => RoomType::StillWaterCavern,
            ArgRoomType::SunlitCavern => RoomType::SunlitCavern,
            ArgRoomType::BigBluffCavern => RoomType::BigBluffCavern,
            ArgRoomType::StargleamCavern => RoomType::StargleamCavern,
            ArgRoomType::GlacialCavern => RoomType::GlacialCavern,
            ArgRoomType::BogsunkCavern => RoomType::BogsunkCavern,
            ArgRoomType::TyphloCavern => RoomType::TyphloCavern,
        }
    }
}

fn write_pokemon(pokemon: &Pokemon, string: &mut String) {
    let personal_info = personal_table::BDSP.get_form_entry(pokemon.species as usize, 0);
    let ability = match pokemon.ability {
        0 => personal_info.get_ability_1(),
        _ => personal_info.get_ability_2(),
    };
    writeln!(string, "Species: {}\nPID: {:08X} EC: {:08X} Shiny: {}\nIVs: {:?} Ability: {} Gender: {}\nNature: {} Item: {}{}\n", SPECIES_EN[pokemon.species as usize], pokemon.pid, pokemon.ec, pokemon.shiny, pokemon.ivs, ABILITIES_EN[ability],  GENDER_SYMBOLS[pokemon.gender as usize], NATURES_EN[pokemon.nature as usize].trim(),
             ITEMS_EN[pokemon.item as usize].trim(),
             if let Some(no) = pokemon.egg_move {
                 format!(" Egg Move: {}", MOVES_EN[no as usize].trim())
             } else {
                 "".to_string()
             }).unwrap();
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

    let min_split = cli.min_ivs.split('/');
    let max_split = cli.max_ivs.split('/');

    let mut min_ivs = [0, 0, 0, 0, 0, 0];

    for (i, val) in min_split.take(6).enumerate() {
        if !val.is_empty() {
            min_ivs[i] = val
                .parse::<u8>()
                .unwrap_or_else(|_| panic!("Failed to parse min iv {}", i));
        }
    }

    let mut max_ivs = [31, 31, 31, 31, 31, 31];

    for (i, val) in max_split.take(6).enumerate() {
        if !val.is_empty() {
            max_ivs[i] = val
                .parse::<u8>()
                .unwrap_or_else(|_| panic!("Failed to parse max iv {}", i));
        }
    }

    let nature = cli.nature.map(|s| {
        s.split(',')
            .filter_map(|i| {
                if i.is_empty() {
                    None
                } else {
                    Some(i.parse::<u8>().expect("Failed to parse nature to u8"))
                }
            })
            .collect::<Vec<u8>>()
    });

    let filter = Filter {
        shiny: cli.shiny_only,
        species: cli.species,
        min_ivs,
        max_ivs,
        ability: cli.ability,
        nature,
        item: cli.item,
        egg_move: cli.egg_move,
        gender: cli.gender,
    };

    let rng = XorShift::from_state([s0, s1, s2, s3]);

    let results = run_results(
        cli.advances,
        rng,
        cli.version.into(),
        cli.story_flag,
        cli.room.into(),
        filter,
        cli.diglett,
    );

    let mut print = String::new();

    for result in results.iter() {
        writeln!(
            print,
            "-------------------------------------------\nAdvances: {}",
            result.advance
        )
        .unwrap();
        for pokemon in result.regular_pokemon.iter() {
            write_pokemon(pokemon, &mut print);
        }

        if let Some(pokemon) = &result.rare_pokemon {
            write_pokemon(pokemon, &mut print);
        }
    }

    println!("{}", print);
}

pub const GENDER_SYMBOLS: [char; 3] = ['♂', '♀', '-'];

const SPECIES_EN_RAW: &str = include_str!("../resources/text/other/en/species_en.txt");
const ABILITIES_EN_RAW: &str = include_str!("../resources/text/other/en/abilities_en.txt");
const NATURES_EN_RAW: &str = include_str!("../resources/text/other/en/natures_en.txt");
const MOVES_EN_RAW: &str = include_str!("../resources/text/other/en/moves_en.txt");
const ITEMS_EN_RAW: &str = include_str!("../resources/text/items/items_en.txt");

lazy_static! {
    pub static ref SPECIES_EN: Vec<&'static str> = load_string_list(SPECIES_EN_RAW);
    pub static ref ABILITIES_EN: Vec<&'static str> = load_string_list(ABILITIES_EN_RAW);
    pub static ref NATURES_EN: Vec<&'static str> = load_string_list(NATURES_EN_RAW);
    pub static ref MOVES_EN: Vec<&'static str> = load_string_list(MOVES_EN_RAW);
    pub static ref ITEMS_EN: Vec<&'static str> = load_string_list(ITEMS_EN_RAW);
}
