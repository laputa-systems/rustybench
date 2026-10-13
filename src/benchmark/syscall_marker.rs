use std::{
    ffi::{CStr, c_char, c_int, c_ulong},
    io::{self, Result},
};

// Linux prctl operation numbers for setting and reading the calling task name.
const PR_SET_NAME: c_int = 15;
const PR_GET_NAME: c_int = 16;
pub(super) const TASK_NAME_LENGTH: usize = 16;
pub(super) type TaskName = [c_char; TASK_NAME_LENGTH];

unsafe extern "C" {
    fn prctl(option: c_int, ...) -> c_int;
}

/// Reads the calling Linux task's original name so diagnostic markers can be
/// removed after each sample.
pub(super) fn current_task_name() -> Result<TaskName> {
    let mut name = [0; TASK_NAME_LENGTH];
    // SAFETY: PR_GET_NAME writes a NUL-terminated name into the 16-byte buffer.
    let result = unsafe {
        prctl(
            PR_GET_NAME,
            name.as_mut_ptr(),
            0 as c_ulong,
            0 as c_ulong,
            0 as c_ulong,
        )
    };
    ensure_success(result)?;
    Ok(name)
}

/// Marks the start of a benchmark sample in a Linux syscall trace.
pub(super) fn begin() -> Result<()> {
    set_task_name(c"BENCH_BEGIN")
}

/// Marks the end of a benchmark sample in a Linux syscall trace.
pub(super) fn end() -> Result<()> {
    set_task_name(c"BENCH_END")
}

/// Restores the calling task's name after a diagnostic sample.
pub(super) fn restore(name: &TaskName) -> Result<()> {
    // SAFETY: `name` is the NUL-terminated 16-byte buffer returned by
    // PR_GET_NAME. PR_SET_NAME reads it without modifying the buffer.
    let result = unsafe {
        prctl(
            PR_SET_NAME,
            name.as_ptr(),
            0 as c_ulong,
            0 as c_ulong,
            0 as c_ulong,
        )
    };
    ensure_success(result)
}

fn set_task_name(name: &CStr) -> Result<()> {
    // SAFETY: `name` is a NUL-terminated static C string. PR_SET_NAME reads it
    // and ignores the remaining arguments.
    let result = unsafe {
        prctl(
            PR_SET_NAME,
            name.as_ptr(),
            0 as c_ulong,
            0 as c_ulong,
            0 as c_ulong,
        )
    };
    ensure_success(result)
}

fn ensure_success(result: c_int) -> Result<()> {
    if result == -1 {
        Err(io::Error::last_os_error())
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn marker_names_are_set_and_the_original_name_can_be_restored() {
        std::thread::spawn(|| {
            let original = current_task_name().unwrap();

            begin().unwrap();
            assert_eq!(current_task_name().unwrap(), task_name(b"BENCH_BEGIN"));

            end().unwrap();
            assert_eq!(current_task_name().unwrap(), task_name(b"BENCH_END"));

            restore(&original).unwrap();
            assert_eq!(current_task_name().unwrap(), original);
        })
        .join()
        .unwrap();
    }

    fn task_name(value: &[u8]) -> TaskName {
        let mut name = [0; TASK_NAME_LENGTH];
        for (destination, source) in name.iter_mut().zip(value) {
            *destination = *source as c_char;
        }
        name
    }
}
