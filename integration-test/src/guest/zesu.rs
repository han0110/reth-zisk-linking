pub mod input;
pub mod output;

use anyhow::Result;
use stateless::StatelessInput;

/// Returns input and expected output.
pub fn io(stateless_input: &StatelessInput, success: bool) -> Result<(Vec<u8>, Vec<u8>)> {
    let input = input::from_fixture(stateless_input, success)?;
    let expected_output = output::expected_output(&input, success);
    Ok((input.encode(), expected_output))
}
