use crate::personal_info::{form_index, PersonalInfo};
use crate::personal_info_bdsp;
use lazy_static::lazy_static;
use personal_info_bdsp::PersonalInfoBDSP;
use std::ops::{Index, IndexMut};

const PERSONAL_BDSP: &[u8] = include_bytes!("../resources/byte/personal/personal_bdsp");

lazy_static! {
    pub static ref BDSP: PersonalTable<PersonalInfoBDSP> =
        PersonalTable::new(PERSONAL_BDSP.to_vec());
}

pub struct PersonalTable<T: PersonalInfo> {
    table: Vec<T>,
}

impl<T: PersonalInfo> Index<usize> for PersonalTable<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        &self.table[index]
    }
}

impl<T: PersonalInfo> IndexMut<usize> for PersonalTable<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.table[index]
    }
}

impl<T: PersonalInfo> PersonalTable<T> {
    pub fn new(data: Vec<u8>) -> Self {
        let size = personal_info_bdsp::SIZE;
        let count = data.len() / size;
        let mut table = Vec::with_capacity(count);
        for i in 0..count {
            table.push(T::new(data[(size * i)..((size * i) + size)].to_vec()))
        }
        Self { table }
    }

    pub fn get_form_index(&self, species: usize, form: usize) -> usize {
        if species <= 493 {
            form_index(&self[species], species, form)
        } else {
            0
        }
    }

    pub fn get_form_entry(&self, species: usize, form: usize) -> &T {
        &self[self.get_form_index(species, form)]
    }
}
