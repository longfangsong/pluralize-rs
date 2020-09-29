#[macro_use]
extern crate lazy_static;

use regex::{Regex, RegexBuilder};
use voca_rs::case;

fn restore_case(origin: &str, to_restore: &str) -> String {
    if origin == to_restore {
        to_restore.to_string()
    } else if origin == case::lower_case(origin) {
        case::lower_case(to_restore)
    } else if origin == case::upper_case(origin) {
        case::upper_case(to_restore)
    } else if origin == case::upper_first(origin) {
        case::upper_first(to_restore)
    } else if origin == case::camel_case(origin) {
        case::camel_case(to_restore)
    } else {
        case::lower_case(to_restore)
    }
}

macro_rules! load_config_map {
    ($filename:expr) => {
        include_str!($filename)
            .split('\n')
            .filter(|it| it.trim() != "")
            .map(|it| {
                let mut splitted = it
                    .split('=')
                    .map(|it| it.trim().trim_matches(|ch| ch == '\"'));
                (splitted.next().unwrap(), splitted.next().unwrap())
            })
            .map(|(k, v)| {
                (
                    RegexBuilder::new(k).case_insensitive(true).build().unwrap(),
                    v.to_string(),
                )
            })
            .rev()
            .collect()
    };
}

lazy_static! {
    static ref IRREGULAR: Vec<(&'static str, &'static str)> =
        include_str!("../rules/irregular.txt")
            .split('\n')
            .filter(|it| it.trim() != "")
            .map(|it| {
                let mut splitted = it
                    .split('=')
                    .map(|it| it.trim().trim_matches(|ch| ch == '\"'));
                (splitted.next().unwrap(), splitted.next().unwrap())
            })
            .collect();
    static ref PLURAL_RULES: Vec<(Regex, String)> = load_config_map!("../rules/plural.txt");
    static ref SINGLAR_RULES: Vec<(Regex, String)> = load_config_map!("../rules/singular.txt");
    static ref UNCOUNTABLE: Vec<Regex> = include_str!("../rules/uncountable.txt")
        .split('\n')
        .filter(|it| it.trim() != "")
        .map(|it| RegexBuilder::new(it)
            .case_insensitive(true)
            .build()
            .unwrap())
        .collect();
}

/// Returns whether a noun is uncountable
///
/// # Arguments
///
/// * `word` - The noun
///
/// # Examples
///
/// ```
/// use pluralize_rs::is_uncountable;
/// assert!(is_uncountable("water"));
/// ```
pub fn is_uncountable(word: &str) -> bool {
    let lower_case = case::lower_case(word);
    for (singular, plural) in IRREGULAR.iter() {
        if lower_case == *singular || lower_case == *plural {
            return false;
        }
    }
    for r in UNCOUNTABLE.iter() {
        if r.find(&lower_case).is_some() {
            return true;
        }
    }
    false
}

fn replace_with_rules(
    word: &str,
    mut rules: impl Iterator<Item=&'static (Regex, String)>,
) -> String {
    if let Some((m, mut r)) = rules
        .find_map(|(re, replace_to)| re.captures(&word).map(move |it| (it, replace_to.clone())))
    {
        if r == "$0" {
            return word.to_string();
        }
        let mut result = word[0..m.get(0).unwrap().start()].to_string();
        for (i, content) in ["$1", "$2"].iter().enumerate() {
            r = if let Some(replace_to) = m.get(i + 1).map(|it| &word[it.start()..it.end()]) {
                r.replace(content, replace_to)
            } else {
                r.replace(content, "")
            }
        }
        result.push_str(&r);
        result
    } else {
        word.to_string()
    }
}

/// Returns a noun's plural form, if it is uncountable, the origin value will be returned
///
/// # Arguments
///
/// * `word` - The noun
///
/// # Examples
///
/// ```
/// use pluralize_rs::to_plural;
/// assert_eq!(to_plural("word"), "words");
/// ```
pub fn to_plural(word: &str) -> String {
    if is_uncountable(word) {
        word.to_string()
    } else {
        let lower_case = case::lower_case(word);
        for (singular, plural) in IRREGULAR.iter() {
            if lower_case == *singular {
                return restore_case(word, plural);
            }
        }
        restore_case(word, &replace_with_rules(&word, PLURAL_RULES.iter()))
    }
}

/// Returns wheter the noun is plural, if it is uncountable, will return true
///
/// # Arguments
///
/// * `word` - The noun
///
/// # Examples
///
/// ```
/// use pluralize_rs::is_plural;
/// assert!(is_plural("words"));
/// assert!(!is_plural("word"));
/// ```
pub fn is_plural(word: &str) -> bool {
    if is_uncountable(word) {
        false
    } else {
        let lower_case = case::lower_case(word);
        for (singular, plural) in IRREGULAR.iter() {
            if lower_case == *singular {
                return false;
            } else if lower_case == *plural {
                return true;
            }
        }
        lower_case == replace_with_rules(&lower_case, PLURAL_RULES.iter())
    }
}

/// Returns a noun's singular form, if it is uncountable, the origin value will be returned
///
/// # Arguments
///
/// * `word` - The noun
///
/// # Examples
///
/// ```
/// use pluralize_rs::to_singular;
/// assert_eq!(to_singular("words"), "word");
/// ```
pub fn to_singular(word: &str) -> String {
    if is_uncountable(word) {
        word.to_string()
    } else {
        let lower_case = case::lower_case(word);
        for (singular, plural) in IRREGULAR.iter() {
            if lower_case == *plural {
                return restore_case(word, singular);
            }
        }
        restore_case(word, &replace_with_rules(&word, SINGLAR_RULES.iter()))
    }
}

/// Returns wheter the noun is singular, if it is uncountable, will return true
///
/// # Arguments
///
/// * `word` - The noun
///
/// # Examples
///
/// ```
/// use pluralize_rs::is_singular;
/// assert!(!is_singular("words"));
/// assert!(is_singular("word"));
/// ```
pub fn is_singular(word: &str) -> bool {
    if is_uncountable(word) {
        false
    } else {
        let lower_case = case::lower_case(word);
        for (singular, plural) in IRREGULAR.iter() {
            if lower_case == *plural {
                return false;
            } else if lower_case == *singular {
                return true;
            }
        }
        lower_case == replace_with_rules(&lower_case, SINGLAR_RULES.iter())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let cases: Vec<_> = include_str!("../test-cases.txt")
            .split('\n')
            .filter(|it| !it.trim().is_empty())
            .map(|it| {
                let mut splitted = it.split(',');
                (
                    splitted.next().unwrap().trim(),
                    splitted.next().unwrap().trim(),
                )
            })
            .collect();
        for (singular, plural) in cases {
            println!("{} <=> {}", singular, plural);
            assert_eq!(to_plural(singular), plural);
            assert_eq!(to_singular(plural), singular);
            if !is_uncountable(singular) {
                assert!(is_singular(singular));
                assert!(is_plural(plural));
            }
        }
    }
}
