# Wustite
A custom toy operating system written mostly in the Rust programming language. The name is based of of a mineral made up of iron oxide.

# Build Dependencies

* nasm
* qemu
* mtools
* gcc

# Roadmap

- [x] Bootloader
  - [x] Stage 1
    - [x] FAT12 header in boot sector
    - [x] Disk read
    - [x] Screen BIOS print routines
    - [x] Minimal FAT12 read support
    - [x] Second stage binary file loading
  - [x] Stage 2
    - [x] Disk read
    - [x] BIOS print routines
    - [x] Minimal FAT12 read support
      - [x] Root directory lookup
      - [x] FAT lookup
      - [x] Read arbitrary file into memory
    - [x] Minimal ELF executable loading
    - [x] Protected mode entry
    - [x] Long mode entry
      - [x] Enabling PAE
      - [x] Set 64-bit page table
      - [x] Set long mode enable bit
      - [x] Enable paging
      - [x] Set up 64-bit GDT
    - [x] BIOS memory map request
    - [x] Providing boot info to kernel
- [ ] Kernel
  - [x] Bare bones printing
    - [x] VGA text buffer support
    - [x] Print macros
  - [x] Panic handling
  - [ ] Interrupt handling
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
