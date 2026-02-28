//! Generic baseline benchmark runners against FANDANGO.

extern crate alloc;

use alloc::vec::Vec;
use linfa::traits::Fit;
use linfa::{Dataset, DatasetBase};
use linfa_linear::FittedLinearRegression;
use ndarray::{Array, Array2, ArrayBase, FixedInitializer, Ix1, Ix2, OwnedRepr};
use serde::{Deserialize, Serialize};

/// A collection of models which represents operations of a particular implementation of FANDANGO
#[derive(Serialize, Deserialize)]
pub struct OperationModel {
    /// The crossover operation model
    pub crossover: FittedLinearRegression<f64>,
    /// The evaluation (check) operation model
    pub evaluate: Option<FittedLinearRegression<f64>>,
    /// The fix operation model
    pub fix: Option<FittedLinearRegression<f64>>,
    /// The generate operation model
    pub generate: FittedLinearRegression<f64>,
    /// The mutate operation model
    pub mutate: FittedLinearRegression<f64>,
}

/// The representation used for sampling data that we will perform linear regression over
pub type DataRepr = DatasetBase<ArrayBase<OwnedRepr<f64>, Ix2>, ArrayBase<OwnedRepr<f64>, Ix1>>;

/// A model and the corresponding dataset which produced it
pub type ModelWithDataset = (FittedLinearRegression<f64>, DataRepr);

/// Regress over a given set of measurements
#[must_use]
#[allow(clippy::missing_panics_doc)]
pub fn regress<const DIM: usize>(
    measurements: Vec<(f64, [f64; DIM])>,
    features: [&'static str; DIM],
) -> ModelWithDataset
where
    [f64; DIM]: FixedInitializer<Elem = f64>, // give the compiler a little help
{
    let targets = Array::from_iter(measurements.iter().map(|(t, _)| *t));
    let data = Array2::from(
        measurements
            .into_iter()
            .map(|(_, e)| e)
            .collect::<Vec<[f64; DIM]>>(),
    );
    let dataset = Dataset::new(data, targets).with_feature_names(features.to_vec());
    let regression = linfa_linear::LinearRegression::new();
    (
        regression.with_intercept(false).fit(&dataset).unwrap(),
        dataset,
    )
}
