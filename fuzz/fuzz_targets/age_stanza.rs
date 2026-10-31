#![no_main]
use age_core::format::{read, write};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok((leftover, stanza)) = read::age_stanza(data) {
        let mut buf = Vec::with_capacity(data.len());
        write::age_stanza(&mut buf, stanza.tag, &stanza.args, &stanza.body())
            .expect("can write to Vec");
        assert_eq!(buf, &data[0..data.len() - leftover.len()]);
    }
});
