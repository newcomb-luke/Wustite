use block_device::impls::FileBlockDevice;
use mock_vfat32_driver::VFAT32Driver;

fn main() {
    let file = std::fs::File::open("../vfat32-core/test-fat32.img").unwrap();
    let block_device = FileBlockDevice::new(file);

    let mut driver = VFAT32Driver::new(block_device).unwrap();

    let path = "/mydir/";

    println!("Files in {}:", path);

    let dir = driver.open_dir(path).unwrap();
    for entry in driver.read_dir(dir).unwrap() {
        let entry = entry.unwrap();

        if entry.is_directory() {
            println!("<DIR> {}", entry.name());
        } else {
            println!("      {}", entry.name());
        }
    }

    let file = driver.open("/test.txt").unwrap();

    println!("File size: {}", file.size());

    let mut buffer = vec![0; file.size()];
    let read = driver.read_file(file, 0, &mut buffer).unwrap();
    buffer.truncate(read);

    let contents = String::from_utf8(buffer).unwrap();

    print!("{contents}");
}
