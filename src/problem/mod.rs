use sha2::{Sha256, Digest};
use std::sync::atomic::{AtomicBool, Ordering::Relaxed};
use std::time::{Duration, Instant};

pub trait Combinable {
    fn total_combinations(&self) -> usize;
}

#[derive(Debug, Clone, PartialEq)]
pub enum PartOfAProblemState {
    NotDistributed,
    Distributed,
    SearchedAndNotFound,
}

#[derive(Debug, Clone)]
pub struct PartOfAProblem {
    pub start: String,
    pub end: String,
    pub alphabet: String,
    pub hash: String,
    pub state: PartOfAProblemState,
}

impl PartOfAProblem {
    pub fn new_from_problem(problem: &Problem, start: String, end: String) -> Self {
        PartOfAProblem {
            start,
            end,
            alphabet: problem.alphabet.clone(),
            hash: problem.hash.clone(),
            state: PartOfAProblemState::NotDistributed,
        }
    }
}

impl Combinable for PartOfAProblem {
    fn total_combinations(&self) -> usize {
        let alphabet_size = self.alphabet.len();
        let str_to_index = |s: &str| -> usize {
            s.chars().fold(0, |acc, c| {
                acc * alphabet_size + self.alphabet.find(c).unwrap()
            })
        };
        let start_index = str_to_index(&self.start);
        let end_index = str_to_index(&self.end);
        if end_index >= start_index {
            end_index - start_index + 1
        } else {
            0
        }
    }
}


pub fn sort_vector_of_parts(parts: &mut Vec<PartOfAProblem>) {
    // sort
    let alphabet = parts[0].alphabet.clone();
    let alphabet_str = &alphabet;
    let str_to_index = |s: &str| -> usize {
        let alphabet_size = alphabet_str.len();
        s.chars().fold(0, |acc, c| {
            acc * alphabet_size + alphabet_str.find(c).unwrap()
        })
    };
    parts.sort_by_key(|p| str_to_index(&p.start));
}

// merges as not distributed
pub fn merge_parts(parts: &Vec<PartOfAProblem>) -> PartOfAProblem {    
    sort_vector_of_parts(&mut parts.clone());
    let alphabet = parts[0].alphabet.clone();
    let hash = parts[0].hash.clone();
    let start = parts.first().unwrap().start.clone();
    let end = parts.last().unwrap().end.clone();
    PartOfAProblem {
        start,
        end,
        alphabet,
        hash,
        state: PartOfAProblemState::NotDistributed,
    }
}

// vector of parts 
pub fn update_state_of_parts(parts: &mut Vec<PartOfAProblem>, updated_part: &PartOfAProblem) {
    sort_vector_of_parts(parts);

    let mut new_parts = Vec::new();
    let mut i = 0;
    let n = parts.len();
    let mut updated = false;

    while i < n {
        let part = &parts[i];
        // If no overlap, just push
        if updated_part.end < part.start || updated_part.start > part.end {
            new_parts.push(part.clone());
            i += 1;
            continue;
        }

        // There is overlap, may need to split
        // 1. Left non-overlapping part
        if updated_part.start > part.start {
            let left = PartOfAProblem {
                start: part.start.clone(),
                end: prev_str(&updated_part.start, &part.alphabet),
                alphabet: part.alphabet.clone(),
                hash: part.hash.clone(),
                state: part.state.clone(),
            };
            new_parts.push(left);
        }
        // 2. Middle (overlapping) part: use updated_part's state
        let overlap_start = std::cmp::max(part.start.clone(), updated_part.start.clone());
        let overlap_end = std::cmp::min(part.end.clone(), updated_part.end.clone());
        let middle = PartOfAProblem {
            start: overlap_start,
            end: overlap_end,
            alphabet: part.alphabet.clone(),
            hash: part.hash.clone(),
            state: updated_part.state.clone(),
        };
        new_parts.push(middle);
        updated = true;

        // 3. Right non-overlapping part
        if updated_part.end < part.end {
            let right = PartOfAProblem {
                start: next_str(&updated_part.end, &part.alphabet),
                end: part.end.clone(),
                alphabet: part.alphabet.clone(),
                hash: part.hash.clone(),
                state: part.state.clone(),
            };
            new_parts.push(right);
        }
        i += 1;
    }

    // If no overlap found, just insert the updated_part
    if !updated {
        new_parts.push(updated_part.clone());
    }

    // Merge adjacent parts with same state
    let mut merged: Vec<PartOfAProblem> = Vec::new();
    for part in new_parts.into_iter() {
        if let Some(last) = merged.last_mut() {
            if last.end == prev_str(&part.start, &part.alphabet) && last.state == part.state {
                last.end = part.end.clone();
                continue;
            }
        }
        merged.push(part);
    }
    *parts = merged;
}

// Helper: get previous string in alphabet order
fn prev_str(s: &str, alphabet: &str) -> String {
    let mut chars: Vec<char> = s.chars().collect();
    for i in (0..chars.len()).rev() {
        let pos = alphabet.find(chars[i]).unwrap();
        if pos > 0 {
            chars[i] = alphabet.chars().nth(pos - 1).unwrap();
            break;
        } else {
            chars[i] = alphabet.chars().last().unwrap();
        }
    }
    chars.iter().collect()
}

// Helper: get next string in alphabet order
fn next_str(s: &str, alphabet: &str) -> String {
    let mut chars: Vec<char> = s.chars().collect();
    let base = alphabet.len();
    for i in (0..chars.len()).rev() {
        let pos = alphabet.find(chars[i]).unwrap();
        if pos + 1 < base {
            chars[i] = alphabet.chars().nth(pos + 1).unwrap();
            break;
        } else {
            chars[i] = alphabet.chars().nth(0).unwrap();
        }
    }
    chars.iter().collect()
}

#[derive(Debug, Clone)]
pub struct Problem {
    pub alphabet: String,
    pub start: String,
    pub end: String,
    pub hash: String,
    pub current: String,
}

impl Combinable for Problem {
    fn total_combinations(&self) -> usize {
        let alphabet_size = self.alphabet.len();
        let str_to_index = |s: &str| -> usize {
            s.chars().fold(0, |acc, c| {
                acc * alphabet_size + self.alphabet.find(c).unwrap()
            })
        };
        let start_index = str_to_index(&self.start);
        let end_index = str_to_index(&self.end);
        if end_index >= start_index {
            end_index - start_index + 1
        } else {
            0
        }
    }
}

impl Problem {
    pub fn new(
        alphabet: String,
        start: String,
        end: String,
        hash: String,
    ) -> Self {
        Problem {
            alphabet,
            start: start.clone(),
            end: end.clone(),
            hash,
            current: start,
        }
    }

    pub fn brute_force(&mut self, stop_flag: &AtomicBool) -> Option<String> {
        loop {
            if stop_flag.load(Relaxed) {
                println!("Brute force stopped by stop flag.");
                return None;
            }
            if self.check_hash(&self.current) {
                return Some(self.current.clone());
            }
            if let None = self.next() {
                break;
            }
        }
        None
    }

    /// Helper: convert a string to its index in the given alphabet base
    fn str_to_index(&self, s: &str) -> usize {
        let alphabet_size = self.alphabet.len();
        s.chars().fold(0, |acc, c| {
            acc * alphabet_size + self.alphabet.find(c).unwrap()
        })
    }

    /// Helper: convert an index to a string in the given alphabet base, with minimum length
    fn index_to_str(&self, mut idx: usize, min_len: usize) -> String {
        let alphabet: Vec<char> = self.alphabet.chars().collect();
        let base = alphabet.len();
        let mut chars = Vec::new();
        while idx > 0 {
            chars.push(alphabet[idx % base]);
            idx /= base;
        }
        while chars.len() < min_len {
            chars.push(alphabet[0]);
        }
        chars.reverse();
        chars.iter().collect()
    }

    /// Divide the problem into n parts, each with roughly the same number of combinations
    pub fn divide_into_n(&self, n: usize) -> Vec<PartOfAProblem> {
        let total = self.total_combinations();
        if n == 0 || total == 0 {
            return vec![];
        }
        let num_parts = n.min(total); // never create more parts than total combinations
        let min_len = self.start.len().max(self.end.len());
        let start_idx = self.str_to_index(&self.start);
        let end_idx = self.str_to_index(&self.end);
        let mut parts = Vec::new();
        let mut prev_start = start_idx;
        let mut remaining = total;
        for i in 0..num_parts {
            let part_size = if i == num_parts - 1 {
                remaining
            } else {
                (remaining + (num_parts - i) - 1) / (num_parts - i) // ceil division for fair split
            };
            let part_end = if i == num_parts - 1 {
                end_idx
            } else {
                prev_start + part_size - 1
            };
            if part_end > end_idx {
                break;
            }
            let part = PartOfAProblem {
                start: self.index_to_str(prev_start, min_len),
                end: self.index_to_str(part_end, min_len),
                alphabet: self.alphabet.clone(),
                hash: self.hash.clone(),
                state: PartOfAProblemState::NotDistributed,
            };
            parts.push(part);
            prev_start = part_end + 1;
            if remaining < part_size { break; }
            remaining -= part_size;
            if prev_start > end_idx { break; }
        }
        parts
    }

    /// Divides the problem into n+1 parts:
    /// - First n parts: roughly (100 - percentage)% of the problem, divided equally
    /// - Last part: roughly percentage% of the problem as one piece
    pub fn divide_into_n_and_keep_percentage(&self, n: usize, percentage: f64) -> Vec<PartOfAProblem> {
        let total = self.total_combinations();
        if n == 0 || total == 0 || percentage < 0.0 || percentage > 100.0 {
            return vec![];
        }

        let min_len = self.start.len().max(self.end.len());
        let start_idx = self.str_to_index(&self.start);
        let end_idx = self.str_to_index(&self.end);

        // Calculate the split point
        let first_part_ratio = (100.0 - percentage) / 100.0;
        let first_part_total = ((total as f64) * first_part_ratio).round() as usize;
        
        if first_part_total == 0 {
            // Everything goes to the last piece
            return vec![PartOfAProblem {
                start: self.index_to_str(start_idx, min_len),
                end: self.index_to_str(end_idx, min_len),
                alphabet: self.alphabet.clone(),
                hash: self.hash.clone(),
                state: PartOfAProblemState::NotDistributed,
            }];
        }

        let mut parts = Vec::new();
        
        // Divide the first (100 - percentage)% into n parts
        let num_parts = n.min(first_part_total);
        let mut prev_start = start_idx;
        let mut remaining = first_part_total;
        
        for i in 0..num_parts {
            let part_size = if i == num_parts - 1 {
                remaining
            } else {
                (remaining + (num_parts - i) - 1) / (num_parts - i) // ceil division for fair split
            };
            let part_end = prev_start + part_size - 1;
            
            if part_end > end_idx {
                break;
            }
            
            let part = PartOfAProblem {
                start: self.index_to_str(prev_start, min_len),
                end: self.index_to_str(part_end, min_len),
                alphabet: self.alphabet.clone(),
                hash: self.hash.clone(),
                state: PartOfAProblemState::NotDistributed,
            };
            parts.push(part);
            prev_start = part_end + 1;
            if remaining < part_size { break; }
            remaining -= part_size;
            if prev_start > end_idx { break; }
        }

        // Add the last piece (the remaining percentage%)
        if prev_start <= end_idx {
            let last_part = PartOfAProblem {
                start: self.index_to_str(prev_start, min_len),
                end: self.index_to_str(end_idx, min_len),
                alphabet: self.alphabet.clone(),
                hash: self.hash.clone(),
                state: PartOfAProblemState::NotDistributed,
            };
            parts.push(last_part);
        }

        parts
    }

    pub fn next(&mut self) -> Option<String> {
        if self.current == self.end {
            return None;
        }
        let mut chars: Vec<char> = self.current.chars().collect();
        for i in (0..chars.len()).rev() {
            if let Some(pos) = self.alphabet.find(chars[i]) {
                if pos + 1 < self.alphabet.len() {
                    chars[i] = self.alphabet.chars().nth(pos + 1).unwrap();
                    self.current = chars.iter().collect();
                    return Some(self.current.clone());
                } else {
                    chars[i] = self.alphabet.chars().nth(0).unwrap();
                }
            }
        }
        // All characters wrapped, increase length by one
        chars.insert(0, self.alphabet.chars().nth(0).unwrap());
        self.current = chars.iter().collect();
        Some(self.current.clone())
    }

    pub fn check_hash(&self, candidate: &str) -> bool {
        let mut hasher = Sha256::new();
        hasher.update(candidate.as_bytes());
        let result = hasher.finalize();
        let hash_string = format!("{:x}", result);
        hash_string == self.hash
    }
}

/// Brute forces an extremely hard problem for 1 second and returns the number of combinations checked.
pub fn solve_for_one_sec() -> usize {
    let alphabet = "abcdefghijklmnopqrstuvwxyz0123456789".to_string();
    let start = "aaaaaaa".to_string(); // 7 character strings
    let end = "zzzzzzz".to_string();
    let hash = "0000000000000000000000000000000000000000000000000000000000000000".to_string();
    
    let mut problem = Problem::new(alphabet, start, end, hash);
    
    let start_time = Instant::now();
    let duration = Duration::from_secs(1);
    let mut count: usize = 0;
    
    loop {
        if start_time.elapsed() >= duration {
            break;
        }
        
        problem.check_hash(&problem.current);
        count += 1;
        
        if problem.next().is_none() {
            break;
        }
    }
    count
}

