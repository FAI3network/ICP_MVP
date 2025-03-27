use crate::{Model};
use candid::Principal;

pub fn is_owner(model: &Model, caller: Principal) {
    if model.owners.iter().all(|id| *id != caller) {
        ic_cdk::api::trap("Unauthorized: You are not the owner of this model");
    }
}

pub fn select_random_element<T, I>(iter: I, seed: u32) -> Option<T> 
where
    I: Iterator<Item = T>,
    T: Clone,
{
    let elements: Vec<T> = iter.collect();
    if elements.is_empty() {
        None
    } else {
        Some(elements[(seed as usize) % elements.len()].clone())
    }
}

pub fn seeded_vector_shuffle<T: Clone>(mut elements: Vec<T>, seed: u32) -> Vec<T> {
    let len = elements.len();
    if len <= 1 {
        return elements; // No need to shuffle
    }

    let mut rng = seed as u64; // Use u64 to prevent overflow

    for i in (1..len).rev() {
        rng ^= rng << 13;
        rng ^= rng >> 7;
        rng ^= rng << 17;

        let j = (rng as usize) % (i + 1); // Generate pseudo-random index
        elements.swap(i, j);
    }

    elements
}

