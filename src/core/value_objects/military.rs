pub const VALID_CITIES: [&str; 52] = [
    "ALTA FLORESTA D'OESTE",
    "ALTO ALEGRE DOS PARECIS",
    "ALTO PARAÍSO",
    "ALVORADA D'OESTE",
    "ARIQUEMES",
    "BURITIS",
    "CABIXI",
    "CACAULÂNDIA",
    "CACOAL",
    "CAMPO NOVO DE RONDÔNIA",
    "CANDEIAS DO JAMARI",
    "CASTANHEIRAS",
    "CEREJEIRAS",
    "CHUPINGUAIA",
    "COLORADO DO OESTE",
    "CORUMBIARA",
    "COSTA MARQUES",
    "CUJUBIM",
    "ESPIGÃO D'OESTE",
    "GOVERNADOR JORGE TEIXEIRA",
    "GUAJARÁ-MIRIM",
    "ITAPUÃ DO OESTE",
    "JARU",
    "JI-PARANÁ",
    "MACHADINHO D'OESTE",
    "MINISTRO ANDREAZZA",
    "MIRANTE DA SERRA",
    "MONTE NEGRO",
    "NOVA BRASILÂNDIA D'OESTE",
    "NOVA MAMORÉ",
    "NOVA UNIÃO",
    "NOVO HORIZONTE DO OESTE",
    "OURO PRETO DO OESTE",
    "PARECIS",
    "PIMENTA BUENO",
    "PIMENTEIRAS DO OESTE",
    "PORTO VELHO",
    "PRESIDENTE MÉDICI",
    "PRIMAVERA DE RONDÔNIA",
    "RIO CRESPO",
    "ROLIM DE MOURA",
    "SANTA LUZIA D'OESTE",
    "SÃO FELIPE D'OESTE",
    "SÃO FRANCISCO DO GUAPORÉ",
    "SÃO MIGUEL DO GUAPORÉ",
    "SERINGUEIRAS",
    "TEIXEIRÓPOLIS",
    "THEOBROMA",
    "URUPÁ",
    "VALE DO ANARI",
    "VALE DO PARAÍSO",
    "VILHENA",
];

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

pub const VALID_STATES: [&str; 1] = ["RO"];

pub const REGISTRATION_PREFIX: &str = "1000";
pub const REGISTRATION_MAX_LENGTH: usize = 9;

pub const VALID_RANKS: [&str; 14] = [
    "CEL PM",
    "TC PM",
    "MAJ PM",
    "CAP PM",
    "1º TEN PM",
    "2º TEN PM",
    "ASP OF PM",
    "CAD PM",
    "ST PM",
    "1º SGT PM",
    "2º SGT PM",
    "3º SGT PM",
    "CB PM",
    "SD PM",
];

pub fn is_valid_rank(rank: &str) -> bool {
    VALID_RANKS.contains(&rank)
}

pub fn is_valid_city_name(name: &str) -> bool {
    VALID_CITIES.contains(&name.to_uppercase().as_str())
}

pub fn is_valid_battalion(battalion: &str) -> bool {
    VALID_BATTALIONS.contains(&battalion)
}

pub fn is_valid_state(state: &str) -> bool {
    VALID_STATES.contains(&state.to_uppercase().as_str())
}

pub fn is_valid_registration(registration: &str) -> bool {
    registration.len() <= REGISTRATION_MAX_LENGTH
        && registration.starts_with(REGISTRATION_PREFIX)
        && registration.chars().all(|c| c.is_ascii_digit())
}
