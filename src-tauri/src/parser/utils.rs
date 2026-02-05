use rand::Rng;

pub const DEFAULT_USER_AGENT: &str = "Mozilla/5.0 (iPhone; CPU iPhone OS 16_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/16.0 Mobile/15E148 Safari/604.1";

pub fn rand_seq(n: usize) -> String {
    let letters = "0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";
    let mut rng = rand::thread_rng();
    (0..n)
        .map(|_| {
            let idx = rng.gen_range(0..letters.len());
            letters.chars().nth(idx).unwrap()
        })
        .collect()
}


pub fn generate_fixed_length_numeric_id(length: usize) -> String {
    let mut rng = rand::thread_rng();
    let mut max = 1u64;
    for _ in 0..length {
        max *= 10;
    }
    let random_num = rng.gen_range(0..max);
    format!("{:0width$}", random_num, width = length)
}

pub fn regexp_match_url_from_string(share_msg: &str) -> Option<String> {
    let re = regex::Regex::new(r"http[s]?://[a-zA-Z0-9\.\-_/\?=&%]+").unwrap();
    re.find(share_msg).map(|m| m.as_str().to_string())
}
