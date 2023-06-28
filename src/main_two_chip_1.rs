use std::marker::PhantomData;

use halo2_proofs::{
    arithmetic::Field,
    circuit::{AssignedCell, Chip, Layouter, Region, SimpleFloorPlanner, Value},
    plonk::{Advice, Circuit, Column, ConstraintSystem, Error, Instance, Selector},
    poly::Rotation,
};

// ANCHOR: field-instructions
#[derive(Clone)]
struct Number<F: Field>(AssignedCell<F, F>);

trait FieldInstructions<F: Field>: CompareInstructions<F> {
    type Num;

    // Loads a number into the circuit as a private input.
    fn load_private(
        &self,
        layouter: impl Layouter<F>,
        a: Value<F>,
    ) -> Result<<Self as FieldInstructions<F>>::Num, Error>;

    // Returns (a,b) if a > b, (b, a) o.w
    fn compare(
        &self,
        layouter: &mut impl Layouter<F>,
        a: <Self as FieldInstructions<F>>::Num,
        b: <Self as FieldInstructions<F>>::Num,
    ) -> Result<(<Self as FieldInstructions<F>>::Num, <Self as FieldInstructions<F>>::Num), Error>;

    // Exposes a number as a public input to the circuit.
    // fn expose_public(
    //     &self,
    //     layouter: impl Layouter<F>,
    //     num: <Self as FieldInstructions<F>>::Num,
    //     row: usize,
    // ) -> Result<(), Error>;
}
// ANCHOR_END: field-instructions

// ANCHOR: compare-instructions
trait CompareInstructions<F: Field>: Chip<F> {
    // Variable representing a number.
    type Num;

    fn compare(
        &self,
        layouter: impl Layouter<F>,
        a: Self::Num,
        b: Self::Num,
    ) -> Result<(Self::Num, Self::Num), Error>;
}
// ANCHOR_END: compare-instructions

// ANCHOR: field-config
// The top-level config that provides all necessary columns and permutations
// for the other configs.
#[derive(Clone, Debug)]
struct FieldConfig {
    advice: [Column<Advice>; 5],

    // instance: [Column<Instance>; 5],

    compare_config: CompareConfig,
}
// ANCHOR END: field-config

// ANCHOR: compare-config
#[derive(Clone, Debug)]
struct CompareConfig {
    advice: [Column<Advice>; 2],
    s_compare: Selector,
}
// ANCHOR_END: compare-config


// ANCHOR: field-chip
/// The top-level chip that will implement the `FieldInstructions`.
struct FieldChip<F: Field> {
    config: FieldConfig,
    _marker: PhantomData<F>,
}
// ANCHOR_END: field-chip

// ANCHOR: compare-chip
struct CompareChip<F: Field> {
    config: CompareConfig,
    _marker: PhantomData<F>,
}
// ANCHOR END: compare-chip

// ANCHOR: compare-chip-trait-impl
impl<F: Field> Chip<F> for CompareChip<F> {
    type Config = CompareConfig;
    type Loaded = ();

    fn config(&self) -> &Self::Config {
        &self.config
    }

    fn loaded(&self) -> &Self::Loaded {
        &()
    }
}
// ANCHOR END: compare-chip-trait-impl

// ANCHOR: compare-chip-impl
impl<F: Field> CompareChip<F> {
    fn construct(config: <Self as Chip<F>>::Config, _loaded: <Self as Chip<F>>::Loaded) -> Self {
        Self {
            config,
            _marker: PhantomData,
        }
    }

    fn configure(
        meta: &mut ConstraintSystem<F>,
        advice: [Column<Advice>; 2],
    ) -> <Self as Chip<F>>::Config {
        let s_comp = meta.selector();

        // Define our compareition gate!
        meta.create_gate("compare", |meta| {
            todo!()
        });

        CompareConfig { advice, s_comp }
    }
}
// ANCHOR END: compare-chip-impl

// ANCHOR: compare-instructions-impl
impl<F: Field> CompareInstructions<F> for FieldChip<F> {
    type Num = Number<F>;
    fn compare(
        &self,
        layouter: impl Layouter<F>,
        a: Self::Num,
        b: Self::Num,
    ) -> Result<Self::Num, Error> {
        let config = self.config().compare_config.clone();

        let compare_chip = CompareChip::<F>::construct(config, ());
        compare_chip.compare(layouter, a, b)
    }
}

impl<F: Field> CompareInstructions<F> for CompareChip<F> {
    type Num = Number<F>;

    fn compare(
        &self,
        mut layouter: impl Layouter<F>,
        a: Self::Num,
        b: Self::Num,
    ) -> Result<(Self::Num, Self::Num), Error> {
        let config = self.config();

        layouter.assign_region(
            || "compare",
            |mut region: Region<'_, F>| {
                todo!()
            },
        )
    }
}
// ANCHOR END: compare-instructions-impl



// ANCHOR: field-chip-trait-impl
impl<F: Field> Chip<F> for FieldChip<F> {
    type Config = FieldConfig;
    type Loaded = ();

    fn config(&self) -> &Self::Config {
        &self.config
    }

    fn loaded(&self) -> &Self::Loaded {
        &()
    }
}
// ANCHOR_END: field-chip-trait-impl

// ANCHOR: field-chip-impl
impl<F: Field> FieldChip<F> {
    fn construct(config: <Self as Chip<F>>::Config, _loaded: <Self as Chip<F>>::Loaded) -> Self {
        Self {
            config,
            _marker: PhantomData,
        }
    }

    fn configure(
        meta: &mut ConstraintSystem<F>,
        advice: [Column<Advice>; 5],
        instance: [Column<Instance>; 5],
    ) -> <Self as Chip<F>>::Config {
        let comp_config = CompareChip::configure(meta, advice);

        meta.enable_equality(instance[0]);
        meta.enable_equality(instance[1]);
        meta.enable_equality(instance[2]);
        meta.enable_equality(instance[3]);
        meta.enable_equality(instance[4]);

        FieldConfig {
            advice,
            instance,
            comp_config,
        }
    }
}
// ANCHOR_END: field-chip-impl

// ANCHOR: field-instructions-impl
impl<F: Field> FieldInstructions<F> for FieldChip<F> {
    type Num = Number<F>;

    fn load_private(
        &self,
        mut layouter: impl Layouter<F>,
        value: Value<F>,
    ) -> Result<<Self as FieldInstructions<F>>::Num, Error> {
        let config = self.config();

        layouter.assign_region(
            || "load private",
            |mut region| {
                region
                    .assign_advice(|| "private input", config.advice[0], 0, || value)
                    .map(Number)
            },
        )
    }
    fn sort(
        &self,
        layouter: &mut impl Layouter<F>,
        nums: [<Self as FieldInstructions<F>>::Num; 5],
    ) -> Result<[<Self as FieldInstructions<F>>::Num; 5], Error> {
        
    }

    // fn expose_public(
    //     &self,
    //     mut layouter: impl Layouter<F>,
    //     num: <Self as FieldInstructions<F>>::Num,
    //     row: usize,
    // ) -> Result<(), Error> {
    //     let config = self.config();

    //     layouter.constrain_instance(num.0.cell(), config.instance, row)
    // }
}
// ANCHOR_END: field-instructions-impl

// ANCHOR: circuit
#[derive(Default)]
struct MyCircuit<F: Field> {
    a: F,
    b: F,
    c: F,
    d: F,
    e: F,
}

impl<F: Field> Circuit<F> for MyCircuit<F> {
    type Config = FieldConfig;
    type FloorPlanner = SimpleFloorPlanner;
    #[cfg(feature = "circuit-params")]
    type Params = ();

    fn without_witnesses(&self) -> Self {
        Self::default()
    }

    fn configure(meta: &mut ConstraintSystem<F>) -> Self::Config {
        // We create the two advice columns that FieldChip uses for I/O.
        let advice = [meta.advice_column(), meta.advice_column(), meta.advice_column(), meta.advice_column(), meta.advice_column()];

        // // We also need an instance column to store public inputs.
        // let instance = [meta.instance_column(), meta.instance_column(), meta.instance_column(), meta.instance_column(), meta.instance_column()];

        FieldChip::configure(meta, advice)
    }

    fn synthesize(
        &self,
        config: Self::Config,
        mut layouter: impl Layouter<F>,
    ) -> Result<(), Error> {
        let field_chip = FieldChip::<F>::construct(config, ());

        // Load our private values into the circuit.
        let a = field_chip.load_private(layouter.namespace(|| "load a"), self.a)?;
        let b = field_chip.load_private(layouter.namespace(|| "load b"), self.b)?;
        let c = field_chip.load_private(layouter.namespace(|| "load c"), self.c)?;

        // Use `compare_and_mul` to get `d = (a + b) * c`.
        let d = field_chip.compare_and_mul(&mut layouter, a, b, c)?;

        // Expose the result as a public input to the circuit.
        field_chip.expose_public(layouter.namespace(|| "expose d"), d, 0)
    }
}
// ANCHOR_END: circuit

#[allow(clippy::many_single_char_names)]
fn main() {
    use halo2_proofs::dev::MockProver;

    // ANCHOR: test-circuit
    // The number of rows in our circuit cannot exceed 2^k. Since our example
    // circuit is very small, we can pick a very small value here.
    let k = 4;

    // Prepare the private and public inputs to the circuit!
    todo!()
}