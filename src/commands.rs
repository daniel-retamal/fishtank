use crate::entities::species::FishSpecies;

pub struct Completion {
    pub ghost: String,
    pub tab_result: Option<String>,
}

static COMMAND_NAMES: &[&str] = &["exit", "feed", "fps", "index", "mutate", "names", "spawn"];

static MUTATION_NAMES: &[&str] = &[
    "bodycolor",
    "bodyvariant",
    "colorpatch",
    "doublefish",
    "eye_left+",
    "eye_left-",
    "eye_right+",
    "eye_right-",
    "eyecolor",
    "glistencolor",
    "glistenfast",
    "glisten_slow",
    "glistenmode",
    "mitosis",
    "mouthvariant",
    "size+",
    "size-",
    "tailvariant",
];

static SPECIES_NAMES: &[&str] = &[
    "aka",
    "anchoveta",
    "betta",
    "carpin",
    "chromis",
    "deadfish",
    "goldenfish",
    "goldfish",
    "jellyfish",
    "koi",
    "kuro",
    "merluza",
    "mutantfish",
    "nishiki",
    "salmon",
    "snapper",
    "tang",
    "turbofish",
];

pub fn autocomplete(input: &str, fish_names: &[&str]) -> Option<Completion> {
    if !input.starts_with('/') {
        return None;
    }
    let body = &input[1..];

    match body.split_once(' ') {
        None => complete_command(body),
        Some((cmd, rest)) => match cmd {
            "feed" => complete_feed(rest),
            "fps" => complete_fps(rest),
            "mutate" => complete_mutate(rest, fish_names),
            "spawn" => complete_spawn(rest),
            _ => None,
        },
    }
}

pub fn tab_complete(input: &str, fish_names: &[&str]) -> Option<String> {
    autocomplete(input, fish_names).and_then(|c| c.tab_result)
}

fn complete_command(partial: &str) -> Option<Completion> {
    if partial.is_empty() {
        return None;
    }

    if COMMAND_NAMES.contains(&partial) {
        let args = command_args_placeholder(partial);
        return if args.is_empty() {
            None
        } else {
            Some(Completion {
                ghost: format!(" {}", args),
                tab_result: Some(format!("/{} ", partial)),
            })
        };
    }

    let matches: Vec<&str> = COMMAND_NAMES
        .iter()
        .copied()
        .filter(|&n| n.starts_with(partial))
        .collect();

    if matches.is_empty() {
        return None;
    }

    let first = matches[0];
    let args = command_args_placeholder(first);
    let ghost = if args.is_empty() {
        first[partial.len()..].to_string()
    } else {
        format!("{} {}", &first[partial.len()..], args)
    };

    let tab_result = if matches.len() == 1 {
        Some(format!("/{} ", first))
    } else {
        let cp = longest_common_prefix(&matches);
        if cp.len() > partial.len() {
            Some(format!("/{}", cp))
        } else {
            Some(format!("/{} ", first))
        }
    };

    Some(Completion { ghost, tab_result })
}

fn complete_feed(rest: &str) -> Option<Completion> {
    if rest.is_empty() {
        Some(Completion {
            ghost: "<amount>".to_string(),
            tab_result: None,
        })
    } else {
        None
    }
}

fn complete_fps(rest: &str) -> Option<Completion> {
    if rest.is_empty() {
        Some(Completion {
            ghost: "<n>".to_string(),
            tab_result: None,
        })
    } else {
        None
    }
}

fn complete_mutate(rest: &str, fish_names: &[&str]) -> Option<Completion> {
    if rest.is_empty() {
        return Some(Completion {
            ghost: "\"<name>\" <mutation>".to_string(),
            tab_result: None,
        });
    }

    let inner = rest.strip_prefix('"')?;

    if let Some(end_pos) = inner.find('"') {
        let name = &inner[..end_pos];
        let after = inner[end_pos + 1..].trim_start();

        if after.is_empty() {
            return Some(Completion {
                ghost: "<mutation>".to_string(),
                tab_result: Some(format!("/mutate \"{}\" ", name)),
            });
        }

        let matches: Vec<&str> = MUTATION_NAMES
            .iter()
            .copied()
            .filter(|&m| m.starts_with(after))
            .collect();
        if matches.is_empty() {
            return None;
        }
        let first = matches[0];
        let ghost = first[after.len()..].to_string();
        let tab_result = if matches.len() == 1 {
            Some(format!("/mutate \"{}\" {}", name, first))
        } else {
            let cp = longest_common_prefix(&matches);
            if cp.len() > after.len() {
                Some(format!("/mutate \"{}\" {}", name, cp))
            } else {
                Some(format!("/mutate \"{}\" {}", name, first))
            }
        };
        return Some(Completion { ghost, tab_result });
    }

    let matches: Vec<&str> = fish_names
        .iter()
        .copied()
        .filter(|&n| n.starts_with(inner))
        .collect();
    if matches.is_empty() {
        return Some(Completion {
            ghost: "\" <mutation>".to_string(),
            tab_result: None,
        });
    }
    let first = matches[0];
    let ghost = format!("{}\" <mutation>", &first[inner.len()..]);
    let tab_result = if matches.len() == 1 {
        Some(format!("/mutate \"{}\" ", first))
    } else {
        let cp = longest_common_prefix(&matches);
        if cp.len() > inner.len() {
            Some(format!("/mutate \"{}", cp))
        } else {
            Some(format!("/mutate \"{}\" ", first))
        }
    };
    Some(Completion { ghost, tab_result })
}

fn complete_spawn(rest: &str) -> Option<Completion> {
    match rest.split_once(' ') {
        None => complete_species(rest),
        Some((_, name_rest)) => {
            if name_rest.is_empty() {
                Some(Completion {
                    ghost: "\"<name>\"".to_string(),
                    tab_result: None,
                })
            } else {
                None
            }
        }
    }
}

fn complete_species(partial: &str) -> Option<Completion> {
    if partial.is_empty() {
        return Some(Completion {
            ghost: "<species> \"<name>\"".to_string(),
            tab_result: None,
        });
    }

    let matches: Vec<&str> = SPECIES_NAMES
        .iter()
        .copied()
        .filter(|&n| n.starts_with(partial))
        .collect();

    if matches.is_empty() {
        return None;
    }

    let first = matches[0];
    let ghost = format!("{} \"<name>\"", &first[partial.len()..]);

    let tab_result = if matches.len() == 1 {
        Some(format!("/spawn {} ", first))
    } else {
        let cp = longest_common_prefix(&matches);
        if cp.len() > partial.len() {
            Some(format!("/spawn {}", cp))
        } else {
            Some(format!("/spawn {} ", first))
        }
    };

    Some(Completion { ghost, tab_result })
}

fn command_args_placeholder(cmd: &str) -> &'static str {
    match cmd {
        "feed" => "<amount>",
        "fps" => "<n>",
        "mutate" => "\"<name>\" <mutation>",
        "spawn" => "<species> \"<name>\"",
        _ => "",
    }
}

fn longest_common_prefix<'a>(strings: &[&'a str]) -> &'a str {
    if strings.is_empty() {
        return "";
    }
    let first = strings[0];
    let len = strings.iter().skip(1).fold(first.len(), |acc, s| {
        first
            .chars()
            .zip(s.chars())
            .take_while(|(a, b)| a == b)
            .count()
            .min(acc)
    });
    &first[..len]
}

pub enum Action {
    Feed(usize),
    SetFps(f32),
    Spawn(FishSpecies, String),
    Mutate(String, String),
    Index { all: bool },
    ToggleNames,
    Exit,
    Unknown,
}

fn parse_mutate_args(rest: &str) -> Option<(String, String)> {
    let rest = rest.trim();
    let (name, mutation_str) = if let Some(inner) = rest.strip_prefix('\'') {
        let end = inner.find('\'')?;
        (&inner[..end], inner[end + 1..].trim())
    } else if let Some(inner) = rest.strip_prefix('"') {
        let end = inner.find('"')?;
        (&inner[..end], inner[end + 1..].trim())
    } else {
        return None;
    };
    if name.is_empty() || mutation_str.is_empty() {
        return None;
    }
    Some((name.to_string(), mutation_str.to_string()))
}

pub fn parse(input: &str) -> Action {
    let input = input.trim();
    if !input.starts_with('/') {
        return Action::Unknown;
    }
    let body = &input[1..];

    if let Some(rest) = body.strip_prefix("mutate ") {
        return match parse_mutate_args(rest) {
            Some((name, mutation)) => Action::Mutate(name, mutation),
            None => Action::Unknown,
        };
    }

    let parts: Vec<&str> = body.splitn(3, ' ').collect();
    match parts.as_slice() {
        ["feed"] => Action::Feed(0),
        ["feed", n] => n.parse().map(Action::Feed).unwrap_or(Action::Unknown),
        ["exit"] => Action::Exit,
        ["index"] => Action::Index { all: false },
        ["index", "all"] => Action::Index { all: true },
        ["names"] => Action::ToggleNames,
        ["fps", n] => n.parse().map(Action::SetFps).unwrap_or(Action::Unknown),
        ["spawn", species_str, name_raw] => {
            let name = name_raw.trim().trim_matches('"').to_string();
            if name.is_empty() {
                return Action::Unknown;
            }
            match FishSpecies::from_str(species_str) {
                Some(species) => Action::Spawn(species, name),
                None => Action::Unknown,
            }
        }
        _ => Action::Unknown,
    }
}
