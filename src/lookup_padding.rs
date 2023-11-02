/// A circuit to demonstrate we can do lookup on different rows in different columns
// mod bad_lookup;

use std::marker::PhantomData;

use halo2_proofs::{
    circuit::{Layouter, SimpleFloorPlanner, Value},
    plonk::*,
    poly::Rotation,
};
use halo2curves::ff::PrimeField;



#[derive(Clone)]
struct LookupConfig {
    a: Column<Advice>,
    s: Selector,
    t1: TableColumn,
    t2: Column<Advice>
}

struct LookupChip<F: PrimeField> {
    config: LookupConfig,
    _marker: PhantomData<F>,
}

impl<F: PrimeField> LookupChip<F> {
    fn construct(config: LookupConfig) -> Self {
        LookupChip {
            config,
            _marker: PhantomData,
        }
    }

    fn configure(meta: &mut ConstraintSystem<F>) -> LookupConfig {
        let a = meta.advice_column();
        let s = meta.complex_selector();
        let t1 = meta.lookup_table_column();
        let t2 = meta.advice_column();

        meta.enable_equality(a);

        let one = Expression::Constant(F::ONE);

        // meta.lookup("lookup", |meta| {
        //     let cur_a = meta.query_advice(a, Rotation::cur());
        //     let s = meta.query_selector(s);
        //     // we'll assgin (0,0) in t1,t2 table
        //     // so the default condition for other rows without need to lookup will also satisfy this constriant
        //     vec![(s.clone() * cur_a + ( one.clone() - s) * one.clone(), t1)]
        // });

        meta.lookup_any("lookup_any", |meta| {
            let cur_a = meta.query_advice(a, Rotation::cur());
            let table = meta.query_advice(t2, Rotation::cur());
            let s = meta.query_selector(s);
            // we'll assgin (0,0) in t1,t2 table
            // so the default condition for other rows without need to lookup will also satisfy this constriant
            vec![(s.clone() * cur_a + ( one.clone() - s) * one.clone(), table)]
        });

        LookupConfig { a,  s, t1, t2}
    }

    fn assign(
        &self,
        mut layouter: impl Layouter<F>,
        a_arr: &Vec<Value<F>>,
    ) -> Result<(), Error> {
        layouter.assign_region(
            || "a,b",
            |mut region| {
                for i in 0..a_arr.len() {
                    self.config.s.enable(&mut region, i)?;
                    region.assign_advice(|| "a col", self.config.a, i, || a_arr[i])?;
                }
                Ok(())
            },
        )?;

        layouter.assign_region(
            || "t2",
            |mut region| {
                region.assign_advice(|| "t2 col", self.config.t2, 0, || Value::known(F::from(1 as u64)))?;
                for i in 1..10 {
                    region.assign_advice(|| "t2 col", self.config.t2, i, || Value::known(F::from(i as u64)))?;
                }
                Ok(())
            },
        )?;

        layouter.assign_table(
            || "t1",
            |mut table| {
                table.assign_cell(
                    || "t1",
                    self.config.t1,
                    0,
                    || Value::known(F::from(1 as u64)),
                )?;
                for i in 1..10 {
                    table.assign_cell(
                        || "t1",
                        self.config.t1,
                        i,
                        || Value::known(F::from(i as u64)),
                    )?;
                }

                Ok(())
            },
        )?;

        Ok(())
    }
}

#[derive(Default)]
struct MyCircuit<F: PrimeField> {
    a: Vec<Value<F>>,
}

impl<F: PrimeField> Circuit<F> for MyCircuit<F> {
    type Config = LookupConfig;
    type FloorPlanner = SimpleFloorPlanner;

    fn without_witnesses(&self) -> Self {
        MyCircuit::default()
    }

    fn configure(meta: &mut ConstraintSystem<F>) -> Self::Config {
        LookupChip::configure(meta)
    }

    fn synthesize(&self, config: Self::Config, layouter: impl Layouter<F>) -> Result<(), Error> {
        let chip = LookupChip::<F>::construct(config);
        chip.assign(layouter, &self.a)
    }
}

#[cfg(test)]
mod tests {
    use halo2_proofs::{dev::MockProver, halo2curves::bn256::Fr as Fp};

    use super::*;
    #[test]
    fn test_lookup_on_different_rows() {
        //here in the table there is no 0, so we expect the circuit will not pass. 
        // However, when using lookup_any, the unassigned table will be padded with zero, so it will pass
        let k = 5;
        let a = [0, 1, 2, 3];
        let a = a.map(|v| Value::known(Fp::from(v))).to_vec();

        let circuit = MyCircuit { a };
        let prover = MockProver::run(k, &circuit, vec![]).unwrap();
        prover.assert_satisfied();
    }
}


