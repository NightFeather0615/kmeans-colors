use std::cmp::Ordering;

use crate::sort::{CentroidData, Sort};
use rayon::prelude::*;

impl Sort for [f32; 3] {
    fn get_dominant_color(data: &[CentroidData<Self>]) -> Option<Self> {
        data.iter()
            .max_by(|a, b| (a.percentage).partial_cmp(&b.percentage).unwrap())
            .map(|res| res.centroid)
    }

    fn sort_colors(centroids: &[Self]) -> Vec<Self> {
        let mut rgb_colors: Vec<[f32; 3]> = centroids.to_vec();
        let is_percentage =  centroids.par_iter().any(|rgb: &[f32; 3]| rgb.par_iter().any(|v| *v > 1.0));

        if !is_percentage {
            rgb_colors
                .par_iter_mut()
                .map(|rgb: &mut [f32; 3]| [rgb[0] / 255.0, rgb[0] / 255.0, rgb[0] / 255.0])
                .collect::<Vec<[f32; 3]>>();
        }

        let mut hsl_colors: Vec<[f32; 3]> = rgb_colors
            .par_iter()
            .map(|rgb: &[f32; 3]| {
                let (max_value, min_value): (f32, f32) = rgb.iter().fold(
                    (f32::NEG_INFINITY, f32::INFINITY),
                    |(max, min): (f32, f32), &val: &f32| (max.max(val), min.min(val)),
                );
                let luminance: f32 = (max_value + min_value) / 2.0;
                let saturation: f32 = match max_value == min_value {
                    true => 0.0,
                    false if luminance <= 0.5 => (max_value - min_value) / (max_value + min_value),
                    false => (max_value - min_value) / (2.0 - max_value - min_value),
                };
                let mut hue: f32 = if max_value == rgb[0] {
                    ((rgb[1] - rgb[2]) / (max_value - min_value)) * 60.0
                } else if max_value == rgb[1] {
                    (2.0 + (rgb[2] - rgb[0]) / (max_value - min_value)) * 60.0
                } else {
                    (4.0 + (rgb[0] - rgb[1]) / (max_value - min_value)) * 60.0
                };

                if hue.is_sign_negative() {
                    hue += 360.0;
                }

                [hue, saturation, luminance]
            })
            .collect();

        hsl_colors.par_sort_by(|a: &[f32; 3], b: &[f32; 3]| {
            a[1].partial_cmp(&b[1])
                .unwrap_or(Ordering::Equal)
                .then_with(|| a[0].partial_cmp(&b[0]).unwrap_or(Ordering::Equal))
                .then_with(|| a[2].partial_cmp(&b[2]).unwrap_or(Ordering::Equal))
        });

        hsl_colors
            .par_iter_mut()
            .map(|hsl: &mut [f32; 3]| {
                if hsl[1] == 0.0 {
                    return [hsl[2] * 255.0; 3];
                }

                let tmp1: f32 = if hsl[2] < 0.5 {
                    hsl[2] * (1.0 + hsl[1])
                } else {
                    hsl[2] + hsl[1] - hsl[2] * hsl[1]
                };

                let tmp2: f32 = 2.0 * hsl[2] - tmp1;

                hsl[0] /= 360.0;

                let tmp_rgb: [f32; 3] = [hsl[0] + 0.333, hsl[0], hsl[0] - 0.333]
                    .map(|v: f32| {
                        if v > 1.0 {
                            return v - 1.0;
                        } else if v < 0.0 {
                            return v + 1.0;
                        } else {
                            return v;
                        }
                    })
                    .map(|v: f32| {
                        if v * 6.0 < 1.0 {
                            tmp2 + (tmp1 - tmp2) * 6.0 * v
                        } else if v * 2.0 < 1.0 {
                            tmp1
                        } else if v * 3.0 < 2.0 {
                            tmp2 + (tmp1 - tmp2) * (0.666 - v) * 6.0
                        } else {
                            tmp2
                        }
                    });

                if !is_percentage {
                    tmp_rgb.map(|v: f32| v * 255.0);
                }

                tmp_rgb
            })
            .collect()
    }
}
