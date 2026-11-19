//! Decode the ANSI-C quoted values emitted by the bundled shell hooks.

/// Whether Bash emitted this value in its ANSI-C quoting form.
pub fn is_ansi_c_quoted(value: &str) -> bool {
    value.starts_with("$'") && value.ends_with('\'')
}

/// Preserve unquoted text and decode the fixed-width escapes emitted by printf %q.
pub fn parse_ansi_c_quoted_string(value: String) -> String {
    let Some(quoted) = value
        .strip_prefix("$'")
        .and_then(|text| text.strip_suffix('\''))
    else {
        return value;
    };
    let mut output = String::with_capacity(quoted.len());
    let mut characters = quoted.char_indices();
    while let Some((_, character)) = characters.next() {
        if character != '\\' {
            output.push(character);
            continue;
        }
        let Some((offset, escape)) = characters.next() else {
            break;
        };
        let character = match escape {
            'a' => Some('\x07'),
            'b' => Some('\x08'),
            'E' | 'e' => Some('\x1b'),
            'f' => Some('\x0c'),
            'n' => Some('\n'),
            'r' => Some('\r'),
            't' => Some('\t'),
            'v' => Some('\x0b'),
            '\\' | '\'' | '"' | '?' => Some(escape),
            'c' => characters.next().and_then(|(_, code)| match code {
                '@'..='_' | '\x7f' => Some(char::from(code as u8 - 64)),
                _ => None,
            }),
            _ => {
                let (first, count, skip, radix) = match escape {
                    'x' => (offset + 1, 2, 2, 16),
                    'u' => (offset + 1, 4, 4, 16),
                    'U' => (offset + 1, 8, 8, 16),
                    _ => (offset, 3, 2, 8),
                };
                let digits = quoted.get(first..first + count);
                if radix == 8 || digits.is_some() {
                    characters.nth(skip - 1);
                }
                match digits {
                    Some("001" | "002") if radix == 8 => None,
                    Some(digits) => u32::from_str_radix(digits, radix)
                        .ok()
                        .and_then(char::from_u32),
                    None => None,
                }
            }
        };
        output.extend(character);
    }
    output
}

#[cfg(test)]
#[path = "ansi_c_decoder_test.rs"]
mod tests;
