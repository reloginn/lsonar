use crate::{AstNode, Result, engine::find_first_match};

pub struct GMatchIterator {
    pub(super) bytes: Vec<u8>,
    pub(super) pattern_ast: Vec<AstNode>,
    pub(super) current_pos: usize,
    pub(super) is_empty_pattern: bool,
}

impl Iterator for GMatchIterator {
    type Item = Result<Vec<String>>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_pos > self.bytes.len() {
            return None;
        }

        if self.is_empty_pattern {
            let result = Some(Ok(vec![String::new()]));

            self.current_pos += 1;

            return result;
        }

        match find_first_match(&self.pattern_ast, &self.bytes, self.current_pos) {
            Ok(Some((match_range, captures))) => {
                if match_range.start == match_range.end {
                    self.current_pos = match_range.end + 1;
                    if self.current_pos > self.bytes.len() {
                        return None;
                    }
                } else {
                    self.current_pos = match_range.end;
                }

                let result: Vec<String> = if captures.iter().any(|c| c.is_some()) {
                    captures
                        .into_iter()
                        .filter_map(|maybe_range| {
                            maybe_range.map(|range| {
                                String::from_utf8_lossy(&self.bytes[range]).into_owned()
                            })
                        })
                        .collect()
                } else {
                    vec![
                        String::from_utf8_lossy(&self.bytes[match_range.start..match_range.end])
                            .into_owned(),
                    ]
                };

                Some(Ok(result))
            }
            Ok(None) => None,
            Err(e) => Some(Err(e)),
        }
    }
}
