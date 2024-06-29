use bnum::BIntD32;
use sha1_smol::Sha1;

pub fn sha1_notchian_hexdigest(value: &str) -> String {
    let mut hash = Sha1::new();
    hash.update(value.as_bytes());

    notchian_hexdigest(&hash.digest().bytes())
}

pub fn sha1_notchian_hexdigest_arr(values: &[&[u8]]) -> String {
    let mut hash = Sha1::new();

    for value in values {
        hash.update(value);
    }

    notchian_hexdigest(&hash.digest().bytes())
}

/// This implementation uses indeed BigInt (`bnum`),
/// but as far as my benchmarks have concluded: doing it manually (as a intermediate rust user) or using library is essentially the same speed.
/// TODO: possibly, implement it without a library.
pub fn notchian_hexdigest(value: &[u8; 20]) -> String {
    BIntD32::<5>::from_be_slice(value).unwrap().to_str_radix(16)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sha1_hex_digest() {
        assert_eq!(
            "4ed1f46bbe04bc756bcb17c0c7ce3e4632f06a48",
            sha1_notchian_hexdigest("Notch")
        );
        assert_eq!(
            "-7c9d5b0044c130109a5d7b5fb5c317c02b4e28c1",
            sha1_notchian_hexdigest("jeb_")
        );
        assert_eq!(
            "88e16a1019277b15d58faf0541e11910eb756f6",
            sha1_notchian_hexdigest("simon")
        )
    }
}
