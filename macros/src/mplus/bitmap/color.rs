pub fn quantize(image_data: &[u8], image_width: u32, bit_depth: u8) -> Vec<u8> {
    let pixels_per_byte = match bit_depth {
        1 => 8,
        2 => 4,
        4 => 2,
        8 => return image_data.to_vec(),
        x => panic!("expected one of: `1`, `2`, `4`, `8`; found: `{x}`"),
    };
    let mut bytes = Vec::new();
    let mut rows = image_data.chunks_exact(image_width as usize);
    let divisor = u8::try_from(255 / (2u32.pow(bit_depth.into()) - 1))
        .expect("expected one of: `1`, `17`, `85`, `255`");

    for row_data in rows.by_ref() {
        let mut chunks = row_data.chunks_exact(pixels_per_byte as usize);
        for chunk in chunks.by_ref() {
            bytes.push(match *chunk {
                [a] => downsample(a, divisor),
                [b, a] => downsample(a, divisor) | (downsample(b, divisor) << 4),
                [d, c, b, a] => {
                    downsample(a, divisor)
                        | (downsample(b, divisor) << 2)
                        | (downsample(c, divisor) << 4)
                        | (downsample(d, divisor) << 6)
                }
                [h, g, f, e, d, c, b, a] => {
                    downsample(a, divisor)
                        | (downsample(b, divisor) << 1)
                        | (downsample(c, divisor) << 2)
                        | (downsample(d, divisor) << 3)
                        | (downsample(e, divisor) << 4)
                        | (downsample(f, divisor) << 5)
                        | (downsample(g, divisor) << 6)
                        | (downsample(h, divisor) << 7)
                }
                _ => panic!("expected one of: `&[u8; 1]`, `&[i8; 2]`, `&[u8; 4]`, `&[u8; 8]`"),
            });
        }

        let remainder = chunks.remainder();
        if !remainder.is_empty() {
            let remainder = remainder.iter().zip(1u8..);
            let remainder = remainder.fold(0u8, |byte, (value, factor)| {
                byte | (downsample(*value, divisor) << (8 - bit_depth * factor))
            });

            bytes.push(remainder);
        }
    }

    let [] = rows.remainder() else {
        panic!("expected no remainder");
    };

    bytes
}

const fn downsample(value: u8, divisor: u8) -> u8 {
    const SHIFT: usize = 23;
    const CONST_0_5: i32 = 1 << (SHIFT - 1);

    if divisor == 0 {
        return value;
    }

    let result = ((value as i32) << SHIFT) / divisor as i32 + CONST_0_5;

    (result >> SHIFT) as u8
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! test_downsample {
        (
            $(
                $fn_ident:ident, $value:expr, $divisor:expr, $expected:expr,
            )*
        ) => {
            $(
                #[test]
                fn $fn_ident() {
                    let result = downsample($value, $divisor);
                    assert_eq!(result, $expected);
                }
            )*
        }
    }

    test_downsample! {
        downsample_255_to_4bpp, 255, 17, 15,
        downsample_128_to_4bpp, 128, 17, 8,
        downsample_127_to_4bpp, 127, 17, 7,
        downsample_64_to_4bpp, 64, 17, 4,

        downsample_32_to_4bpp, 32, 17, 2,
        downsample_16_to_4bpp, 16, 17, 1,
        downsample_8_to_4bpp, 8, 17, 0,
        downsample_0_to_4bpp, 0, 17, 0,

        downsample_255_to_2bpp, 255, 85, 3,
        downsample_128_to_2bpp, 128, 85, 2,
        downsample_127_to_2bpp, 127, 85, 1,
        downsample_0_to_2bpp, 0, 85, 0,

        downsample_255_to_1bpp, 255, 255, 1,
        downsample_128_to_1bpp, 128, 255, 1,
        downsample_127_to_1bpp, 127, 255, 0,
        downsample_0_to_1bpp, 0, 255, 0,

        downsample_255_no_resample, 255, 1, 255,
        downsample_255_upsample, 255, 0, 255,
    }
}
