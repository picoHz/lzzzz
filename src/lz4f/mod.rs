mod params;
mod sys;

use std::io::Write;
use sys::Preferences;

pub struct CompressorBuilder {
    prefs: Preferences,
}

impl CompressorBuilder {
    pub fn new() -> Self {
        Self {
            prefs: Default::default(),
        }
    }

    pub fn build_writer<'a, W: Write>(self, writer: &'a mut W) -> WriteCompressor<'a, W> {
        WriteCompressor { prefs: self.prefs, writer }
    }
}

pub struct WriteCompressor<'a, W> {
    prefs: Preferences,
    writer: &'a mut W,
}
