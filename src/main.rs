use std::{marker::PhantomData};

use halo2_proofs::{
    arithmetic::FieldExt,
    circuit::*,
    plonk::*,
    poly::Rotation
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
                self.config.s.enable(&mut region, 1)?;


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
        offset: usize,
    ) -> Result<(AssignedCell<F,F>, AssignedCell<F,F>, AssignedCell<F,F>, AssignedCell<F,F>), Error> {
        // do a compare & swap and assign a new region
        println!("offset: {:?}\n", offset);
        let mut arr = [prev_a.value().clone(), prev_b.value().clone(), prev_c.value().clone(), prev_d.value().clone()];

        println!("a_unsorted: {:?}\n", arr[0]);

        println!("b_unsorted: {:?}\n", arr[1]);

        println!("c_unsorted: {:?}\n", arr[2]);
  
        println!("d_unsorted: {:?}\n", arr[3]);
        
        for idx in 0..3 {
            let a = arr[idx].clone();
            let b = arr[idx+1].clone();
            let is_greater = a.zip(b).map(|(a, b)| a > b);
            let output = format!("{:?}", is_greater);
            let is_true = if output.contains("true") {
                true
            } else {
                false
            };
            if is_true {
                arr.swap(idx, idx+1);
            }

        }
        let a_val = arr[0];
        println!("a: {:?}\n", a_val);
        let b_val = arr[1];
        println!("b: {:?}\n", b_val);
        let c_val = arr[2];
        println!("c: {:?}\n", c_val);
        let d_val = arr[3];
        println!("d: {:?}\n", d_val);

        // now load new sorted values into new row 

        /// DEBUG

        Ok(layouter.assign_region(
            || "next row",
            |mut region| {
                

                // Maybe jusy copy every value over instead?
                

                // a
                let a_cell = region.assign_advice(
                    || "a",
                    self.config.a,
                    0,
                    || a_val.copied(),
                )?;

                // b
                let b_cell = region.assign_advice(
                    || "b",
                    self.config.b,
                    0,
                    || b_val.copied(),
                )?;

                // c
                let c_cell = region.assign_advice(
                    || "c",
                    self.config.c,
                    0,
                    || c_val.copied(),
                )?;

                // d
                let d_cell = region.assign_advice(
                    || "d",
                    self.config.d,
                    0,
                    || d_val.copied(),
                )?;

                // enable the selector
                self.config.s.enable(&mut region, 0)?;

                Ok((a_cell, b_cell, c_cell, d_cell))
            },
        )?)
        
    }

}

#[derive(Default)]
struct BubSortCircuit<F> (PhantomData<F>);

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
        )?;
        // rows in the table
        for round in 2..5 {
            let offset: usize = round;
            let (a, b, c, d) = chip.load_row(
                layouter.namespace(|| "next row"),
                &prev_a,
                &prev_b,
                &prev_c,
                &prev_d,
                offset,
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
    use std::marker::PhantomData;
    use halo2_proofs::{dev::MockProver, pasta::Fp};
    // Prepare the private and public inputs to the circuit!
    let a = Fp::from(100);
    let b = Fp::from(90);
    let c = Fp::from(80);
    let d = Fp::from(70);

    // Instantiate the circuit with the private inputs.
    let circuit = BubSortCircuit(PhantomData);
    let public_input = vec![a,b,c,d];

    // Set circuit size
    let k = 5;

    // Given the correct public input, our circuit will verify.
    let prover = MockProver::run(k, &circuit, vec![vec![a], vec![b], vec![c], vec![d]]).unwrap();
    prover.assert_satisfied();
}

