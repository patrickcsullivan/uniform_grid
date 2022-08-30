pub fn min_f32(x: f32, y: f32) -> f32 {
    match x.partial_cmp(&y) {
        Some(std::cmp::Ordering::Less) => x,
        Some(std::cmp::Ordering::Greater) => y,
        Some(std::cmp::Ordering::Equal) => x,
        None => panic!("Cannot compare NaN."),
    }
}

pub fn max_f32(x: f32, y: f32) -> f32 {
    match x.partial_cmp(&y) {
        Some(std::cmp::Ordering::Less) => y,
        Some(std::cmp::Ordering::Greater) => x,
        Some(std::cmp::Ordering::Equal) => x,
        None => panic!("Cannot compare NaN."),
    }
}
