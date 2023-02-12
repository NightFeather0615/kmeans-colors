use rand::Rng;
use rayon::prelude::*;

use crate::kmeans::{Calculate, Hamerly, HamerlyCentroids, HamerlyPoint};

impl Calculate for [f32; 3] {
    fn get_closest_centroid(rgb: &[[f32; 3]], centroids: &[[f32; 3]], indices: &mut Vec<u8>) {
        rgb.into_iter().for_each(|&color: &[f32; 3]| {
            let index: u8 = centroids
                .into_par_iter()
                .map(|c: &[f32; 3]| Self::difference(&color, c))
                .enumerate()
                .reduce(
                    || (0, f32::INFINITY),
                    |(i1, d1): (usize, f32), (i2, d2): (usize, f32)| if d1 < d2 { (i1, d1) } else { (i2, d2) },
                )
                .0 as u8;
            indices.push(index);
        });
    }

    fn recalculate_centroids(
        mut rng: &mut impl Rng,
        buf: &[[f32; 3]],
        centroids: &mut [[f32; 3]],
        indices: &[u8],
    ) {
        centroids.iter_mut().enumerate().for_each(|(i, centroid): (usize, &mut [f32; 3])| {
            let (red, green, blue, count): (f32, f32, f32, i32) = indices
                .into_par_iter()
                .zip(buf.into_par_iter())
                .filter(|(&index, _)| index == i as u8)
                .fold(
                    || (0.0, 0.0, 0.0, 0),
                    |(r, g, b, c): (f32, f32, f32, i32), (_, color)| (r + color[0], g + color[1], b + color[2], c + 1),
                )
                .reduce(
                    || (0.0, 0.0, 0.0, 0),
                    |(r1, g1, b1, c1): (f32, f32, f32, i32), (r2, g2, b2, c2): (f32, f32, f32, i32)| (r1 + r2, g1 + g2, b1 + b2, c1 + c2),
                );

            if count != 0 {
                *centroid = [
                    red / count as f32,
                    green / count as f32,
                    blue / count as f32,
                ];
            } else {
                *centroid = Self::create_random(&mut rng);
            }
        });
    }

    fn check_loop(centroids: &[[f32; 3]], old_centroids: &[[f32; 3]]) -> f32 {
        centroids
            .into_par_iter()
            .zip(old_centroids.into_par_iter())
            .map(|(c1, c2): (&[f32; 3], &[f32; 3])| Self::difference(c1, c2))
            .sum::<f32>()
    }

    #[inline]
    fn create_random(rng: &mut impl Rng) -> [f32; 3] {
        [
            rng.gen_range(0.0..255.0),
            rng.gen_range(0.0..255.0),
            rng.gen_range(0.0..255.0),
        ]
    }

    #[inline]
    fn difference(c1: &[f32; 3], c2: &[f32; 3]) -> f32 {
        (c1[0] - c2[0]).powf(2.0) + (c1[1] - c2[1]).powf(2.0) + (c1[2] - c2[2]).powf(2.0)
    }
}

impl Hamerly for [f32; 3] {
    fn compute_half_distances(centers: &mut HamerlyCentroids<Self>) {
        let centroids: &Vec<[f32; 3]> = &centers.centroids;
        centers.half_distances.par_iter_mut().enumerate().for_each(|(idx, half_dist): (usize, &mut f32)| {
            let min_diff: f32 = (0..centroids.len())
                .filter(|&jdx: &usize| idx != jdx)
                .fold(f32::MAX, |min: f32, jdx: usize| {
                    let diff: f32 = Self::difference(&centroids[idx], &centroids[jdx]);
                    f32::min(min, diff)
                });
            *half_dist = 0.5 * min_diff.sqrt();
        });
    }

    fn get_closest_centroid_hamerly(
        buffer: &[Self],
        centers: &HamerlyCentroids<Self>,
        points: &mut [HamerlyPoint],
    ) {
        points.par_iter_mut().for_each(|point: &mut HamerlyPoint| {
            // Assign max of lower bound and half distance to z
            let z: f32 = centers.half_distances[point.index as usize].max(point.lower_bound);
    
            if point.upper_bound <= z {
                return;
            }
    
            // Tighten upper bound
            let centroid: &[f32; 3] = &centers.centroids[point.index as usize];
            point.upper_bound = Self::difference(&buffer[point.index as usize], centroid).sqrt();
    
            if point.upper_bound <= z {
                return;
            }
    
            // Find the two closest centers to current point and their distances
            if centers.centroids.len() < 2 {
                return;
            }
    
            let (min1, c1): (f32, usize) = centers.centroids.par_iter().enumerate().skip(1).fold(
                || (Self::difference(&buffer[point.index as usize], &centers.centroids[0]), 0),
                |(min, idx): (f32, usize), (j, centroid): (usize, &[f32; 3])| {
                    let diff: f32 = Self::difference(&buffer[point.index as usize], centroid);
                    if diff < min {
                        (diff, j)
                    } else {
                        (min, idx)
                    }
                },
            ).reduce(
                || (f32::MAX, 0),
                |(a, ia): (f32, usize), (b, ib): (f32, usize)| if a < b { (a, ia) } else { (b, ib) }
            );
    
            let mut min2 = f32::MAX;
            centers.centroids.iter().enumerate().for_each(|(j, centroid): (usize, &[f32; 3])| {
                if j != c1 && j != point.index as usize {
                    let diff:f32 = Self::difference(&buffer[point.index as usize], centroid);
                    if diff < min2 {
                        min2 = diff;
                    }
                }
            });
    
            if c1 != point.index as usize {
                point.index = c1 as u8;
                point.upper_bound = min1.sqrt();
            }
            point.lower_bound = min2.sqrt();
        });
    }

    fn recalculate_centroids_hamerly(
        mut rng: &mut impl Rng,
        buf: &[Self],
        centers: &mut HamerlyCentroids<Self>,
        points: &[HamerlyPoint],
    ) {
        (0..centers.centroids.len()).for_each(|idx: usize| {
            let mut red: f32 = 0.0;
            let mut green: f32 = 0.0;
            let mut blue: f32 = 0.0;
            let mut counter: u64 = 0;
            (0..points.len()).for_each(|jdx: usize| {
                if points[jdx].index == idx as u8 {
                    red += buf[jdx][0];
                    green += buf[jdx][1];
                    blue += buf[jdx][2];
                    counter += 1;
                }
            });
            if counter != 0 {
                let new_color: [f32; 3] = [
                    red / (counter as f32),
                    green / (counter as f32),
                    blue / (counter as f32),
                ];
                centers.deltas[idx] = Self::difference(&centers.centroids[idx], &new_color).sqrt();
                centers.centroids[idx] = new_color;
            } else {
                let new_color: [f32; 3] = Self::create_random(&mut rng);
                centers.deltas[idx] = Self::difference(&centers.centroids[idx], &new_color).sqrt();
                centers.centroids[idx] = new_color;
            }
        });
    }

    fn update_bounds(centers: &HamerlyCentroids<Self>, points: &mut [HamerlyPoint]) {
        let delta_p: f32 = centers.deltas.iter().fold(0.0, |max_delta: f32, &delta: &f32| delta.max(max_delta));

        points.iter_mut().for_each(|point: &mut HamerlyPoint| {
            point.upper_bound += centers.deltas[point.index as usize];
            point.lower_bound -= delta_p;
        });
    }
}

/// A trait for mapping colors to their corresponding centroids.
pub trait MapColor: Sized {
    /// Map pixel indices to each centroid for output buffer.
    fn map_indices_to_centroids(centroids: &[Self], indices: &[u8]) -> Vec<Self>;
}

impl MapColor for [f32; 3] {
    #[inline]
    fn map_indices_to_centroids(centroids: &[Self], indices: &[u8]) -> Vec<Self> {
        indices
            .par_iter()
            .map(|x: &u8| {
                *centroids
                    .get(*x as usize)
                    .unwrap_or_else(|| centroids.last().unwrap())
            })
            .collect()
    }
}
