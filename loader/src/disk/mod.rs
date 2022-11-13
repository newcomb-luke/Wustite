const IO_PORT: u16 = 0x01f0;
const CONTROL_PORT: u16 = 0x3f6;

#[repr(u16)]
#[derive(Copy, Clone)]
enum ReadableIORegister {
    Data = IO_PORT,
    Error = IO_PORT + 1,
    SectorCount = IO_PORT + 2,
    SectorNumber = IO_PORT + 3,
    CylinderLow = IO_PORT + 4,
    CylinderHigh = IO_PORT + 5,
    DriveAndHead = IO_PORT + 6,
    Status = IO_PORT + 7,
}

#[repr(u16)]
#[derive(Copy, Clone)]
enum WriteableIORegister {
    Data = IO_PORT,
    Features = IO_PORT + 1,
    SectorCount = IO_PORT + 2,
    SectorNumber = IO_PORT + 3,
    CylinderLow = IO_PORT + 4,
    CylinderHigh = IO_PORT + 5,
    DriveAndHead = IO_PORT + 6,
    Command = IO_PORT + 7,
}

#[repr(u16)]
#[derive(Copy, Clone)]
enum ReadableControlRegister {
    AlternateStatus = CONTROL_PORT,
    DeviceAddress = CONTROL_PORT + 1,
}

#[repr(u16)]
#[derive(Copy, Clone)]
enum WriteableControlRegister {
    DeviceControl = CONTROL_PORT,
}

#[repr(u8)]
#[derive(Copy, Clone)]
enum DiskError {
    AddressMarkNotFound = 1,
    TrackZeroNotFound = 2,
    AbortedCommand = 4,
    MediaChangeRequest = 8,
    IDNotFound = 16,
    MediaChanged = 32,
    UncorrectableDataError = 64,
    BadBlock = 128,
}

pub unsafe fn read_disk(cylinder: u16, head: u8, sector: u8) {}
