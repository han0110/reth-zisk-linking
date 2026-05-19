use alloc::{vec, vec::Vec};

use zkvm_interface::{
    zkvm_blake2f_message, zkvm_blake2f_offset, zkvm_blake2f_state, zkvm_bls12_381_fp,
    zkvm_bls12_381_fp2, zkvm_bls12_381_g1_msm_pair, zkvm_bls12_381_g1_point,
    zkvm_bls12_381_g2_msm_pair, zkvm_bls12_381_g2_point, zkvm_bls12_381_pairing_pair,
    zkvm_bn254_g1_point, zkvm_bn254_pairing_pair, zkvm_bn254_scalar, zkvm_bytes_32,
    zkvm_kzg_commitment, zkvm_kzg_field_element, zkvm_kzg_proof, zkvm_ripemd160_hash,
    zkvm_secp256k1_hash, zkvm_secp256k1_pubkey, zkvm_secp256k1_signature, zkvm_secp256r1_hash,
    zkvm_secp256r1_pubkey, zkvm_secp256r1_signature, zkvm_sha256_hash,
};

type Result<T> = core::result::Result<T, ()>;

pub fn zkvm_sha256(input: &[u8]) -> Result<[u8; 32]> {
    let mut output = zkvm_sha256_hash { data: [0; 32] };
    let ret = unsafe { zkvm_interface::zkvm_sha256(input.as_ptr(), input.len(), &mut output) };
    (ret == 0).then_some(output.data).ok_or(())
}

pub fn zkvm_keccak256(input: &[u8]) -> Result<[u8; 32]> {
    let mut output = zkvm_bytes_32 { data: [0; 32] };
    let ret = unsafe { zkvm_interface::zkvm_keccak256(input.as_ptr(), input.len(), &mut output) };
    (ret == 0).then_some(output.data).ok_or(())
}

pub fn zkvm_ripemd160(input: &[u8]) -> Result<[u8; 32]> {
    let mut output = zkvm_ripemd160_hash { data: [0; 32] };
    let ret = unsafe { zkvm_interface::zkvm_ripemd160(input.as_ptr(), input.len(), &mut output) };
    (ret == 0).then_some(output.data).ok_or(())
}

pub fn zkvm_modexp(base: &[u8], exp: &[u8], modulus: &[u8]) -> Result<Vec<u8>> {
    let mut output = vec![0u8; modulus.len()];
    let ret = unsafe {
        zkvm_interface::zkvm_modexp(
            base.as_ptr(),
            base.len(),
            exp.as_ptr(),
            exp.len(),
            modulus.as_ptr(),
            modulus.len(),
            output.as_mut_ptr(),
        )
    };
    (ret == 0).then_some(output).ok_or(())
}

pub fn zkvm_blake2f(
    rounds: u32,
    state: [u64; 8],
    message: &[u64; 16],
    offset: &[u64; 2],
    final_block: bool,
) -> Result<[u64; 8]> {
    let mut state_struct = zkvm_blake2f_state {
        data: unsafe { core::mem::transmute::<[u64; 8], [u8; 64]>(state) },
    };
    let message_struct = zkvm_blake2f_message {
        data: unsafe { core::mem::transmute::<[u64; 16], [u8; 128]>(*message) },
    };
    let mut offset_bytes = [0u8; 16];
    offset_bytes[..8].copy_from_slice(&offset[0].to_ne_bytes());
    offset_bytes[8..].copy_from_slice(&offset[1].to_ne_bytes());
    let offset_struct = zkvm_blake2f_offset { data: offset_bytes };

    let ret = unsafe {
        zkvm_interface::zkvm_blake2f(
            rounds,
            &mut state_struct,
            &message_struct,
            &offset_struct,
            final_block as u8,
        )
    };
    if ret != 0 {
        return Err(());
    }
    Ok(unsafe { core::mem::transmute::<[u8; 64], [u64; 8]>(state_struct.data) })
}

pub fn zkvm_secp256k1_ecrecover(msg: &[u8; 32], sig: &[u8; 64], recid: u8) -> Result<[u8; 64]> {
    let msg_struct = zkvm_secp256k1_hash { data: *msg };
    let sig_struct = zkvm_secp256k1_signature { data: *sig };
    let mut output = zkvm_secp256k1_pubkey { data: [0; 64] };
    let ret = unsafe {
        zkvm_interface::zkvm_secp256k1_ecrecover(&msg_struct, &sig_struct, recid, &mut output)
    };
    (ret == 0).then_some(output.data).ok_or(())
}

pub fn zkvm_secp256k1_verify(msg: &[u8; 32], sig: &[u8; 64], pubkey: &[u8; 64]) -> Result<bool> {
    let msg_struct = zkvm_secp256k1_hash { data: *msg };
    let sig_struct = zkvm_secp256k1_signature { data: *sig };
    let pubkey_struct = zkvm_secp256k1_pubkey { data: *pubkey };
    let mut verified = false;
    let ret = unsafe {
        zkvm_interface::zkvm_secp256k1_verify(
            &msg_struct,
            &sig_struct,
            &pubkey_struct,
            &mut verified,
        )
    };
    (ret == 0).then_some(verified).ok_or(())
}

pub fn zkvm_secp256r1_verify(msg: &[u8; 32], sig: &[u8; 64], pubkey: &[u8; 64]) -> Result<bool> {
    let msg_struct = zkvm_secp256r1_hash { data: *msg };
    let sig_struct = zkvm_secp256r1_signature { data: *sig };
    let pubkey_struct = zkvm_secp256r1_pubkey { data: *pubkey };
    let mut verified = false;
    let ret = unsafe {
        zkvm_interface::zkvm_secp256r1_verify(
            &msg_struct,
            &sig_struct,
            &pubkey_struct,
            &mut verified,
        )
    };
    (ret == 0).then_some(verified).ok_or(())
}

pub fn zkvm_bn254_g1_add(p1: &[u8; 64], p2: &[u8; 64]) -> Result<[u8; 64]> {
    let p1_struct = zkvm_bn254_g1_point { data: *p1 };
    let p2_struct = zkvm_bn254_g1_point { data: *p2 };
    let mut output = zkvm_bn254_g1_point { data: [0; 64] };
    let ret = unsafe { zkvm_interface::zkvm_bn254_g1_add(&p1_struct, &p2_struct, &mut output) };
    (ret == 0).then_some(output.data).ok_or(())
}

pub fn zkvm_bn254_g1_mul(point: &[u8; 64], scalar: &[u8; 32]) -> Result<[u8; 64]> {
    let point_struct = zkvm_bn254_g1_point { data: *point };
    let scalar_struct = zkvm_bn254_scalar { data: *scalar };
    let mut output = zkvm_bn254_g1_point { data: [0; 64] };
    let ret =
        unsafe { zkvm_interface::zkvm_bn254_g1_mul(&point_struct, &scalar_struct, &mut output) };
    (ret == 0).then_some(output.data).ok_or(())
}

pub fn zkvm_bn254_pairing(pairs: &[[u8; 192]]) -> Result<bool> {
    let mut verified = false;
    let ret = unsafe {
        zkvm_interface::zkvm_bn254_pairing(
            pairs.as_ptr() as *const zkvm_bn254_pairing_pair,
            pairs.len(),
            &mut verified,
        )
    };
    (ret == 0).then_some(verified).ok_or(())
}

pub fn zkvm_bls12_g1_add(p1: &[u8; 96], p2: &[u8; 96]) -> Result<[u8; 96]> {
    let p1_struct = zkvm_bls12_381_g1_point { data: *p1 };
    let p2_struct = zkvm_bls12_381_g1_point { data: *p2 };
    let mut output = zkvm_bls12_381_g1_point { data: [0; 96] };
    let ret = unsafe { zkvm_interface::zkvm_bls12_g1_add(&p1_struct, &p2_struct, &mut output) };
    (ret == 0).then_some(output.data).ok_or(())
}

pub fn zkvm_bls12_g1_msm(pairs: &[[u8; 128]]) -> Result<[u8; 96]> {
    let mut output = zkvm_bls12_381_g1_point { data: [0; 96] };
    let ret = unsafe {
        zkvm_interface::zkvm_bls12_g1_msm(
            pairs.as_ptr() as *const zkvm_bls12_381_g1_msm_pair,
            pairs.len(),
            &mut output,
        )
    };
    (ret == 0).then_some(output.data).ok_or(())
}

pub fn zkvm_bls12_g2_add(p1: &[u8; 192], p2: &[u8; 192]) -> Result<[u8; 192]> {
    let p1_struct = zkvm_bls12_381_g2_point { data: *p1 };
    let p2_struct = zkvm_bls12_381_g2_point { data: *p2 };
    let mut output = zkvm_bls12_381_g2_point { data: [0; 192] };
    let ret = unsafe { zkvm_interface::zkvm_bls12_g2_add(&p1_struct, &p2_struct, &mut output) };
    (ret == 0).then_some(output.data).ok_or(())
}

pub fn zkvm_bls12_g2_msm(pairs: &[[u8; 224]]) -> Result<[u8; 192]> {
    let mut output = zkvm_bls12_381_g2_point { data: [0; 192] };
    let ret = unsafe {
        zkvm_interface::zkvm_bls12_g2_msm(
            pairs.as_ptr() as *const zkvm_bls12_381_g2_msm_pair,
            pairs.len(),
            &mut output,
        )
    };
    (ret == 0).then_some(output.data).ok_or(())
}

pub fn zkvm_bls12_pairing(pairs: &[[u8; 288]]) -> Result<bool> {
    let mut verified = false;
    let ret = unsafe {
        zkvm_interface::zkvm_bls12_pairing(
            pairs.as_ptr() as *const zkvm_bls12_381_pairing_pair,
            pairs.len(),
            &mut verified,
        )
    };
    (ret == 0).then_some(verified).ok_or(())
}

pub fn zkvm_bls12_map_fp_to_g1(fp: &[u8; 48]) -> Result<[u8; 96]> {
    let fp_struct = zkvm_bls12_381_fp { data: *fp };
    let mut output = zkvm_bls12_381_g1_point { data: [0; 96] };
    let ret = unsafe { zkvm_interface::zkvm_bls12_map_fp_to_g1(&fp_struct, &mut output) };
    (ret == 0).then_some(output.data).ok_or(())
}

pub fn zkvm_bls12_map_fp2_to_g2(fp2: &[u8; 96]) -> Result<[u8; 192]> {
    let fp2_struct = zkvm_bls12_381_fp2 { data: *fp2 };
    let mut output = zkvm_bls12_381_g2_point { data: [0; 192] };
    let ret = unsafe { zkvm_interface::zkvm_bls12_map_fp2_to_g2(&fp2_struct, &mut output) };
    (ret == 0).then_some(output.data).ok_or(())
}

pub fn zkvm_kzg_point_eval(
    commitment: &[u8; 48],
    z: &[u8; 32],
    y: &[u8; 32],
    proof: &[u8; 48],
) -> Result<bool> {
    let commitment_struct = zkvm_kzg_commitment { data: *commitment };
    let z_struct = zkvm_kzg_field_element { data: *z };
    let y_struct = zkvm_kzg_field_element { data: *y };
    let proof_struct = zkvm_kzg_proof { data: *proof };
    let mut verified = false;
    let ret = unsafe {
        zkvm_interface::zkvm_kzg_point_eval(
            &commitment_struct,
            &z_struct,
            &y_struct,
            &proof_struct,
            &mut verified,
        )
    };
    (ret == 0).then_some(verified).ok_or(())
}
