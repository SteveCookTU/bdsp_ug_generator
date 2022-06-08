use crate::{
    personal_table, PersonalInfo, PokeRate, RoomType, Sheet1, TamagoWazaIgnoreTable,
    TamagoWazaTable, TypeAndSize, TypeRate, UgEncount, UgEncountSheet, UgPokemonData,
    UgRandMarkSheet, UgSpecialPokemon, Version, XorShift, ABILITIES_EN, GENDER_SYMBOLS, ITEMS_EN,
    MOVES_EN, NATURES_EN, SPECIES_EN, TAMAGO_WAZA_IGNORE_TABLE, TAMAGO_WAZA_TABLE, UG_ENCOUNT_02,
    UG_ENCOUNT_03, UG_ENCOUNT_04, UG_ENCOUNT_05, UG_ENCOUNT_06, UG_ENCOUNT_07, UG_ENCOUNT_08,
    UG_ENCOUNT_09, UG_ENCOUNT_10, UG_ENCOUNT_11, UG_ENCOUNT_12, UG_ENCOUNT_20, UG_POKEMON_DATA,
    UG_RAND_MARK, UG_SPECIAL_POKEMON,
};

pub struct Advance {
    pub regular_pokemon: Vec<Pokemon>,
    pub rare_pokemon: Option<Pokemon>,
}

pub struct Pokemon {
    pub species: u16,
    pub ec: u32,
    pub pid: u32,
    pub shiny: bool,
    pub ivs: [u8; 6],
    pub ability: u16,
    pub gender: u8,
    pub nature: u8,
    pub item: u16,
    pub egg_move: Option<u16>,
}

pub fn run_results(
    advances: u32,
    s0: u32,
    s1: u32,
    s2: u32,
    s3: u32,
    version: Version,
    story_flag: u8,
    room: RoomType,
    shiny_only: bool,
) -> Vec<Advance> {
    let mut results = Vec::with_capacity(advances as usize);

    let special_pokemon = serde_json::from_str::<UgSpecialPokemon>(UG_SPECIAL_POKEMON).unwrap();

    let special_pokemon = special_pokemon
        .sheet_sheet_1
        .into_iter()
        .filter_map(|s| if s.id == room as u8 { Some(s) } else { None })
        .collect::<Vec<Sheet1>>();

    let mut special_pokemon_rates = special_pokemon
        .iter()
        .map(|s| PokeRate {
            monsno: s.monsno,
            rate: {
                if version == Version::BD {
                    s.d_special_rate
                } else {
                    s.p_special_rate
                }
            },
        })
        .collect::<Vec<PokeRate>>();
    special_pokemon_rates.sort_by(|pr, pr2| pr2.rate.cmp(&pr.rate));
    let special_rates_sum = special_pokemon_rates
        .iter()
        .map(|pr| pr.rate as f32)
        .sum::<f32>();

    let ug_pokemon_data = serde_json::from_str::<UgPokemonData>(UG_POKEMON_DATA).unwrap();

    let mut ug_rand_mark = serde_json::from_str::<UgRandMarkSheet>(UG_RAND_MARK).unwrap();

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
    let mut ug_encount = serde_json::from_str::<UgEncountSheet>(ug_encount_str).unwrap();

    let opposite_version = match version {
        Version::BD => Version::SP,
        Version::SP => Version::BD,
    };

    let enabled_pokemon = ug_encount
        .table
        .into_iter()
        .filter(|e| e.version != opposite_version as u8 && e.zukan_flag <= story_flag)
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

    let mut type_rates = ug_rand_mark
        .table
        .iter()
        .find(|t| t.id == room as u8)
        .unwrap()
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

    let rand_mark_data = ug_rand_mark
        .table
        .iter()
        .find(|t| t.id == room as u8)
        .unwrap()
        .clone();
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
    let mut egg_move_ignore_table =
        serde_json::from_str::<TamagoWazaIgnoreTable>(TAMAGO_WAZA_IGNORE_TABLE).unwrap();

    let mut rng = XorShift::from_state([s0, s1, s2, s3]);
    let secret_base_used_tiles = 0;
    for _ in 0..=advances {
        let mut contains_shiny = false;
        let mut spawn_count = rand_mark_data.min;
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

        let mut advance = Advance {
            regular_pokemon: Vec::with_capacity(spawn_count as usize),
            rare_pokemon: None,
        };

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
                    rate: pokemon_data.flag_rate[story_flag as usize - 1] as u16,
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

            let mut ivs = [0; 6];

            ivs[0] = (clone.next() % 32) as u8; //IV 1
            ivs[1] = (clone.next() % 32) as u8; //IV 2
            ivs[2] = (clone.next() % 32) as u8; //IV 3
            ivs[3] = (clone.next() % 32) as u8; //IV 4
            ivs[4] = (clone.next() % 32) as u8; //IV 5
            ivs[5] = (clone.next() % 32) as u8; //IV 6
            let ability_rand = clone.next() % 2;
            let ability = match ability_rand {
                //ability
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

            let hatch_species = personal_info.get_hatch_species();

            let mut egg_move_no = None;

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
                if !egg_move_table.is_empty() {
                    let egg_move_rand = clone.rand_range(0, egg_move_table.len() as u32) as usize;
                    egg_move_no = Some(egg_move_table[egg_move_rand]);
                }
            }

            advance.regular_pokemon.push(Pokemon {
                species,
                ec,
                pid: curr_pid,
                shiny: is_shiny,
                ivs,
                ability: ability as u16,
                gender: gender as u8,
                nature: nature as u8,
                item: item as u16,
                egg_move: egg_move_no,
            });
        }

        if rare_check < 50 {
            writeln!(
                log,
                "Rare Species: {}",
                SPECIES_EN[rare_mons_no as usize].trim()
            )
            .unwrap();
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

            let mut ivs = [0; 6];

            ivs[0] = (clone.next() % 32) as u8; //IV 1
            ivs[1] = (clone.next() % 32) as u8; //IV 2
            ivs[2] = (clone.next() % 32) as u8; //IV 3
            ivs[3] = (clone.next() % 32) as u8; //IV 4
            ivs[4] = (clone.next() % 32) as u8; //IV 5
            ivs[5] = (clone.next() % 32) as u8; //IV 6
            let ability_rand = clone.next() % 2;
            let ability = match ability_rand {
                //ability
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
            let hatch_species = personal_info.get_hatch_species();

            let mut egg_move_no = None;

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
                if !egg_move_table.is_empty() {
                    let egg_move_rand = clone.rand_range(0, egg_move_table.len() as u32) as usize;
                    egg_move_no = Some(egg_move_table[egg_move_rand]);
                }
            }

            advance.rare_pokemon = Some(Pokemon {
                species,
                ec,
                pid: curr_pid,
                shiny: is_shiny,
                ivs,
                ability: ability as u16,
                gender: gender as u8,
                nature: nature as u8,
                item: item as u16,
                egg_move: egg_move_no,
            });
        }

        if contains_shiny || !shiny_only {
            results.push(advance);
        }
        rng.next();
    }
    results
}
