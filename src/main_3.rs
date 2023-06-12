/// Referenced https://github.com/icemelon/halo2-tutorial/blob/master/src/example2.rs
/// Implement a circuit for bubblesort, I hope it works

use std::{marker::PhantomData};

use halo2_proofs::{
    arithmetic::FieldExt,
    circuit::{AssignedCell, Layouter, SimpleFloorPlanner},
    plonk::{Advice, Circuit, Column, ConstraintSystem, Error, Instance, Selector},
    poly::Rotation,
};

#[derive(Clone)]
struct Number<F: FieldExt>(AssignedCell<F, F>);

#[derive(Debug, Clone)]
struct FiboConfig {
    a: Column<Advice>,
    b: Column<Advice>,
    c: Column<Advice>,
    d: Column<Advice>,
    selector: Selector,
    instance: Column<Instance>,
}

struct FiboChip<F: FieldExt> {
    config: FiboConfig,
    _marker: PhantomData<F>,
}

impl<F: FieldExt> FiboChip<F> {
    fn construct(config: FiboConfig) -> Self {
        Self {
            config,
            _marker: PhantomData,
        }
    }

    fn configure(
        meta: &mut ConstraintSystem<F>,
        advice: [Column<Advice>; 4],
        selector: Selector,
        instance: Column<Instance>,
    ) -> FiboConfig {
        let a = advice[0];
        let b = advice[1];
        let c = advice[2];
        let d = advice[3];

        meta.enable_equality(b);
        meta.enable_equality(instance);

        meta.create_gate("bubble-sort-block", |meta| {
            let s = meta.query_selector(selector);
            let a1 = meta.query_advice(a, Rotation::prev());
            let b1 = meta.query_advice(b, Rotation::prev());
            let c1 = meta.query_advice(c, Rotation::prev());
            let d1 = meta.query_advice(d, Rotation::prev());
            let a2 = meta.query_advice(a, Rotation::cur());
            let b2 = meta.query_advice(b, Rotation::cur());
            let c2 = meta.query_advice(c, Rotation::cur());
            let d2 = meta.query_advice(d, Rotation::cur());
            vec![
                s * (a2+ b2 + c2 +d2 -a1 - b1 -c1 - d1),
            ]
        });

        FiboConfig {
            a, b, c, d, selector, instance,
        }
    }

    fn load(
        &self,
        mut layouter: impl Layouter<F>,
        a: F,
        b: F,
        c: F,
        d: F,
        nrows: usize,
    ) -> Result<(Number<F>, Number<F>), Error> {
        layouter.assign_region(
            || "entire block",
            |mut region| {
                // assign first row
                let mut a = region.assign_advice(
                    || "a",
                    self.config.a,
                    0,
                    || Ok(a),
                ).map(Number)?;

                let mut b = region.assign_advice(
                    || "b",
                    self.config.b,
                    0,
                    || Ok(b),
                ).map(Number)?;
                // println!("[0] a = {:?} b = {:?}", a.0, b.0);

                let mut c = region.assign_advice(
                    || "c",
                    self.config.c,
                    0,
                    || Ok(c),
                ).map(Number)?;

                let mut d = region.assign_advice(
                    || "d",
                    self.config.d,
                    0,
                    || Ok(d),
                ).map(Number)?;

                let mut arr = [a,b,c,d];

                for idx in 1..nrows {
                    self.config.selector.enable(&mut region, idx)?;
                    
                    for i in 0..3{
                        if arr[i].0.value() > arr[i+1].0.value() {
                            arr.swap(arr[i],arr[i+1]);
                        } 
                    }

                    
                }

                Ok((a, b, c, d))
            },
        )
    }

    fn expose_public(
        &self,
        mut layouter: impl Layouter<F>,
        num: Number<F>,
        row: usize,
    ) -> Result<(), Error> {
        layouter.constrain_instance(num.0.cell(), self.config.instance, row)
    }
}

#[derive(Default)]
struct FiboCircuit<F> {
    a: F,
    b: F,
    num: usize,
}

impl<F: FieldExt> Circuit<F> for FiboCircuit<F> {
    type Config = FiboConfig;
    type FloorPlanner = SimpleFloorPlanner;

    fn without_witnesses(&self) -> Self {
        Self::default()
    }

    fn configure(meta: &mut ConstraintSystem<F>) -> Self::Config {
        let advice = [
            meta.advice_column(),
            meta.advice_column(),
        ];
        let selector = meta.selector();
        let instance = meta.instance_column();
        FiboChip::configure(meta, advice, selector, instance)
    }

    fn synthesize(
        &self,
        config: Self::Config,
        mut layouter: impl Layouter<F>
    ) -> Result<(), Error> {
        let chip = FiboChip::construct(config);
        let nrows = (self.num + 1) / 2;
        let (_, b) = chip.load(
            layouter.namespace(|| "block"),
            self.a,
            self.b,
            nrows)?;
        chip.expose_public(layouter.namespace(|| "expose b"), b, 0)?;
        Ok(())
    }
}

fn get_fibo_seq(a: u64, b: u64, num: usize) -> Vec<u64> {
    let mut seq = vec![0; num];
    seq[0] = a;
    seq[1] = b;
    for i in 2..num {
        seq[i] = seq[i - 1] + seq[i - 2];
    }
    seq
}

fn main() {
    use halo2_proofs::{dev::MockProver, pairing::bn256::Fr as Fp};

    // Prepare the private and public inputs to the circuit!
    let num = 16;
    let seq = get_fibo_seq(1, 1, num);
    let res = Fp::from(seq[num - 1]);
    println!("{:?}", seq);

    // Instantiate the circuit with the private inputs.
    let circuit = FiboCircuit {
        a: Fp::from(seq[0]),
        b: Fp::from(seq[1]),
        num,
    };

    // Arrange the public input. We expose the multiplication result in row 0
    // of the instance column, so we position it there in our public inputs.
    let mut public_inputs = vec![res];

    // Set circuit size
    let k = 4;

    // Given the correct public input, our circuit will verify.
    let prover = MockProver::run(k, &circuit, vec![public_inputs.clone()]).unwrap();
    assert_eq!(prover.verify(), Ok(()));

    // If we try some other public input, the proof will fail!
    public_inputs[0] += Fp::one();
    let prover = MockProver::run(k, &circuit, vec![public_inputs]).unwrap();
    assert!(prover.verify().is_err());
}
