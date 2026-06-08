pub const VALID_BATTALIONS: [&str; 16] = [
    "1ºBPM",
    "2ºBPM",
    "3ºBPM",
    "4ºBPM",
    "5ºBPM",
    "6ºBPM",
    "7ºBPM",
    "8ºBPM",
    "9ºBPM",
    "10ºBPM",
    "11ºBPM",
    "BPTRAN",
    "BOPE",
    "BPCHOQUE",
    "BPTAR",
    "CIPO/BURITIS",
];

pub fn is_valid_battalion(battalion: &str) -> bool {
    VALID_BATTALIONS.contains(&battalion)
}
