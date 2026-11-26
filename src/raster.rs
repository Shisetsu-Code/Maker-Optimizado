//! Bounded raster preflight, before decoding in the browser.
//! This checks container headers and dimensions, not compressed pixel integrity.
const PNG: &[u8] = b"\x89PNG\r\n\x1a\n";
const MAX_UPLOAD_BYTES: usize = 8 * 1024 * 1024;
const MAX_UPLOAD_PIXELS: u64 = 40_000_000;
const MAX_PROJECT_SIDE: u32 = 4096;
const MAX_PROJECT_PIXELS: u64 = 16_000_000;

fn be32(b: &[u8]) -> Option<u32> {
    Some(u32::from_be_bytes(b.try_into().ok()?))
}
fn le32(b: &[u8]) -> Option<u32> {
    Some(u32::from_le_bytes(b.try_into().ok()?))
}
fn le24(b: &[u8]) -> Option<u32> {
    Some(u32::from(b[0]) | u32::from(b[1]) << 8 | u32::from(b[2]) << 16)
}
fn png(data: &[u8]) -> Option<(u32, u32)> {
    if !data.starts_with(PNG) || data.get(8..16)? != b"\0\0\0\rIHDR" {
        return None;
    }
    let size = (be32(data.get(16..20)?)?, be32(data.get(20..24)?)?);
    let mut pos = 8;
    let mut pixels = false;
    while pos < data.len() {
        let header = data.get(pos..pos.checked_add(8)?)?;
        let len = usize::try_from(be32(&header[..4])?).ok()?;
        let end = pos.checked_add(12)?.checked_add(len)?;
        if end > data.len() {
            return None;
        }
        match &header[4..8] {
            b"acTL" | b"fcTL" | b"fdAT" => return None,
            b"IDAT" => pixels = true,
            b"IEND" => return (len == 0 && pixels && end == data.len()).then_some(size),
            _ => {}
        }
        pos = end;
    }
    None
}
fn jpeg(data: &[u8]) -> Option<(u32, u32)> {
    if !data.starts_with(b"\xff\xd8") {
        return None;
    }
    let mut pos = 2;
    while pos < data.len() {
        if *data.get(pos)? != 0xff {
            return None;
        }
        while data.get(pos) == Some(&0xff) {
            pos += 1;
        }
        let marker = *data.get(pos)?;
        pos += 1;
        if matches!(marker, 0x00 | 0xd8 | 0xd9 | 0xda | 0xd0..=0xd7) {
            return None;
        }
        if marker == 0x01 {
            continue;
        }
        let len = usize::from(u16::from_be_bytes(
            data.get(pos..pos.checked_add(2)?)?.try_into().ok()?,
        ));
        if len < 2 {
            return None;
        }
        let end = pos.checked_add(len)?;
        let part = data.get(pos..end)?;
        if matches!(marker, 0xc0..=0xc3 | 0xc5..=0xc7 | 0xc9..=0xcb | 0xcd..=0xcf) {
            if len < 8 {
                return None;
            }
            let h = u16::from_be_bytes(part[3..5].try_into().ok()?);
            let w = u16::from_be_bytes(part[5..7].try_into().ok()?);
            return Some((u32::from(w), u32::from(h)));
        }
        pos = end;
    }
    None
}
fn webp(data: &[u8]) -> Option<(u32, u32)> {
    if data.get(..4)? != b"RIFF"
        || data.get(8..12)? != b"WEBP"
        || usize::try_from(le32(data.get(4..8)?)?).ok()?.checked_add(8)? != data.len()
    {
        return None;
    }
    let mut pos = 12;
    let mut canvas = None;
    let mut frame = None;
    while pos < data.len() {
        let header = data.get(pos..pos.checked_add(8)?)?;
        let len = usize::try_from(le32(&header[4..8])?).ok()?;
        let start = pos.checked_add(8)?;
        let end = start.checked_add(len)?;
        let part = data.get(start..end)?;
        let size = match &header[..4] {
            b"ANIM" | b"ANMF" => return None,
            b"VP8X" => {
                if canvas.is_some() || pos != 12 || len != 10 || part[0] & 2 != 0 {
                    return None;
                }
                canvas = Some((le24(&part[4..7])? + 1, le24(&part[7..10])? + 1));
                None
            }
            b"VP8 " => {
                if len < 10 || part[0] & 1 != 0 || part[3..6] != [0x9d, 0x01, 0x2a] {
                    return None;
                }
                let w = u16::from_le_bytes(part[6..8].try_into().ok()?) & 0x3fff;
                let h = u16::from_le_bytes(part[8..10].try_into().ok()?) & 0x3fff;
                Some((u32::from(w), u32::from(h)))
            }
            b"VP8L" => {
                if len < 5 || part[0] != 0x2f {
                    return None;
                }
                let bits = le32(&part[1..5])?;
                if bits >> 29 != 0 {
                    return None;
                }
                Some(((bits & 0x3fff) + 1, ((bits >> 14) & 0x3fff) + 1))
            }
            _ => None,
        };
        if let Some(size) = size {
            if frame.replace(size).is_some() {
                return None;
            }
        }
        pos = end.checked_add(len % 2)?;
        if pos > data.len() {
            return None;
        }
    }
    let size = frame?;
    if canvas.is_some_and(|c| c != size) {
        return None;
    }
    Some(size)
}

fn dimensions(data: &[u8], mime: &str) -> Option<(u32, u32)> {
    match mime {
        "image/png" => png(data),
        "image/jpeg" => jpeg(data),
        "image/webp" => webp(data),
        _ => None,
    }
}
fn within(data: &[u8], mime: &str, side: u32, pixels: u64) -> bool {
    dimensions(data, mime).is_some_and(|(w, h)| {
        w > 0 && h > 0 && w <= side && h <= side && u64::from(w) * u64::from(h) <= pixels
    })
}
pub fn valid_upload(data: &[u8], mime: &str) -> bool {
    data.len() <= MAX_UPLOAD_BYTES && within(data, mime, 16_384, MAX_UPLOAD_PIXELS)
}

fn sextet(b: u8) -> Option<u8> {
    Some(match b {
        b'A'..=b'Z' => b - b'A',
        b'a'..=b'z' => b - b'a' + 26,
        b'0'..=b'9' => b - b'0' + 52,
        b'+' => 62,
        b'/' => 63,
        _ => return None,
    })
}
fn decode(data: &str) -> Option<Vec<u8>> {
    let bytes = data.as_bytes();
    if bytes.is_empty() || !bytes.len().is_multiple_of(4) {
        return None;
    }
    let mut out = Vec::with_capacity(bytes.len() / 4 * 3);
    let count = bytes.len() / 4;
    for (i, c) in bytes.chunks_exact(4).enumerate() {
        let a = sextet(c[0])?;
        let b = sextet(c[1])?;
        out.push(a << 2 | b >> 4);
        if c[2] == b'=' {
            if i + 1 != count || c[3] != b'=' || b & 15 != 0 {
                return None;
            }
            break;
        }
        let third = sextet(c[2])?;
        out.push(b << 4 | third >> 2);
        if c[3] == b'=' {
            if i + 1 != count || third & 3 != 0 {
                return None;
            }
            break;
        }
        out.push(third << 6 | sextet(c[3])?);
    }
    Some(out)
}

pub fn valid_data_url(value: &str, max: usize) -> bool {
    if value.len() > max {
        return false;
    }
    let Some((header, payload)) = value.split_once(',') else {
        return false;
    };
    let Some(mime) = header
        .strip_prefix("data:")
        .and_then(|h| h.strip_suffix(";base64"))
    else {
        return false;
    };
    if !matches!(mime, "image/png" | "image/jpeg" | "image/webp") {
        return false;
    }
    decode(payload).is_some_and(|data| {
        within(&data, mime, MAX_PROJECT_SIDE, MAX_PROJECT_PIXELS)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const PIXEL: &str = "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mP8/x8AAwMCAO+aX1sAAAAASUVORK5CYII=";

    #[test]
    fn rejects_type_mismatch_and_oversized_png_before_decode() {
        let data = decode(PIXEL).unwrap();
        assert!(valid_upload(&data, "image/png"));
        assert!(!valid_upload(&data, "image/webp"));
        let mut huge = data.clone();
        huge[16..20].copy_from_slice(&100_000u32.to_be_bytes());
        assert!(!valid_upload(&huge, "image/png"));
        for n in 0..data.len() {
            assert!(!valid_upload(&data[..n], "image/png"));
        }
    }

    #[test]
    fn malformed_base64_and_non_raster_are_rejected() {
        assert!(valid_data_url(
            &format!("data:image/png;base64,{PIXEL}"),
            600_000
        ));
        for value in [
            "data:image/png;base64,YWJj",
            "data:image/svg+xml;base64,YWJj",
            "data:image/png;base64,AA=A",
            "data:image/png;base64,A===",
            "data:image/png;base64,AB==",
        ] {
            assert!(!valid_data_url(value, 600_000));
        }
    }
}
