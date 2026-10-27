//! TSF Composition management for CHKey.

use windows::Win32::UI::TextServices::ITfComposition;

/// Quản lý TSF Text Composition.
#[derive(Default)]
pub struct CompositionManager {
    active_composition: Option<ITfComposition>,
}

impl CompositionManager {
    pub fn new() -> Self {
        Self {
            active_composition: None,
        }
    }

    pub fn clear(&mut self) {
        self.active_composition = None;
    }
}
