use std::alloc::{alloc, dealloc, Layout};
use std::ptr::NonNull;
use zeroize::Zeroize;

use crate::SecurityError;

pub struct SecureMemory {
    ptr: NonNull<u8>,
    layout: Layout,
}

impl SecureMemory {
    pub fn new(size: usize) -> Result<Self, SecurityError> {
        let layout = Layout::array::<u8>(size)
            .map_err(|e| SecurityError::MemoryError(e.to_string()))?;

        let ptr = unsafe {
            NonNull::new(alloc(layout))
                .ok_or_else(|| SecurityError::MemoryError("Memory allocation failed".to_string()))?
        };

        Ok(Self { ptr, layout })
    }

    pub fn write(&mut self, data: &[u8]) -> Result<(), SecurityError> {
        if data.len() > self.layout.size() {
            return Err(SecurityError::MemoryError("Data too large for allocated memory".to_string()));
        }

        unsafe {
            std::ptr::copy_nonoverlapping(data.as_ptr(), self.ptr.as_ptr(), data.len());
        }

        Ok(())
    }

    pub fn read(&self, len: usize) -> Result<Vec<u8>, SecurityError> {
        if len > self.layout.size() {
            return Err(SecurityError::MemoryError("Read length exceeds allocated memory".to_string()));
        }

        let mut result = vec![0u8; len];
        unsafe {
            std::ptr::copy_nonoverlapping(self.ptr.as_ptr(), result.as_mut_ptr(), len);
        }

        Ok(result)
    }

    pub fn clear(&mut self) {
        unsafe {
            std::ptr::write_bytes(self.ptr.as_ptr(), 0, self.layout.size());
        }
    }
}

impl Drop for SecureMemory {
    fn drop(&mut self) {
        self.clear();
        unsafe {
            dealloc(self.ptr.as_ptr(), self.layout);
        }
    }
}

pub struct MemoryGuard<T: Zeroize> {
    data: T,
}

impl<T: Zeroize> MemoryGuard<T> {
    pub fn new(data: T) -> Self {
        Self { data }
    }

    pub fn get(&self) -> &T {
        &self.data
    }

    pub fn get_mut(&mut self) -> &mut T {
        &mut self.data
    }
}

impl<T: Zeroize> Drop for MemoryGuard<T> {
    fn drop(&mut self) {
        self.data.zeroize();
    }
} 