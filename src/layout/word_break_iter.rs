use std::iter::Peekable;
use std::str::CharIndices;

pub struct WordBreakIterator<'a> {
    string: &'a str,
    iter: Peekable<CharIndices<'a>>,
}

impl<'a> Iterator for WordBreakIterator<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<&'a str> {
        let mut first_index = None;
        let mut seen_non_ws = false;
        loop {
            let result = self.iter.peek();
            let index = if let Some(&(index, _)) = result {
                index
            } else {
                self.string.len()
            };
            if first_index.is_none() {
                first_index = Some(index);
            }
            let is_whitespace_or_end = if let Some((_, ch)) = result {
                ch.is_whitespace()
            } else {
                true
            };
            if seen_non_ws && is_whitespace_or_end {
                return Some(&self.string[first_index.unwrap()..index]);
            }
            if !is_whitespace_or_end {
                seen_non_ws = true;
            }

            if result.is_none() && !seen_non_ws {
                return None;
            }

            self.iter.next();
        }
    }
}

impl<'a> WordBreakIterator<'a> {
    pub fn new(string: &'a str) -> WordBreakIterator<'a> {
        WordBreakIterator {
            string,
            iter: string.char_indices().peekable(),
        }
    }
}

#[cfg(test)]
mod test {
    use super::WordBreakIterator;

    #[test]
    fn test_iter() {
        let string = "foo bar  baz";
        let expect = vec!["foo", " bar", "  baz"];
        let result = WordBreakIterator::new(string).collect::<Vec<_>>();
        println!("{:#?}", result);
        assert!(expect.len() == result.len());
        assert!(expect[0] == result[0]);
        assert!(expect[1] == result[1]);
        assert!(expect[2] == result[2]);
    }

    #[test]
    fn trailing_ws() {
        let string = "foo  ";
        let expect = vec!["foo"];
        let result = WordBreakIterator::new(string).collect::<Vec<_>>();
        println!("{:#?}", result);
        assert!(expect.len() == result.len());
        assert!(expect[0] == result[0]);
    }

    #[test]
    fn head_ws() {
        let string = "   foo";
        let expect = vec!["   foo"];
        let result = WordBreakIterator::new(string).collect::<Vec<_>>();
        println!("{:#?}", result);
        assert!(expect.len() == result.len());
        assert!(expect[0] == result[0]);
    }
}
