use block_device::impls::FileBlockDevice;
use mock_vfat32_driver::VFAT32Driver;

fn main() {
    let file = std::fs::File::open("../vfat32-core/test-fat32.img").unwrap();
    let block_device = FileBlockDevice::new(file);

    let mut driver = VFAT32Driver::new(block_device).unwrap();

    driver.find_root_entry("kernel.o").unwrap();
}
