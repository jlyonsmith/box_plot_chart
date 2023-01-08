use std::error::Error;

#[derive(Debug, PartialEq)]
pub struct Quartile {
    lower_outliers: Vec<f64>,
    lower_fence: f64,
    min_before_lower_fence: f64,
    lower_median: f64,
    median: f64,
    upper_median: f64,
    max_before_upper_fence: f64,
    upper_fence: f64,
    upper_outliers: Vec<f64>,
    iqr: f64,
}

impl Quartile {
    pub fn new(values: &[f64]) -> Result<Quartile, Box<dyn Error>> {
        if values.len() < 3 {
            return Err(From::from(format!(
                "Minimum of 3 values needed for a quartile range"
            )));
        }

        let mut arr = values.to_vec();

        arr.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let len = arr.len();
        let midpoint = len / 2;
        let median: f64;
        let upper_median: f64;

        if len % 2 == 0 {
            // Even sized array
            median = (arr[midpoint - 1] + arr[midpoint]) / 2.0;
            upper_median = arr[midpoint + midpoint / 2];
        } else {
            // Odd sized array
            median = arr[midpoint];
            upper_median = arr[midpoint + 1 + midpoint / 2];
        }

        let lower_median = arr[midpoint / 2];
        let iqr = upper_median - lower_median;
        let lower_fence = lower_median - 1.5f64 * iqr;
        let upper_fence = upper_median + 1.5f64 * iqr;
        let lower_outliers: Vec<f64> = arr
            .iter()
            .take_while(|n| **n < lower_fence)
            .cloned()
            .collect();
        let upper_outliers: Vec<f64> = arr
            .iter()
            .take_while(|n| **n > upper_fence)
            .cloned()
            .collect();
        let min_before_lower_fence = arr[lower_outliers.len()];
        let max_before_upper_fence = arr[arr.len() - upper_outliers.len() - 1];

        Ok(Quartile {
            lower_outliers,
            lower_fence,
            min_before_lower_fence,
            lower_median,
            median,
            upper_median,
            max_before_upper_fence,
            upper_fence,
            upper_outliers,
            iqr,
        })
    }

    pub fn lower_outliers(&self) -> Vec<f64> {
        self.lower_outliers.clone()
    }

    pub fn lower_fence(&self) -> f64 {
        self.lower_fence
    }

    pub fn min_before_lower_fence(&self) -> f64 {
        self.min_before_lower_fence
    }

    pub fn lower_median(&self) -> f64 {
        self.lower_median
    }

    pub fn median(&self) -> f64 {
        self.median
    }

    pub fn upper_median(&self) -> f64 {
        self.upper_median
    }

    pub fn max_before_upper_fence(&self) -> f64 {
        self.max_before_upper_fence
    }

    pub fn upper_fence(&self) -> f64 {
        self.upper_fence
    }

    pub fn iqr(&self) -> f64 {
        self.iqr
    }

    pub fn upper_outliers(&self) -> Vec<f64> {
        self.upper_outliers.clone()
    }

    pub fn min_value(&self) -> f64 {
        if self.lower_outliers.is_empty() {
            self.min_before_lower_fence
        } else {
            self.lower_outliers[0]
        }
    }

    pub fn max_value(&self) -> f64 {
        if self.upper_outliers.is_empty() {
            self.max_before_upper_fence
        } else {
            *self.upper_outliers.last().unwrap()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn even_test() {
        let quartile =
            Quartile::new(&[48.0, 52.0, 57.0, 64.0, 72.0, 76.0, 77.0, 81.0, 85.0, 88.0]).unwrap();

        assert_eq!(quartile.iqr(), 24.0);
        assert_eq!(quartile.median(), 74.0);
        assert_eq!(quartile.lower_median(), 57.0);
        assert_eq!(quartile.upper_median(), 81.0);
        assert_eq!(quartile.lower_fence(), 21.0);
        assert_eq!(quartile.upper_fence(), 117.0);
        assert_eq!(quartile.min_before_lower_fence(), 48.0);
        assert_eq!(quartile.max_before_upper_fence(), 88.0);
        assert_eq!(quartile.lower_outliers(), vec![]);
        assert_eq!(quartile.upper_outliers(), vec![]);
        assert_eq!(quartile.min_value(), 48.0);
        assert_eq!(quartile.max_value(), 88.0);
    }

    #[test]
    fn odd_test_with_outliers() {
        let quartile = Quartile::new(&[
            5.0, 6.0, 48.0, 52.0, 57.0, 61.0, 64.0, 72.0, 76.0, 77.0, 81.0, 85.0, 88.0,
        ])
        .unwrap();

        assert_eq!(quartile.iqr(), 29.0);
        assert_eq!(quartile.median(), 64.0);
        assert_eq!(quartile.lower_median(), 52.0);
        assert_eq!(quartile.upper_median(), 81.0);
        assert_eq!(quartile.lower_fence(), 8.5);
        assert_eq!(quartile.upper_fence(), 124.5);
        assert_eq!(quartile.min_before_lower_fence(), 48.0);
        assert_eq!(quartile.max_before_upper_fence(), 88.0);
        assert_eq!(quartile.lower_outliers(), vec![5.0, 6.0]);
        assert_eq!(quartile.upper_outliers(), vec![]);
        assert_eq!(quartile.min_value(), 5.0);
        assert_eq!(quartile.max_value(), 88.0);
    }
}
