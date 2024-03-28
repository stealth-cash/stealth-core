const PRIME_Q: u128 = 199;

pub struct G1Point {
    x: u128,
    y: u128
}

pub struct G2Point {
    x: (u128, u128),
    y: (u128, u128)
}

pub fn negate(p: G1Point) -> G1Point {
    if p.x == 0 && p.y == 0 {
        return G1Point { x: 0, y: 0 };
    }
    return G1Point { x: p.x, y: PRIME_Q - (p.y % PRIME_Q) };
}

pub fn plus(p1: G1Point, p2: G1Point) -> G1Point {
    if p1.x == 0 && p1.y == 0 {
        return p2;
    }
    
    if p2.x == 0 && p2.y == 0 {
        return p1;
    }
    
    let mut slope: u128;
    if p1.x == p2.x && p1.y != p2.y {
        return G1Point { x: 0, y: 0 };
    } else if p1.x == p2.x && p1.y == p2.y {
        slope = (3 * p1.x * p1.x) % PRIME_Q;
        let temp = (2 * p1.y) % PRIME_Q;
        let temp_inv = mod_inverse(temp, PRIME_Q);
        slope = (slope * temp_inv) % PRIME_Q;
    } else {
        slope = ((p2.y - p1.y) * mod_inverse(p2.x - p1.x, PRIME_Q)) % PRIME_Q;
    }

    let x3 = (slope * slope - p1.x - p2.x) % PRIME_Q;
    let y3 = (slope * (p1.x - x3) - p1.y) % PRIME_Q;

    G1Point { x: x3, y: y3 }
}

fn mod_inverse(a: u128, m: u128) -> u128 {
    let mut mn = (m, a);
    let mut xy = (0, 1);

    while mn.1 != 0 {
        xy = (xy.1, xy.0 - (mn.0 / mn.1) * xy.1);
        mn = (mn.1, mn.0 % mn.1);
    }

    while xy.0 < 0 {
        xy.0 += m;
    }

    xy.0
}