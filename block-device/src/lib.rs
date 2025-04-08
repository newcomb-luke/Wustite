#![no_std]

#[cfg(feature = "std")]
extern crate std;

pub mod impls;

pub trait BlockDevice {
    type Error;

    fn block_size(&self) -> u64;

    fn read_block(&mut self, lba: u64, buffer: &mut [u8]) -> Result<(), Self::Error>;

    fn write_block(&mut self, lba: u64, buffer: &[u8]) -> Result<(), Self::Error>;
}

pub trait OffsetRead {
    type Error;

    fn read(&mut self, offset: u64, buffer: &mut [u8]) -> Result<(), Self::Error>;

    fn write(&mut self, offset: u64, buffer: &[u8]) -> Result<(), Self::Error>;
}

// impl<T: BlockDevice> OffsetRead for T {
//     type Error = T::Error;
// 
//     fn read(&mut self, offset: u64, buffer: &mut [u8]) -> Result<(), Self::Error> {
//         todo!()
//     }
// 
//     fn write(&mut self, offset: u64, buffer: &[u8]) -> Result<(), Self::Error> {
//         todo!()
//     }
// }