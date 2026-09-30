// Copyright (c) 2023 Intel Corporation
//
// SPDX-License-Identifier: Apache-2.0
//

use anyhow::{anyhow, Context, Result};
use base64::{
    engine::general_purpose::{STANDARD, URL_SAFE_NO_PAD},
    Engine,
};

/// Parse `runtime_data` from the evidence query string.
///
/// When `encoding` is `base64`, `runtime_data` is decoded as URL-safe base64 (no padding).
/// Otherwise the raw UTF-8 bytes of the query value are used (legacy behavior).
///
/// Cohere fork compatibility, kept until no deployed TNG needs it (TNG <= v0.7.0):
/// - `base64` also accepts standard base64 with padding, which the fork served
///   before upstream added this parameter. The URL-safe and standard alphabets
///   differ only in `-_` vs `+/`, so a value is tried as URL-safe first.
/// - `hex` is accepted.
pub fn decode_runtime_data(runtime_data: &str, encoding: Option<&str>) -> Result<Vec<u8>> {
    match encoding {
        None => Ok(runtime_data.as_bytes().to_vec()),
        Some("base64") => URL_SAFE_NO_PAD
            .decode(runtime_data)
            .or_else(|_| STANDARD.decode(runtime_data))
            .context("invalid base64 runtime_data"),
        Some("hex") => hex::decode(runtime_data).context("invalid hex runtime_data"),
        Some(other) => Err(anyhow!("unsupported runtime_data encoding: {other}")),
    }
}

pub fn split_nth_slash(url: &str, n: usize) -> Option<(&str, &str)> {
    let mut split_pos = None;
    let mut splits = url.match_indices('/');

    for _ in 0..n {
        split_pos = splits.next();
    }

    split_pos.map(|(idx, pat)| url.split_at(idx + pat.len() - 1))
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case("hello", None, true, Some(b"hello".as_slice()))]
    #[case("cmVwb3J0", Some("base64"), true, Some(b"report".as_slice()))]
    #[case("!!!", Some("base64"), false, None)]
    // Cohere fork compatibility: standard base64 with padding and the `+/` alphabet.
    #[case("cmVwb3J0Cg==", Some("base64"), true, Some(b"report\n".as_slice()))]
    #[case("+/8=", Some("base64"), true, Some([0xfb, 0xff].as_slice()))]
    // URL-safe alphabet without padding still decodes the upstream way.
    #[case("-_8", Some("base64"), true, Some([0xfb, 0xff].as_slice()))]
    // Cohere fork compatibility: hex.
    #[case("7265706f7274", Some("hex"), true, Some(b"report".as_slice()))]
    #[case("zz", Some("hex"), false, None)]
    #[case("data", Some("utf16"), false, None)]
    fn test_decode_runtime_data(
        #[case] runtime_data: &str,
        #[case] encoding: Option<&str>,
        #[case] expect_ok: bool,
        #[case] expected: Option<&[u8]>,
    ) {
        let result = decode_runtime_data(runtime_data, encoding);
        if expect_ok {
            assert_eq!(result.unwrap(), expected.unwrap());
        } else {
            assert!(result.is_err());
        }
    }

    #[test]
    fn test_split_nth_slash() {
        let url_path = "/cdh/resource/default/";
        assert_eq!(split_nth_slash(url_path, 0), None);
        assert_eq!(
            split_nth_slash(url_path, 1),
            Some(("", "/cdh/resource/default/"))
        );
        assert_eq!(
            split_nth_slash(url_path, 2),
            Some(("/cdh", "/resource/default/"))
        );
        assert_eq!(
            split_nth_slash(url_path, 3),
            Some(("/cdh/resource", "/default/"))
        );
        assert_eq!(
            split_nth_slash(url_path, 4),
            Some(("/cdh/resource/default", "/"))
        );
        assert_eq!(split_nth_slash(url_path, 5), None);
    }
}
