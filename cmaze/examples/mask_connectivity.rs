use cmaze::{algorithms::CellMask, array::Array3D, dims::Dims3D};

fn m<const W: usize, const H: usize>(buf: [[u8; W]; H]) -> CellMask {
    CellMask::from(
        Array3D::from_buf(
            buf.iter()
                .flat_map(|row| row.iter())
                .copied()
                .collect::<Vec<_>>(),
            4,
            4,
            1,
        )
        .map(|v| v != 0),
    )
}

fn print_mask(mask: &CellMask) {
    println!("+{}+", "-".repeat(mask.size().0 as usize));
    for y in 0..mask.size().1 {
        print!("|");
        for x in 0..mask.size().0 {
            if mask[Dims3D(x, y, 0)] {
                print!("•");
            } else {
                print!(" ");
            }
        }
        print!("|");
        println!();
    }
    println!("+{}+", "-".repeat(mask.size().0 as usize));
}

fn main() {
    let a = m([[1, 0, 0, 0], [0, 0, 1, 0], [0, 1, 1, 0], [0, 0, 0, 0]]);
    print_mask(&a);
    print_mask(&a.connected(Dims3D(2, 1, 0)));
    print_mask(&a.connected(Dims3D(0, 0, 0)));

    println!("Disjoint parts:");
    for mask in a.disjoint_parts() {
        print_mask(&mask);
    }
}
