/// Shape-aware glyph selection for the 3D projected terminal view.
///
/// Core idea (adapted from Alex Harri's "ASCII characters are not pixels"):
/// Each ASCII character has a visual shape that occupies specific regions of its
/// cell. We precompute a 4-value descriptor for each glyph (coverage in TL, TR,
/// BL, BR quadrants). At render time, we compute the same descriptor from
/// projected voxel geometry and pick the nearest-match glyph.

/// A glyph with its precomputed shape descriptor.
struct GlyphEntry {
    ch: char,
    /// Coverage in [top-left, top-right, bottom-left, bottom-right], each 0.0–1.0.
    coverage: [f32; 4],
}

/// Atlas of curated glyphs with precomputed shape descriptors.
pub struct GlyphAtlas {
    glyphs: Vec<GlyphEntry>,
}

impl GlyphAtlas {
    /// Build the curated glyph atlas.
    ///
    /// Descriptors are hand-tuned to match the visual weight of each character
    /// in a typical monospace terminal font. These don't need to be precise —
    /// they capture relative shape differences.
    pub fn new() -> Self {
        let glyphs = vec![
            // Empty / very light
            GlyphEntry { ch: ' ', coverage: [0.0, 0.0, 0.0, 0.0] },
            GlyphEntry { ch: '.', coverage: [0.0, 0.0, 0.1, 0.1] },
            GlyphEntry { ch: ',', coverage: [0.0, 0.0, 0.1, 0.0] },
            GlyphEntry { ch: '\'', coverage: [0.1, 0.0, 0.0, 0.0] },
            GlyphEntry { ch: '`', coverage: [0.1, 0.0, 0.0, 0.0] },

            // Horizontal / top-surface
            GlyphEntry { ch: '_', coverage: [0.0, 0.0, 0.9, 0.9] },
            GlyphEntry { ch: '-', coverage: [0.3, 0.3, 0.5, 0.5] },
            GlyphEntry { ch: '=', coverage: [0.5, 0.5, 0.5, 0.5] },
            GlyphEntry { ch: '~', coverage: [0.3, 0.4, 0.4, 0.3] },

            // Vertical / walls
            GlyphEntry { ch: '|', coverage: [0.4, 0.4, 0.4, 0.4] },
            GlyphEntry { ch: '!', coverage: [0.5, 0.5, 0.1, 0.1] },

            // Diagonal / contour edges
            GlyphEntry { ch: '/', coverage: [0.0, 0.7, 0.7, 0.0] },
            GlyphEntry { ch: '\\', coverage: [0.7, 0.0, 0.0, 0.7] },

            // Half-fills
            GlyphEntry { ch: '"', coverage: [0.5, 0.5, 0.0, 0.0] },

            // Dense / full fill
            GlyphEntry { ch: '#', coverage: [0.8, 0.8, 0.8, 0.8] },
            GlyphEntry { ch: '%', coverage: [0.7, 0.7, 0.7, 0.7] },
            GlyphEntry { ch: '@', coverage: [0.9, 0.9, 0.9, 0.9] },
            GlyphEntry { ch: '*', coverage: [0.5, 0.5, 0.5, 0.5] },

            // Partial fills — left/right/top/bottom heavy
            GlyphEntry { ch: '[', coverage: [0.7, 0.2, 0.7, 0.2] },
            GlyphEntry { ch: ']', coverage: [0.2, 0.7, 0.2, 0.7] },
            GlyphEntry { ch: '^', coverage: [0.5, 0.5, 0.2, 0.2] },

            // Corner pieces
            GlyphEntry { ch: 'r', coverage: [0.0, 0.0, 0.6, 0.6] },
        ];

        Self { glyphs }
    }

    /// Find the glyph whose shape descriptor is closest to the given coverage.
    ///
    /// Uses squared Euclidean distance — no sqrt needed for comparison.
    pub fn best_match(&self, coverage: [f32; 4]) -> char {
        let mut best_ch = ' ';
        let mut best_dist = f32::MAX;

        for entry in &self.glyphs {
            let dist: f32 = (0..4)
                .map(|i| {
                    let d = coverage[i] - entry.coverage[i];
                    d * d
                })
                .sum();

            if dist < best_dist {
                best_dist = dist;
                best_ch = entry.ch;
            }
        }

        best_ch
    }
}
