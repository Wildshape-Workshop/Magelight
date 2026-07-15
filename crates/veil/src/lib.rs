//! magelight-veil: the fog-of-war mask format.
//!
//! A [`FogMask`] is a coarse per-block alpha grid: one byte per 8x8 pixel block,
//! where 255 means fully hidden and 0 means fully revealed. The stored byte is
//! the value the player composites directly as the fog layer's alpha, so the
//! storage format and the render format are the same number.

/// Side length in pixels of one mask block. One alpha byte covers an 8x8 square.
const BLOCK_SIZE: u32 = 8;

/// Length of the header with the information we encode
const HEADER_LENGTH: usize = 9;

/// Why decoding a byte buffer into a [`FogMask`] failed.
#[derive(Debug, PartialEq, Eq)]
pub enum DecodeError {
    /// The leading bytes were not the veil magic, so this is not a mask blob.
    BadMagic,
    /// The version byte names a format this build does not understand.
    UnsupportedVersion(u8),
    /// The buffer was not the length the header and dimensions demand.
    SizeError { expected: usize, found: usize },
}

/// A rectangle in map pixel coordinates.
pub struct Rect {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

/// A fog-of-war mask: one alpha byte per 8x8 pixel block, 255 hidden, 0 revealed.
#[derive(Debug, PartialEq, Eq)]
pub struct FogMask {
    width_blocks: usize,
    height_blocks: usize,
    blocks: Vec<u8>,
}

impl FogMask {
    /// Build a fully fogged mask covering a `width_px` by `height_px` map.
    ///
    /// Block counts round up so the final partial block still covers the map edge.
    /// Every block starts at 255 (solid fog).
    pub fn new(width_px: u32, height_px: u32) -> Self {
        let width_blocks = width_px.div_ceil(BLOCK_SIZE) as usize;
        let height_blocks = height_px.div_ceil(BLOCK_SIZE) as usize;
        let blocks = vec![255u8; width_blocks * height_blocks];

        FogMask {
            width_blocks,
            height_blocks,
            blocks,
        }
    }

    /// Reveal every block the rectangle touches (rounds outward).
    pub fn reveal(&mut self, rect: Rect) {
        let col_start = (rect.x / BLOCK_SIZE) as usize;
        let row_start = (rect.y / BLOCK_SIZE) as usize;

        let col_end = (rect.x + rect.width).div_ceil(BLOCK_SIZE) as usize;
        let row_end = (rect.y + rect.height).div_ceil(BLOCK_SIZE) as usize;

        let col_end = col_end.min(self.width_blocks);
        let row_end = row_end.min(self.height_blocks);

        for row in row_start..row_end {
            for col in col_start..col_end {
                self.blocks[row * self.width_blocks + col] = 0;
            }
        }
    }

    /// Serialize to the on-disk byte format (header + one byte per block).
    pub fn encode(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(HEADER_LENGTH + self.blocks.len());
        bytes.extend_from_slice(b"VEIL");
        bytes.push(1);
        bytes.extend_from_slice(&(self.width_blocks as u16).to_be_bytes());
        bytes.extend_from_slice(&(self.height_blocks as u16).to_be_bytes());
        bytes.extend_from_slice(&self.blocks);
        bytes
    }

    /// Parse a byte buffer back into a mask, validating the header.
    pub fn decode(bytes: &[u8]) -> Result<FogMask, DecodeError> {
        if bytes.get(..HEADER_LENGTH) == None {
            return Err(DecodeError::SizeError {
                expected: HEADER_LENGTH,
                found: bytes.len(),
            });
        }

        if &bytes[0..4] != b"VEIL" {
            return Err(DecodeError::BadMagic);
        }

        let version = bytes[4];
        if version != 1 {
            return Err(DecodeError::UnsupportedVersion(version));
        }

        let width_blocks = u16::from_be_bytes([bytes[5], bytes[6]]) as usize;
        let height_blocks = u16::from_be_bytes([bytes[7], bytes[8]]) as usize;

        let expected = HEADER_LENGTH + (width_blocks * height_blocks);

        if expected != bytes.len() {
            return Err(DecodeError::SizeError {
                expected,
                found: bytes.len(),
            });
        }

        let blocks = bytes[HEADER_LENGTH..].to_vec();

        Ok(FogMask {
            width_blocks,
            height_blocks,
            blocks,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_builds_a_fully_fogged_grid() {
        // 2401px wide must round up to 301 blocks so the last 1px strip is still
        // covered; 16px tall is exactly 2 blocks.
        let mask = FogMask::new(2401, 16);

        assert_eq!(mask.width_blocks, 301);
        assert_eq!(mask.height_blocks, 2);
        assert_eq!(mask.blocks.len(), 301 * 2);
        assert!(
            mask.blocks.iter().all(|&b| b == 255),
            "every block starts hidden"
        );
    }

    #[test]
    fn reveal_rounds_outward_to_every_touched_block() {
        // 32x16 px -> a 4x2 block grid (row 0 = blocks[0..4], row 1 = blocks[4..8]).
        let mut mask = FogMask::new(32, 16);

        // Pixels x:[9,17) straddle the boundary between block col 1 and col 2,
        // so outward rounding must clear BOTH. y:[0,1) touches only row 0.
        mask.reveal(Rect {
            x: 9,
            y: 0,
            width: 8,
            height: 1,
        });

        assert_eq!(mask.blocks[1], 0, "col 1 touched");
        assert_eq!(mask.blocks[2], 0, "col 2 touched");
        assert_eq!(mask.blocks[0], 255, "col 0 untouched");
        assert_eq!(mask.blocks[3], 255, "col 3 untouched");
        assert!(
            mask.blocks[4..].iter().all(|&b| b == 255),
            "row 1 untouched"
        );
    }

    #[test]
    fn encode_lays_out_header_then_row_major_payload() {
        // 24x16 px -> a 3x2 block grid (asymmetric on purpose: a width/height
        // swap or a transpose would survive a square grid but not this one).
        let mut mask = FogMask::new(24, 16);

        // Reveal only the top-left block so the payload is not all-255 and the
        // row-major order is observable: blocks[0] must be the first payload byte.
        mask.reveal(Rect {
            x: 0,
            y: 0,
            width: 8,
            height: 1,
        });

        let bytes = mask.encode();

        #[rustfmt::skip]
        let expected = vec![
            86, 69, 73, 76, // magic b"VEIL"
            1,              // version
            0, 3,           // width_blocks = 3 (u16 big-endian)
            0, 2,           // height_blocks = 2 (u16 big-endian)
            0, 255, 255,    // payload row 0: top-left revealed, rest hidden
            255, 255, 255,  // payload row 1: all hidden
        ];

        assert_eq!(bytes, expected);
    }

    #[test]
    fn decode_round_trips_an_encoded_mask() {
        // The headline guarantee: encode then decode yields an equal mask.
        let mut mask = FogMask::new(24, 16);
        mask.reveal(Rect {
            x: 0,
            y: 0,
            width: 8,
            height: 1,
        });

        let bytes = mask.encode();

        assert_eq!(FogMask::decode(&bytes), Ok(mask));
    }

    #[test]
    fn decode_rejects_a_foreign_blob() {
        // A valid-length buffer whose first bytes are not the magic is not ours.
        let mut bytes = FogMask::new(24, 16).encode();
        bytes[0] = b'X';

        assert_eq!(FogMask::decode(&bytes), Err(DecodeError::BadMagic));
    }

    #[test]
    fn decode_rejects_an_unknown_version() {
        // A future format must be refused, not misread, and report which version.
        let mut bytes = FogMask::new(24, 16).encode();
        bytes[4] = 2;

        assert_eq!(
            FogMask::decode(&bytes),
            Err(DecodeError::UnsupportedVersion(2))
        );
    }

    #[test]
    fn decode_rejects_a_truncated_buffer() {
        // A 3x2 grid demands HEADER_LENGTH + 6 = 15 bytes; one short must fail with the
        // exact counts, not panic by indexing past the slice.
        let mut bytes = FogMask::new(24, 16).encode();
        bytes.pop();

        assert_eq!(
            FogMask::decode(&bytes),
            Err(DecodeError::SizeError {
                expected: 15,
                found: 14
            })
        );
    }

    #[test]
    fn reveal_clamps_a_rect_past_the_map_edge() {
        // A brush dragged off the map edge must clamp to the grid, not index
        // past the buffer and panic.
        let mut mask = FogMask::new(32, 16);

        mask.reveal(Rect {
            x: 24,
            y: 8,
            width: 999,
            height: 999,
        });

        // Only the bottom-right block (row 1, col 3 = index 7) is in range.
        assert_eq!(mask.blocks[7], 0, "bottom-right block revealed");
    }
}
