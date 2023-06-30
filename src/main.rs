use std::ops::{Mul, Div, Index};
use std::marker::PhantomData;
use halo2_proofs::{
    arithmetic::{FieldExt, BaseExt},
    circuit::*,
    plonk::*,
    poly::Rotation,
};
use num_bigint::BigUint;
   use halo2_proofs::pairing::bn256::Fr;


// borrowed from 
pub fn field_to_bn<F: BaseExt>(f: &F) -> BigUint {
    let mut bytes: Vec<u8> = Vec::new();
    f.write(&mut bytes).unwrap();
    BigUint::from_bytes_le(&bytes[..])
}


#[derive(Clone, Debug)]
struct Limb<F: FieldExt> {
    cell: Option<AssignedCell<F, F>>,
    value: F
}

impl<F: FieldExt> Limb<F> {
    fn new(cell: Option<AssignedCell<F, F>>, value: F) -> Self {
        Limb { cell, value }
    }
}

#[derive(Clone,Debug)]
struct CompareConfig {
    lhs: Column<Advice>,
    rhs: Column<Advice>,
    result: Column<Advice>,
    cond: Column<Advice>,
    s_comp: Selector,
}

#[derive(Clone,Debug)]
struct MainConfig{
    nums: [Column<Advice>; 5],

    compareconfig: CompareConfig,
}

struct CompareChip<F: FieldExt> {
    config: CompareConfig,
    _marker: PhantomData<F>,
}

impl<F: FieldExt> CompareChip<F>{
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
        let rhs = meta.advice_column();
        let lhs = meta.advice_column();
        let cond = meta.advice_column();
        let result = meta.advice_column();
        let s_comp = meta.selector();

        meta.enable_equality(rhs);
        meta.enable_equality(lhs);
        meta.enable_equality(result);
        meta.enable_equality(cond); // not sure whether this is necessary

        // define the custom gate
        meta.create_gate("cond", |meta| {
            let lhs = meta.query_advice(lhs, Rotation::cur());
            let rhs = meta.query_advice(rhs, Rotation::cur());
            let cond = meta.query_advice(cond, Rotation::cur());
            let result = meta.query_advice(result, Rotation::cur());
            let s_comp = meta.query_selector(s_comp);

            // make sure cond is 1 when result is lhs, cond is 0 when result is rhs
            vec![s_comp*(lhs.clone() - result - cond.clone()*lhs + cond*rhs)] 
        });

        CompareConfig {
            lhs, rhs, result, cond, s_comp
        }
    }


    /// todo
    /// implement a decompose number method using the config we have
    /// constrain the value of decomposition in some way, maybe not
    fn decompose_limb(
        &self,
        region: &mut Region<F>,
        limb: &Limb<F>,
        limbsize: usize,
    ) -> Vec<Limb<F>> {
        // borrowed from zkWASM-host-circuits
        // ignoring constraining in this part for now
        let mut limbs = vec![];
        let mut bool_limbs = field_to_bn(&limb.value).to_radix_le(2);
        bool_limbs.truncate(limbsize);
        bool_limbs.resize_with(limbsize, | | 0);
        bool_limbs.reverse();
        let mut v = F::zero();
        for i in 0..(limbsize/4) {
            let l0 = F::from_u128(bool_limbs[4*i] as u128);
            let l1 = F::from_u128(bool_limbs[4*i+1] as u128);
            let l2 = F::from_u128(bool_limbs[4*i+2] as u128);
            let l3 = F::from_u128(bool_limbs[4*i+3] as u128);
            let v_next = v * F::from_u128(16u128)
                + l0 * F::from_u128(8u128)
                + l1 * F::from_u128(4u128)
                + l2 * F::from_u128(2u128)
                + l3 * F::from_u128(1u128);
                let l = [
                    Limb::new(None, l0),
                    Limb::new(None, l1),
                    Limb::new(None, l2),
                    Limb::new(None, l3),
                    Limb::new(None, v),
                    Limb::new(None, v_next),
                ];
            limbs.append(&mut l.to_vec()[0..4].to_vec());
            v = v_next;
        }
        limbs
    }
    

    // cond就是 y = 2^x - (b-a) 的第一位数字
    fn select(
        &self, 
        region: &mut Region<F>,
        arr: &mut [Limb<F>; 5],
        offset: &mut usize,
        idx: usize,
    ) -> Result<[Limb<F>; 5], Error>{
        let lhs = arr[idx].clone();
        let rhs = arr[idx+1].clone();
        // 先把a和b都转换成二进制
        // a和b做减法然后看diff+2^x

        // let x = u32::pow(2,8);
        // let y = F::from(x) - (rhs.value - lhs.value);
        // now check first digit of y

        let x = F::from(256);
        // check if y has 1 on first digit: 1xxxxxxx
        let y = x - (lhs.value - rhs.value);
        // this limits largest ele of nums to be 127
        let y_bin = self.decompose_limb(region,  &Limb::new(None, y), 8); 
        let cond = y_bin[0].clone();
        let (result_1, result_2) = if cond.value == F::zero() {(lhs.clone(), rhs.clone())} else {(rhs.clone(), lhs.clone())};
        // constrain condition
        region.assign_advice(|| "lhs", self.config.lhs, *offset, || Ok(lhs.value))?;
        region.assign_advice(|| "rhs", self.config.rhs, *offset, || Ok(rhs.value))?;
        region.assign_advice(|| "cond", self.config.cond, *offset, || Ok(cond.value))?;
        result_1.clone().cell.unwrap().copy_advice(|| "result", region, self.config.result, *offset)?;        
        self.config.s_comp.enable(region, *offset)?;

        arr[idx] = result_1.clone();
        arr[idx+1] = result_2.clone();
        Ok(arr.clone())
    }
}

struct MainChip<F: FieldExt>{
    config: MainConfig,
    _marker: PhantomData<F>,
}

impl<F: FieldExt> MainChip<F> {
    fn construct(config: MainConfig) -> Self {
        Self {
            config,
            _marker: PhantomData,
        }
    }

    fn configure(
        meta: &mut ConstraintSystem<F>,
    ) -> MainConfig {
        let compareconfig = CompareChip::configure(meta);
        // create columns
        let a = meta.advice_column();
        let b = meta.advice_column();
        let c = meta.advice_column();
        let d = meta.advice_column();
        let e = meta.advice_column();

        // enable permutation checks for the following columns
        // we use copy_advice to perform such checks
        meta.enable_equality(a);
        meta.enable_equality(b);
        meta.enable_equality(c);
        meta.enable_equality(d);
        meta.enable_equality(e);

        MainConfig {
            nums: [a, b, c, d, e],
            compareconfig,
        }
    }

    fn load_first_row(
        &self, 
        mut layouter: impl Layouter<F>,
        a: F,
        b: F,
        c: F,
        d: F,
        e: F
    ) -> Result<(Limb<F>, Limb<F>, Limb<F>, Limb<F>, Limb<F>), Error> {
            layouter.assign_region(||"first row", |mut region| {
                let a_cell = region.assign_advice(
                    ||"a_0",
                    self.config.nums[0],
                    0,
                    || Ok(a),
                )?;

                let b_cell = region.assign_advice(
                    ||"b_0",
                    self.config.nums[1],
                    0,
                    || Ok(b),
                )?;

                let c_cell = region.assign_advice(
                    ||"c_0",
                    self.config.nums[2],
                    0,
                    || Ok(c),
                )?;

                let d_cell = region.assign_advice(
                    ||"d_0",
                    self.config.nums[3],
                    0,
                    || Ok(d),
                )?;

                let e_cell = region.assign_advice(
                    ||"e_0",
                    self.config.nums[4],
                    0,
                    || Ok(e),
                )?;

                Ok((Limb {cell: Some(a_cell), value: a},
                    Limb {cell: Some(b_cell), value: b},
                    Limb {cell: Some(c_cell), value: c},
                    Limb {cell: Some(d_cell), value: d},
                    Limb {cell: Some(e_cell), value: e},
                ))
            },)
        }

    fn load_row(
        &self, 
        region: &mut Region<F>,
        a: &Limb<F>,
        b: &Limb<F>,
        c: &Limb<F>,
        d: &Limb<F>,
        e: &Limb<F>,
        offset: &mut usize,
    ) -> Result<(), Error> {
        // use copy advice to do permutation checks
        a.cell.clone().unwrap().copy_advice(||"copied", region,self.config.nums[0],*offset,)?;
        b.cell.clone().unwrap().copy_advice(||"copied", region,self.config.nums[1],*offset,)?;
        c.cell.clone().unwrap().copy_advice(||"copied", region,self.config.nums[2],*offset,)?;
        d.cell.clone().unwrap().copy_advice(||"copied", region,self.config.nums[3],*offset,)?;
        e.cell.clone().unwrap().copy_advice(||"copied", region,self.config.nums[4],*offset,)?;
        Ok(())
    }
}

#[derive(Debug, Default)]
struct BubSortCircuit<F> {
    arr: [F; 5],
}

impl<F: FieldExt> Circuit<F> for BubSortCircuit<F> {
    type Config = MainConfig;
    type FloorPlanner = SimpleFloorPlanner;

    fn without_witnesses(&self) -> Self {
        Self::default()
    }

    fn configure(meta: &mut ConstraintSystem<F>) -> Self::Config {
        MainChip::configure(meta)
    }

    fn synthesize(
        &self,
        config: Self::Config,
        mut layouter: impl Layouter<F>
    ) -> Result<(), Error> {
        let chip = MainChip::construct(config.clone());
        let comp_chip = CompareChip::construct(config.clone().compareconfig);
        // println!("{:?}", self.arr);
        let ( prev_a,  prev_b,  prev_c,  prev_d,  prev_e) = chip.load_first_row(
            layouter.namespace(|| "first row"),
            self.arr[0],
            self.arr[1],
            self.arr[2],
            self.arr[3],
            self.arr[4],
        )?;
        // rows in the table
        let mut v = [prev_a, prev_b, prev_c, prev_d, prev_e];
        layouter.assign_region(|| "row", |mut region|{
            for round in 0..5 {
                let mut offset = round;
                for idx in 0..4 {
                    let idx: usize = idx as usize;
                    let v: [Limb<F>; 5] = comp_chip.select(&mut region, &mut v, &mut offset, idx)?;
                    chip.load_row(&mut region, &v[0], &v[1], &v[2], &v[3], &v[4], &mut offset)?;
                    for element in &v {
                        println!("{:?}", element.value);
                    }
                }
            }
            Ok(())
        },)
    }

}



fn main(){
    use halo2_proofs::dev::MockProver;
    // Prepare the private and public inputs to the circuit!
    let a = Fr::from(100);
    let b = Fr::from(90);
    let c = Fr::from(80);
    let d = Fr::from(70);
    let e = Fr::from(66);

    // Instantiate the circuit with the private inputs.
    let circuit = BubSortCircuit{
        arr: [a, b, c, d, e],
    };

    // Set circuit size
    let k = 18;

    // Given the correct public input, our circuit will verify.
    let prover = MockProver::run(k, &circuit, vec![]).unwrap();
    assert_eq!(prover.verify(), Ok(()));
}