use crate::{Advance, Pokemon};

#[derive(Default, Debug)]
pub struct Filter {
    pub shiny: bool,
    pub species: Option<u16>,
    pub min_ivs: [u8; 6],
    pub max_ivs: [u8; 6],
    pub ability: Option<u8>,
    pub nature: Option<Vec<u8>>,
    pub item: Option<u16>,
    pub egg_move: Option<u16>,
    pub gender: Option<u8>,
}

impl Filter {
    pub fn shiny(mut self, shiny: bool) -> Filter {
        self.shiny = shiny;
        self
    }

    pub fn species(mut self, species: u16) -> Filter {
        self.species = Some(species);
        self
    }

    pub fn min_ivs(mut self, min_ivs: [u8; 6]) -> Filter {
        self.min_ivs = min_ivs;
        self
    }

    pub fn max_ivs(mut self, max_ivs: [u8; 6]) -> Filter {
        self.max_ivs = max_ivs;
        self
    }

    pub fn ability(mut self, ability: u8) -> Filter {
        self.ability = Some(ability);
        self
    }

    pub fn nature(mut self, nature: Vec<u8>) -> Filter {
        self.nature = Some(nature);
        self
    }

    pub fn item(mut self, item: u16) -> Filter {
        self.item = Some(item);
        self
    }

    pub fn egg_move(mut self, egg_move: u16) -> Filter {
        self.egg_move = Some(egg_move);
        self
    }

    pub fn gender(mut self, gender: u8) -> Filter {
        self.gender = Some(gender);
        self
    }

    pub fn passes_filter(&self, advance: &Advance) -> bool {
        for pokemon in advance.regular_pokemon.iter() {
            if self.check_pokemon(pokemon) {
                return true;
            }
        }

        if let Some(pokemon) = &advance.rare_pokemon {
            if self.check_pokemon(pokemon) {
                return true;
            }
        }

        false
    }

    fn check_pokemon(&self, pokemon: &Pokemon) -> bool {
        if let Some(species) = self.species {
            if pokemon.species != species {
                return false;
            }
        }

        if self.shiny && !pokemon.shiny {
            return false;
        }

        let mut passes_ivs = true;
        for (i, iv) in pokemon.ivs.iter().enumerate() {
            if !(self.min_ivs[i]..=self.max_ivs[i]).contains(iv) {
                passes_ivs = false;
                break;
            }
        }

        if !passes_ivs {
            return false;
        }

        if let Some(ability) = self.ability {
            if pokemon.ability != ability {
                return false;
            }
        }

        if let Some(nature) = &self.nature {
            if !nature.contains(&pokemon.nature) {
                return false;
            }
        }

        if let Some(item) = self.item {
            if pokemon.item != item {
                return false;
            }
        }

        if let Some(egg_move) = self.egg_move {
            if let Some(p_egg_move) = self.egg_move {
                if egg_move != p_egg_move {
                    return false;
                }
            }
        }

        if let Some(gender) = self.gender {
            if pokemon.gender != gender {
                return false;
            }
        }

        true
    }
}
