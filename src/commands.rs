use crate::entities::species::FishSpecies;

pub struct Completion {
    pub ghost: String,
    pub tab_result: Option<String>,
}

static COMMAND_NAMES: &[&str] = &[
    "add",
    "consume",
    "exit",
    "feed",
    "fish",
    "fishtanks",
    "fps",
    "index",
    "inventory",
    "move",
    "mutate",
    "names",
    "shop",
    "show",
    "spawn",
    "stats",
    "subtract",
    "switch",
    "voidspawn",
];

static RESOURCE_NAMES: &[&str] = &["food", "junk", "cash"];

static MUTATION_NAMES: &[&str] = &[
    "bodycolor",
    "bodyvariant",
    "colorpatch",
    "doublefish",
    "eye+",
    "eye-",
    "eyecolor",
    "glistencolor",
    "glistendisable",
    "glistenenable",
    "glistenfast",
    "glistenslow",
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

pub fn autocomplete(
    input: &str,
    fish_names: &[&str],
    consumable_names: &[&str],
    tank_names: &[&str],
    current_tank: &str,
    fish_in_tanks: &[(&str, &str)],
) -> Option<Completion> {
    if !input.starts_with('/') {
        return None;
    }
    let body = &input[1..];

    match body.split_once(' ') {
        None => complete_command(body),
        Some((cmd, rest)) => match cmd.to_ascii_lowercase().as_str() {
            "add" => complete_add_subtract("add", rest),
            "consume" => complete_consume(rest, consumable_names),
            "feed" => complete_feed(rest),
            "fps" => complete_fps(rest),
            "index" => complete_index(rest, tank_names),
            "move" => complete_move(rest, fish_names, tank_names, fish_in_tanks),
            "mutate" => complete_mutate(rest, fish_names),
            "show" => complete_show(rest, fish_in_tanks),
            "spawn" => complete_spawn(rest),
            "subtract" => complete_add_subtract("subtract", rest),
            "switch" => complete_switch(rest, tank_names, current_tank),
            _ => None,
        },
    }
}

pub fn tab_complete(
    input: &str,
    fish_names: &[&str],
    consumable_names: &[&str],
    tank_names: &[&str],
    current_tank: &str,
    fish_in_tanks: &[(&str, &str)],
) -> Option<String> {
    autocomplete(
        input,
        fish_names,
        consumable_names,
        tank_names,
        current_tank,
        fish_in_tanks,
    )
    .and_then(|c| c.tab_result)
}

fn complete_command(partial: &str) -> Option<Completion> {
    if partial.is_empty() {
        return None;
    }
    let partial_lower = partial.to_ascii_lowercase();

    if COMMAND_NAMES.iter().any(|&n| n == partial_lower) {
        let args = command_args_placeholder(&partial_lower);
        return if args.is_empty() {
            None
        } else {
            Some(Completion {
                ghost: format!(" {}", args),
                tab_result: Some(format!("/{} ", partial_lower)),
            })
        };
    }

    let matches: Vec<&str> = COMMAND_NAMES
        .iter()
        .copied()
        .filter(|&n| n.starts_with(partial_lower.as_str()))
        .collect();

    if matches.is_empty() {
        return None;
    }

    let first = matches[0];
    let args = command_args_placeholder(first);
    let ghost = if args.is_empty() {
        first[partial_lower.len()..].to_string()
    } else {
        format!("{} {}", &first[partial_lower.len()..], args)
    };

    let tab_result = if matches.len() == 1 {
        Some(format!("/{} ", first))
    } else {
        let cp = longest_common_prefix(&matches);
        if cp.len() > partial_lower.len() {
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

fn complete_add_subtract(cmd: &str, rest: &str) -> Option<Completion> {
    match rest.split_once(' ') {
        None => {
            if rest.is_empty() {
                return Some(Completion {
                    ghost: "<resource> <amount>".to_string(),
                    tab_result: None,
                });
            }
            let rest_lower = rest.to_ascii_lowercase();
            let matches: Vec<&str> = RESOURCE_NAMES
                .iter()
                .copied()
                .filter(|&r| r.starts_with(rest_lower.as_str()))
                .collect();
            if matches.is_empty() {
                return None;
            }
            let first = matches[0];
            let ghost = format!("{} <amount>", &first[rest.len()..]);
            let tab_result = if matches.len() == 1 {
                Some(format!("/{} {} ", cmd, first))
            } else {
                let cp = longest_common_prefix(&matches);
                if cp.len() > rest.len() {
                    Some(format!("/{} {}", cmd, cp))
                } else {
                    Some(format!("/{} {} ", cmd, first))
                }
            };
            Some(Completion { ghost, tab_result })
        }
        Some((_, amount_rest)) => {
            if amount_rest.is_empty() {
                Some(Completion {
                    ghost: "<amount>".to_string(),
                    tab_result: None,
                })
            } else {
                None
            }
        }
    }
}

fn complete_mutate(rest: &str, fish_names: &[&str]) -> Option<Completion> {
    if rest.is_empty() {
        return Some(Completion {
            ghost: "<name> <mutation>".to_string(),
            tab_result: None,
        });
    }

    let (quoted, inner) = if let Some(s) = rest.strip_prefix('"') {
        (true, s)
    } else {
        (false, rest)
    };

    if quoted {
        if let Some(end_pos) = inner.find('"') {
            let name = &inner[..end_pos];
            let after = inner[end_pos + 1..].trim_start();
            if after.is_empty() {
                return Some(Completion {
                    ghost: "<mutation>".to_string(),
                    tab_result: Some(format!("/mutate \"{}\" ", name)),
                });
            }
            return complete_mutation_part(after, name, true);
        }
        let inner_lower = inner.to_ascii_lowercase();
        let matches: Vec<&str> = fish_names
            .iter()
            .copied()
            .filter(|&n| n.to_ascii_lowercase().starts_with(inner_lower.as_str()))
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
        return Some(Completion { ghost, tab_result });
    }

    let words: Vec<&str> = inner.split_whitespace().collect();
    for end in (1..=words.len()).rev() {
        let candidate = words[..end].join(" ");
        let candidate_lower = candidate.to_ascii_lowercase();
        if let Some(&display) = fish_names
            .iter()
            .find(|&&n| n.to_ascii_lowercase() == candidate_lower)
        {
            let after_words = &words[end..];
            if after_words.is_empty() {
                let trailing = rest.ends_with(' ');
                return Some(Completion {
                    ghost: if trailing {
                        "<mutation>".to_string()
                    } else {
                        " <mutation>".to_string()
                    },
                    tab_result: Some(format!("/mutate {} ", display)),
                });
            }
            let after = after_words.join(" ");
            return complete_mutation_part(&after, display, false);
        }
    }

    let partial_lower = inner.to_ascii_lowercase();
    let matches: Vec<&str> = fish_names
        .iter()
        .copied()
        .filter(|&n| n.to_ascii_lowercase().starts_with(partial_lower.as_str()))
        .collect();
    if matches.is_empty() {
        return None;
    }
    let first = matches[0];
    let ghost = format!("{} <mutation>", &first[inner.len()..]);
    let tab_result = if matches.len() == 1 {
        Some(format!("/mutate {} ", first))
    } else {
        let cp = longest_common_prefix(&matches);
        if cp.len() > inner.len() {
            Some(format!("/mutate {}", cp))
        } else {
            Some(format!("/mutate {} ", first))
        }
    };
    Some(Completion { ghost, tab_result })
}

fn complete_mutation_part(after: &str, name: &str, quoted: bool) -> Option<Completion> {
    let after_lower = after.to_ascii_lowercase();
    let matches: Vec<&str> = MUTATION_NAMES
        .iter()
        .copied()
        .filter(|&m| m.starts_with(after_lower.as_str()))
        .collect();
    if matches.is_empty() {
        return None;
    }
    let first = matches[0];
    let ghost = first[after.len()..].to_string();
    let prefix = if quoted {
        format!("/mutate \"{}\" ", name)
    } else {
        format!("/mutate {} ", name)
    };
    let tab_result = if matches.len() == 1 {
        Some(format!("{}{}", prefix, first))
    } else {
        let cp = longest_common_prefix(&matches);
        if cp.len() > after.len() {
            Some(format!("{}{}", prefix, cp))
        } else {
            Some(format!("{}{}", prefix, first))
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
                    ghost: "<name>".to_string(),
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
            ghost: "<species> <name>".to_string(),
            tab_result: None,
        });
    }
    let partial_lower = partial.to_ascii_lowercase();
    let matches: Vec<&str> = SPECIES_NAMES
        .iter()
        .copied()
        .filter(|&n| n.starts_with(partial_lower.as_str()))
        .collect();
    if matches.is_empty() {
        return None;
    }
    let first = matches[0];
    let ghost = format!("{} <name>", &first[partial.len()..]);
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

fn complete_consume(rest: &str, consumable_names: &[&str]) -> Option<Completion> {
    if rest.contains(' ') {
        return None;
    }
    if rest.is_empty() {
        if consumable_names.is_empty() {
            return None;
        }
        return Some(Completion {
            ghost: "<consumable>".to_string(),
            tab_result: None,
        });
    }
    let rest_lower = rest.to_ascii_lowercase();
    let matches: Vec<&str> = consumable_names
        .iter()
        .copied()
        .filter(|&n| n.to_ascii_lowercase().starts_with(rest_lower.as_str()))
        .collect();
    if matches.is_empty() {
        return None;
    }
    let first = matches[0];
    let ghost = first[rest.len()..].to_string();
    let tab_result = if matches.len() == 1 {
        Some(format!("/consume {}", first))
    } else {
        let cp = longest_common_prefix(&matches);
        if cp.len() > rest.len() {
            Some(format!("/consume {}", cp))
        } else {
            Some(format!("/consume {}", first))
        }
    };
    Some(Completion { ghost, tab_result })
}

fn complete_switch(rest: &str, tank_names: &[&str], current_tank: &str) -> Option<Completion> {
    let available: Vec<&str> = tank_names
        .iter()
        .copied()
        .filter(|&n| !n.eq_ignore_ascii_case(current_tank))
        .collect();
    if available.is_empty() {
        return None;
    }
    if rest.is_empty() {
        return Some(Completion {
            ghost: "<name>".to_string(),
            tab_result: None,
        });
    }
    let (quoted, inner) = if let Some(s) = rest.strip_prefix('"') {
        (true, s)
    } else {
        (false, rest)
    };
    if quoted && inner.contains('"') {
        return None;
    }
    let inner_lower = inner.to_ascii_lowercase();
    let matches: Vec<&str> = available
        .iter()
        .copied()
        .filter(|&n| n.to_ascii_lowercase().starts_with(inner_lower.as_str()))
        .collect();
    if matches.is_empty() {
        if quoted {
            return Some(Completion {
                ghost: "\"".to_string(),
                tab_result: None,
            });
        }
        return None;
    }
    let first = matches[0];
    let ghost = if quoted {
        format!("{}\"", &first[inner.len()..])
    } else {
        first[inner.len()..].to_string()
    };
    let tab_result = if matches.len() == 1 {
        if quoted {
            Some(format!("/switch \"{}\"", first))
        } else {
            Some(format!("/switch {}", first))
        }
    } else {
        let cp = longest_common_prefix(&matches);
        if cp.len() > inner.len() {
            if quoted {
                Some(format!("/switch \"{}", cp))
            } else {
                Some(format!("/switch {}", cp))
            }
        } else if quoted {
            Some(format!("/switch \"{}\"", first))
        } else {
            Some(format!("/switch {}", first))
        }
    };
    Some(Completion { ghost, tab_result })
}

fn complete_move(
    rest: &str,
    fish_names: &[&str],
    tank_names: &[&str],
    fish_in_tanks: &[(&str, &str)],
) -> Option<Completion> {
    if rest.is_empty() {
        return Some(Completion {
            ghost: "<fish> <tank>".to_string(),
            tab_result: None,
        });
    }

    let (fish_quoted, inner) = if let Some(s) = rest.strip_prefix('"') {
        (true, s)
    } else {
        (false, rest)
    };

    if fish_quoted {
        if let Some(end_pos) = inner.find('"') {
            let fish_name = &inner[..end_pos];
            let after_fish = inner[end_pos + 1..].trim_start();
            if after_fish.is_empty() {
                return Some(Completion {
                    ghost: "<tank>".to_string(),
                    tab_result: Some(format!("/move \"{}\" \"", fish_name)),
                });
            }
            let (tank_quoted, tank_inner) = if let Some(s) = after_fish.strip_prefix('"') {
                (true, s)
            } else {
                (false, after_fish)
            };
            if tank_quoted && tank_inner.contains('"') {
                return None;
            }
            return complete_move_tank(
                fish_name,
                tank_inner,
                tank_names,
                fish_in_tanks,
                true,
                tank_quoted,
            );
        }
        let inner_lower = inner.to_ascii_lowercase();
        let matches: Vec<&str> = fish_names
            .iter()
            .copied()
            .filter(|&n| n.to_ascii_lowercase().starts_with(inner_lower.as_str()))
            .collect();
        if matches.is_empty() {
            return Some(Completion {
                ghost: "\" <tank>\"".to_string(),
                tab_result: None,
            });
        }
        let first = matches[0];
        let ghost = format!("{}\" <tank>\"", &first[inner.len()..]);
        let tab_result = if matches.len() == 1 {
            Some(format!("/move \"{}\" \"", first))
        } else {
            let cp = longest_common_prefix(&matches);
            if cp.len() > inner.len() {
                Some(format!("/move \"{}", cp))
            } else {
                Some(format!("/move \"{}\" \"", first))
            }
        };
        return Some(Completion { ghost, tab_result });
    }

    let words: Vec<&str> = inner.split_whitespace().collect();
    for end in (1..=words.len()).rev() {
        let candidate = words[..end].join(" ");
        if let Some(&display) = fish_names
            .iter()
            .find(|&&n| n.eq_ignore_ascii_case(&candidate))
        {
            let after_words = &words[end..];
            if after_words.is_empty() {
                let trailing = rest.ends_with(' ');
                return Some(Completion {
                    ghost: if trailing {
                        "<tank>".to_string()
                    } else {
                        " <tank>".to_string()
                    },
                    tab_result: Some(format!("/move {} ", display)),
                });
            }
            let tank_partial = after_words.join(" ");
            return complete_move_tank(
                display,
                &tank_partial,
                tank_names,
                fish_in_tanks,
                false,
                false,
            );
        }
    }

    let partial_lower = inner.to_ascii_lowercase();
    let matches: Vec<&str> = fish_names
        .iter()
        .copied()
        .filter(|&n| n.to_ascii_lowercase().starts_with(partial_lower.as_str()))
        .collect();
    if matches.is_empty() {
        return None;
    }
    let first = matches[0];
    let ghost = format!("{} <tank>", &first[inner.len()..]);
    let tab_result = if matches.len() == 1 {
        Some(format!("/move {} ", first))
    } else {
        let cp = longest_common_prefix(&matches);
        if cp.len() > inner.len() {
            Some(format!("/move {}", cp))
        } else {
            Some(format!("/move {} ", first))
        }
    };
    Some(Completion { ghost, tab_result })
}

fn complete_move_tank(
    fish_name: &str,
    tank_partial: &str,
    tank_names: &[&str],
    fish_in_tanks: &[(&str, &str)],
    fish_quoted: bool,
    tank_quoted: bool,
) -> Option<Completion> {
    let fish_home: Option<&str> = fish_in_tanks
        .iter()
        .find(|(fn_, _)| fn_.eq_ignore_ascii_case(fish_name))
        .map(|(_, tn)| *tn);
    let available: Vec<&str> = tank_names
        .iter()
        .copied()
        .filter(|&t| fish_home.is_none_or(|h| !t.eq_ignore_ascii_case(h)))
        .collect();
    let partial_lower = tank_partial.to_ascii_lowercase();
    let matches: Vec<&str> = available
        .iter()
        .copied()
        .filter(|&n| n.to_ascii_lowercase().starts_with(partial_lower.as_str()))
        .collect();
    let fish_part = if fish_quoted {
        format!("/move \"{}\" ", fish_name)
    } else {
        format!("/move {} ", fish_name)
    };
    if matches.is_empty() {
        if tank_quoted {
            return Some(Completion {
                ghost: "\"".to_string(),
                tab_result: None,
            });
        }
        return None;
    }
    let first = matches[0];
    let ghost = if tank_quoted {
        format!("{}\"", &first[tank_partial.len()..])
    } else {
        first[tank_partial.len()..].to_string()
    };
    let tab_result = if matches.len() == 1 {
        if tank_quoted {
            Some(format!("{}\"{}\"", fish_part, first))
        } else {
            Some(format!("{}{}", fish_part, first))
        }
    } else {
        let cp = longest_common_prefix(&matches);
        if cp.len() > tank_partial.len() {
            if tank_quoted {
                Some(format!("{}\"{}", fish_part, cp))
            } else {
                Some(format!("{}{}", fish_part, cp))
            }
        } else if tank_quoted {
            Some(format!("{}\"{}\"", fish_part, first))
        } else {
            Some(format!("{}{}", fish_part, first))
        }
    };
    Some(Completion { ghost, tab_result })
}

fn complete_index(rest: &str, tank_names: &[&str]) -> Option<Completion> {
    if tank_names.is_empty() {
        return None;
    }
    if rest.is_empty() {
        return Some(Completion {
            ghost: "<name>".to_string(),
            tab_result: None,
        });
    }
    let (quoted, inner) = if let Some(s) = rest.strip_prefix('"') {
        (true, s)
    } else {
        (false, rest)
    };
    if quoted && inner.contains('"') {
        return None;
    }
    let inner_lower = inner.to_ascii_lowercase();
    let matches: Vec<&str> = tank_names
        .iter()
        .copied()
        .filter(|&n| n.to_ascii_lowercase().starts_with(inner_lower.as_str()))
        .collect();
    if matches.is_empty() {
        if quoted {
            return Some(Completion {
                ghost: "\"".to_string(),
                tab_result: None,
            });
        }
        return None;
    }
    let first = matches[0];
    let ghost = if quoted {
        format!("{}\"", &first[inner.len()..])
    } else {
        first[inner.len()..].to_string()
    };
    let tab_result = if matches.len() == 1 {
        if quoted {
            Some(format!("/index \"{}\"", first))
        } else {
            Some(format!("/index {}", first))
        }
    } else {
        let cp = longest_common_prefix(&matches);
        if cp.len() > inner.len() {
            if quoted {
                Some(format!("/index \"{}", cp))
            } else {
                Some(format!("/index {}", cp))
            }
        } else if quoted {
            Some(format!("/index \"{}\"", first))
        } else {
            Some(format!("/index {}", first))
        }
    };
    Some(Completion { ghost, tab_result })
}

fn complete_show(rest: &str, fish_in_tanks: &[(&str, &str)]) -> Option<Completion> {
    let all_names: Vec<&str> = fish_in_tanks.iter().map(|(n, _)| *n).collect();
    if all_names.is_empty() {
        return None;
    }
    if rest.is_empty() {
        return Some(Completion {
            ghost: "<name>".to_string(),
            tab_result: None,
        });
    }
    let (quoted, inner) = if let Some(s) = rest.strip_prefix('"') {
        (true, s)
    } else {
        (false, rest)
    };
    if quoted && inner.contains('"') {
        return None;
    }
    let inner_lower = inner.to_ascii_lowercase();
    let matches: Vec<&str> = all_names
        .iter()
        .copied()
        .filter(|&n| n.to_ascii_lowercase().starts_with(inner_lower.as_str()))
        .collect();
    if matches.is_empty() {
        if quoted {
            return Some(Completion {
                ghost: "\"".to_string(),
                tab_result: None,
            });
        }
        return None;
    }
    let first = matches[0];
    let ghost = if quoted {
        format!("{}\"", &first[inner.len()..])
    } else {
        first[inner.len()..].to_string()
    };
    let tab_result = if matches.len() == 1 {
        if quoted {
            Some(format!("/show \"{}\"", first))
        } else {
            Some(format!("/show {}", first))
        }
    } else {
        let cp = longest_common_prefix(&matches);
        if cp.len() > inner.len() {
            if quoted {
                Some(format!("/show \"{}", cp))
            } else {
                Some(format!("/show {}", cp))
            }
        } else if quoted {
            Some(format!("/show \"{}\"", first))
        } else {
            Some(format!("/show {}", first))
        }
    };
    Some(Completion { ghost, tab_result })
}

fn command_args_placeholder(cmd: &str) -> &'static str {
    match cmd {
        "add" | "subtract" => "<resource> <amount>",
        "consume" => "<consumable>",
        "feed" => "<amount>",
        "fps" => "<n>",
        "index" | "show" | "switch" => "<name>",
        "move" => "<fish> <tank>",
        "mutate" => "<name> <mutation>",
        "spawn" => "<species> <name>",
        _ => "",
    }
}

fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(c) => c.to_uppercase().to_string() + chars.as_str(),
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
    Show {
        name: String,
        all: bool,
    },
    Index {
        all: bool,
        tank_filter: Option<String>,
    },
    Fish {
        no_death: bool,
        no_fish: bool,
    },
    Inventory,
    Shop,
    Consume {
        name: String,
    },
    ToggleNames,
    ToggleStats,
    ModResource {
        name: String,
        delta: i32,
    },
    Switch(String),
    Move {
        fish: String,
        tank: String,
    },
    Fishtanks,
    Exit,
    VoidSpawn,
    StartVoidWish {
        skip: bool,
    },
    Unknown,
}

fn greedy_name_words<'a>(words: &[&str], names: &[&'a str]) -> Option<(usize, &'a str)> {
    for end in (1..=words.len()).rev() {
        let candidate = words[..end].join(" ");
        if let Some(&display) = names.iter().find(|&&n| n.eq_ignore_ascii_case(&candidate)) {
            return Some((end, display));
        }
    }
    None
}

fn parse_raw_arg(rest: &str) -> String {
    let rest = rest.trim();
    if let Some(inner) = rest.strip_prefix('"')
        && let Some(end) = inner.find('"')
    {
        return inner[..end].to_string();
    }
    if let Some(inner) = rest.strip_prefix('\'')
        && let Some(end) = inner.find('\'')
    {
        return inner[..end].to_string();
    }
    rest.split_whitespace().next().unwrap_or("").to_string()
}

fn parse_name_greedy(rest: &str, names: &[&str]) -> Option<String> {
    let rest = rest.trim();
    if rest.is_empty() {
        return None;
    }
    if rest.starts_with('"') || rest.starts_with('\'') {
        let q = rest.chars().next().unwrap();
        let inner = &rest[1..];
        let end = inner.find(q)?;
        let name = &inner[..end];
        if name.is_empty() {
            return None;
        }
        return Some(name.to_string());
    }
    let words: Vec<&str> = rest.split_whitespace().collect();
    let (_, display) = greedy_name_words(&words, names)?;
    Some(display.to_string())
}

fn parse_name_and_all_flag(rest: &str, names: &[&str]) -> Option<(String, bool)> {
    let rest = rest.trim();
    if rest.starts_with('"') || rest.starts_with('\'') {
        let q = rest.chars().next().unwrap();
        let inner = &rest[1..];
        let end = inner.find(q)?;
        let name = inner[..end].to_string();
        if name.is_empty() {
            return None;
        }
        let after = inner[end + 1..].trim();
        return Some((name, after.eq_ignore_ascii_case("all")));
    }
    let words: Vec<&str> = rest.split_whitespace().collect();
    let (consumed, display) = greedy_name_words(&words, names)?;
    let remaining = words[consumed..].join(" ");
    let all_flag = remaining.trim().eq_ignore_ascii_case("all");
    Some((display.to_string(), all_flag))
}

fn parse_name_and_mutation(rest: &str, fish_names: &[&str]) -> Option<(String, String)> {
    let rest = rest.trim();
    if rest.starts_with('"') || rest.starts_with('\'') {
        let q = rest.chars().next().unwrap();
        let inner = &rest[1..];
        let end = inner.find(q)?;
        let name = inner[..end].to_string();
        if name.is_empty() {
            return None;
        }
        let mutation = inner[end + 1..].trim().to_string();
        return Some((name, mutation));
    }
    let words: Vec<&str> = rest.split_whitespace().collect();
    let (consumed, display) = greedy_name_words(&words, fish_names)?;
    let mutation = words[consumed..].join(" ");
    Some((display.to_string(), mutation))
}

fn parse_fish_tank_args(
    rest: &str,
    fish_names: &[&str],
    tank_names: &[&str],
) -> Option<(String, String)> {
    let rest = rest.trim();
    let (fish, tank_rest) = if rest.starts_with('"') || rest.starts_with('\'') {
        let q = rest.chars().next().unwrap();
        let inner = &rest[1..];
        let end = inner.find(q)?;
        let fish = inner[..end].to_string();
        (fish, inner[end + 1..].to_string())
    } else {
        let words: Vec<&str> = rest.split_whitespace().collect();
        let (consumed, display) = greedy_name_words(&words, fish_names)?;
        let tank_rest = words[consumed..].join(" ");
        (display.to_string(), tank_rest)
    };
    if fish.is_empty() {
        return None;
    }
    let tank = parse_name_greedy(tank_rest.trim(), tank_names)?;
    Some((fish, tank))
}

pub fn parse(input: &str, fish_names: &[&str], tank_names: &[&str]) -> Action {
    let input = input.trim();
    if !input.starts_with('/') {
        return Action::Unknown;
    }
    let body = &input[1..];

    let (cmd_lower, rest) = match body.find(' ') {
        Some(pos) => (body[..pos].to_ascii_lowercase(), body[pos + 1..].trim()),
        None => (body.to_ascii_lowercase(), ""),
    };

    match cmd_lower.as_str() {
        "mutate" => {
            if rest.is_empty() {
                return Action::Unknown;
            }
            match parse_name_and_mutation(rest, fish_names) {
                Some((name, mutation)) => Action::Mutate(name, mutation),
                None => Action::Unknown,
            }
        }
        "switch" => match parse_name_greedy(rest, tank_names) {
            Some(name) if !name.is_empty() => Action::Switch(name),
            _ => Action::Unknown,
        },
        "move" => {
            if rest.is_empty() {
                return Action::Unknown;
            }
            match parse_fish_tank_args(rest, fish_names, tank_names) {
                Some((fish, tank)) => Action::Move { fish, tank },
                None => Action::Unknown,
            }
        }
        "show" => {
            if rest.is_empty() {
                return Action::Unknown;
            }
            match parse_name_and_all_flag(rest, fish_names) {
                Some((name, all)) => Action::Show { name, all },
                None => Action::Unknown,
            }
        }
        "index" => {
            if rest.is_empty() {
                return Action::Index {
                    all: false,
                    tank_filter: None,
                };
            }
            if rest.eq_ignore_ascii_case("all") {
                return Action::Index {
                    all: true,
                    tank_filter: None,
                };
            }
            match parse_name_greedy(rest, tank_names) {
                Some(name) if !name.is_empty() => Action::Index {
                    all: false,
                    tank_filter: Some(name),
                },
                _ => Action::Unknown,
            }
        }
        "consume" => {
            let name = parse_raw_arg(rest);
            if name.is_empty() {
                Action::Unknown
            } else {
                Action::Consume { name }
            }
        }
        "feed" => {
            if rest.is_empty() {
                return Action::Feed(0);
            }
            rest.parse().map(Action::Feed).unwrap_or(Action::Unknown)
        }
        "fps" => rest.parse().map(Action::SetFps).unwrap_or(Action::Unknown),
        "add" | "subtract" => {
            let mut parts = rest.splitn(2, ' ');
            let resource = match parts.next() {
                Some(r) if !r.is_empty() => r,
                _ => return Action::Unknown,
            };
            let n_str = match parts.next() {
                Some(n) => n.trim(),
                None => return Action::Unknown,
            };
            let Ok(qty) = n_str.parse::<u32>() else {
                return Action::Unknown;
            };
            let delta = if cmd_lower == "add" {
                qty as i32
            } else {
                -(qty as i32)
            };
            Action::ModResource {
                name: capitalize(resource),
                delta,
            }
        }
        "spawn" => {
            let mut parts = rest.splitn(2, ' ');
            let species_str = match parts.next() {
                Some(s) if !s.is_empty() => s,
                _ => return Action::Unknown,
            };
            let name_raw = match parts.next() {
                Some(n) => n.trim().trim_matches('"').to_string(),
                None => return Action::Unknown,
            };
            if name_raw.is_empty() {
                return Action::Unknown;
            }
            match FishSpecies::from_str(species_str) {
                Some(species) => Action::Spawn(species, name_raw),
                None => Action::Unknown,
            }
        }
        "fish" => {
            let flags: Vec<&str> = rest.split_whitespace().collect();
            Action::Fish {
                no_death: flags.contains(&"--no-death"),
                no_fish: flags.contains(&"--no-fish"),
            }
        }
        "exit" => Action::Exit,
        "voidspawn" => Action::VoidSpawn,
        "startvoidwish" => {
            let flags: Vec<&str> = rest.split_whitespace().collect();
            Action::StartVoidWish {
                skip: flags.contains(&"--skip"),
            }
        }
        "fishtanks" => Action::Fishtanks,
        "inventory" => Action::Inventory,
        "shop" => Action::Shop,
        "names" => Action::ToggleNames,
        "stats" => Action::ToggleStats,
        _ => Action::Unknown,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn no_names() -> (&'static [&'static str], &'static [&'static str]) {
        (&[], &[])
    }

    #[test]
    fn parse_feed_no_arg_returns_feed_zero() {
        let (fish, tanks) = no_names();
        assert!(matches!(parse("/feed", fish, tanks), Action::Feed(0)));
    }

    #[test]
    fn parse_feed_with_number() {
        let (fish, tanks) = no_names();
        assert!(matches!(parse("/feed 5", fish, tanks), Action::Feed(5)));
    }

    #[test]
    fn parse_feed_invalid_arg_is_unknown() {
        let (fish, tanks) = no_names();
        assert!(matches!(parse("/feed abc", fish, tanks), Action::Unknown));
    }

    #[test]
    fn parse_fps() {
        let (fish, tanks) = no_names();
        assert!(matches!(parse("/fps 60", fish, tanks), Action::SetFps(v) if v == 60.0));
    }

    #[test]
    fn parse_fps_invalid_is_unknown() {
        let (fish, tanks) = no_names();
        assert!(matches!(parse("/fps nope", fish, tanks), Action::Unknown));
    }

    #[test]
    fn parse_spawn_quoted_name() {
        let (fish, tanks) = no_names();
        let action = parse("/spawn merluza \"Nemo\"", fish, tanks);
        assert!(
            matches!(&action, Action::Spawn(s, n) if *s == FishSpecies::Merluza && n == "Nemo")
        );
    }

    #[test]
    fn parse_spawn_unquoted_name() {
        let (fish, tanks) = no_names();
        let action = parse("/spawn salmon Bob", fish, tanks);
        assert!(matches!(&action, Action::Spawn(s, n) if *s == FishSpecies::Salmon && n == "Bob"));
    }

    #[test]
    fn parse_spawn_unknown_species_is_unknown() {
        let (fish, tanks) = no_names();
        assert!(matches!(
            parse("/spawn unknownspecies Blob", fish, tanks),
            Action::Unknown
        ));
    }

    #[test]
    fn parse_spawn_missing_name_is_unknown() {
        let (fish, tanks) = no_names();
        assert!(matches!(
            parse("/spawn merluza", fish, tanks),
            Action::Unknown
        ));
    }

    #[test]
    fn parse_exit() {
        let (fish, tanks) = no_names();
        assert!(matches!(parse("/exit", fish, tanks), Action::Exit));
    }

    #[test]
    fn parse_fishtanks() {
        let (fish, tanks) = no_names();
        assert!(matches!(
            parse("/fishtanks", fish, tanks),
            Action::Fishtanks
        ));
    }

    #[test]
    fn parse_inventory() {
        let (fish, tanks) = no_names();
        assert!(matches!(
            parse("/inventory", fish, tanks),
            Action::Inventory
        ));
    }

    #[test]
    fn parse_shop() {
        let (fish, tanks) = no_names();
        assert!(matches!(parse("/shop", fish, tanks), Action::Shop));
    }

    #[test]
    fn parse_names() {
        let (fish, tanks) = no_names();
        assert!(matches!(parse("/names", fish, tanks), Action::ToggleNames));
    }

    #[test]
    fn parse_stats() {
        let (fish, tanks) = no_names();
        assert!(matches!(parse("/stats", fish, tanks), Action::ToggleStats));
    }

    #[test]
    fn parse_voidspawn() {
        let (fish, tanks) = no_names();
        assert!(matches!(
            parse("/voidspawn", fish, tanks),
            Action::VoidSpawn
        ));
    }

    #[test]
    fn parse_fish_no_flags() {
        let (fish, tanks) = no_names();
        assert!(matches!(
            parse("/fish", fish, tanks),
            Action::Fish {
                no_death: false,
                no_fish: false
            }
        ));
    }

    #[test]
    fn parse_fish_no_death_flag() {
        let (fish, tanks) = no_names();
        assert!(matches!(
            parse("/fish --no-death", fish, tanks),
            Action::Fish {
                no_death: true,
                no_fish: false
            }
        ));
    }

    #[test]
    fn parse_fish_no_fish_flag() {
        let (fish, tanks) = no_names();
        assert!(matches!(
            parse("/fish --no-fish", fish, tanks),
            Action::Fish {
                no_death: false,
                no_fish: true
            }
        ));
    }

    #[test]
    fn parse_fish_both_flags() {
        let (fish, tanks) = no_names();
        assert!(matches!(
            parse("/fish --no-death --no-fish", fish, tanks),
            Action::Fish {
                no_death: true,
                no_fish: true
            }
        ));
    }

    #[test]
    fn parse_index_no_arg() {
        let (fish, tanks) = no_names();
        assert!(matches!(
            parse("/index", fish, tanks),
            Action::Index {
                all: false,
                tank_filter: None
            }
        ));
    }

    #[test]
    fn parse_index_all() {
        let (fish, tanks) = no_names();
        assert!(matches!(
            parse("/index all", fish, tanks),
            Action::Index {
                all: true,
                tank_filter: None
            }
        ));
    }

    #[test]
    fn parse_index_tank_filter() {
        let fish: &[&str] = &[];
        let tanks: &[&str] = &["Ocean"];
        assert!(matches!(
            parse("/index Ocean", fish, tanks),
            Action::Index { all: false, tank_filter: Some(ref n) } if n == "Ocean"
        ));
    }

    #[test]
    fn parse_show_unquoted() {
        let fish: &[&str] = &["Nemo"];
        let tanks: &[&str] = &[];
        assert!(
            matches!(parse("/show Nemo", fish, tanks), Action::Show { ref name, all: false } if name == "Nemo")
        );
    }

    #[test]
    fn parse_show_quoted() {
        let fish: &[&str] = &["Blue Tang"];
        let tanks: &[&str] = &[];
        assert!(matches!(
            parse("/show \"Blue Tang\"", fish, tanks),
            Action::Show { ref name, all: false } if name == "Blue Tang"
        ));
    }

    #[test]
    fn parse_show_with_all_flag() {
        let fish: &[&str] = &["Nemo"];
        let tanks: &[&str] = &[];
        assert!(matches!(
            parse("/show Nemo all", fish, tanks),
            Action::Show { ref name, all: true } if name == "Nemo"
        ));
    }

    #[test]
    fn parse_show_no_arg_is_unknown() {
        let (fish, tanks) = no_names();
        assert!(matches!(parse("/show", fish, tanks), Action::Unknown));
    }

    #[test]
    fn parse_switch_known_tank() {
        let fish: &[&str] = &[];
        let tanks: &[&str] = &["Ocean"];
        assert!(matches!(
            parse("/switch Ocean", fish, tanks),
            Action::Switch(ref n) if n == "Ocean"
        ));
    }

    #[test]
    fn parse_switch_no_arg_is_unknown() {
        let (fish, tanks) = no_names();
        assert!(matches!(parse("/switch", fish, tanks), Action::Unknown));
    }

    #[test]
    fn parse_move_known_fish_and_tank() {
        let fish: &[&str] = &["Nemo"];
        let tanks: &[&str] = &["Ocean"];
        assert!(matches!(
            parse("/move Nemo Ocean", fish, tanks),
            Action::Move { ref fish, ref tank } if fish == "Nemo" && tank == "Ocean"
        ));
    }

    #[test]
    fn parse_move_quoted_args() {
        let fish: &[&str] = &["Blue Tang"];
        let tanks: &[&str] = &["Coral Reef"];
        assert!(matches!(
            parse("/move \"Blue Tang\" \"Coral Reef\"", fish, tanks),
            Action::Move { ref fish, ref tank } if fish == "Blue Tang" && tank == "Coral Reef"
        ));
    }

    #[test]
    fn parse_mutate_unquoted() {
        let fish: &[&str] = &["Nemo"];
        let tanks: &[&str] = &[];
        assert!(matches!(
            parse("/mutate Nemo eye+", fish, tanks),
            Action::Mutate(ref n, ref m) if n == "Nemo" && m == "eye+"
        ));
    }

    #[test]
    fn parse_mutate_quoted_name() {
        let fish: &[&str] = &["Blue Tang"];
        let tanks: &[&str] = &[];
        assert!(matches!(
            parse("/mutate \"Blue Tang\" size+", fish, tanks),
            Action::Mutate(ref n, ref m) if n == "Blue Tang" && m == "size+"
        ));
    }

    #[test]
    fn parse_add_resource() {
        let (fish, tanks) = no_names();
        assert!(matches!(
            parse("/add food 10", fish, tanks),
            Action::ModResource { ref name, delta: 10 } if name == "Food"
        ));
    }

    #[test]
    fn parse_subtract_resource() {
        let (fish, tanks) = no_names();
        assert!(matches!(
            parse("/subtract cash 5", fish, tanks),
            Action::ModResource { ref name, delta: -5 } if name == "Cash"
        ));
    }

    #[test]
    fn parse_consume_unquoted() {
        let (fish, tanks) = no_names();
        assert!(matches!(
            parse("/consume coffee", fish, tanks),
            Action::Consume { ref name } if name == "coffee"
        ));
    }

    #[test]
    fn parse_unknown_command() {
        let (fish, tanks) = no_names();
        assert!(matches!(parse("/glorp", fish, tanks), Action::Unknown));
    }

    #[test]
    fn parse_no_slash_is_unknown() {
        let (fish, tanks) = no_names();
        assert!(matches!(parse("feed 5", fish, tanks), Action::Unknown));
    }

    #[test]
    fn autocomplete_partial_command_ghost() {
        let result = autocomplete("/fe", &[], &[], &[], "", &[]);
        let c = result.unwrap();
        assert!(c.ghost.starts_with("ed"));
    }

    #[test]
    fn autocomplete_exact_command_with_args_ghost() {
        let result = autocomplete("/feed", &[], &[], &[], "", &[]);
        let c = result.unwrap();
        assert_eq!(c.ghost, " <amount>");
    }

    #[test]
    fn autocomplete_exact_command_tab_result_adds_space() {
        let result = autocomplete("/feed", &[], &[], &[], "", &[]);
        let c = result.unwrap();
        assert_eq!(c.tab_result, Some("/feed ".to_string()));
    }

    #[test]
    fn autocomplete_spawn_partial_species_ghost() {
        let result = autocomplete("/spawn mer", &[], &[], &[], "", &[]);
        let c = result.unwrap();
        assert!(c.ghost.contains("luza"));
    }

    #[test]
    fn autocomplete_unknown_prefix_returns_none() {
        let result = autocomplete("/zzz", &[], &[], &[], "", &[]);
        assert!(result.is_none());
    }

    #[test]
    fn autocomplete_fish_name_ghost_in_mutate() {
        let result = autocomplete("/mutate Ne", &["Nemo"], &[], &[], "", &[]);
        let c = result.unwrap();
        assert!(c.ghost.contains("mo"));
    }

    #[test]
    fn autocomplete_mutation_name_ghost() {
        let result = autocomplete("/mutate Nemo eye", &["Nemo"], &[], &[], "", &[]);
        let c = result.unwrap();
        assert!(!c.ghost.is_empty());
    }

    #[test]
    fn autocomplete_show_fish_name() {
        let result = autocomplete("/show Ne", &[], &[], &[], "", &[("Nemo", "Tank1")]);
        let c = result.unwrap();
        assert!(c.ghost.contains("mo"));
    }

    #[test]
    fn autocomplete_no_input_returns_none() {
        let result = autocomplete("", &[], &[], &[], "", &[]);
        assert!(result.is_none());
    }

    #[test]
    fn parse_startvoidwish_no_flags() {
        let (fish, tanks) = no_names();
        assert!(matches!(
            parse("/startvoidwish", fish, tanks),
            Action::StartVoidWish { skip: false }
        ));
    }

    #[test]
    fn parse_startvoidwish_skip_flag() {
        let (fish, tanks) = no_names();
        assert!(matches!(
            parse("/startvoidwish --skip", fish, tanks),
            Action::StartVoidWish { skip: true }
        ));
    }
}
