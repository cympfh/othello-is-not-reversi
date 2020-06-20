pub fn uabs(x: usize, y: usize) -> usize {
    if x < y {
        y - x
    } else {
        x - y
    }
}

pub fn mdist((x, y): (usize, usize), (u, v): (usize, usize)) -> usize {
    uabs(x, u) + uabs(y, v)
}
