use uefi::{
    prelude::BootServices,
    proto::{
        loaded_image::LoadedImage,
        media::{
            file::{File, FileAttribute, FileInfo, FileMode, RegularFile},
            fs::SimpleFileSystem,
        },
    },
    table::boot::MemoryType,
    CStr16,
};

#[derive(Debug, Clone, Copy)]
pub enum FindFileError {
    IsDirectoryError,
    PathTooLongError,
    PathSegmentWasFileError,
    PathDoesNotExistError,
    UEFIError,
}

/// Finds a file handle from the provided path if one exists. This does traverse directory
/// structures. This will fail if the provided path leads to a directory.
///
/// The path is relative to the root of wherever this booted off of. A leading '/' is allowed.
///
/// Relative paths are NOT allowed
///
pub fn find_file(path: &str, boot_services: &BootServices) -> Result<RegularFile, FindFileError> {
    let loaded_image = boot_services
        .open_protocol_exclusive::<LoadedImage>(boot_services.image_handle())
        .map_err(|_| FindFileError::UEFIError)?;

    let mut volume_handle = boot_services
        .open_protocol_exclusive::<SimpleFileSystem>(loaded_image.device())
        .map_err(|_| FindFileError::UEFIError)?;

    let mut current_directory = volume_handle
        .open_volume()
        .map_err(|_| FindFileError::UEFIError)?;

    // The number of path segments is always equal to the number of '/' + 1
    let num_segments = path.chars().filter(|c| *c == '/').count() + 1;

    for (num, segment) in path.split("/").enumerate() {
        // Skip the root '/' if there is one, and consequently allow paths like
        // "//////file.txt" because I don't care :(
        if segment.is_empty() {
            continue;
        }

        let mut segment_buffer = [0u16; 256];
        let segment_cstr = CStr16::from_str_with_buf(segment, &mut segment_buffer)
            .map_err(|_| FindFileError::PathTooLongError)?;

        // If this is the last segment, it should be a file, not a directory
        if num == num_segments - 1 {
            let maybe_file = current_directory
                .open(segment_cstr, FileMode::Read, FileAttribute::empty())
                .map_err(|_| FindFileError::PathDoesNotExistError)?;

            if !maybe_file
                .is_regular_file()
                .map_err(|_| FindFileError::UEFIError)?
            {
                return Err(FindFileError::IsDirectoryError);
            }

            return maybe_file
                .into_regular_file()
                .ok_or(FindFileError::UEFIError);
        } else {
            let next_directory = current_directory
                .open(segment_cstr, FileMode::Read, FileAttribute::empty())
                .map_err(|_| FindFileError::PathDoesNotExistError)?;

            if !next_directory
                .is_directory()
                .map_err(|_| FindFileError::UEFIError)?
            {
                return Err(FindFileError::PathSegmentWasFileError);
            }

            current_directory = next_directory
                .into_directory()
                .ok_or(FindFileError::UEFIError)?;
        }
    }

    // Either the path was empty (therefore the root directory), or we never found
    // a file
    Err(FindFileError::IsDirectoryError)
}

#[derive(Debug, Clone, Copy)]
pub enum ReadFileError {
    FileReadError,
    UEFIError,
}

/// Loads a file into memory. Returns a slice to wherever the buffer we allocated
/// from UEFI put it.
pub fn read_file(
    mut file: RegularFile,
    boot_services: &BootServices,
) -> Result<&'static mut [u8], ReadFileError> {
    let mut file_info_buffer = [0u8; 1024];

    let file_size = file
        .get_info::<FileInfo>(&mut file_info_buffer)
        .map_err(|_| ReadFileError::UEFIError)?
        .file_size();

    let load_area_start = boot_services
        .allocate_pool(MemoryType::LOADER_DATA, file_size.try_into().unwrap())
        .map_err(|_| ReadFileError::UEFIError)?;

    // SAFETY: We know that the buffer we asked for is at least file_size bytes long
    // otherwise the call would have failed, and the pointer is where it told us
    // the start was.
    let file_buffer =
        unsafe { core::slice::from_raw_parts_mut(load_area_start, file_size.try_into().unwrap()) };

    file.read(file_buffer)
        .map_err(|_| ReadFileError::FileReadError)?;

    Ok(file_buffer)
}
