use std::marker::PhantomData;
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
    // used to check value after swapping, we could add another set to check unswapped values but
    // I am not doing that here
    s_a: Selector,
    s_b: Selector,
    s_c: Selector,
    s_d: Selector,
    s_unchanged_a: Selector,
    s_unchanged_b: Selector,
    s_unchanged_c: Selector,
    s_unchanged_d: Selector,
}

// need a chip to perform compare
// we need decompose and compare
// fn decompose 
// fn compare
// do: get F: 1111111, subtract Value
// use value api


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
        let s_a = meta.complex_selector();
        let s_b = meta.complex_selector();
        let s_c = meta.complex_selector();
        let s_d = meta.complex_selector();
        let s_unchanged_a = meta.selector();
        let s_unchanged_b = meta.selector();
        let s_unchanged_c = meta.selector();
        let s_unchanged_d = meta.selector();

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
            // selector for swapped columns
            let s_a = meta.query_selector(s_a);
            let s_b = meta.query_selector(s_b);
            let s_c = meta.query_selector(s_c);
            let s_d = meta.query_selector(s_d);
            // unchanged selector
            let s_a2 = meta.query_selector(s_unchanged_a);
            let s_b2 = meta.query_selector(s_unchanged_b);
            let s_c2 = meta.query_selector(s_unchanged_c);
            let s_d2 = meta.query_selector(s_unchanged_d);

            let a_cur = meta.query_advice(a, Rotation::cur());
            let b_cur = meta.query_advice(b, Rotation::cur());
            let c_cur = meta.query_advice(c, Rotation::cur());
            let d_cur = meta.query_advice(d, Rotation::cur());

            let a_next = meta.query_advice(a, Rotation::next());
            let b_next = meta.query_advice(b, Rotation::next());
            let c_next = meta.query_advice(c, Rotation::next());
            let d_next = meta.query_advice(d, Rotation::next());

            // first row is all unchanged
            // swapped x and y
            // => x_prev - x + y_prev - y = 0
            vec![s_a2*(a_next.clone() - a_cur.clone()),
                s_b2*(b_next.clone() - b_cur.clone()),
                s_c2*(c_next.clone() - c_cur.clone()),
                s_d2*(d_next.clone() - d_cur.clone()),
                s_a*(a_next - a_cur) + s_b*(b_next - b_cur) + s_c*(c_next - c_cur) + s_d*(d_next - d_cur)]
        });

        BubSortConfig {
            a, b, c, d, 
            a_0, b_0, c_0, d_0, 
            s_a, s_b, s_c, s_d,
            s_unchanged_a, s_unchanged_b, s_unchanged_c, s_unchanged_d,
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

    // this function performs a compare (and swap if necessary) on two designated cells
    fn load_row(
        &self,
        mut layouter: impl Layouter<F>,
        prev_a: &AssignedCell<F,F>,
        prev_b: &AssignedCell<F,F>,
        prev_c: &AssignedCell<F,F>,
        prev_d: &AssignedCell<F,F>,
        idx: usize,
    ) -> Result<(AssignedCell<F,F>, AssignedCell<F,F>, AssignedCell<F,F>, AssignedCell<F,F>), Error> {
        // do one compare & swap and assign a new region
        let mut arr = [prev_a.value().clone(), prev_b.value().clone(), prev_c.value().clone(), prev_d.value().clone()];

        // compare
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
        let a_val = arr[0];
        let b_val = arr[1];
        let c_val = arr[2];
        let d_val = arr[3];

        if is_true {
            Ok(layouter.assign_region(
                || "swap and assign to next row",
                |mut region| {               
                    // enable only unchanged
                    if idx == 0 {
                        self.config.s_a.enable(&mut region, 0)?;
                        self.config.s_b.enable(&mut region, 0)?;
                        self.config.s_unchanged_c.enable(&mut region, 0)?;
                        self.config.s_unchanged_d.enable(&mut region, 0)?;

                    } else if idx == 1{
                        self.config.s_b.enable(&mut region, 0)?;
                        self.config.s_c.enable(&mut region, 0)?;
                        self.config.s_unchanged_a.enable(&mut region, 0)?;
                        self.config.s_unchanged_d.enable(&mut region, 0)?;

                    } else if idx == 2{
                        self.config.s_c.enable(&mut region, 0)?;
                        self.config.s_d.enable(&mut region, 0)?;
                        self.config.s_unchanged_a.enable(&mut region, 0)?;
                        self.config.s_unchanged_b.enable(&mut region, 0)?;

                    }
                    // row 1 
                    prev_a.copy_advice(||"copied", &mut region,self.config.a,0,)?;
                    prev_b.copy_advice(||"copied", &mut region,self.config.b,0,)?;
                    prev_c.copy_advice(||"copied", &mut region,self.config.c,0,)?;
                    prev_d.copy_advice(||"copied", &mut region,self.config.d,0,)?;
                    
                    // row 2
                    // a
                    let a_cell = region.assign_advice(
                        || "a",
                        self.config.a,
                        1,
                        || a_val.copied(),
                    )?;
    
                    // b
                    let b_cell = region.assign_advice(
                        || "b",
                        self.config.b,
                        1,
                        || b_val.copied(),
                    )?;
    
                    // c
                    let c_cell = region.assign_advice(
                        || "c",
                        self.config.c,
                        1,
                        || c_val.copied(),
                    )?;
    
                    // d
                    let d_cell = region.assign_advice(
                        || "d",
                        self.config.d,
                        1,
                        || d_val.copied(),
                    )?;
    
                    
                    Ok((a_cell, b_cell, c_cell, d_cell))
                },
            )?)
        } else {
            // no swap happens
            Ok(layouter.assign_region(
                || "no swap",
                |mut region| { 
                    // copy unchanged over           
                    self.config.s_unchanged_a.enable(&mut region, 0)?;
                    self.config.s_unchanged_b.enable(&mut region, 0)?;
                    self.config.s_unchanged_c.enable(&mut region, 0)?;
                    self.config.s_unchanged_d.enable(&mut region, 0)?;
                    prev_a.copy_advice(||"copied", &mut region,self.config.a,0,)?;
                    prev_b.copy_advice(||"copied", &mut region,self.config.b,0,)?;
                    prev_c.copy_advice(||"copied", &mut region,self.config.c,0,)?;
                    prev_d.copy_advice(||"copied", &mut region,self.config.d,0,)?;


                    // next row in the region

                    // a
                    let a_cell = region.assign_advice(
                        || "a",
                        self.config.a,
                        1,
                        || a_val.copied(),
                    )?;
    
                    // b
                    let b_cell = region.assign_advice(
                        || "b",
                        self.config.b,
                        1,
                        || b_val.copied(),
                    )?;
    
                    // c
                    let c_cell = region.assign_advice(
                        || "c",
                        self.config.c,
                        1,
                        || c_val.copied(),
                    )?;
    
                    // d
                    let d_cell = region.assign_advice(
                        || "d",
                        self.config.d,
                        1,
                        || d_val.copied(),
                    )?;
    
                    
                    Ok((a_cell, b_cell, c_cell, d_cell))
                },
            )?)
        }
            
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
        for _round in 1..4 {
            for idx in 0..3 {
                let (a, b, c, d) = chip.load_row(
                    layouter.namespace(|| "next row"),
                    &prev_a,
                    &prev_b,
                    &prev_c,
                    &prev_d,
                    idx,
                )?;
                prev_a = a;
                prev_b = b;
                prev_c = c;
                prev_d = d;
            }
        }

        Ok(())
    }
}


fn main() {
    use halo2_proofs::{dev::MockProver, pasta::Fp};
    // Prepare the private and public inputs to the circuit!
    let a = Fp::from(100);
    let b = Fp::from(90);
    let c = Fp::from(80);
    let d = Fp::from(70);

    // Instantiate the circuit with the private inputs.
    let circuit = BubSortCircuit(PhantomData);

    // Set circuit size
    let k = 6;

    // Given the correct public input, our circuit will verify.
    let prover = MockProver::run(k, &circuit, vec![vec![a], vec![b], vec![c], vec![d]]).unwrap();
    prover.assert_satisfied();
}
