use crate::constants::*;

#[derive(Clone, Debug, PartialEq)]
pub enum Style {
    Old,
    OldInverse,
    New,
    NewInverse,
    Error,
}

#[derive(Clone, Debug, PartialEq)]
pub struct StyledToken {
    token: String,
    style: Style,
}

impl StyledToken {
    pub fn new(token: String, style: Style) -> StyledToken {
        return StyledToken { token, style };
    }

    pub fn is_whitespace(&self) -> bool {
        let mut chars_iterator = self.token.chars();
        let first_char = chars_iterator.next().unwrap();
        if chars_iterator.next().is_some() {
            // Multiple chars found in this token, but whitespace will only be
            // one per token.
            return false;
        }

        return first_char.is_whitespace();
    }

    pub fn is_word(&self) -> bool {
        let mut chars_iterator = self.token.chars();
        let first_char = chars_iterator.next().unwrap();
        let second_char = chars_iterator.next();
        if second_char.is_some() {
            // We consist of multiple characters, that means we are a word
            return true;
        }

        // If we get here, it means our token consists of one character only. If
        // that single character is alphanumeric, we are a word, otherwise not.
        return first_char.is_alphanumeric();
    }
}

pub struct TokenCollector {
    line_prefix: StyledToken,
    tokens: Vec<StyledToken>,
    bytes_count: usize,
    highlighted_bytes_count: usize,
    rendered: bool,
}

impl Style {
    #[must_use]
    pub fn is_inverse(&self) -> bool {
        match self {
            Style::OldInverse | Style::NewInverse | Style::Error => {
                return true;
            }
            _ => {
                return false;
            }
        }
    }

    pub fn inverted(&self) -> Style {
        return match self {
            Style::Old => Style::OldInverse,
            Style::New => Style::NewInverse,
            Style::OldInverse => Style::OldInverse,
            Style::NewInverse => Style::NewInverse,
            Style::Error => Style::Error,
        };
    }

    #[must_use]
    pub fn color<'a>(&self) -> &'a str {
        match self {
            Style::Old => {
                return OLD;
            }
            Style::OldInverse => {
                return OLD;
            }
            Style::New => {
                return NEW;
            }
            Style::NewInverse => {
                return NEW;
            }
            Style::Error => {
                return ERROR;
            }
        }
    }
}

impl TokenCollector {
    #[must_use]
    pub fn create(line_prefix: StyledToken) -> Self {
        return TokenCollector {
            line_prefix,
            tokens: Vec::new(),
            bytes_count: 0,
            highlighted_bytes_count: 0,
            rendered: false,
        };
    }

    pub fn push(&mut self, token: StyledToken) {
        self.tokens.push(token);
    }

    #[must_use]
    fn render_row(&self, row: &mut [StyledToken]) -> String {
        let mut rendered = String::new();

        if row.is_empty() {
            return rendered;
        }

        if self.line_prefix.style == Style::New {
            highlight_trailing_whitespace(row);
            highlight_nonleading_tab(row);
        }
        highlight_space_between_words(row);

        // Set inverse from prefix
        let mut is_inverse = self.line_prefix.style.is_inverse();
        if is_inverse {
            rendered.push_str(INVERSE_VIDEO);
        }

        // Set line color from prefix
        let mut color = self.line_prefix.style.color();
        rendered.push_str(self.line_prefix.style.color());

        // Render prefix
        rendered.push_str(&self.line_prefix.token);

        for token in row {
            if token.style.is_inverse() && !is_inverse {
                rendered.push_str(INVERSE_VIDEO);
            }
            if is_inverse && !token.style.is_inverse() {
                rendered.push_str(NOT_INVERSE_VIDEO);
            }
            is_inverse = token.style.is_inverse();

            if token.style.color() != color {
                rendered.push_str(token.style.color());
                color = token.style.color();
            }

            rendered.push_str(&token.token);
        }

        rendered.push_str(NORMAL);

        return rendered;
    }

    #[must_use]
    pub fn render(&mut self) -> String {
        assert!(!self.rendered);
        let mut current_row: Vec<StyledToken> = Vec::new();
        let mut rendered = String::new();

        let tokens = std::mem::take(&mut self.tokens);
        for token in tokens {
            self.bytes_count += token.token.len();
            if token.style.is_inverse() {
                self.highlighted_bytes_count += token.token.len();
            }

            if token.token == "\n" {
                let rendered_row = &self.render_row(&mut current_row);
                rendered.push_str(rendered_row);
                rendered.push('\n');
                current_row.clear();
                continue;
            }

            current_row.push(token);
        }

        if !current_row.is_empty() {
            let rendered_row = &self.render_row(&mut current_row);
            rendered.push_str(rendered_row);
        }

        self.rendered = true;
        return rendered;
    }

    pub fn chars_count(&self) -> usize {
        assert!(self.rendered); // It's the rendering that does the counting
        return self.bytes_count;
    }

    pub fn highlighted_chars_count(&self) -> usize {
        assert!(self.rendered); // It's the rendering that does the counting
        return self.highlighted_bytes_count;
    }
}

fn highlight_trailing_whitespace(row: &mut [StyledToken]) {
    for token in row.iter_mut().rev() {
        if !token.is_whitespace() {
            return;
        }

        token.style = Style::Error;
    }
}

fn highlight_nonleading_tab(row: &mut [StyledToken]) {
    let mut token_iter = row.iter_mut();

    // Skip leading TABs
    loop {
        let next = token_iter.next();
        if next.is_none() {
            // Done!
            return;
        }

        let token = next.unwrap();
        if token.token != "\t" {
            // Not a TAB, this means we're out of skipping the leading TABs
            break;
        }
    }

    // Scan the rest of the line for non-leading TABs
    for token in token_iter {
        if token.token != "\t" {
            // Not a TAB, never mind
            continue;
        }

        // Non-leading TAB, mark it!
        token.style = Style::Error;
    }
}

/// Highlight single space between two highlighted words
fn highlight_space_between_words(row: &mut [StyledToken]) {
    enum FoundState {
        Nothing,
        HighlightedWord,
        WordSpace,
    };

    let mut found_state = FoundState::Nothing;
    let mut previous_token: Option<&mut StyledToken> = None;
    for token in row.iter_mut() {
        match found_state {
            FoundState::Nothing => {
                if token.style.is_inverse() && token.is_word() {
                    // Found "Monkey"
                    found_state = FoundState::HighlightedWord;
                }
            }

            FoundState::HighlightedWord => {
                if token.is_whitespace() {
                    // Found "Monkey " (note trailing space)
                    found_state = FoundState::WordSpace;
                } else if token.style.is_inverse() && token.is_word() {
                    found_state = FoundState::HighlightedWord;
                } else {
                    found_state = FoundState::Nothing;
                }
            }

            FoundState::WordSpace => {
                if token.style.is_inverse() && token.is_word() {
                    // Found "Monkey Dance"
                    if let Some(_previous_token) = previous_token {
                        _previous_token.style = _previous_token.style.inverted();
                    }

                    found_state = FoundState::HighlightedWord;
                } else {
                    found_state = FoundState::Nothing;
                }
            }
        }

        previous_token = Some(token);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(test)]
    use pretty_assertions::assert_eq;

    #[test]
    fn test_basic() {
        let mut test_me = TokenCollector::create(StyledToken {
            token: "+".to_string(),
            style: Style::New,
        });

        test_me.push(StyledToken {
            token: "hej".to_string(),
            style: Style::New,
        });

        test_me.push(StyledToken {
            token: "\n".to_string(),
            style: Style::New,
        });

        let rendered = test_me.render();
        assert_eq!(rendered, format!("{}+hej{}\n", NEW, NORMAL));
    }

    #[test]
    fn test_add_trailing_whitespace() {
        // Just a whitespace
        let mut row = [StyledToken::new(" ".to_string(), Style::New)];
        highlight_trailing_whitespace(&mut row);
        assert_eq!(row, [StyledToken::new(" ".to_string(), Style::Error)]);

        // Trailing whitespace
        let mut row = [
            StyledToken::new("x".to_string(), Style::New),
            StyledToken::new(" ".to_string(), Style::New),
        ];
        highlight_trailing_whitespace(&mut row);
        assert_eq!(
            row,
            [
                StyledToken::new("x".to_string(), Style::New),
                StyledToken::new(" ".to_string(), Style::Error),
            ]
        );

        // Leading whitespace
        let mut row = [
            StyledToken::new(" ".to_string(), Style::New),
            StyledToken::new("x".to_string(), Style::New),
        ];
        highlight_trailing_whitespace(&mut row);
        assert_eq!(
            row,
            [
                StyledToken::new(" ".to_string(), Style::New),
                StyledToken::new("x".to_string(), Style::New),
            ]
        );
    }

    #[test]
    fn test_removed_trailing_whitespace() {
        // It shouldn't be highlighted, just added ones should
        let mut test_me = TokenCollector::create(StyledToken::new("-".to_string(), Style::Old));
        test_me.push(StyledToken::new(" ".to_string(), Style::Old));
        let actual = test_me.render();

        assert_eq!(actual, format!("{}- {}", OLD, NORMAL));
    }

    #[test]
    fn test_add_nonleading_tab() {
        // Trailing TAB
        let mut row = [
            StyledToken::new("x".to_string(), Style::New),
            StyledToken::new("\t".to_string(), Style::New),
        ];
        highlight_nonleading_tab(&mut row);
        assert_eq!(
            row,
            [
                StyledToken::new("x".to_string(), Style::New),
                StyledToken::new("\t".to_string(), Style::Error),
            ]
        );

        // Middle TAB
        let mut row = [
            StyledToken::new("x".to_string(), Style::New),
            StyledToken::new("\t".to_string(), Style::New),
            StyledToken::new("y".to_string(), Style::New),
        ];
        highlight_nonleading_tab(&mut row);
        assert_eq!(
            row,
            [
                StyledToken::new("x".to_string(), Style::New),
                StyledToken::new("\t".to_string(), Style::Error),
                StyledToken::new("y".to_string(), Style::New),
            ]
        );

        // Leading TAB (don't highlight)
        let mut row = [
            StyledToken::new("\t".to_string(), Style::New),
            StyledToken::new("x".to_string(), Style::New),
        ];
        highlight_nonleading_tab(&mut row);
        assert_eq!(
            row,
            [
                StyledToken::new("\t".to_string(), Style::New),
                StyledToken::new("x".to_string(), Style::New),
            ]
        );

        // Single TAB (don't highlight because it is leading)
        let mut row = [StyledToken::new("\t".to_string(), Style::New)];
        highlight_nonleading_tab(&mut row);
        assert_eq!(row, [StyledToken::new("\t".to_string(), Style::New),]);
    }

    #[test]
    fn test_removed_nonleading_tab() {
        // It shouldn't be highlighted, just added ones should
        let mut test_me = TokenCollector::create(StyledToken::new("-".to_string(), Style::Old));
        test_me.push(StyledToken::new("x".to_string(), Style::Old));
        test_me.push(StyledToken::new("\t".to_string(), Style::Old));
        let actual = test_me.render();

        assert_eq!(actual, format!("{}-x\t{}", OLD, NORMAL));
    }

    #[test]
    fn test_highlight_space_between_words() {
        let mut row = [
            StyledToken::new("Monkey".to_string(), Style::NewInverse),
            StyledToken::new(" ".to_string(), Style::New),
            StyledToken::new("Dance".to_string(), Style::NewInverse),
        ];

        highlight_space_between_words(&mut row);

        assert_eq!(
            row,
            [
                StyledToken::new("Monkey".to_string(), Style::NewInverse),
                StyledToken::new(" ".to_string(), Style::NewInverse),
                StyledToken::new("Dance".to_string(), Style::NewInverse),
            ]
        );
    }
}
