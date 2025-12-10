use esp_hal::{
    hmac::{Hmac, HmacPurpose, KeyId},
    rng::Rng,
};
use nb::block;

pub fn derive_sw_key(hmac: &mut Hmac, pin: &[u8], key_id: KeyId) -> [u8; 32] {
    let rng = Rng::new();
    let mut buffer = [0u8; 32];
    rng.read(buffer.as_mut_slice());

    let mut sw_key = [0u8; 32];
    hmac.init();
    block!(hmac.configure(HmacPurpose::ToUser, key_id)).expect("key purpose missmatch");
    block!(hmac.update(pin)).expect("it takes any message");
    block!(hmac.finalize(sw_key.as_mut_slice())).unwrap();

    sw_key
}
