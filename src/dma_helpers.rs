use esp_hal::{
    Blocking,
    dma::{DmaError, SimpleMem2Mem},
};

pub fn write_msg_to_mem<'d>(
    m2m: &mut SimpleMem2Mem<'d, Blocking>,
    rx: &mut [u8],
    tx: &mut [u8],
    msg: &[u8],
) -> Result<(), DmaError> {
    // Fill the buffer predictably
    for (i, byte) in tx.iter_mut().enumerate() {
        *byte = (i % 256) as u8;
    }
    tx[..msg.len()].copy_from_slice(msg);

    let transfer = m2m.start_transfer(rx, tx)?;
    transfer.wait()
}
