use rand::Rng;
use rand::SeedableRng;
use rand::rngs::StdRng;

const WORD_LIST: &str = include_str!("../../../languages/english1000.txt");

const CHUNK_SIZE: u32 = 25;

fn words() -> Vec<&'static str> {
    WORD_LIST.lines().map(|s| s.trim()).filter(|s| !s.is_empty()).collect()
}

pub fn generate_next_chunk(
    seed: u64,
    value: u32,
    words_so_far: u32,
) -> Option<String> {
    let word_list = words();
    if word_list.is_empty() {
        return Some("the quick brown fox".to_string());
    }
    let mut rng = StdRng::seed_from_u64(seed);
    let (take, need_advance) = (CHUNK_SIZE, words_so_far);
    if take == 0 {
        return None;
    }
    for _ in 0..need_advance {
        let _ = rng.gen_range(0..word_list.len());
    }
    let indices: Vec<usize> = (0..take).map(|_| rng.gen_range(0..word_list.len())).collect();
    let chunk = indices
        .into_iter()
        .map(|i| word_list[i])
        .collect::<Vec<&str>>()
        .join(" ");
    Some(chunk)
}
