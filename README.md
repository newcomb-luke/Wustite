# Wustite
A custom toy operating system written mostly in the Rust programming language. The name is based of of a mineral made up of iron oxide.

# Build Dependencies

* qemu
* mtools
* cargo

# Roadmap

- [ ] Bootloader
  - [ ] Stage 1
    - [ ] FAT12 header in boot sector
    - [ ] Disk read
    - [ ] Screen BIOS print routines
    - [ ] Minimal FAT12 read support
    - [ ] Second stage binary file loading
  - [ ] Stage 2
    - [ ] Disk read
    - [ ] BIOS print routines
    - [ ] Minimal FAT12 read support
      - [ ] Root directory lookup
      - [ ] FAT lookup
      - [ ] Read arbitrary file into memory
    - [ ] Minimal ELF executable loading
    - [ ] Protected mode entry
    - [ ] Long mode entry
      - [ ] Enabling PAE
      - [ ] Set 64-bit page table
      - [ ] Set long mode enable bit
      - [ ] Enable paging
      - [ ] Set up 64-bit GDT
    - [ ] BIOS memory map request
    - [ ] Providing boot info to kernel
- [ ] Kernel
  - [x] Bare bones printing
    - [x] VGA text buffer support
    - [x] Print macros
  - [x] Panic handling
  - [x] Interrupt handling
  - [x] Kernel Paging Setup
  - [ ] Memory Map Coalescence
  - [ ] Kernel Frame Allocation
  - [ ] Kernel Page Table Allocation
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
