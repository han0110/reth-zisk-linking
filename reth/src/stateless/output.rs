pub const STATELESS_VALIDATOR_OUTPUT_SIZE: usize = 33;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct StatelessValidatorOutput {
    pub new_payload_request_root: [u8; 32],
    pub successful_block_validation: bool,
}

impl StatelessValidatorOutput {
    pub fn new(new_payload_request_root: [u8; 32], successful_block_validation: bool) -> Self {
        Self {
            new_payload_request_root,
            successful_block_validation,
        }
    }

    pub fn serialize(&self) -> [u8; STATELESS_VALIDATOR_OUTPUT_SIZE] {
        let mut buf = [0u8; STATELESS_VALIDATOR_OUTPUT_SIZE];
        buf[..32].copy_from_slice(&self.new_payload_request_root);
        buf[32] = self.successful_block_validation as u8;
        buf
    }
}
