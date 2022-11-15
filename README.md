# Wustite
A custom toy operating system written mostly in the Rust programming language. The name is based of of a mineral made up of iron oxide.

# Build Dependencies

* nasm
* qemu
* mtools
* open watcom 2.0 compiler tools

# Roadmap

- [ ] Bootloader
  - [x] Stage 1
    - [x] FAT12 header in boot sector
    - [x] Disk read
    - [x] Screen BIOS print routines
    - [x] Minimal FAT12 read support
    - [x] Second stage binary file loading
  - [ ] Stage 2
    - [x] Disk read
    - [x] BIOS print routines
    - [ ] Full FAT12 read support
      - [x] Root directory lookup
      - [ ] FAT lookup
      - [ ] Subdirectory lookup
    - [ ] Minimal ELF executable loading
    - [ ] Protected mode entry
    - [ ] Long mode entry
    - [ ] BIOS memory map request
    - [ ] Graphics mode setting
- [ ] Kernel
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
