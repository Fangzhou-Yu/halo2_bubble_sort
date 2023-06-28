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
    nums: [Column<Advice>; 5],
    a_0 : Column<Instance>,
    b_0 : Column<Instance>,
    c_0 : Column<Instance>,
    d_0 : Column<Instance>,
    e_0: Column<Instance>,
}


struct CompareConfig {
    p: Column<Advice>,
    q: Column<Advice>,
    s_comp: Selector,
}

struct CompareChip<F:FieldExt> {
    config: CompareConfig,
    _marker: PhantomData<F>,
}

impl<F: FieldExt> CompareChip<F> {
    fn construct(config: CompareConfig) -> Self {
        Self {
            config,
            _marker: PhantomData,
        }
    }

    fn configure(
        meta: &mut ConstraintSystem<F>,
    ) -> CompareConfig {
        // create columns
        let p = meta.advice_column();
        let q = meta.advice_column();
        let s_comp = meta.selector();

        // convert to bit

        meta.enable_equality(p);
        meta.enable_equality(q);

        // define the custom gate
        // let's assume p and q are all 8 bit for now
        meta.create_gate("p > q", |meta| {
            let p = meta.query_advice(p, Rotation::cur());
            let q = meta.query_advice(q, Rotation::cur());
            let s_comp = meta.query_selector(s_comp);
            // 假设a,b是x比特， 计算y =  2^x - (b - a), y是（x+1)比特的数。如果a < b，那么y<2^x, y的最高位是0。反之，y>=2^x，y的最高位是1
            todo!()
        });

        CompareConfig {
            p, 
            q, 
            s_comp
        }
    }

    fn compare(&self,
        mut layouter: impl Layouter<F>,
        p: F,
        q: F,
    ) -> Result<(Number<F>, Number<F>) ,Error> {
        if p > q {
            layouter.assign_region(
                || "compare",
                |mut region| {
                    self.config.s_comp.enable(&mut region, 0);
                    let p_cell = region.assign_advice(
                        || "a",
                        self.config.p,
                        0,
                        || Value::known(p),
                    ).map(Number)?;
    
                    let q_cell = region.assign_advice(
                        || "b",
                        self.config.q,
                        0,
                        || Value::known(q),
                    ).map(Number)?;

                    Ok((p_cell, q_cell))
                },
            )
        } else {
            layouter.assign_region(
                || "compare",
                |mut region| {
                    self.config.s_comp.enable(&mut region, 0);
                    let p_cell = region.assign_advice(
                        || "a",
                        self.config.p,
                        0,
                        || Value::known(q),
                    ).map(Number)?;
    
                    let q_cell = region.assign_advice(
                        || "b",
                        self.config.q,
                        0,
                        || Value::known(p),
                    ).map(Number)?;

                    Ok((p_cell, q_cell))
                },
            )
        }
    }
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
        let e = meta.advice_column();
        let a_0 = meta.instance_column();
        let b_0 = meta.instance_column();
        let c_0 = meta.instance_column();
        let d_0 = meta.instance_column();
        let e_0 = meta.instance_column();

        // enable permutation checks for the following columns
        // we use copy_advice to perform such checks
        meta.enable_equality(a);
        meta.enable_equality(b);
        meta.enable_equality(c);
        meta.enable_equality(d);
        meta.enable_equality(e);

        BubSortConfig {
            nums: [a, b, c, d, e],
            a_0,
            b_0,
            c_0,
            d_0,
            e_0,
        }
    }

    fn load_first_row(
        &self,
        mut layouter: impl Layouter<F>,
    ) -> Result<(Number<F>, Number<F>, Number<F>, Number<F>, Number<F>), Error> {
        // load first row
        layouter.assign_region(
            || "first row",
            |mut region| {

                let a_num = region.assign_advice_from_instance(
                    || "a",
                    self.config.a_0, // column a
                    0, // rotation
                    self.config.nums[0],
                    0,
                ).map(Number)?;
                
                let b_num = region.assign_advice_from_instance(
                    || "b",
                    self.config.b_0, // column b
                    0, 
                    self.config.nums[1],
                    0,
                ).map(Number)?;
                
                let c_num = region.assign_advice_from_instance(
                    || "c",
                    self.config.c_0, // column c
                    0, 
                    self.config.nums[2],
                    0,
                ).map(Number)?;

                let d_num = region.assign_advice_from_instance(
                    || "d",
                    self.config.d_0, // column c
                    0, // rotation
                    self.config.nums[3],
                    0,
                ).map(Number)?;

                let e_num = region.assign_advice_from_instance(
                    || "e",
                    self.config.e_0, // column c
                    0, // rotation
                    self.config.nums[4],
                    0,
                ).map(Number)?;

                Ok((a_num, b_num, c_num, d_num, e_num))
            },
        )
    }

    // this function performs a compare (and swap if necessary) on two designated cells
    fn load_row(
        &self,
        mut layouter: impl Layouter<F>,
        n: [Number<F>; 5],
    ) -> Result<(), Error> {
        Ok(layouter.assign_region(|| "load row", |mut region|Ok({
            n[0].0.copy_advice(||"copied", &mut region,self.config.nums[0],0,)?;
            n[1].0.copy_advice(||"copied", &mut region,self.config.nums[1],0,)?;
            n[2].0.copy_advice(||"copied", &mut region,self.config.nums[2],0,)?;
            n[3].0.copy_advice(||"copied", &mut region,self.config.nums[3],0,)?;
            n[4].0.copy_advice(||"copied", &mut region,self.config.nums[4],0,)?;
        }),
    )?)
    }


}

#[derive(Default)]
struct BubSortCircuit<F> {
    arr: [F; 5],
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
        let (mut prev_a, mut prev_b, mut prev_c, mut prev_d, mut prev_e) = chip.load_first_row(
            layouter.namespace(|| "first row"),
        )?;
        // rows in the table
        for _round in 1..5 {
            for idx in 0..4 {
                todo!()
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
    let e = Fp::from(128);

    // Instantiate the circuit with the private inputs.
    let circuit = BubSortCircuit{
        arr: [Fp::from(100),
        Fp::from(90),
        Fp::from(80),
        Fp::from(70),
        Fp::from(128)]
    };

    // Set circuit size
    let k = 6;

    // Given the correct public input, our circuit will verify.
    let prover = MockProver::run(k, &circuit, vec![vec![a], vec![b], vec![c], vec![d], vec![e]]).unwrap();
    prover.assert_satisfied();
}

