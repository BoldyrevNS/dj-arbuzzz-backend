/// DFPWM (Dynamic Filter Pulse Width Modulation) encoder
/// Implementation based on the 1a specification used by ComputerCraft
pub struct DfpwmEncoder {
    charge: i32,
    strength: i32,
    previous_bit: bool,
}

impl DfpwmEncoder {
    pub fn new() -> Self {
        Self {
            charge: 0,
            strength: 0,
            previous_bit: false,
        }
    }

    /// Encode a slice of signed 8-bit PCM samples into DFPWM format
    /// Returns the encoded bytes (8 samples = 1 byte)
    pub fn encode(&mut self, input: &[i8], output: &mut Vec<u8>) {
        let mut byte = 0u8;
        let mut bit_pos = 0;

        for &sample in input {
            // Scale 8-bit sample to encoder's working range (-32768..32767)
            let target = (sample as i32) << 8; // * 256 для полного использования диапазона

            // Determine output bit
            let current_bit = target > self.charge;

            // Update charge
            let mut next_charge = self.charge;
            if current_bit {
                next_charge += self.strength;
            } else {
                next_charge -= self.strength;
            }

            // Clamp charge to valid range
            next_charge = next_charge.clamp(-32768, 32767);

            // Update strength (adaptive)
            let mut next_strength = self.strength;
            if current_bit == self.previous_bit {
                next_strength += 1;
            } else {
                next_strength -= 1;
            }
            next_strength = next_strength.clamp(0, 32767);

            // Apply exponential decay
            if next_strength > 0 {
                next_strength = ((next_strength as i64 * 2047) / 2048) as i32;
            }
            if next_strength < 0 {
                next_strength = 0;
            }

            // Ensure minimum strength
            if next_strength < 8 {
                next_strength = 8;
            }

            self.charge = next_charge;
            self.strength = next_strength;
            self.previous_bit = current_bit;

            // Pack bit into byte (LSB first)
            if current_bit {
                byte |= 1 << bit_pos;
            }

            bit_pos += 1;
            if bit_pos == 8 {
                output.push(byte);
                byte = 0;
                bit_pos = 0;
            }
        }

        // Flush remaining bits if any
        if bit_pos > 0 {
            output.push(byte);
        }
    }
}

impl Default for DfpwmEncoder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encoder_basic() {
        let mut encoder = DfpwmEncoder::new();
        let input = vec![0i8, 64, 127, 64, 0, -64, -128, -64];
        let mut output = Vec::new();

        encoder.encode(&input, &mut output);

        assert_eq!(output.len(), 1); // 8 samples = 1 byte
    }
}
