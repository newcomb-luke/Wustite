# Wustite
A custom toy operating system written mostly in the Rust programming language. The name is based of of a mineral made up of iron oxide.

# Build Dependencies

* qemu
* mtools
* cargo

# Roadmap

- [x] Bootloader
	- [x] Read kernel from disk
	- [x] Read initramfs from disk
	- [x] Load kernel
		- [x] Read ELF files
		- [x] Perform relocations
	- [x] Get ACPI RSDP
	- [x] Write kernel boot info to memory
	- [x] Get memory map
    - [x] Memory Map Coalescence
	- [x] Initialize kernel paging
		- [x] Bootloader code identity mapping
		- [x] Physical memory mapping
		- [x] Higher-Half kernel mapping
		- [x] Kernel stack mapping
	- [x] Exit boot services
	- [x] Set kernel stack as active stack
	- [x] Jump to kernel
- [ ] Kernel
  - [ ] Bare bones printing
    - [x] Serial port 0 driver
    - [ ] Print macros
  - [ ] Panic handling
  - [ ] Interrupt handling
  - [ ] Kernel Paging Setup
  - [ ] Kernel Frame Allocation
  - [ ] Kernel Page Table Allocation
  - [ ] Disk drivers
  - [ ] FAT32 read-only drivers
      - [ ] Root directory lookup
      - [ ] FAT lookup
      - [ ] Subdirectory lookup
  - [ ] FAT32 read/write drivers
  - [ ] EXT4 read-only drivers
      - [ ] Read superblock
      - [ ] Root directory reading
      - [ ] Directory reading
      - [ ] Inode reading
      - [ ] File reading
  - [ ] EXT4 read/write drivers
  - [ ] Virtual file system
  - [ ] ELF executable loading
  - [ ] PCI
    - [ ] PCI device enumeration
    - [ ] PCI host bridge enumeration
  - [ ] Graphics
    - [ ] VMWARE SVGA card driver
    - [ ] Bitmap font rendering
