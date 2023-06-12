use std::{marker::PhantomData};

use halo2_proofs::{
    arithmetic::FieldExt,
    circuit::{AssignedCell, Layouter, SimpleFloorPlanner},
    plonk::{Advice, Circuit, Column, ConstraintSystem, Error, Instance, Selector},
    poly::Rotation,
};

#[derive(Clone)]
struct Number<F: FieldExt>(AssignedCell<F, F>);

// Config that contains the columns used in the circuit
#[derive(Debug, Clone)]
struct BubSortConfig {
    a: Column<Advice>,
    b: Column<Advice>,
    c: Column<Advice>,
    d: Column<Advice>,
    s: Selector,
}

// The chip that configures the gate and fills in the witness
struct BubSortChip<F: FieldExt> {
    config: BubSortConfig,
    _marker: PhantomData<F>,
}

impl<F: FieldExt> BubSortChip<F> {
    fn construct(config: BubSortConfig) -> Self {
        Self {
            config,
            _marker: PhantomData,
        }
    }

    fn configure(
        meta: &mut ConstraintSystem<F>,
    ) -> BubSortConfig {
        // create columns
        let a = meta.advice_column();
        let b = meta.advice_column();
        let c = meta.advice_column();
        let d = meta.advice_column();
        let s = meta.selector();

        // enable permutation checks for the following columns
        meta.enable_equality(a);
        meta.enable_equality(b);
        meta.enable_equality(c);
        meta.enable_equality(d);
        meta.enable_equality(i);

        // define the custom gate
        meta.create_gate("swap", |meta| {
            let s = meta.query_selector(selector);
            let term_a = meta.query_advice(a, Rotation::cur());
            let term_b = meta.query_advice(b, Rotation::cur());
            let term_c = meta.query_advice(c, Rotation::cur());
            let term_d = meta.query_advice(d, Rotation::cur());
            let term_a_next = meta.query_advice(a, Rotation::next());
            let term_b_next = meta.query_advice(b, Rotation::next());
            let term_c_next = meta.query_advice(c, Rotation::next());
            let term_d_next = meta.query_advice(d, Rotation::next());
            vec![s * (term_a + term_b + term_c + term_d - term_a_next - term_b_next - term_c_next - term_d_next)]
        });

        BubSortConfig {
            a, b, c, d, i, s,
        }
    }

    fn load_first_row(
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
                    || Ok(a),
                ).map(Number)?;

                let b_num = region.assign_advice(
                    || "b",
                    self.config.b, // column b
                    0, // rotation
                    || Ok(b),
                ).map(Number)?;

                let c_num = region.assign_advice(
                    || "c",
                    self.config.c, // column c
                    0, // rotation
                    || Ok(c),
                ).map(Number)?;

                let d_num = region.assign_advice(
                    || "d",
                    self.config.d, // column c
                    0, // rotation
                    || Ok(d),
                ).map(Number)?;

                Ok((a_num, b_num, c_num, d_num))
            },
        )
    }

    fn load_row(
        &self,
        mut layouter: impl Layouter<F>,
        a: F,
        b: F,
        c: F,
        d: F,
    ) -> Result<(a_num, b_num, c_num, d_num), Error> {
        // do a compare & swap and assign a new region
        let mut arr = [prev_a.clone(), prev_b.clone(), prev_c.clone(), prev_d.clone()];
        for idx in 0..3 {
            let mut a = arr[idx];
            let mut b = arr[idx+1]

            if Some(true) == a.0.value().and_then(|a| b.0.value().map(|b| *a > *b)) {
                arr.swap(arr[idx], arr[idx+1])
            }
        }


        layouter.assign_region(
            || "row",
            |mut region| {
                // enable the selector
                self.config.s.enable(&mut region, 0)?;

                // copy the cell from previous row
                let a_val = arr[0].0.value();
                let b_val = arr[1].0.value();
                let c_val = arr[2].0.value();
                let d_val = arr[3].0.value();

                // a
                region.assign_advice(
                    || "a",
                    self.config.a,
                    0,
                    || a_val.ok_or(Error::Synthesis),
                ).map(Number)

                // b
                region.assign_advice(
                    || "b",
                    self.config.b,
                    0,
                    || b_val.ok_or(Error::Synthesis),
                ).map(Number)

                // c
                region.assign_advice(
                    || "c",
                    self.config.c,
                    0,
                    || c_val.ok_or(Error::Synthesis),
                ).map(Number)

                // d
                region.assign_advice(
                    || "d",
                    self.config.d,
                    0,
                    || d_val.ok_or(Error::Synthesis),
                ).map(Number)

                Ok((arr[0].clone(), arr[1].clone(),arr[2].clone(),arr[3].clone()))
            },
        )
        
    }

}

#[derive(Default)]
struct BubSortCircuit<F> {
    a: F,
    b: F,
    c: F,
    d: F,
}

impl<F: FieldExt> Circuit<F> for BubSortCircuit<F> {
    type Config = BubSortConfig;
    type FloorPlanner = SimpleFloorPlanner;

    fn without_witnesses(&self) -> Self {
        Self::default()
    }

    fn configure(meta: &mut ConstraintSystem<F>) -> Self::Config {
        BubSortChip::configure(meta)
    }

    fn synthesize(
        &self,
        config: Self::Config,
        mut layouter: impl Layouter<F>
    ) -> Result<(), Error> {
        let chip = BubSortChip::construct(config);
        let (mut prev_a, mut prev_b, mut prev_c, mut prev_d) = chip.load_first_row(
            layouter.namespace(|| "first row"),
            self.a,
            self.b,
            self.c,
            self.d,
        )?;
        for _ in 0..4 {
            let (a, b, c, d) = chip.load_row(
                layouter.namespace(|| "row"),
                &prev_a,
                &prev_b,
                &prev_c,
                &prev_d,
            )?;
            prev_a = a;
            prev_b = b;
            prev_c = c;
            prev_d = d;
        }
        Ok(())
    }
}


fn main() {
    use halo2_proofs::{dev::MockProver, pairing::bn256::Fr as Fp};

    // Prepare the private and public inputs to the circuit!
    let arr = [13,324,3,88]

    // Instantiate the circuit with the private inputs.
    let circuit = FiboCircuit {
        a: Fp::from(arr[0]),
        b: Fp::from(arr[1]),
        c: Fp::from(arr[2]),
        d: Fp::from(arr[3]),
    };


    // Set circuit size
    let k = 4;

    // Given the correct public input, our circuit will verify.
    let prover = MockProver::run(k, &circuit, _).unwrap();
    assert_eq!(prover.verify(), Ok(()));
}

