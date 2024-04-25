use crate::circuit_config::StandardPlonkConfig;
use halo2_proofs::arithmetic::Field;
use halo2_proofs::plonk::ConstraintSystem;
use halo2_proofs::poly::Rotation;

/// Register the universal gate in the constraint system using registered columns.
pub fn create_universal_plonk_gate<Fr: Field>(
    meta: &mut ConstraintSystem<Fr>,
    config: &StandardPlonkConfig,
) {
    meta.create_gate(
        "q_left·left_input + q_right·right_input + q_output·output + q_product·left_input·right_input + constant + instance = 0",
        |meta| {
            // Query the advice columns. `Rotation` is the relative offset from the current table row.
            let [left_input, right_input, output] = [config.left_input, config.right_input, config.output].map(|column| meta.query_advice(column, Rotation::cur()));

            // Query the selectors.
            let [q_left, q_right, q_product, q_output] = [config.q_left, config.q_right, config.q_product, config.q_output].map(|column| meta.query_fixed(column));

            // Query the constant column.
            let constant = meta.query_fixed(config.constant);

            // Query the instance column.
            let instance = meta.query_instance(config.instance, Rotation::cur());

            // Return a list of expressions that should be equal to zero.
            vec![
                q_left * left_input.clone() +
                q_right * right_input.clone() +
                q_product * left_input * right_input +
                q_output * output +
                constant +
                instance
            ]
        },
    );
}
