# Wustite
A custom toy operating system written mostly in the Rust programming language. The name is based of of a mineral made up of iron oxide.

# Build Dependencies

* qemu
* mtools
* cargo

# Roadmap

- [ ] Bootloader
	- [x] Read kernel from disk
	- [x] Read initramfs from disk
	- [x] Load kernel
		- [x] Read ELF files
		- [x] Perform relocations
	- [x] Get ACPI RSDP
	- [x] Write kernel boot info to memory
	- [x] Get memory map
	- [x] Initialize kernel paging
		- [x] Bootloader code identity mapping
		- [x] Physical memory mapping
		- [x] Higher-Half kernel mapping
		- [x] Kernel stack mapping
	- [x] Exit boot services
	- [ ] Set kernel stack as active stack
	- [ ] Jump to kernel
- [ ] Kernel
  - [x] Bare bones printing
    - [x] VGA text buffer support
    - [x] Print macros
  - [x] Panic handling
  - [x] Interrupt handling
  - [x] Kernel Paging Setup
  - [x] Memory Map Coalescence
  - [x] Kernel Frame Allocation
  - [x] Kernel Page Table Allocation
  - [ ] Disk drivers
  - [ ] FAT12 read only drivers
      - [ ] Root directory lookup
      - [ ] FAT lookup
      - [ ] Subdirectory lookup
  - [ ] FAT16 read/write drivers
  - [ ] Virtual file system
  - [ ] ELF executable loading
  - [ ] Print routines
    - [ ] Bitmap font rendering
