use crate::Model;
use regex::Regex;
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

/// Shuffless a vector in a cross-platform safe way
pub fn seeded_vector_shuffle<T: Clone>(mut elements: Vec<T>, seed: u32) -> Vec<T> {
    let len = elements.len();
    if len <= 1 {
        return elements;
    }
    
    // Use a simple but consistent PRNG algorithm with fixed-width types
    let mut state = seed as u32;
    
    for i in (1..len).rev() {
        // Update state using a simple but consistent algorithm
        state = state.wrapping_mul(1664525).wrapping_add(1013904223); // Linear congruential generator
        
        // Convert to range [0, i] in a consistent way
        let j = (state as u32) % (i as u32 + 1);
        elements.swap(i, j as usize);
    }
    
    elements
}

pub fn clean_llm_response(text: &String) -> String {
    let re = Regex::new(r"(?s)<think>.*?</think>").unwrap();
    re.replace_all(text, "")
        .replace("\n", " ")
        .replace("\r", " ")
        .trim()
        .to_string()
}
