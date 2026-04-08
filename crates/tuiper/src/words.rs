use rand::Rng;
use rand::SeedableRng;
use rand::rngs::StdRng;

const WORD_LIST: &str = include_str!("../../../languages/english1000.txt");

const CHUNK_SIZE: u32 = 25;

fn words() -> Vec<&'static str> {
    WORD_LIST.lines().map(|s| s.trim()).filter(|s| !s.is_empty()).collect()
}

pub fn generate_words_text(
    seed: u64,
    value: u32,
    total_words: u32,
) -> Option<String> {
    let word_list = words();

    if word_list.is_empty() {
        return Some("the quick brown fox".to_string());
    }

    let mut rng = StdRng::seed_from_u64(seed);

    let text = (0..total_words)
        .map(|_| {
            let i = rng.gen_range(0..word_list.len());
            word_list[i]
        })
        .collect::<Vec<_>>()
        .join(" ");

    Some(text)
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

    for _ in 0..words_so_far {
        rng.gen_range(0..word_list.len());
    }

    let chunk = (0..CHUNK_SIZE)
        .map(|_| {
            let i = rng.gen_range(0..word_list.len());
            word_list[i]
        })
        .collect::<Vec<_>>()
        .join(" ");

    Some(chunk)
}
