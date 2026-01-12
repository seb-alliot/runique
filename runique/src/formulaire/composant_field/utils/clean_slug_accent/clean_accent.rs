pub fn remove_accents(s: &str) -> String {
    s.chars().map(|c| match c {
        'é' | 'è' | 'ê' | 'ë' => 'e',
        'à' | 'â' => 'a',
        'ç' => 'c',
        'î' | 'ï' => 'i',
        'ô' => 'o',
        'û' | 'ù' => 'u',
        'É' | 'È' | 'Ê' | 'Ë' => 'e',
        _ => c
    }).collect()
}