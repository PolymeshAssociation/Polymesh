use frame_support::weights::{
    WeightToFeeCoefficient, WeightToFeeCoefficients, WeightToFeePolynomial,
};
use smallvec::smallvec;
pub use sp_runtime::Perbill;

use polymesh_primitives::Balance;

pub struct PrivateRuntimeWeightToFee;

impl WeightToFeePolynomial for PrivateRuntimeWeightToFee {
    type Balance = Balance;

    fn polynomial() -> WeightToFeeCoefficients<Self::Balance> {
        smallvec![WeightToFeeCoefficient {
            degree: 0,
            coeff_frac: Perbill::zero(),
            coeff_integer: 0,
            negative: false,
        }]
    }
}
