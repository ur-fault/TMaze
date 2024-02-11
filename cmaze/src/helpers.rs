pub fn get_2d<T>(vec: &Vec<Vec<T>>, x: usize, y: usize) -> Option<&T> {
    vec.get(y).and_then(|row| row.get(x))
}

pub fn get_2d_mut<T>(vec: &mut Vec<Vec<T>>, x: usize, y: usize) -> Option<&mut T> {
    vec.get_mut(y).and_then(|row| row.get_mut(x))
}

pub fn get_3d<T>(vec: &Vec<Vec<Vec<T>>>, x: usize, y: usize, z: usize) -> Option<&T> {
    vec.get(z)
        .and_then(|floor| floor.get(y))
        .and_then(|row| row.get(x))
}

pub fn get_3d_mut<T>(vec: &mut Vec<Vec<Vec<T>>>, x: usize, y: usize, z: usize) -> Option<&mut T> {
    vec.get_mut(z)
        .and_then(|floor| floor.get_mut(y))
        .and_then(|row| row.get_mut(x))
}
