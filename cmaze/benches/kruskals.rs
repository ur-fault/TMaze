use std::sync::mpsc::channel;

use cmaze::maze::{
    Cell, CellWall, Dims3D, Maze, MazeAlgorithm, Progress, RndKruskals, StopGenerationFlag,
};
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use rand::{seq::SliceRandom, thread_rng};

const DIMS: Dims3D = Dims3D(30, 30, 10);

pub fn kruskals_floors(c: &mut Criterion) {
    let mut group = c.benchmark_group("kruskals_floors");
    group.bench_with_input("kruskals", &false, |b, par| {
        b.iter(|| {
            let (handle, _stop, _progress) =
                RndKruskals::generate(black_box(DIMS), black_box(false), *par).unwrap();

            let _ = handle.join().unwrap();
        });
    });
    group.bench_with_input("kruskals_par", &true, |b, par| {
        b.iter(|| {
            let (handle, _stop, _progress) =
                RndKruskals::generate(black_box(DIMS), black_box(true), *par).unwrap();

            let _ = handle.join().unwrap();
        });
    });

    group.finish();
}

pub fn kruskals_hashmap(c: &mut Criterion) {
    use cmaze::maze::{CellWall::*, GenerationErrorInstant, GenerationErrorThreaded};
    let mut group = c.benchmark_group("kruskals_hashmap");
    for input in [
        (10, 10, 1),
        (20, 20, 1),
        (50, 50, 1),
        (100, 100, 1),
        (10, 10, 10),
        (15, 15, 15),
        (200, 100, 1),
    ]
    .iter()
    {
        group.bench_with_input(
            BenchmarkId::new("std", format!("{:?}", input)),
            &input,
            |b, d| {
                b.iter(|| {
                    use std::collections::HashSet;

                    let size = Dims3D(d.0, d.1, d.2);
                    let (progress, _r) = channel();
                    let stopper = StopGenerationFlag::new();

                    if size.0 == 0 || size.1 == 0 || size.2 == 0 {
                        return Err(GenerationErrorThreaded::GenerationError(
                            GenerationErrorInstant::InvalidSize(size),
                        ));
                    }

                    let Dims3D(w, h, d) = size;
                    let (wu, hu, du) = (w as usize, h as usize, d as usize);
                    let cell_count = wu * hu * du;

                    let mut cells: Vec<Vec<Vec<Cell>>> = vec![vec![Vec::with_capacity(wu); hu]; du];

                    for z in 0..d {
                        for y in 0..h {
                            for x in 0..w {
                                cells[z as usize][y as usize].push(Cell::new(Dims3D(x, y, z)));
                            }
                        }
                    }

                    let wall_count = (hu * (wu - 1) + wu * (hu - 1)) * du + wu * hu * (du - 1);
                    let mut walls: Vec<(Dims3D, CellWall)> = Vec::with_capacity(wall_count);

                    for (iz, floor) in cells.iter().enumerate() {
                        for (iy, row) in floor.iter().enumerate() {
                            for ix in 0..row.len() {
                                if ix != wu - 1 {
                                    walls.push((Dims3D(ix as i32, iy as i32, iz as i32), Right));
                                }

                                if iy != hu - 1 {
                                    walls.push((Dims3D(ix as i32, iy as i32, iz as i32), Bottom));
                                }

                                if iz != du - 1 {
                                    walls.push((Dims3D(ix as i32, iy as i32, iz as i32), Up));
                                }
                            }
                        }
                    }

                    let mut sets = Vec::<HashSet<Dims3D>>::with_capacity(cell_count);
                    for iz in 0..cells.len() {
                        for iy in 0..cells[0].len() {
                            for ix in 0..cells[0][0].len() {
                                sets.push(
                                    vec![Dims3D(ix as i32, iy as i32, iz as i32)]
                                        .into_iter()
                                        .collect(),
                                );
                            }
                        }
                    }

                    let mut maze = Maze::new(cells);

                    walls.shuffle(&mut thread_rng());
                    while let Some((pos0, wall)) = walls.pop() {
                        let pos1 = pos0 + wall.to_coord();

                        let set0_i = sets.iter().position(|set| set.contains(&pos0)).unwrap();

                        if sets[set0_i].contains(&pos1) {
                            continue;
                        }

                        let set1_i = sets.iter().position(|set| set.contains(&pos1)).unwrap();

                        maze.get_cell_mut(pos0).unwrap().remove_wall(wall);
                        maze.get_cell_mut(pos1)
                            .unwrap()
                            .remove_wall(wall.reverse_wall());
                        let set0 = sets.swap_remove(set0_i);

                        let set1_i = if set1_i == sets.len() - 1 {
                            sets.len() - 1
                        } else {
                            sets.iter().position(|set| set.contains(&pos1)).unwrap()
                        };
                        sets[set1_i].extend(set0);

                        progress
                            .send(Progress {
                                done: wall_count - walls.len(),
                                from: wall_count,
                            })
                            .unwrap();

                        if stopper.is_stopped() {
                            return Err(GenerationErrorThreaded::AbortGeneration);
                        }
                    }

                    Ok(maze)
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("hashbrown", format!("{:?}", input)),
            &input,
            |b, d| {
                b.iter(|| {
                    use hashbrown::HashSet;

                    let size = Dims3D(d.0, d.1, d.2);
                    let (progress, _r) = channel();
                    let stopper = StopGenerationFlag::new();

                    if size.0 == 0 || size.1 == 0 || size.2 == 0 {
                        return Err(GenerationErrorThreaded::GenerationError(
                            GenerationErrorInstant::InvalidSize(size),
                        ));
                    }

                    let Dims3D(w, h, d) = size;
                    let (wu, hu, du) = (w as usize, h as usize, d as usize);
                    let cell_count = wu * hu * du;

                    let mut cells: Vec<Vec<Vec<Cell>>> = vec![vec![Vec::with_capacity(wu); hu]; du];

                    for z in 0..d {
                        for y in 0..h {
                            for x in 0..w {
                                cells[z as usize][y as usize].push(Cell::new(Dims3D(x, y, z)));
                            }
                        }
                    }

                    let wall_count = (hu * (wu - 1) + wu * (hu - 1)) * du + wu * hu * (du - 1);
                    let mut walls: Vec<(Dims3D, CellWall)> = Vec::with_capacity(wall_count);

                    for (iz, floor) in cells.iter().enumerate() {
                        for (iy, row) in floor.iter().enumerate() {
                            for ix in 0..row.len() {
                                if ix != wu - 1 {
                                    walls.push((Dims3D(ix as i32, iy as i32, iz as i32), Right));
                                }

                                if iy != hu - 1 {
                                    walls.push((Dims3D(ix as i32, iy as i32, iz as i32), Bottom));
                                }

                                if iz != du - 1 {
                                    walls.push((Dims3D(ix as i32, iy as i32, iz as i32), Up));
                                }
                            }
                        }
                    }

                    let mut sets = Vec::<HashSet<Dims3D>>::with_capacity(cell_count);
                    for iz in 0..cells.len() {
                        for iy in 0..cells[0].len() {
                            for ix in 0..cells[0][0].len() {
                                sets.push(
                                    vec![Dims3D(ix as i32, iy as i32, iz as i32)]
                                        .into_iter()
                                        .collect(),
                                );
                            }
                        }
                    }

                    let mut maze = Maze::new(cells);

                    walls.shuffle(&mut thread_rng());
                    while let Some((pos0, wall)) = walls.pop() {
                        let pos1 = pos0 + wall.to_coord();

                        let set0_i = sets.iter().position(|set| set.contains(&pos0)).unwrap();

                        if sets[set0_i].contains(&pos1) {
                            continue;
                        }

                        let set1_i = sets.iter().position(|set| set.contains(&pos1)).unwrap();

                        maze.get_cell_mut(pos0).unwrap().remove_wall(wall);
                        maze.get_cell_mut(pos1)
                            .unwrap()
                            .remove_wall(wall.reverse_wall());
                        let set0 = sets.swap_remove(set0_i);

                        let set1_i = if set1_i == sets.len() - 1 {
                            sets.len() - 1
                        } else {
                            sets.iter().position(|set| set.contains(&pos1)).unwrap()
                        };
                        sets[set1_i].extend(set0);

                        progress
                            .send(Progress {
                                done: wall_count - walls.len(),
                                from: wall_count,
                            })
                            .unwrap();

                        if stopper.is_stopped() {
                            return Err(GenerationErrorThreaded::AbortGeneration);
                        }
                    }

                    Ok(maze)
                });
            },
        );
    }

    group.finish();
}

criterion_group! {name = benches; config = Criterion::default().sample_size(10); targets = kruskals_floors, kruskals_hashmap}
criterion_main!(benches);
