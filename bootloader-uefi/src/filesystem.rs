use uefi::{
    prelude::BootServices,
    proto::{
        loaded_image::LoadedImage,
        media::{
            file::{File, FileAttribute, FileHandle, FileMode, RegularFile},
            fs::SimpleFileSystem,
        },
    },
    CStr16,
};
use uefi_services::println;

#[derive(Debug, Clone, Copy)]
pub enum FindFileError {
    IsDirectoryError,
    PathTooLongError,
    PathSegmentWasFileError,
    VolumeAlreadyOpenError,
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
        .map_err(|_| FindFileError::VolumeAlreadyOpenError)?;

    // The number of path segments is always equal to the number of '/' + 1
    let mut num_segments = path.chars().filter(|c| *c == '/').count() + 1;

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
