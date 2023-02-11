use rand::distributions::{Distribution, WeightedIndex};
use rand::Rng;
use rayon::prelude::*;

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
pub fn init_plus_plus<C: crate::Calculate + Clone>(
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

    let mut weights: Vec<f32> = (0..buf_len).into_par_iter().map(|_| 0.0).collect();

    // Choose first centroid at random, uniform sampling from input buffer
    centroids.push(buf.get(rng.gen_range(0..buf_len)).unwrap().to_owned());

    // Pick a new centroid with weighted probability of `D(x)^2 / sum(D(x)^2)`,
    // where `D(x)^2` is the distance to the closest centroid
    for _ in 1..k {
        // Calculate the distances to nearest centers, accumulate a sum
        let mut sum = 0.0;
        for idx in 0..buf_len {
            let mut diff;
            let mut min = core::f32::MAX;
            for cent_idx in 0..centroids.len() {
                diff = C::difference(&buf[idx], &centroids[cent_idx]);
                if diff < min {
                    min = diff;
                }
            }
            weights[idx] = min;
            sum += min;
        }

        // If centroids match all colors, return early
        if !sum.is_normal() {
            return;
        }

        // Divide distances by sum to find D^2 weighting for distribution
        weights.par_iter_mut().for_each(|x: &mut f32| *x /= sum);

        // Choose next centroid based on weights
        let sampler: WeightedIndex<f32> = WeightedIndex::new(&weights).unwrap();
        centroids.push(buf.get(sampler.sample(&mut rng)).unwrap().to_owned());
    }
}
