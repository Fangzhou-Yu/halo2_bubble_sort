use std::{marker::PhantomData};

use halo2_proofs::{
    arithmetic::FieldExt,
    circuit::*,
    plonk::*,
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
    a_0 : Column<Instance>,
    b_0 : Column<Instance>,
    c_0 : Column<Instance>,
    d_0 : Column<Instance>,
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
        let a_0 = meta.instance_column();
        let b_0 = meta.instance_column();
        let c_0 = meta.instance_column();
        let d_0 = meta.instance_column();
        let s = meta.selector();

        // enable permutation checks for the following columns
        meta.enable_equality(a);
        meta.enable_equality(b);
        meta.enable_equality(c);
        meta.enable_equality(d);
        meta.enable_equality(a_0);
        meta.enable_equality(b_0);
        meta.enable_equality(c_0);
        meta.enable_equality(d_0);

        // define the custom gate
        meta.create_gate("swap", |meta| {
            let s = meta.query_selector(s);
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
            a, b, c, d, a_0, b_0, c_0, d_0, s,
        }
    }

    fn load_first_row(
        &self,
        mut layouter: impl Layouter<F>,
    ) -> Result<(AssignedCell<F,F>, AssignedCell<F,F>, AssignedCell<F,F>, AssignedCell<F,F>), Error> {
        // load first row
        layouter.assign_region(
            || "first row",
            |mut region| {
                // enable the selector
                self.config.s.enable(&mut region, 0)?;


                let a_num = region.assign_advice_from_instance(
                    || "a",
                    self.config.a_0, // column a
                    0, // rotation
                    self.config.a,
                    0,
                )?;
                
                let b_num = region.assign_advice_from_instance(
                    || "b",
                    self.config.b_0, // column b
                    0, 
                    self.config.b,
                    0,
                )?;
                
                let c_num = region.assign_advice_from_instance(
                    || "c",
                    self.config.c_0, // column c
                    0, 
                    self.config.c,
                    0,
                )?;

                let d_num = region.assign_advice_from_instance(
                    || "d",
                    self.config.d_0, // column c
                    0, // rotation
                    self.config.d,
                    0,
                )?;

                Ok((a_num, b_num, c_num, d_num))
            },
        )
    }


    /// JUST THIS FUCKING FUNC NOW
    fn load_row(
        &self,
        mut layouter: impl Layouter<F>,
        prev_a: &AssignedCell<F,F>,
        prev_b: &AssignedCell<F,F>,
        prev_c: &AssignedCell<F,F>,
        prev_d: &AssignedCell<F,F>,
    ) -> (AssignedCell<F,F>, AssignedCell<F,F>, AssignedCell<F,F>, AssignedCell<F,F>) {
        // do a compare & swap and assign a new region
        let mut arr = [prev_a.clone(), prev_b.clone(), prev_c.clone(), prev_d.clone()];
        for idx in 0..3 {
            let mut a = arr[idx];
            let mut b = arr[idx+1];

            if a.value() > b.value() {
                arr.swap(idx, idx+1)
            }
        }

        /// DEBUG

        layouter.assign_region(
            || "next row",
            |mut region| {
                // enable the selector
                self.config.s.enable(&mut region, 0)?;

                // Maybe jusy copy every value over instead?
                

                // a
                let a_cell = region.assign_advice(
                    || "a",
                    self.config.a,
                    0,
                    || arr[0].value().copied(),
                )?;

                // b
                let b_cell = region.assign_advice(
                    || "b",
                    self.config.b,
                    0,
                    || arr[1].value().copied(),
                )?;

                // c
                let c_cell = region.assign_advice(
                    || "c",
                    self.config.c,
                    0,
                    || arr[2].value().copied(),
                )?;

                // d
                let d_cell = region.assign_advice(
                    || "d",
                    self.config.d,
                    0,
                    || arr[3].value().copied(),
                )?;

                Ok((a_cell, b_cell, c_cell, d_cell))
            },
        )?
        
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
        let Ok((mut prev_a, mut prev_b, mut prev_c, mut prev_d)) = chip.load_first_row(
            layouter.namespace(|| "first row"),
        );
        for _ in 0..4 {
            let (a, b, c, d) = chip.load_row(
                layouter.namespace(|| "row"),
                &prev_a,
                &prev_b,
                &prev_c,
                &prev_d,
            );
            prev_a = a;
            prev_b = b;
            prev_c = c;
            prev_d = d;
        }
        Ok(())
    }
}


fn main() {
    use halo2_proofs::{dev::MockProver, pasta::Fp};
    // Prepare the private and public inputs to the circuit!
    let arr = [13,324,3,88];

    // Instantiate the circuit with the private inputs.
    let circuit = BubSortCircuit {
        a: Fp::from(arr[0]),
        b: Fp::from(arr[1]),
        c: Fp::from(arr[2]),
        d: Fp::from(arr[3]),
    };


    // Set circuit size
    let k = 4;

    // Given the correct public input, our circuit will verify.
    let prover = MockProver::run(k, &circuit, vec![]).unwrap();
    assert_eq!(prover.verify(), Ok(()));
}

