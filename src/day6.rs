fn doit(t: usize, d: usize) -> (f64, f64) {
    let (tf, df) = (t as f64, d as f64);
    return (
        -((tf.powf(2.0) - 4.0 * df).sqrt() - tf) / 2.0,
        ((tf.powf(2.0) - 4.0 * df).sqrt() + tf) / 2.0,
    );
}

fn doit2(t: usize, d: usize) -> usize {
    let (a, b) = doit(t, d);
    let above = (a + 1.0).floor() as usize;
    let below = (b - 1.0).ceil() as usize;
    let result = below - above + 1;
    result as usize
}

pub fn solve() {
    let a = doit2(49, 263);
    let b = doit2(97, 1532);
    let c = doit2(94, 1378);
    let d = doit2(94, 1851);
    let pt1 = a * b * c * d;

    let pt2 = doit2(49979494, 263153213781851);
    println!("Day 6");
    println!("Part 1: {pt1}");
    println!("Part 2: {pt2}");
}
