use std::env;
use std::hash::DefaultHasher;

use std::hash::{Hash as _, Hasher as _};

use cmaze::array::Array3D;
use cmaze::{
    dims::Dims3D,
    gameboard::algorithms::{Generator, Random},
};

use rand::{thread_rng, Rng as _, SeedableRng as _};

fn main() {
    let args = env::args()
        .skip(1)
        .take(4)
        .map(|s| s.parse())
        .collect::<Result<Vec<i128>, _>>()
        .expect("Expected 3 integers");

    assert!(
        args.len() == 3 || args.len() == 4,
        "Expected 3 or 4 integers"
    );

    let input_seed = args.get(3).copied().map(|seed| seed as u64);
    let seed = input_seed.unwrap_or_else(|| thread_rng().gen());
    let mut rng = Random::seed_from_u64(seed);

    if input_seed.is_none() {
        println!("Seed: {}", seed);
    }

    let base_hash = rng.gen::<u64>();

    let size = Dims3D(args[0] as i32, args[1] as i32, 1);
    let point_count = args[2] as u8;

    let points = Generator::randon_points(size, point_count, &mut rng);
    let groups = Generator::split_groups(points, size, &mut rng);

    let mut mask = Array3D::new_dims(false, size).unwrap();
    let (_, borders) = Generator::build_region_graph(&groups);
    for border in borders {
        mask[border.0.0] = true;
        mask[border.1.0] = true;
    }

    let groups = groups.mask(&mask).unwrap();

    let groups = groups.layer(0).unwrap();
    for cell in 0..groups.len() {
        let group = groups[groups.idx_to_dim(cell).unwrap()];

        if group.is_none() {
            print!(" ");
        } else {
            let mut hasher = DefaultHasher::new();
            group.hash(&mut hasher);

            let hash = hasher.finish().wrapping_add(base_hash);
            let (r, g, b) = ((hash >> 16) as u8, (hash >> 8) as u8, hash as u8);

            print!("\x1b[48;2;{r};{g};{b}m \x1b[0m");
        }
        if cell as i32 % size.0 == size.0 - 1 {
            println!();
        }
    }
}
