//! Brief introduction to halo2.

#![deny(missing_docs)]

use halo2_proofs::circuit::{Layouter, SimpleFloorPlanner, Value};
use halo2_proofs::dev::MockProver;
use halo2_proofs::plonk::{Circuit, ConstraintSystem, Error};
use halo2curves::bn256::Fr;

use crate::circuit_config::StandardPlonkConfig;
use crate::gate::create_universal_plonk_gate;

/// Column setup.
mod circuit_config;
/// Universal PLONK gate.
mod gate;

/// Simple addition relation expressing: `a + b = c`, where:
/// - `a` is private input (aka advice, aka witness)
/// - `b` is public input (aka instance)
/// - `c` is constant (aka fixed)
#[derive(Default)]
struct SimpleRelation {
    /// Private summand.
    a: Fr,
    /// Public summand.
    b: Fr,
    /// Constant result.
    c: Fr,
}

impl Circuit<Fr> for SimpleRelation {
    // We are using our own column setup.
    type Config = StandardPlonkConfig;
    // We use the simplest layouting.
    type FloorPlanner = SimpleFloorPlanner;

    fn without_witnesses(&self) -> Self {
        Self::default()
    }

    // Setting up the table shape (its columns and available gates).
    fn configure(meta: &mut ConstraintSystem<Fr>) -> Self::Config {
        // ====== Columns ======
        let config = StandardPlonkConfig::new(meta);
        // ====== Gates   ======
        create_universal_plonk_gate(meta, &config);

        config
    }

    fn synthesize(
        &self,
        config: Self::Config,
        mut layouter: impl Layouter<Fr>,
    ) -> Result<(), Error> {
        layouter.assign_region(
            || "first row",
            |mut region| {
                region.assign_advice(
                    || "assign `a` as the left input",
                    config.left_input,
                    0,
                    || Value::known(self.a),
                )?;
                region.assign_fixed(
                    || "turn on selector for the left input",
                    config.q_left,
                    0,
                    || Value::known(Fr::one()),
                )?;
                region.assign_fixed(
                    || "assign constant result",
                    config.constant,
                    0,
                    || Value::known(self.c.neg()),
                )?;

                // `b` will be implicitly present in the `config.instance` column

                Ok(())
            },
        )
    }
}

fn main() {
    let relation_instance = SimpleRelation {
        a: Fr::from(2),
        b: Fr::from(3),
        c: Fr::from(5),
    };

    MockProver::run(3, &relation_instance, vec![vec![relation_instance.b]])
        .unwrap()
        .assert_satisfied();

    let faulty_relation_instance = SimpleRelation {
        a: Fr::from(2),
        b: Fr::from(2),
        c: Fr::from(5),
    };

    assert!(MockProver::run(
        3,
        &faulty_relation_instance,
        vec![vec![faulty_relation_instance.b]]
    )
    .unwrap()
    .verify()
    .is_err());
}
