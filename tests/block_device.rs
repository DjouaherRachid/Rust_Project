use rust_project::device::block_device::{
    BlockDevice,
    BlockDeviceError,
    MemoryBlockDevice,
};

#[test]
fn read_single_byte() {
    let data = [0xAA, 0xBB, 0xCC];
    let device = MemoryBlockDevice::new(&data);

    let mut buf = [0u8; 1];
    device.read_at(1, &mut buf).unwrap();

    assert_eq!(buf[0], 0xBB);
}

#[test]
fn read_multiple_bytes() {
    let data = [1, 2, 3, 4, 5];
    let device = MemoryBlockDevice::new(&data);

    let mut buf = [0u8; 3];
    device.read_at(1, &mut buf).unwrap();

    assert_eq!(buf, [2, 3, 4]);
}

#[test]
fn read_at_zero() {
    let data = [9, 8, 7];
    let device = MemoryBlockDevice::new(&data);

    let mut buf = [0u8; 2];
    device.read_at(0, &mut buf).unwrap();

    assert_eq!(buf, [9, 8]);
}

#[test]
fn read_out_of_bounds() {
    let data = [1, 2, 3];
    let device = MemoryBlockDevice::new(&data);

    let mut buf = [0u8; 4];
    let result = device.read_at(0, &mut buf);

    assert_eq!(result, Err(BlockDeviceError::OutOfBounds));
}

#[test]
fn read_with_offset_out_of_bounds() {
    let data = [1, 2, 3];
    let device = MemoryBlockDevice::new(&data);

    let mut buf = [0u8; 1];
    let result = device.read_at(10, &mut buf);

    assert_eq!(result, Err(BlockDeviceError::OutOfBounds));
}
