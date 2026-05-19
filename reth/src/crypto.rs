use alloc::{boxed::Box, vec::Vec};

use alloy_consensus::crypto::{CryptoProvider, RecoveryError, install_default_provider};
use alloy_primitives::{Address, sync::Arc};
use libssz_merkle::Sha256Hasher;
use revm::precompile::{
    Crypto, PrecompileHalt,
    bls12_381::{G1Point, G1PointScalar, G2Point, G2PointScalar},
};

use crate::crypto::zkvm_interface::{
    zkvm_blake2f, zkvm_bls12_g1_add, zkvm_bls12_g1_msm, zkvm_bls12_g2_add, zkvm_bls12_g2_msm,
    zkvm_bls12_map_fp_to_g1, zkvm_bls12_map_fp2_to_g2, zkvm_bls12_pairing, zkvm_bn254_g1_add,
    zkvm_bn254_g1_mul, zkvm_bn254_pairing, zkvm_keccak256, zkvm_kzg_point_eval, zkvm_modexp,
    zkvm_ripemd160, zkvm_secp256k1_ecrecover, zkvm_secp256k1_verify, zkvm_secp256r1_verify,
    zkvm_sha256,
};

mod zkvm_interface;

pub fn install_crypto() {
    assert!(revm::install_crypto(ZkVMCrypto));
    let boxed: Box<dyn CryptoProvider> = Box::new(ZkVMCrypto);
    install_default_provider(Arc::from(boxed)).unwrap();
}

#[derive(Debug, Default)]
pub struct ZkVMCrypto;

impl Sha256Hasher for ZkVMCrypto {
    fn hash(&self, data: &[u8]) -> [u8; 32] {
        zkvm_sha256(data).unwrap()
    }
}

impl Crypto for ZkVMCrypto {
    #[inline]
    fn sha256(&self, input: &[u8]) -> [u8; 32] {
        zkvm_sha256(input).unwrap()
    }

    #[inline]
    fn blake2_compress(&self, rounds: u32, h: &mut [u64; 8], m: &[u64; 16], t: &[u64; 2], f: bool) {
        *h = zkvm_blake2f(rounds, *h, m, t, f).unwrap();
    }

    #[inline]
    fn ripemd160(&self, input: &[u8]) -> [u8; 32] {
        zkvm_ripemd160(input).unwrap()
    }

    #[inline]
    fn modexp(&self, base: &[u8], exp: &[u8], modulus: &[u8]) -> Result<Vec<u8>, PrecompileHalt> {
        zkvm_modexp(base, exp, modulus).map_err(|()| PrecompileHalt::other("zkvm_modexp failed"))
    }

    #[inline]
    fn secp256k1_ecrecover(
        &self,
        sig: &[u8; 64],
        recid: u8,
        msg: &[u8; 32],
    ) -> Result<[u8; 32], PrecompileHalt> {
        let pubkey = zkvm_secp256k1_ecrecover(msg, sig, recid)
            .map_err(|()| PrecompileHalt::Secp256k1RecoverFailed)?;
        let mut hash = zkvm_keccak256(&pubkey).unwrap();
        hash[..12].fill(0);
        Ok(hash)
    }

    #[inline]
    fn secp256r1_verify_signature(&self, msg: &[u8; 32], sig: &[u8; 64], pk: &[u8; 64]) -> bool {
        zkvm_secp256r1_verify(msg, sig, pk).unwrap_or(false)
    }

    #[inline]
    fn bn254_g1_add(&self, p1: &[u8], p2: &[u8]) -> Result<[u8; 64], PrecompileHalt> {
        let p1: &[u8; 64] = p1
            .try_into()
            .map_err(|_| PrecompileHalt::other("bn254 g1_add bad p1 len"))?;
        let p2: &[u8; 64] = p2
            .try_into()
            .map_err(|_| PrecompileHalt::other("bn254 g1_add bad p2 len"))?;
        zkvm_bn254_g1_add(p1, p2).map_err(|()| PrecompileHalt::other("bn254_g1_add failed"))
    }

    #[inline]
    fn bn254_g1_mul(&self, point: &[u8], scalar: &[u8]) -> Result<[u8; 64], PrecompileHalt> {
        let point: &[u8; 64] = point
            .try_into()
            .map_err(|_| PrecompileHalt::other("bn254 g1_mul bad point len"))?;
        let scalar: &[u8; 32] = scalar
            .try_into()
            .map_err(|_| PrecompileHalt::other("bn254 g1_mul bad scalar len"))?;
        zkvm_bn254_g1_mul(point, scalar).map_err(|()| PrecompileHalt::other("bn254_g1_mul failed"))
    }

    #[inline]
    fn bn254_pairing_check(&self, pairs: &[(&[u8], &[u8])]) -> Result<bool, PrecompileHalt> {
        let pairs_typed: Vec<[u8; 192]> = pairs
            .iter()
            .map(|(g1, g2)| {
                let mut buf = [0u8; 192];
                buf[..64].copy_from_slice(g1);
                buf[64..].copy_from_slice(g2);
                buf
            })
            .collect();
        zkvm_bn254_pairing(&pairs_typed).map_err(|()| PrecompileHalt::other("bn254_pairing failed"))
    }

    fn bls12_381_g1_add(&self, a: G1Point, b: G1Point) -> Result<[u8; 96], PrecompileHalt> {
        let mut a_bytes = [0u8; 96];
        a_bytes[..48].copy_from_slice(&a.0);
        a_bytes[48..].copy_from_slice(&a.1);
        let mut b_bytes = [0u8; 96];
        b_bytes[..48].copy_from_slice(&b.0);
        b_bytes[48..].copy_from_slice(&b.1);
        zkvm_bls12_g1_add(&a_bytes, &b_bytes).map_err(|()| PrecompileHalt::Bls12381G1NotOnCurve)
    }

    fn bls12_381_g1_msm(
        &self,
        pairs: &mut dyn Iterator<Item = Result<G1PointScalar, PrecompileHalt>>,
    ) -> Result<[u8; 96], PrecompileHalt> {
        let mut pairs_typed: Vec<[u8; 128]> = Vec::new();
        for pair in pairs {
            let (point, scalar) = pair?;
            let mut buf = [0u8; 128];
            buf[..48].copy_from_slice(&point.0);
            buf[48..96].copy_from_slice(&point.1);
            buf[96..].copy_from_slice(&scalar);
            pairs_typed.push(buf);
        }
        zkvm_bls12_g1_msm(&pairs_typed).map_err(|()| PrecompileHalt::Bls12381G1NotOnCurve)
    }

    fn bls12_381_g2_add(&self, a: G2Point, b: G2Point) -> Result<[u8; 192], PrecompileHalt> {
        let mut a_bytes = [0u8; 192];
        a_bytes[..48].copy_from_slice(&a.0);
        a_bytes[48..96].copy_from_slice(&a.1);
        a_bytes[96..144].copy_from_slice(&a.2);
        a_bytes[144..].copy_from_slice(&a.3);
        let mut b_bytes = [0u8; 192];
        b_bytes[..48].copy_from_slice(&b.0);
        b_bytes[48..96].copy_from_slice(&b.1);
        b_bytes[96..144].copy_from_slice(&b.2);
        b_bytes[144..].copy_from_slice(&b.3);
        zkvm_bls12_g2_add(&a_bytes, &b_bytes).map_err(|()| PrecompileHalt::Bls12381G2NotOnCurve)
    }

    fn bls12_381_g2_msm(
        &self,
        pairs: &mut dyn Iterator<Item = Result<G2PointScalar, PrecompileHalt>>,
    ) -> Result<[u8; 192], PrecompileHalt> {
        let mut pairs_typed: Vec<[u8; 224]> = Vec::new();
        for pair in pairs {
            let (point, scalar) = pair?;
            let mut buf = [0u8; 224];
            buf[..48].copy_from_slice(&point.0);
            buf[48..96].copy_from_slice(&point.1);
            buf[96..144].copy_from_slice(&point.2);
            buf[144..192].copy_from_slice(&point.3);
            buf[192..].copy_from_slice(&scalar);
            pairs_typed.push(buf);
        }
        zkvm_bls12_g2_msm(&pairs_typed).map_err(|()| PrecompileHalt::Bls12381G2NotOnCurve)
    }

    fn bls12_381_pairing_check(
        &self,
        pairs: &[(G1Point, G2Point)],
    ) -> Result<bool, PrecompileHalt> {
        let pairs_typed: Vec<[u8; 288]> = pairs
            .iter()
            .map(|(g1, g2)| {
                let mut buf = [0u8; 288];
                buf[..48].copy_from_slice(&g1.0);
                buf[48..96].copy_from_slice(&g1.1);
                buf[96..144].copy_from_slice(&g2.0);
                buf[144..192].copy_from_slice(&g2.1);
                buf[192..240].copy_from_slice(&g2.2);
                buf[240..].copy_from_slice(&g2.3);
                buf
            })
            .collect();
        zkvm_bls12_pairing(&pairs_typed).map_err(|()| PrecompileHalt::Bls12381G1NotOnCurve)
    }

    fn bls12_381_fp_to_g1(&self, fp: &[u8; 48]) -> Result<[u8; 96], PrecompileHalt> {
        zkvm_bls12_map_fp_to_g1(fp).map_err(|()| PrecompileHalt::other("bls12_381_fp_to_g1 failed"))
    }

    fn bls12_381_fp2_to_g2(&self, fp2: ([u8; 48], [u8; 48])) -> Result<[u8; 192], PrecompileHalt> {
        let mut fp2_bytes = [0u8; 96];
        fp2_bytes[..48].copy_from_slice(&fp2.0);
        fp2_bytes[48..].copy_from_slice(&fp2.1);
        zkvm_bls12_map_fp2_to_g2(&fp2_bytes)
            .map_err(|()| PrecompileHalt::other("bls12_381_fp2_to_g2 failed"))
    }

    #[inline]
    fn verify_kzg_proof(
        &self,
        z: &[u8; 32],
        y: &[u8; 32],
        commitment: &[u8; 48],
        proof: &[u8; 48],
    ) -> Result<(), PrecompileHalt> {
        match zkvm_kzg_point_eval(commitment, z, y, proof) {
            Ok(true) => Ok(()),
            _ => Err(PrecompileHalt::BlobVerifyKzgProofFailed),
        }
    }
}

impl CryptoProvider for ZkVMCrypto {
    fn recover_signer_unchecked(
        &self,
        sig: &[u8; 65],
        msg: &[u8; 32],
    ) -> Result<Address, RecoveryError> {
        let sig_bytes: &[u8; 64] = sig[..64].try_into().unwrap();
        let recid = sig[64];
        let pubkey =
            zkvm_secp256k1_ecrecover(msg, sig_bytes, recid).map_err(|()| RecoveryError::new())?;
        let hash = zkvm_keccak256(&pubkey).unwrap();
        Ok(Address::from_slice(&hash[12..]))
    }

    fn verify_and_compute_signer_unchecked(
        &self,
        pubkey: &[u8; 65],
        sig: &[u8; 64],
        msg: &[u8; 32],
    ) -> Result<Address, RecoveryError> {
        let pk_bytes: &[u8; 64] = pubkey[1..].try_into().unwrap();
        match zkvm_secp256k1_verify(msg, sig, pk_bytes) {
            Ok(true) => {
                let hash = zkvm_keccak256(pk_bytes).unwrap();
                Ok(Address::from_slice(&hash[12..]))
            }
            _ => Err(RecoveryError::new()),
        }
    }
}
