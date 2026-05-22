#![no_std]

use zkvm_interface::*;

const BN254_G1: zkvm_bn254_g1_point = zkvm_bn254_g1_point {
    data: hex_literal::hex!(
        "0000000000000000000000000000000000000000000000000000000000000001"
        "0000000000000000000000000000000000000000000000000000000000000002"
    ),
};

const BN254_G2: zkvm_bn254_g2_point = zkvm_bn254_g2_point {
    data: hex_literal::hex!(
        "198e9393920d483a7260bfb731fb5d25f1aa493335a9e71297e485b7aef312c2"
        "1800deef121f1e76426a00665e5c4479674322d4f75edadd46debd5cd992f6ed"
        "090689d0585ff075ec9e99ad690c3395bc4b313370b38ef355acdadcd122975b"
        "12c85ea5db8c6deb4aab71808dcb408fe3d1e7690c43d37b4ce6cc0166fa7daa"
    ),
};

const BLS12_381_G1: zkvm_bls12_381_g1_point = zkvm_bls12_381_g1_point {
    data: hex_literal::hex!(
        "17f1d3a73197d7942695638c4fa9ac0fc3688c4f9774b905a14e3a3f171bac586c55e83ff97a1aeffb3af00adb22c6bb"
        "08b3f481e3aaa0f1a09e30ed741d8ae4fcf5e095d5d00af600db18cb2c04b3edd03cc744a2888ae40caa232946c5e7e1"
    ),
};

const BLS12_381_G2: zkvm_bls12_381_g2_point = zkvm_bls12_381_g2_point {
    data: hex_literal::hex!(
        "024aa2b2f08f0a91260805272dc51051c6e47ad4fa403b02b4510b647ae3d1770bac0326a805bbefd48056c8c121bdb8"
        "13e02b6052719f607dacd3a088274f65596bd0d09920b61ab5da61bbdc7f5049334cf11213945d57e5ac7d055d042b7e"
        "0ce5d527727d6e118cc9cdc6da2e351aadfd9baa8cbdd3a76d429a695160d12c923ac9cc3baca289e193548608b82801"
        "0606c4a02ea734cc32acd2b02bc28b99cb3e287e85a763af267492ab572e99ab3f370d275cec1da1aaa9075ff05f79be"
    ),
};

#[unsafe(no_mangle)]
pub extern "C" fn main() {
    let mut max_base = [0u8; 1024];
    max_base[1023] = 3;
    let mut max_exp = [0u8; 1024];
    max_exp[1023] = 4;
    let mut max_mod = [0u8; 1024];
    max_mod[1023] = 7;
    let mut max_out = [0u8; 1024];
    assert_zero(unsafe {
        zkvm_modexp(
            max_base.as_ptr(),
            1024,
            max_exp.as_ptr(),
            1024,
            max_mod.as_ptr(),
            1024,
            max_out.as_mut_ptr(),
        )
    });

    let input = [0; 32];
    let mut output = zkvm_sha256_hash { data: [0u8; 32] };
    assert_zero(unsafe { zkvm_sha256(input.as_ptr(), input.len(), &mut output) });

    let input = [0; 32];
    let mut output = zkvm_keccak256_hash { data: [0u8; 32] };
    assert_zero(unsafe { zkvm_keccak256(input.as_ptr(), input.len(), &mut output) });

    let input = [0; 32];
    let mut output = zkvm_ripemd160_hash { data: [0u8; 32] };
    assert_zero(unsafe { zkvm_ripemd160(input.as_ptr(), input.len(), &mut output) });

    let mut output = zkvm_bn254_g1_point { data: [0u8; 64] };
    assert_zero(unsafe { zkvm_bn254_g1_add(&BN254_G1, &BN254_G1, &mut output) });

    let scalar = zkvm_bn254_scalar { data: [0u8; 32] };
    let mut output = zkvm_bn254_g1_point { data: [0u8; 64] };
    assert_zero(unsafe { zkvm_bn254_g1_mul(&BN254_G1, &scalar, &mut output) });

    let pair = zkvm_bn254_pairing_pair {
        g1: BN254_G1,
        g2: BN254_G2,
    };
    let pairs = [pair; 13];
    let mut output = false;
    assert_zero(unsafe { zkvm_bn254_pairing(pairs.as_ptr(), 13, &mut output) });

    let mut output = zkvm_blake2f_state { data: [0u8; 64] };
    let m = zkvm_blake2f_message { data: [0u8; 128] };
    let t = zkvm_blake2f_offset { data: [0u8; 16] };
    assert_zero(unsafe { zkvm_blake2f(0, &mut output, &m, &t, 0) });

    let commitment = zkvm_kzg_commitment { data: [0u8; 48] };
    let z = zkvm_kzg_field_element { data: [0u8; 32] };
    let y = zkvm_kzg_field_element { data: [0u8; 32] };
    let proof = zkvm_kzg_proof { data: [0u8; 48] };
    let mut output = false;
    assert_zero(unsafe { zkvm_kzg_point_eval(&commitment, &z, &y, &proof, &mut output) });

    let mut output = zkvm_bls12_381_g1_point { data: [0u8; 96] };
    assert_zero(unsafe { zkvm_bls12_g1_add(&BLS12_381_G1, &BLS12_381_G1, &mut output) });

    let pair = zkvm_bls12_381_g1_msm_pair {
        point: BLS12_381_G1,
        scalar: zkvm_bls12_381_scalar { data: [1u8; 32] },
    };
    let mut output = zkvm_bls12_381_g1_point { data: [0u8; 96] };
    assert_zero(unsafe { zkvm_bls12_g1_msm(&pair, 1, &mut output) });

    let mut output = zkvm_bls12_381_g2_point { data: [0u8; 192] };
    assert_zero(unsafe { zkvm_bls12_g2_add(&BLS12_381_G2, &BLS12_381_G2, &mut output) });

    let pair = zkvm_bls12_381_g2_msm_pair {
        point: BLS12_381_G2,
        scalar: zkvm_bls12_381_scalar { data: [0u8; 32] },
    };
    let mut output = zkvm_bls12_381_g2_point { data: [0u8; 192] };
    assert_zero(unsafe { zkvm_bls12_g2_msm(&pair, 1, &mut output) });

    let pair = zkvm_bls12_381_pairing_pair {
        g1: BLS12_381_G1,
        g2: BLS12_381_G2,
    };
    let pairs = [pair; 18];
    let mut output = false;
    assert_zero(unsafe { zkvm_bls12_pairing(pairs.as_ptr(), 18, &mut output) });

    let mut fp = zkvm_bls12_381_fp { data: [0u8; 48] };
    fp.data[47] = 1;
    let mut output = zkvm_bls12_381_g1_point { data: [0u8; 96] };
    assert_zero(unsafe { zkvm_bls12_map_fp_to_g1(&fp, &mut output) });

    let mut fp2 = zkvm_bls12_381_fp2 { data: [0u8; 96] };
    fp2.data[47] = 1;
    let mut output = zkvm_bls12_381_g2_point { data: [0u8; 192] };
    assert_zero(unsafe { zkvm_bls12_map_fp2_to_g2(&fp2, &mut output) });

    let msg = zkvm_secp256r1_hash { data: [0u8; 32] };
    let sig = zkvm_secp256r1_signature { data: [0u8; 64] };
    let pubkey = zkvm_secp256r1_pubkey { data: [0u8; 64] };
    let mut output = false;
    assert_zero(unsafe { zkvm_secp256r1_verify(&msg, &sig, &pubkey, &mut output) });

    let msg = zkvm_secp256k1_hash { data: [0u8; 32] };
    let sig = zkvm_secp256k1_signature { data: [0u8; 64] };
    let pubkey = zkvm_secp256k1_pubkey { data: [0u8; 64] };
    let mut output = false;
    assert_zero(unsafe { zkvm_secp256k1_verify(&msg, &sig, &pubkey, &mut output) });

    let msg = zkvm_secp256k1_hash {
        data: hex_literal::hex!("00000000000000000000000000000000000000000000000000000000deadbeef"),
    };
    let sig = zkvm_secp256k1_signature {
        data: hex_literal::hex!(
            "c6047f9441ed7d6d3045406e95c07cd85c778e4b8cef3ca7abac09b95c709ee5"
            "63023fca20f6beb69822a0374ae03e6c2e3bc725c6779e53d5d604dd1d8f2eea"
        ),
    };
    let mut output = zkvm_secp256k1_pubkey { data: [0u8; 64] };
    assert_zero(unsafe { zkvm_secp256k1_ecrecover(&msg, &sig, 0, &mut output) });

    let output = [0xFFu8; 32];
    unsafe { write_output(output.as_ptr(), output.len()) };
}

#[inline]
fn assert_zero(ret: i32) {
    if ret != 0 {
        panic!();
    }
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}
