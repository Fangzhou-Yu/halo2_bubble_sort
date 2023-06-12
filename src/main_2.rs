/// Circuit Implementation for Bubble Sort
/// Purpose: generate zero-knowledge proof for a bubble sort algo with halo2
/// Fangzhou Yu, Summer 23


/// This version doesn't implement a check for correctness of result
/// So it ignores many necessary constraint and just sort it in a circuit

use ff::{BatchInvert, FromUniformBytes};
use halo2_proofs::{
    arithmetic::{CurveAffine, Field},
    circuit::{floor_planner::V1, Layouter, Value},
    dev::{metadata, FailureLocation, MockProver, VerifyFailure},
    halo2curves::pasta::EqAffine,
    plonk::*,
    poly::{
        commitment::ParamsProver,
        ipa::{
            commitment::{IPACommitmentScheme, ParamsIPA},
            multiopen::{ProverIPA, VerifierIPA},
            strategy::AccumulatorStrategy,
        },
        VerificationStrategy,
    },
    transcript::{
        Blake2bRead, Blake2bWrite, Challenge255, TranscriptReadBuffer, TranscriptWriterBuffer,
    },
};
use rand_core::{OsRng, RngCore};
use std::iter;

fn rand_1d_array<F: Field, R: RngCore>(rng: &mut R) -> [[F; 4]] {
    [(); H].map(|_| F::random(&mut *rng))
}

fn sorted<F: Field>(
    unsorted: [F; 4],
) -> [F; 4] {
    let mut sorted = unsorted;

    for i in (0..4) {
        for j in (1..4-i-1) {
            if sorted[j] > sorted[j+1]{
                sorted.swap(j,j+1);
            }    
        }
    }

    sorted
}

#[derive(Clone)]
struct MyConfig<F: FieldExt> {
    unsorted: [Column<Advice>; 4],
    sorted: [Column<Advice>; 4],
    instance: Column<Instance>,
}

// impl<const W: usize> MyConfig<W> {
//     fn configure<F: Field>(meta: &mut ConstraintSystem<F>) -> Self {
//         let [q_shuffle, q_first, q_last] = [(); 3].map(|_| meta.selector());
//         // First phase, nothing has changed
//         let unsorted = [(); W].map(|_| meta.advice_column_in(FirstPhase));
//         let sorted = [(); W].map(|_| meta.advice_column_in(FirstPhase));
//         let [theta, gamma] = [(); 2].map(|_| meta.challenge_usable_after(FirstPhase));
//         // Second phase
//         let z = meta.advice_column_in(SecondPhase);

//         meta.create_gate("z should start with 1", |_| {
//             let one = Expression::Constant(F::ONE);

//             vec![q_first.expr() * (one - z.cur())]
//         });

//         meta.create_gate("z should end with 1", |_| {
//             let one = Expression::Constant(F::ONE);

//             vec![q_last.expr() * (one - z.cur())]
//         });

//         meta.create_gate("z should have valid transition", |_| {
//             let q_shuffle = q_shuffle.expr();
//             let unsorted = unsorted.map(|advice| advice.cur());
//             let sorted = sorted.map(|advice| advice.cur());
//             let [theta, gamma] = [theta, gamma].map(|challenge| challenge.expr());

//             // Compress
//             let unsorted = unsorted
//                 .iter()
//                 .cloned()
//                 .reduce(|acc, a| acc * theta.clone() + a)
//                 .unwrap();
//             let sorted = sorted
//                 .iter()
//                 .cloned()
//                 .reduce(|acc, a| acc * theta.clone() + a)
//                 .unwrap();

//             vec![q_shuffle * (z.cur() * (unsorted + gamma.clone()) - z.next() * (sorted + gamma))]
//         });

//         Self {
//             q_shuffle,
//             q_first,
//             q_last,
//             unsorted,
//             sorted,
//             theta,
//             gamma,
//             z,
//         }
//     }
// }

#[derive(Clone, Default)]
struct MyCircuit<F: Field> {
    unsorted: Value<[F; H]>,
    sorted: Value<[F; H]>,
}

impl<F: Field> MyCircuit<F> {
    fn rand<R: RngCore>(rng: &mut R) -> Self {
        let original = rand_1d_array::<F, _>(rng);
        let sorted = shuffled(original, rng);

        Self {
            original: Value::known(original),
            sorted: Value::known(shuffled),
        }
    }
}

impl<F: Field> Circuit<F> for MyCircuit<F> {
    type Config = MyConfig;
    type FloorPlanner = V1;
    #[cfg(feature = "circuit-params")]
    type Params = ();

    fn without_witnesses(&self) -> Self {
        Self::default()
    }

    fn configure(meta: &mut ConstraintSystem<F>) -> Self::Config {
        MyConfig::configure(meta)
    }

    fn synthesize(
        &self,
        config: Self::Config,
        mut layouter: impl Layouter<F>,
    ) -> Result<(), Error> {
        for i in 0..4 {
            for j in (i + 1)..4 {
                let less = layouter.assign_region(
                    || format!("less[{}][{}]", i, j),
                    |region| {
                        let less = region.assign_advice(
                            || format!("less[{}][{}]", i, j),
                            || sorted[i] - sorted[j],
                        )?;
    
                        Ok(less)
                    },
                )?;
    
                let swap = layouter.assign_region(
                    || format!("swap[{}][{}]", i, j),
                    |region| {
                        let swap = region.assign_advice(
                            || format!("swap[{}][{}]", i, j),
                            || sorted[i] - less,
                        )?;
    
                        Ok(swap)
                    },
                )?;
    
                layouter.constrain_equal(
                    || format!("constrain[{}][{}]", i, j),
                    swap,
                    || sorted[j],
                );
    
                layouter.constrain_equal(
                    || format!("constrain[{}][{}]", i, j),
                    less,
                    || sorted[i],
                );
    
                sorted[i] = layouter.get_value(swap)?;
                sorted[j] = layouter.get_value(less)?;
            }
        }
    }
}

fn main() {
\;

    // let circuit = &MyCircuit::<_, W, H>::rand(&mut OsRng);
    // let a = Fp::from(1);
    // let b = Fp::from(7);
    // let c = Fp::from(20);
    // let d = Fp::from(15);
    // let mut public_input = vec![a, b, c, d];

    let circuit = MyCircuit<_>::rand(&mut OsRng);

    let prover = MockProver::run(k, &circuit, vec![public_input.clone()]).unwrap();
    prover.assert_satisfied();
}