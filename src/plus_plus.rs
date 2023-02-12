use rand::distributions::{Distribution, WeightedIndex};
use rand::Rng;
use rayon::prelude::*;

const F32_MAX: f32 = core::f32::MAX;

/// k-means++ centroid initialization.
///
/// # Panics
///
/// Panics if buffer is empty.
///
/// # Reference
///
/// Based on Section 2.2 from `k-means++: The Advantages of Careful Seeding` by
/// Arthur and Vassilvitskii (2007).
pub fn init_plus_plus<C: crate::Calculate + Clone + Sync + Send>(
    k: usize,
    mut rng: &mut impl Rng,
    buf: &[C],
    centroids: &mut Vec<C>,
) {
    if k == 0 {
        return;
    }
    let buf_len: usize = buf.len();
    assert!(buf_len > 0);

    let mut weights: Vec<f32> = vec![0.0; buf_len];

    // Choose first centroid at random, uniform sampling from input buffer
    centroids.push(buf.get(rng.gen_range(0..buf_len)).unwrap().to_owned());

    // Pick a new centroid with weighted probability of `D(x)^2 / sum(D(x)^2)`,
    // where `D(x)^2` is the distance to the closest centroid
    (1..k).for_each(|_| {
        // Calculate the distances to nearest centers, accumulate a sum
        let mut sum: f32 = 0.0;
        weights
            .iter_mut()
            .enumerate()
            .for_each(|(idx, weight): (usize, &mut f32)| {
                let mut min: f32 = F32_MAX;
                centroids.iter().for_each(|cent: &C| {
                    let diff: f32 = C::difference(&buf[idx], cent);
                    if diff < min {
                        min = diff;
                    }
                });
                *weight = min;
                sum += min;
            });

        // If centroids match all colors, return early
        if !sum.is_normal() {
            return;
        }

        // Divide distances by sum to find D^2 weighting for distribution
        weights.par_iter_mut().for_each(|x: &mut f32| *x /= sum);

        // Choose next centroid based on weights
        let sampler: WeightedIndex<f32> =
            WeightedIndex::new(&weights).expect("Failed to create weighted index.");
        centroids.push(buf.get(sampler.sample(&mut rng)).unwrap().to_owned());
    });
}
