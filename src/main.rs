/// Circuit Implementation for Bubble Sort
/// Purpose: generate zero-knowledge proof for a bubble sort algo with halo2
/// Fangzhou Yu, Summer 23

// Import
use halo2_proofs::{arithmetic::FieldExt, circuit::*, plonk::*, poly::Rotation,dev::MockProver, pasta::Fp};
use std::marker::PhantomData;

#[derive(Debug, Clone)]

struct Sizes {
    RANGE: usize,
    NUM_BITS: usize,
    LOOKUP_RANGE: usize,
}

// we do it for array of size 5
// diff is for col(5-i+1) - col(5-i) in round i (1..5), see README.md for more discussion
#[derive(Debug)]
struct BubbleSortConfig <F: FieldExt>{
    pub col_a: Column<Advice>,
    pub col_b: Column<Advice>,
    pub col_c: Column<Advice>,
    pub col_d: Column<Advice>,
    pub col_e: Column<Advice>,
    pub diff: Column<Advice>,
    pub selector: Selector,
    pub instance: Column<Instance>,
    pub range_check: Selector,
    _marker: PhantomData<F>, 
}


/// Todo
/// Add Implementation of BubbleSortConfig to assign value of diff and make sure it is within some range
/// Let's hard code the range to 0,127

#[derive(Debug, Clone)]
struct BubbleSortChip<F:  FieldExt> {
    config: BubbleSortConfig<F>,
    _marker: PhantomData<F>,                                                                   
}

impl<F: FieldExt> BubbleSortChip<F> {
    pub fn construct (config: BubbleSortConfig <F>) -> Self {
        Self { config, _marker: PhantomData}
    }

    pub fn configure (meta: &mut ConstraintSystem<F>)-> BubbleSortConfig<F> {
        let col_a = meta.advice_column();
        let col_b = meta.advice_column();
        let col_c = meta.advice_column();
        let col_d = meta.advice_column();
        let col_e = meta.advice_column();
        let diff = meta.advice_column();    // the new column after sort
        let selector = meta.selector();
        let range_check = meta.selector();
        let instance = meta.instance_column();

        meta.enable_equality(diff);         // for range check

        meta.create_gate("range check", |meta| {
            let range_check = meta.query_selector(range_check);
            let diff = meta.query_advice(diff, Rotation::cur());
            let range_check = |range: usize, value: Expression<F>| {
                (0..range).fold(
                    diff.clone(),
                    // We do value.clone() above to initialize the types correctly. Since we want it to check 0 equality, it doesn't really matter what it is
                    |acc: halo2_proofs::plonk::Expression<F>, i| {
                        acc * (diff.clone()
                            - halo2_proofs::plonk::Expression::Constant(F::from_u128(i as u128)))
                    },
                )
            };
            Constraints::with_selector(range_check(), [("range check", range_check(20, diff))])
        });


        BubbleSortConfig {
            col_a, col_b, col_c,col_d,col_e, diff,
            selector,
            range_check,
            instance,
        }
    }

    pub fn assign_first_row(
        &self,
        mut layouter: impl Layouter<F>,
    ) -> Result<(AssignedCell<F, F>, AssignedCell<F, F>, AssignedCell<F, F>, AssignedCell<F, F>, AssignedCell<F, F>, AssignedCell<F, F>), Error> {
        layouter.assign_region(
            || "first row",
            |mut region| {
                self.config.selector.enable(&mut region, 0)?;

                let a_cell = region.assign_advice_from_instance(
                    || "a0",
                    self.config.instance,
                    0,
                    self.config.col_a,
                    0,
                )?;

                let b_cell = region.assign_advice_from_instance(
                    || "a1",
                    self.config.instance,
                    1,
                    self.config.col_b,
                    0,
                )?;

                let c_cell = region.assign_advice(
                    || "a2",
                    self.config.instance,
                    0,
                    self.config.col_c,
                )?;

                let d_cell = region.assign_advice(
                    || "a3",
                    self.config.instance,
                    0,
                    self.config.col_d,
                )?;

                let e_cell = region.assign_advice(
                    || "a4",
                    self.config.instance,
                    0,
                    self.config.col_e,
                )?;

                let diff = region.assign_advice(
                    || "diff",
                    self.config.diff,
                    0,
                    || e_cell.value().copied() - e_cell.value().copied(),
                )?;

                Ok((a_cell, b_cell, c_cell, d_cell, e_cell, diff))
            },
        )
    }

    pub fn assign_diff(
        &self,
        mut layouter: impl Layouter<F>,
        p: &AssignedCell<F, F>,
        q: &AssignedCell<F, F>,
    ) ->  Result<AssignedCell<F, F>, Error> {
        layouter.assign_region(
            || "next row",
            |mut region| {
                self.config.selector.enable(&mut region, 0)?;

                let diff = region.assign_advice(
                    || "diff",
                    self.config.diff,
                    0,
                    || p.value().copied() - q.value(),
                )?;

                Ok(diff)
            },
        )
    }


    /// This stuff doesn't work
    fn sort_row(&self, mut layouter: impl Layouter<F>,
        prev_a: &AssignedCell<F, F>,
        prev_b: &AssignedCell<F, F>,
        prev_c: &AssignedCell<F, F>,
        prev_d: &AssignedCell<F, F>,
        prev_e: &AssignedCell<F, F>)
    -> Result<(AssignedCell<F, F>, AssignedCell<F, F>, AssignedCell<F, F>, AssignedCell<F, F>, AssignedCell<F, F>), Error> {
        layouter.assign_region(
            || "next row",
            |mut region| {
                self.config.selector.enable(&mut region, 0)?;

                // copy over
                let mut col_a = region.assign_advice(
                    || "a",
                    self.config.col_a,
                    0,
                    || prev_a.value().copied(),
                )?;
                
                let mut col_b = region.assign_advice(
                    || "b",
                    self.config.col_b,
                    0,
                    || prev_b.value().copied(),
                )?;

                let mut col_c = region.assign_advice(
                    || "c",
                    self.config.col_c,
                    0,
                    || prev_c.value().copied(),
                )?;

                let mut col_d = region.assign_advice(
                    || "d",
                    self.config.col_d,
                    0,
                    || prev_d.value().copied(),
                )?;

                let mut col_e = region.assign_advice(
                    || "e",
                    self.config.col_e,
                    0,
                    || prev_e.value().copied(),
                )?;

                // Compare all columns and swap if needed
                if prev_a.value() > prev_b.value() {
                    let mut col_a = region.assign_advice(
                        || "a",
                        self.config.col_a,
                        0,
                        || prev_b.value().copied(),
                    )?;
                    
                    let mutcol_b = region.assign_advice(
                        || "b",
                        self.config.col_b,
                        0,
                        || prev_a.value().copied(),
                    )?;
                }
                // keep comparing new col b and prev col c and so on
                // Problem: Naming
                if col_b.value() > prev_c.value() {
                    col_b.copy_advice(|| "c", &mut region, self.config.col_c, 0)?; 
                    prev_c.copy_advice(|| "b", &mut region, self.config.col_b, 0)?;
                }
                if col_c.value() > prev_d.value() {
                    col_c.copy_advice(|| "d", &mut region, self.config.col_d, 0)?; 
                    prev_d.copy_advice(|| "c", &mut region, self.config.col_c, 0)?;
                }
                if col_d.value() > prev_e.value() {
                    col_d.copy_advice(|| "e", &mut region, self.config.col_e, 0)?; 
                    prev_e.copy_advice(|| "d", &mut region, self.config.col_d, 0)?;
                }

                Ok((col_a, col_b, col_c, col_d, col_e))
            },
        )
        
    }

    fn assign(
        &self,
        mut layouter: impl Layouter<F>,
        diff: AssignedCell<F, F>,
    ) -> Result<AssignedCell<F, F>, halo2_proofs::plonk::Error> {
        layouter.assign_region(
            || "Range chip brute force",
            |mut region| {
                let offset = 0;
                self.config.range_check.enable(&mut region, offset);
                Ok(diff)
            },
        )
    }


    pub fn expose_public(
        &self,
        mut layouter: impl Layouter<F>,
        cell: &AssignedCell<F, F>,
        row: usize,
    ) -> Result<(), Error> {
        layouter.constrain_instance(cell.cell(), self.config.instance, row)
    }
}

#[derive(Default)]
struct MyCircuit<F, const RANGE: usize> (PhantomData<F>);

impl<F: FieldExt> Circuit<F> for MyCircuit<F, {RANGE}> {
    type Config = BubbleSortConfig<F>;
    type FloorPlanner = SimpleFloorPlanner;

    fn without_witnesses(&self) -> Self {
        Self::default()
    }

    fn configure(meta: &mut ConstraintSystem<F>) -> Self::Config {
        BubbleSortChip::configure(meta)
    }

    fn synthesize(&self, config: Self::Config, mut layouter: impl Layouter<F>) -> Result<(), Error> {
        let chip = BubbleSortChip::construct(config);

        let (mut prev_a, mut prev_b, mut prev_c, mut prev_d, mut prev_e, mut diff) = chip.assign_first_row(layouter.namespace(|| "first row"))?;

        // sort
        for i in 0..5 {
            let (col_a, col_b, col_c, col_d, col_e) = chip.sort_row(
                layouter.namespace(|| "next row"),
                &prev_a,
                &prev_b,
                &prev_c,
                &prev_d,
                &prev_e
            )?;
            // update diff
            // based on property of bubble sort, the last i elements would be already sorted
            if i == 0 {
                let diff = chip.assign_diff(layouter.namespace(|| "diff"), &col_e, col_e);
            }
            if i == 1 {
                let diff = chip.assign_diff(layouter.namespace(|| "diff"), &col_e, col_d);
            }
            if i == 2 {
                let diff = chip.assign_diff(layouter.namespace(|| "diff"), &col_d, col_c);
            }
            if i == 3 {
                let diff = chip.assign_diff(layouter.namespace(|| "diff"), &col_c, col_b);
            }
            if i == 4 {
                let diff = chip.assign_diff(layouter.namespace(|| "diff"), &col_b, col_a);
            }
        }

        // check diff
        chip.assign(layouter.namespace(|| "value_check"), self.diff);

        Ok(())
    }
}

fn main() {
    let k = 5;

    let a = Fp::from(1);
    let b = Fp::from(7);
    let c = Fp::from(20);
    let d = Fp::from(15);
    let e = Fp::from(27);
    const RANGE: usize = 20;
    let mut public_input = vec![a, b, c, d, e];

    let circuit = MyCircuit<Fp, 20>(/* fields */);

    let prover = MockProver::run(k, &circuit, vec![public_input.clone()]).unwrap();
    prover.assert_satisfied();
    
}
