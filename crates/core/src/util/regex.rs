use once_cell::sync::Lazy;
use regex::Regex;

/// Regex for valid role colour
///
/// Allows the use of named colour, rgb(a), variables and all gradients.
///
/// Flags:
/// - Case-insensitive (`i`)
///
/// Source:
/// ```regex
/// VALUE = [a-z ]+|var\(--[a-z\d-]+\)|rgba?\([\d, ]+\)|#[a-f0-9]+
/// ADDITIONAL_VALUE = \d+deg
/// STOP = ([ ]+(\d{1,3}%|0))?
///
/// ^(?:VALUE|(repeating-)?(linear|conic|radial)-gradient\((VALUE|ADDITIONAL_VALUE)STOP(,[ ]*(VALUE)STOP)+\))$
/// ```
pub static RE_COLOUR: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)^(?:[a-z ]+|var\(--[a-z\d-]+\)|rgba?\([\d, ]+\)|#[a-f0-9]+|(repeating-)?(linear|conic|radial)-gradient\(([a-z ]+|var\(--[a-z\d-]+\)|rgba?\([\d, ]+\)|#[a-f0-9]+|\d+deg)([ ]+(\d{1,3}%|0))?(,[ ]*([a-z ]+|var\(--[a-z\d-]+\)|rgba?\([\d, ]+\)|#[a-f0-9]+)([ ]+(\d{1,3}%|0))?)+\))$").unwrap()
});
