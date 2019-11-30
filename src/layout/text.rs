use super::{Glyph, TextFragment};
use crate::util::equal_rc::EqualRc;
use crate::util::word_break_iter;
use euclid::point2;
use moxie::*;
use skribo::{FontCollection, LayoutSession, TextStyle};
use std::cell::RefCell;

pub struct TextLayoutInfo {
    session: RefCell<LayoutSession<String>>,
}

pub struct FilledLine {
    pub width: f32,
    pub height: f32,
    pub ascender: f32,
    pub fragments: Vec<TextFragment>,
    pub text_size: f32,
}

pub struct TextState<'a> {
    offset: usize,
    layout: &'a TextLayoutInfo,
}

impl TextLayoutInfo {
    #[topo::from_env(collection: &EqualRc<FontCollection>)]
    pub fn new(text: String, size: f32) -> Self {
        TextLayoutInfo {
            session: RefCell::new(LayoutSession::create(text, &TextStyle { size }, collection)),
        }
    }
}

impl<'a> TextState<'a> {
    pub fn new(layout: &'a TextLayoutInfo) -> TextState<'a> {
        TextState { offset: 0, layout }
    }

    fn create_fragments(
        &self,
        session: &mut LayoutSession<String>,
        start: usize,
        end: usize,
    ) -> Vec<TextFragment> {
        let mut fragments = vec![];
        let size = session.style().size;
        for run in session.iter_substr(start..end) {
            let font = run.font().to_owned();
            let metrics = font.font.metrics();
            let units_per_px = metrics.units_per_em as f32 / size;
            let baseline_offset = metrics.ascent / units_per_px;

            let glyphs = run
                .glyphs()
                .map(|glyph| Glyph {
                    index: glyph.glyph_id,
                    offset: point2(glyph.offset.x, glyph.offset.y + baseline_offset),
                })
                .collect();
            fragments.push(TextFragment { font, glyphs });
        }

        fragments
    }

    pub fn finished(&self) -> bool {
        self.offset == self.layout.session.borrow().text().len()
    }

    pub fn fill_line(&mut self, width: f32, is_new_line: bool) -> Option<FilledLine> {
        let mut session = self.layout.session.borrow_mut();

        let mut x = 0.0;
        let mut height = 0.0f32;
        let mut ascender = 0.0f32;
        let mut last_word_end = 0;
        let mut last_word_x = 0.0;
        let mut last_word_height = 0.0;
        let mut last_word_ascender = 0.0;
        let size = session.style().size;
        let text = session.text().to_owned();

        if is_new_line {
            let trimmed = text[self.offset..].trim_start();
            self.offset = trimmed.as_ptr() as usize - text.as_ptr() as usize;
        };

        for word in word_break_iter::WordBreakIterator::new(&text[self.offset..]) {
            let start = word.as_ptr() as usize - text.as_ptr() as usize;
            let end = start + word.len();

            for run in session.iter_substr(start..end) {
                let font = run.font();
                let metrics = font.font.metrics();
                let units_per_px = metrics.units_per_em as f32 / size;
                let line_height = (metrics.ascent - metrics.descent) / units_per_px;
                let line_ascent = metrics.ascent / units_per_px;

                for glyph in run.glyphs() {
                    let new_x = glyph.offset.x
                        + font.font.advance(glyph.glyph_id).unwrap().x / units_per_px;

                    if last_word_x + new_x > width {
                        let start = self.offset;
                        self.offset += last_word_end;
                        if last_word_end > 0 {
                            // soft break
                            return Some(FilledLine {
                                fragments: self.create_fragments(&mut *session, start, self.offset),
                                width: last_word_x,
                                height: last_word_height,
                                ascender: last_word_ascender,
                                text_size: size,
                            });
                        } else {
                            // todo: force progress by hard breaking if is_new_line is true
                            return None;
                        }
                    }
                    x = last_word_x + new_x;
                    height = height.max(line_height);
                    ascender = ascender.max(line_ascent);
                }
            }
            last_word_end = end - self.offset;
            last_word_x = x;
            last_word_height = height;
            last_word_ascender = ascender;
        }

        let start = self.offset;
        self.offset += last_word_end;
        if last_word_end > 0 {
            Some(FilledLine {
                fragments: self.create_fragments(&mut *session, start, self.offset),
                width: last_word_x,
                height: last_word_height,
                ascender: last_word_ascender,
                text_size: size,
            })
        } else {
            None
        }
    }
}
