use crate::{
    test_case::TestCase,
    test_utils::{load_yaml, Error},
};
use ethereum_consensus::deneb::{
    mainnet::Blob,
    polynomial_commitments::{
        blob_to_kzg_commitment, compute_blob_kzg_proof, compute_kzg_proof, kzg_settings_from_json,
        verify_blob_kzg_proof, verify_kzg_proof, Error as PolynomialCommitmentsError, FieldElement,
        KzgCommitment, KzgProof, KzgSettings, ProofAndEvaluation,
    },
    presets::TRUSTED_SETUP_JSON,
};

pub fn dispatch(test: &TestCase) -> Result<(), Error> {
    let kzg_settings = kzg_settings_from_json(TRUSTED_SETUP_JSON)?;

    match test.meta.handler.0.as_str() {
        "blob_to_kzg_commitment" => run_blob_to_kzg_commitment_test(test, &kzg_settings),
        "compute_kzg_proof" => run_compute_kzg_proof_test(test, &kzg_settings),
        "verify_kzg_proof" => run_verify_kzg_proof_test(test, &kzg_settings),
        "compute_blob_kzg_proof" => run_compute_blob_kzg_proof_test(test, &kzg_settings),
        "verify_blob_kzg_proof" => run_verify_blob_kzg_proof_test(test, &kzg_settings),
        "verify_blob_kzg_proof_batch" => run_verify_blob_kzg_proof_batch_test(test, &kzg_settings),
        handler => unreachable!("no tests for {handler}"),
    }
}

fn run_blob_to_kzg_commitment_test(
    test: &TestCase,
    kzg_settings: &KzgSettings,
) -> Result<(), Error> {
    let path = &test.data_path;
    // Load test case ----
    let path = path.to_string() + "/data.yaml";
    let test_data: serde_yaml::Value = load_yaml(&path);
    let input_yaml = test_data.get("input").unwrap();
    let blob_yaml = input_yaml.get("blob").unwrap();
    let output_yaml = test_data.get("output").unwrap();

    let input_blob_result: Result<Blob, _> = serde_yaml::from_value(blob_yaml.clone());
    let output_result: Result<Option<KzgCommitment>, _> =
        serde_yaml::from_value(output_yaml.clone());
    let output = output_result.unwrap();

    match (input_blob_result, output) {
        (Ok(blob), Some(expected_commmitment)) => {
            let kzg_commitment = blob_to_kzg_commitment(&blob, kzg_settings).unwrap();
            assert!(kzg_commitment == expected_commmitment);
            Ok(())
        }
        (Err(_), None) => {
            // Note: Expected state for invalid length blob
            Ok(())
        }
        (Ok(blob), None) => {
            let result = blob_to_kzg_commitment(&blob, kzg_settings);
            assert!(matches!(result, Err(PolynomialCommitmentsError::CKzg(..))));
            Ok(())
        }
        _ => unreachable!("not possible"),
    }
}

fn run_compute_kzg_proof_test(test: &TestCase, kzg_settings: &KzgSettings) -> Result<(), Error> {
    let path = &test.data_path;
    // Load test case ----
    let path = path.to_string() + "/data.yaml";
    let test_data: serde_yaml::Value = load_yaml(&path);
    let input_yaml = test_data.get("input").unwrap();
    let blob_yaml = input_yaml.get("blob").unwrap();
    let z_yaml = input_yaml.get("z").unwrap();
    let output_yaml = test_data.get("output").unwrap();

    let input_blob_result: Result<Blob, _> = serde_yaml::from_value(blob_yaml.clone());
    let input_z_result: Result<FieldElement, _> = serde_yaml::from_value(z_yaml.clone());
    let output_result: Result<Option<(KzgProof, FieldElement)>, _> =
        serde_yaml::from_value(output_yaml.clone());
    let output = output_result.unwrap();

    match (input_blob_result, input_z_result, output) {
        // Note: All maps for yaml file deserialized correctly
        (Ok(blob), Ok(z), Some(expected_proof_and_evaluation)) => {
            let proof_and_evaluation = compute_kzg_proof(&blob, &z, kzg_settings).unwrap();
            let expected_proof_and_evaluation = ProofAndEvaluation {
                proof: expected_proof_and_evaluation.0,
                evaluation: expected_proof_and_evaluation.1,
            };
            assert_eq!(proof_and_evaluation, expected_proof_and_evaluation);
            Ok(())
        }
        (Ok(blob), Ok(z), None) => {
            let result = compute_kzg_proof(&blob, &z, kzg_settings);
            assert!(matches!(result, Err(PolynomialCommitmentsError::CKzg(..))));
            Ok(())
        }
        (Err(_), Ok(_), None) => {
            // Note: Expected state for invalid length blob
            Ok(())
        }
        (Ok(_), Err(_), None) => {
            // Note: Expected state for invalid evaluation point
            Ok(())
        }
        _ => unreachable!("not possible"),
    }
}

fn run_verify_kzg_proof_test(test: &TestCase, kzg_settings: &KzgSettings) -> Result<(), Error> {
    let path = &test.data_path;
    // Load test case ----
    let path = path.to_string() + "/data.yaml";
    let test_data: serde_yaml::Value = load_yaml(&path);
    let input_yaml = test_data.get("input").unwrap();
    let commitment_yaml = input_yaml.get("commitment").unwrap();
    let z_yaml = input_yaml.get("z").unwrap();
    let y_yaml = input_yaml.get("y").unwrap();
    let proof_yaml = input_yaml.get("proof").unwrap();

    let output_yaml = test_data.get("output").unwrap();
    let output_result: Result<Option<bool>, _> = serde_yaml::from_value(output_yaml.clone());
    let output = output_result.unwrap();

    // Check the deserialization of each input
    let commitment = match serde_yaml::from_value(commitment_yaml.clone()) {
        Ok(commitment) => commitment,
        Err(_) => {
            assert!(output.is_none());
            return Ok(());
        }
    };

    let z = match serde_yaml::from_value(z_yaml.clone()) {
        Ok(z) => z,
        Err(_) => {
            assert!(output.is_none());
            return Ok(());
        }
    };

    let y = match serde_yaml::from_value(y_yaml.clone()) {
        Ok(y) => y,
        Err(_) => {
            assert!(output.is_none());
            return Ok(());
        }
    };

    let proof = match serde_yaml::from_value(proof_yaml.clone()) {
        Ok(proof) => proof,
        Err(_) => {
            assert!(output.is_none());
            return Ok(());
        }
    };

    let result = verify_kzg_proof(&commitment, &z, &y, &proof, kzg_settings);
    if let Some(expected_validity) = output {
        // some `output` was present, use inner value to determine if the spec code should succeed
        // or fail
        if expected_validity {
            assert!(result.is_ok());
            Ok(())
        } else {
            assert!(result.is_err());
            Ok(())
        }
    } else {
        // `output` is `null`, implying the spec code should always fail
        let result = verify_kzg_proof(&commitment, &z, &y, &proof, kzg_settings);
        assert!(result.is_err());
        Ok(())
    }
}

fn run_compute_blob_kzg_proof_test(
    test: &TestCase,
    kzg_settings: &KzgSettings,
) -> Result<(), Error> {
    let path = &test.data_path;
    // Load test case ----
    let path = path.to_string() + "/data.yaml";
    let test_data: serde_yaml::Value = load_yaml(&path);
    let input_yaml = test_data.get("input").unwrap();
    let blob_yaml = input_yaml.get("blob").unwrap();
    let commitment_yaml = input_yaml.get("commitment").unwrap();
    let output_yaml = test_data.get("output").unwrap();

    let input_blob_result: Result<Blob, _> = serde_yaml::from_value(blob_yaml.clone());
    let input_commitment_result: Result<KzgCommitment, _> =
        serde_yaml::from_value(commitment_yaml.clone());
    let output_result: Result<Option<KzgProof>, _> = serde_yaml::from_value(output_yaml.clone());
    let output = output_result.unwrap();

    match (input_blob_result, input_commitment_result, output) {
        (Ok(blob), Ok(commitment), Some(expected_proof)) => {
            let proof = compute_blob_kzg_proof(&blob, &commitment, kzg_settings).unwrap();
            assert_eq!(proof, expected_proof);
            Ok(())
        }
        (Ok(blob), Ok(commitment), None) => {
            let result = compute_blob_kzg_proof(&blob, &commitment, kzg_settings);
            assert!(matches!(result, Err(PolynomialCommitmentsError::CKzg(..))));
            Ok(())
        }
        (Err(_), Ok(_), None) => Ok(()),
        (Ok(_), Err(_), None) => Ok(()),
        (Err(_), Err(_), None) => Ok(()),
        _ => unreachable!("not possible"),
    }
}

fn run_verify_blob_kzg_proof_test(
    test: &TestCase,
    kzg_settings: &KzgSettings,
) -> Result<(), Error> {
    let path = &test.data_path;
    // Load test case ----
    let path = path.to_string() + "/data.yaml";
    let test_data: serde_yaml::Value = load_yaml(&path);
    let input_yaml = test_data.get("input").unwrap();
    let blob_yaml = input_yaml.get("blob").unwrap();
    let commitment_yaml = input_yaml.get("commitment").unwrap();
    let proof_yaml = input_yaml.get("proof").unwrap();
    let output_yaml = test_data.get("output").unwrap();

    let input_blob_result: Result<Blob, _> = serde_yaml::from_value(blob_yaml.clone());
    let input_commitment_result: Result<KzgCommitment, _> =
        serde_yaml::from_value(commitment_yaml.clone());
    let input_proof_result: Result<KzgProof, _> = serde_yaml::from_value(proof_yaml.clone());
    let output_result: Result<Option<bool>, _> = serde_yaml::from_value(output_yaml.clone());
    let output = output_result.unwrap();

    match (input_blob_result, input_commitment_result, input_proof_result, output) {
        (Ok(blob), Ok(commitment), Ok(proof), Some(_expected_validity)) => {
            let result = verify_blob_kzg_proof(&blob, &commitment, &proof, kzg_settings);
            // Note: expected_validity is never compared.  This is ok, right?
            assert!(result.is_ok());
            Ok(())
        }
        (Ok(blob), Ok(commitment), Ok(proof), None) => {
            let result = verify_blob_kzg_proof(&blob, &commitment, &proof, kzg_settings);
            assert!(matches!(result, Err(PolynomialCommitmentsError::CKzg(..))));
            Ok(())
        }
        // Note: "Err(_), Err(_), ..." and other variants are possible. Should i either match on
        // those cases or check for individual deserializations like
        // `run_verify_kzg_proof_test`?
        (Err(_), Ok(_), Ok(_), None) => Ok(()),
        (Ok(_), Err(_), Ok(_), None) => Ok(()),
        (Ok(_), Ok(_), Err(_), None) => Ok(()),

        _ => unreachable!("not possible"),
    }
}

fn run_verify_blob_kzg_proof_batch_test(
    test: &TestCase,
    _kzg_settings: &KzgSettings,
) -> Result<(), Error> {
    let _path = &test.data_path;

    todo!()
}
