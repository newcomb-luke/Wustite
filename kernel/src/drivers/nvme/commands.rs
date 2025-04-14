use bin_tools::{write_u32_le, write_u64_le};

use crate::logln;

#[derive(Debug, Clone, Copy)]
pub struct CreateIOCompletionQueueCommand {
    address: u64,
    size: u16,
    identifier: u16,
    interrupts_enabled: bool,
}

impl CreateIOCompletionQueueCommand {
    pub fn new(address: u64, size: u16, identifier: u16, interrupts_enabled: bool) -> Self {
        Self {
            address,
            size,
            identifier,
            interrupts_enabled,
        }
    }

    pub fn into_command(self, command_identifier: u16) -> AdminCommand {
        let dword0 = AdminCommandDword0::new(
            command_identifier,
            AdminOpcode::CreateIOCompletionQueue,
            FusedOperation::Normal,
            Location::PRP,
        );

        let mut dwords10_15 = [0u32; 6];
        // Command Dword 10
        dwords10_15[0] = ((self.size as u32) << 16) | (self.identifier as u32);
        // Command Dword 11
        // Bit 1 = Interrupts Enabled
        // Bit 0 = Physically Contiguous
        dwords10_15[1] = if self.interrupts_enabled { 0b10 } else { 0 } | 1u32;

        logln!("Dword 11: {:032b}", dwords10_15[1]);

        AdminCommand {
            dword0,
            namespace_identifier: 0,
            dword2: 0,
            dword3: 0,
            metadata_pointer: 0,
            data_pointer_0: self.address,
            data_pointer_1: 0,
            dwords10_15,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct CreateIOSubmissionQueueCommand {
    address: u64,
    size: u16,
    identifier: u16,
    completion_queue_identifier: u16,
}

impl CreateIOSubmissionQueueCommand {
    pub fn new(address: u64, size: u16, identifier: u16, completion_queue_identifier: u16) -> Self {
        Self {
            address,
            size,
            identifier,
            completion_queue_identifier,
        }
    }

    pub fn into_command(self, command_identifier: u16) -> AdminCommand {
        let dword0 = AdminCommandDword0::new(
            command_identifier,
            AdminOpcode::CreateIOSubmissionQueue,
            FusedOperation::Normal,
            Location::PRP,
        );

        let mut dwords10_15 = [0u32; 6];
        // Command Dword 10
        dwords10_15[0] = ((self.size as u32) << 16) | (self.identifier as u32);
        // Command Dword 11
        // Upper 16 bits = Completion queue identifier
        // Lower 16 bits:
        //     Bit 0 = Physically Contiguous
        dwords10_15[1] = ((self.completion_queue_identifier as u32) << 16) | 1u32;

        AdminCommand {
            dword0,
            namespace_identifier: 0,
            dword2: 0,
            dword3: 0,
            metadata_pointer: 0,
            data_pointer_0: self.address,
            data_pointer_1: 0,
            dwords10_15,
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct AdminCommandDword0 {
    identifier: u16,
    opcode: AdminOpcode,
    fused: FusedOperation,
    location: Location,
}

impl AdminCommandDword0 {
    fn new(
        identifier: u16,
        opcode: AdminOpcode,
        fused: FusedOperation,
        location: Location,
    ) -> Self {
        Self {
            identifier,
            opcode,
            fused,
            location,
        }
    }

    fn as_u32(&self) -> u32 {
        let byte_0 = self.opcode.as_u8();
        let byte_1 = self.fused.as_u8() | (self.location.as_u8() << 6);

        ((self.identifier as u32) << 16) | ((byte_1 as u32) << 8) | (byte_0 as u32)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct AdminCommand {
    dword0: AdminCommandDword0,
    namespace_identifier: u32,
    dword2: u32,
    dword3: u32,
    metadata_pointer: u64,
    data_pointer_0: u64,
    data_pointer_1: u64,
    dwords10_15: [u32; 6],
}

impl AdminCommand {
    pub fn write(&self, buffer: &mut [u8]) {
        write_u32_le(self.dword0.as_u32(), buffer, 0);
        write_u32_le(self.namespace_identifier, buffer, 4);
        write_u32_le(self.dword2, buffer, 8);
        write_u32_le(self.dword3, buffer, 12);
        write_u64_le(self.metadata_pointer, buffer, 16);
        write_u64_le(self.data_pointer_0, buffer, 24);
        write_u64_le(self.data_pointer_1, buffer, 32);

        let mut offset = 40;
        for dword in self.dwords10_15 {
            write_u32_le(dword, buffer, offset);
            offset += 4;
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AdminOpcode {
    CreateIOSubmissionQueue,
    CreateIOCompletionQueue,
    Identify,
}

impl AdminOpcode {
    fn as_u8(&self) -> u8 {
        match self {
            Self::CreateIOSubmissionQueue => 0x01,
            Self::CreateIOCompletionQueue => 0x05,
            Self::Identify => 0x06,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum IOOpcode {
    Read,
    Write,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FusedOperation {
    Normal,
    FirstCommand,
    SecondCommand,
}

impl FusedOperation {
    fn as_u8(&self) -> u8 {
        match self {
            Self::Normal => 0b00,
            Self::FirstCommand => 0b01,
            Self::SecondCommand => 0b10,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Location {
    PRP,
    SGLUsedAddress,
    SGLUsedSegment,
}

impl Location {
    fn as_u8(&self) -> u8 {
        match self {
            Self::PRP => 0b00,
            Self::SGLUsedAddress => 0b01,
            Self::SGLUsedSegment => 0b10,
        }
    }
}
