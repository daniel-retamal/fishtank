use std::collections::HashSet;

pub fn title_case(s: &str) -> String {
    s.split_whitespace()
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(c) => {
                    let upper: String = c.to_uppercase().collect();
                    upper + chars.as_str()
                }
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn unique_name_in(used: &HashSet<String>, requested: &str) -> String {
    if !used.contains(requested) {
        return requested.to_string();
    }
    let (root, n) = parse_name_numeral(requested);
    let mut i = n + 1;
    loop {
        let candidate = format!("{} {}", root, to_roman(i));
        if !used.contains(&candidate) {
            return candidate;
        }
        i += 1;
    }
}

fn parse_name_numeral(name: &str) -> (&str, u32) {
    if let Some(pos) = name.rfind(' ') {
        let suffix = &name[pos + 1..];
        if let Some(n) = roman_to_u32(suffix) {
            return (&name[..pos], n);
        }
    }
    (name, 1)
}

pub fn to_roman(mut n: u32) -> String {
    const VALS: &[(u32, &str)] = &[
        (1000, "M"),
        (900, "CM"),
        (500, "D"),
        (400, "CD"),
        (100, "C"),
        (90, "XC"),
        (50, "L"),
        (40, "XL"),
        (10, "X"),
        (9, "IX"),
        (5, "V"),
        (4, "IV"),
        (1, "I"),
    ];
    let mut s = String::new();
    for &(val, sym) in VALS {
        while n >= val {
            s.push_str(sym);
            n -= val;
        }
    }
    s
}

fn roman_to_u32(s: &str) -> Option<u32> {
    if s.is_empty() {
        return None;
    }
    let digit = |c: char| match c {
        'I' => Some(1u32),
        'V' => Some(5),
        'X' => Some(10),
        'L' => Some(50),
        'C' => Some(100),
        'D' => Some(500),
        'M' => Some(1000),
        _ => None,
    };
    let vals: Option<Vec<u32>> = s.chars().map(digit).collect();
    let vals = vals?;
    let mut total = 0u32;
    let mut prev = 0u32;
    for &v in vals.iter().rev() {
        if v < prev {
            total = total.saturating_sub(v);
        } else {
            total = total.saturating_add(v);
        }
        prev = v;
    }
    if total == 0 { None } else { Some(total) }
}
