use crate::cursor::{Codepoint, Cursor, Iter};

pub struct Tail<'a> {
    text: &'a str,
    iter: Codepoint<'a>,
}

fn is_word_tail(cursor: Cursor, text: &str) -> bool {
    let mut chars = text[cursor.offset..].chars();

    let p = chars.next();
    let q = chars.next();

    let res = !p.map_or(true, char::is_whitespace) && q.map_or(true, char::is_whitespace);

    res
}

impl<'a> Iter<'a> for Tail<'a> {
    fn new(cursor: Cursor, text: &'a str) -> Self {
        Self { text, iter: Codepoint::new(cursor, text) }
    }

    fn at(&self) -> Self::Item {
        self.iter.at()
    }
}

impl Iterator for Tail<'_> {
    type Item = Cursor;

    fn next(&mut self) -> Option<Self::Item> {
        let text = self.text;
        eprintln!("hi");
        self.iter.find(|&p| is_word_tail(p, text))
    }
}

impl DoubleEndedIterator for Tail<'_> {
    fn next_back(&mut self) -> Option<Self::Item> {
        let text = self.text;
        self.iter.rfind(|&p| is_word_tail(p, text))
    }
}

#[cfg(test)]
mod tests {
    use super::Tail;
    use crate::Cursor;

    static LOREM: &str = include_str!("../../assets/lorem.txt");

    #[test]
    fn test_iter() {
        let codepoints = Cursor::origin().iter::<Tail>(LOREM).collect::<Vec<_>>();

        assert_eq!(codepoints, vec![]);
    }
}
