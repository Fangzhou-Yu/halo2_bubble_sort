use std::{marker::PhantomData};
use std::cmp::PartialOrd;
use std::ops::Deref;

use halo2_proofs::{
    arithmetic::FieldExt,
    circuit::{AssignedCell, Layouter, SimpleFloorPlanner, Value},
    plonk::{Advice, Circuit, Column, ConstraintSystem, Error, Instance, Selector},
    poly::Rotation,
};


// let's do bubble sort on an array A of size N
// Each element in the array is of type Number
#[derive(Clone)]
struct Number<F: FieldExt>(AssignedCell<F, F>);

// Config
// define columns
// we maintain N advice columns for each 
#[derive(Debug, Clone)]
struct BubbleSortConfig {
    a: Column<Advice>,
    b: Column<Advice>,
    c: Column<Advice>,
    d: Column<Advice>,
    selector: Selector,
    i: Column<Instance>,
}

// Chip
// define instructions
struct BubbleSortChip<F: FieldExt> {
    config: BubbleSortConfig,
    _marker: PhantomData<F>,
}

impl<F: FieldExt> BubbleSortChip<F> {
    fn construct(config: BubbleSortConfig) -> Self {
        Self {
            config,
            _marker: PhantomData,
        }
    }

    fn configure(
        meta: &mut ConstraintSystem<F>,
        instance: Column<Instance>,
    ) -> BubbleSortConfig {
        // create columns
        let a = meta.advice_column();
        let b = meta.advice_column();
        let c = meta.advice_column();
        let d = meta.advice_column();
        let i = instance;
        let selector = meta.selector();

        // enable permutation checks for the following columns
        meta.enable_equality(a);
        meta.enable_equality(b);
        meta.enable_equality(c);
        meta.enable_equality(d);

        // define the custom gate
        // swap gate?
        meta.create_gate("compare swap", |meta| {
            let s = meta.query_selector(selector);
            let term_a = meta.query_advice(a, Rotation::cur());
            let term_b = meta.query_advice(b, Rotation::cur());
            let term_c = meta.query_advice(c, Rotation::cur());
            let term_c = meta.query_advice(d, Rotation::cur());
            let term_a_next = meta.query_advice(a, Rotation::next());
            let term_b_next = meta.query_advice(b, Rotation::next());
            let term_c_next = meta.query_advice(c, Rotation::next());
            let term_d_next = meta.query_advice(d, Rotation::next());
            vec![s * (term_a + term_b + term_c + term_d - term_a_next - term_b_next - term_c_next - term_d_next)]
        });

        BubbleSortConfig {
            a, b, c, d, i, selector,
        }
    }

    fn load_array(
        &self,
        mut layouter: impl Layouter<F>,
        a: F,
        b: F,
        c: F,
        d: F,
    ) -> Result<(Number<F>, Number<F>, Number<F>, Number<F>), Error> {
        // load first row
        layouter.assign_region(
            || "first row",
            |mut region| {
                // enable the selector
                self.config.s.enable(&mut region, 0)?;

                let a_num = region.assign_advice(
                    || "a",
                    self.config.a, // column a
                    0, // rotation
                    || Ok(self.a),
                ).map(Number)?;

                let b_num = region.assign_advice(
                    || "b",
                    self.config.b, // column b
                    0, // rotation
                    || Ok(self.b),
                ).map(Number)?;

                let c_num = region.assign_advice(
                    || "c",
                    self.config.c, // column c
                    0, // rotation
                    || Ok(self.c),
                ).map(Number)?;

                let d_num = region.assign_advice(
                    || "d",
                    self.config.d, // column c
                    0, // rotation
                    || Ok(self.d),
                ).map(Number)?;

                Ok((a_num, b_num, c_num, d_num))
            },
        )
    }



    fn assign_row(
        &self,
        mut layouter: impl Layouter<F>,
        prev_a: &Number<F>,
        prev_b: &Number<F>,
        prev_c: &Number<F>,
        prev_d: &Number<F>,
    ) -> Result<(Number<F>,Number<F>,Number<F>,Number<F>), Error> {
        // sort one round
        let mut arr = [prev_a.clone(), prev_b.clone(), prev_c.clone(), prev_d.clone()];
        for idx in 0..3 {
            if is_a_greater_than_b(arr[idx].0.value(), arr[idx+1].0.value()) {
                arr.swap(arr[idx], arr[idx+1])
            }
        }

        // assign new region
        layouter.assign_region(
            || "new row with one pair",
            |mut region| {
                // enable the selector
                self.config.selector.enable(&mut region, 0)?;

                // copy the cell from previous row
                let a = arr[0].0.value();
                let b = arr[1].0.value();
                let c = arr[2].0.value();
                let d = arr[3].0.value();

                let a_num = region.assign_advice(
                    || "a",
                    self.config.a,
                    0,
                    || a,
                ).map(Number);

                let b_num = region.assign_advice(
                    || "b",
                    self.config.b,
                    0,
                    || b,
                ).map(Number);

                let c_num = region.assign_advice(
                    || "c",
                    self.config.c,
                    0,
                    || c,
                ).map(Number);

                let d_num = region.assign_advice(
                    || "d",
                    self.config.d,
                    0,
                    || d,
                ).map(Number);

                Ok((a_num, b_num, c_num, d_num))
            },
        )  
    }

    // fn expose_public(
    //     &self,
    //     mut layouter: impl Layouter<F>,
    //     num: Number<F>,
    //     row: usize,
    // ) -> Result<(), Error> {
    //     layouter.constrain_instance(num.cell(), self.config.i, row)
    // }

}

// Circuit
// load stuff into tables and perform operations

#[derive(Default)]
struct BubbleSortCircuit<F> {
    a: F,
    b: F,
    c: F,
    d: F,
}
impl<F: FieldExt> Circuit<F> for BubbleSortCircuit<F> {
    type Config = BubbleSortConfig;
    type FloorPlanner = SimpleFloorPlanner;

    fn without_witnesses(&self) -> Self {
        Self::default()
    }

    fn configure(meta: &mut ConstraintSystem<F>) -> Self::Config {
        BubbleSortChip::configure(meta)
    }

    fn synthesize(
        &self,
        config: Self::Config,
        mut layouter: impl Layouter<F>
    ) -> Result<(), Error> {
        let chip = BubbleSortChip::construct(config);
        let (prev_a, prev_b, prev_c, prev_d) = chip.load_array(
            layouter.namespace(|| "array"),
            &self.array,
        )?;
        chip.assign_row(
            layouter.namespace(|| "bubble_sort"),
            &prev_a,
            &prev_b,
            &prev_c,
            &prev_d,
        )?;
        // chip.expose_public(
        //     layouter.namespace(|| "expose_array"),
        //     prev_a,
        // )?;
        Ok(())
    }
}

fn main() {
    use halo2_proofs::{dev::MockProver, pairing::bn256::Fr as Fp};

    // Prepare the private and public inputs to the circuit!
    let array = vec![5, 2, 8, 1, 9];
    let sorted_array = array.clone().into_iter().sorted().collect::<Vec<_>>();

    // Instantiate the circuit with the private inputs.
    let circuit = BubbleSortCircuit {
        array: array.into_iter().map(Fp::from).collect(),
    };

    // Arrange the public input. We expose the sorted array in the instance column.
    let public_inputs = sorted_array.into_iter().map(Fp::from).collect::<Vec<_>>();

    // Set circuit size
    let k = 5;

    // Given the correct public input, our circuit will verify.
    let prover = MockProver::run(k, &circuit, vec![public_inputs.clone()]).unwrap();
    assert_eq!(prover.verify(), Ok(()));

    // If we try some other public input, the proof will fail!
    let modified_public_inputs = public_inputs.into_iter().map(|mut x| { x += Fp::one(); x }).collect::<Vec<_>>();
    let prover = MockProver::run(k, &circuit, vec![modified_public_inputs]).unwrap();
    assert!(prover.verify().is_err());
}

