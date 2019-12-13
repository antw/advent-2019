use std::fs;

/// Given a layer, multiplies the number of ones and the number of twos.
fn ones_times_twos(layer: &Vec<u8>) -> i64 {
    let mut ones = 0;
    let mut twos = 0;

    for pixel in layer {
        match *pixel {
            1u8 => ones += 1,
            2u8 => twos += 1,
            _ => {}
        }
    }

    ones * twos
}

/// Composes the layers of the image, from the top-most layer to the bottom, into a final image.
/// Pixels with a value of 0 and 1 are opaque, while 2 is treated as transparent. If a higher layer
/// has a transparent value, then an opaque value from a lower layer should show through. An opaque
/// value in a higher layer will obscure any value from a lower layer.
fn compose_image_from_layers(pixels: &Vec<u8>, pixels_per_layer: usize) -> Vec<u8> {
    let mut image = vec![2; pixels_per_layer];
    let layers = pixels.chunks(pixels_per_layer);

    for layer in layers {
        for (index, pixel) in layer.iter().enumerate() {
            if image[index] == 2 {
                image[index] = *pixel;
            }
        }
    }

    image
}

/// Reads pixel data from the file at the given `path`.
fn read_data(path: &str) -> Result<Vec<u8>, std::io::Error> {
    let raw_content = fs::read_to_string(path)?;
    let raw_content = raw_content.trim();

    let mut data = Vec::new();

    for pixel in raw_content.chars() {
        data.push(pixel as u8 - 48);
    }

    Ok(data)
}

/// Receives pixel data and the number of pixels per layer, finds the layer with the least zeros and
/// multiplies the number of ones by twos in that layer. Returns None if the pixel data is empty.
fn part_one(pixels: &Vec<u8>, pixels_per_layer: usize) -> Option<i64> {
    let layers = pixels.chunks(pixels_per_layer);

    let least_zeros = layers.min_by_key(|layer| {
        let zeros: Vec<&u8> = layer.iter().filter(|pixel| **pixel == 0u8).collect();
        zeros.len()
    });

    match least_zeros {
        Some(layer) => Some(ones_times_twos(&layer.to_vec())),
        None => None,
    }
}

/// Composes the individual layers of an image, by overlaying the top-most layer over the layer
/// beneath it, and so on.
///
/// A pixel value of 0 is black, 1 is white, and 2 is transparent.
///
/// Returns the final "image" as a string where each white character is an "o" and each black
/// character is left as whitespace.
fn part_two(pixels: &Vec<u8>, pixels_per_row: usize, pixels_per_layer: usize) -> String {
    let image = compose_image_from_layers(pixels, pixels_per_layer);
    let rows = image.chunks(pixels_per_row);

    let mut rendered =
        String::with_capacity(pixels_per_layer * 2 + pixels_per_layer / pixels_per_row + 1);

    for row in rows {
        for pixel in row {
            match *pixel {
                1u8 => rendered.push('o'),
                _ => rendered.push(' '),
            }

            rendered.push(' ');
        }

        rendered.push('\n')
    }

    rendered
}

fn main() -> Result<(), std::io::Error> {
    let pixels = read_data("data/image.txt")?;
    let pixels_per_layer = 25 * 6;

    match part_one(&pixels, pixels_per_layer) {
        Some(result) => println!("Part one: {}", result),
        None => println!("Part one: No matching layer found."),
    };

    println!("Part two:");
    println!("{}", part_two(&pixels, 25, pixels_per_layer));

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_part_two_example() {
        let data = vec![0, 2, 2, 2, 1, 1, 2, 2, 2, 2, 1, 2, 0, 0, 0, 0];
        let image = compose_image_from_layers(&data, 4);

        assert_eq!(image, vec![0, 1, 1, 0]);
    }
}
