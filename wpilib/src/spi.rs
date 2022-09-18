// Copyright 2018 First Rust Competition Developers.
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Provides an interface to the SPI bus and the automatic SPI transfer engine.
//!
//! Currently does not implement an accumulator.

use std::{io, time};
use wpilib_sys::*;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[repr(i32)]
pub enum Port {
    OnboardCS0 = HAL_SPIPort::HAL_SPI_kOnboardCS0,
    OnboardCS1 = HAL_SPIPort::HAL_SPI_kOnboardCS1,
    OnboardCS2 = HAL_SPIPort::HAL_SPI_kOnboardCS2,
    OnboardCS3 = HAL_SPIPort::HAL_SPI_kOnboardCS3,
    MXP = HAL_SPIPort::HAL_SPI_kMXP,
}

/// Settings for `Spi::set_opts`. These all default to false.
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Hash)]
pub struct SpiOptions {
    pub msb_first: bool,
    pub sample_on_trailing: bool,
    pub clk_idle_high: bool,
}

/// SPI bus interface. Intended to be used by sensor (and other SPI device) drivers.
#[derive(Debug)]
pub struct Spi {
    port: HAL_SPIPort::Type,
    opts: SpiOptions,
}

impl Spi {
    pub fn new(port: Port) -> HalResult<Self> {
        let port = port as HAL_SPIPort::Type;
        hal_call!(HAL_InitializeSPI(port))?;
        usage::report(usage::resource_types::SPI, port as _);
        Ok(Spi {
            port,
            opts: SpiOptions::default(),
        })
    }

    pub fn set_clock_rate(&mut self, hz: i32) {
        unsafe { HAL_SetSPISpeed(self.port, hz as i32) }
    }

    pub fn set_opts(&mut self, opts: SpiOptions) {
        unsafe {
            HAL_SetSPIOpts(
                self.port,
                opts.msb_first as HAL_Bool,
                opts.sample_on_trailing as HAL_Bool,
                opts.clk_idle_high as HAL_Bool,
            )
        }
    }

    #[deprecated(since = "0.5.0", note = "use `set_opts` directly instead")]
    pub fn set_msb_first(&mut self) {
        self.opts.msb_first = true;
        self.set_opts(self.opts)
    }

    #[deprecated(since = "0.5.0", note = "use `set_opts` directly instead")]
    pub fn set_lsb_first(&mut self) {
        self.opts.msb_first = false;
        self.set_opts(self.opts)
    }

    #[deprecated(since = "0.5.0", note = "use `set_opts` directly instead")]
    pub fn set_sample_data_on_leading_edge(&mut self) {
        self.opts.sample_on_trailing = false;
        self.set_opts(self.opts)
    }

    #[deprecated(since = "0.5.0", note = "use `set_opts` directly instead")]
    pub fn set_sample_data_on_trailing_edge(&mut self) {
        self.opts.sample_on_trailing = true;
        self.set_opts(self.opts)
    }

    #[deprecated(since = "0.5.0", note = "use `set_opts` directly instead")]
    pub fn set_clock_active_low(&mut self) {
        self.opts.clk_idle_high = true;
        self.set_opts(self.opts)
    }

    #[deprecated(since = "0.5.0", note = "use `set_opts` directly instead")]
    pub fn set_clock_active_high(&mut self) {
        self.opts.clk_idle_high = false;
        self.set_opts(self.opts)
    }

    pub fn set_chip_select_active_high(&mut self) -> HalResult<()> {
        hal_call!(HAL_SetSPIChipSelectActiveHigh(self.port))
    }

    pub fn set_chip_select_active_low(&mut self) -> HalResult<()> {
        hal_call!(HAL_SetSPIChipSelectActiveLow(self.port))
    }

    pub fn write(&mut self, data: &[u8]) -> io::Result<usize> {
        io_result(unsafe { HAL_WriteSPI(self.port, data.as_ptr(), data.len() as _) })
    }

    pub fn read(&mut self, initiate: bool, buf: &mut [u8]) -> io::Result<usize> {
        if initiate {
            let send_data: Vec<u8> = vec![0; buf.len()];
            return unsafe { self.transaction_into(&send_data, buf.as_mut_ptr()) };
        }

        io_result(unsafe { HAL_ReadSPI(self.port, buf.as_mut_ptr(), buf.len() as _) })
    }

    /// Performs an SPI send/receive transaction.
    pub fn transaction(&mut self, to_send: &[u8]) -> io::Result<Vec<u8>> {
        let size = to_send.len();
        let mut receive_buf = Vec::with_capacity(size);
        let read = unsafe { self.transaction_into(to_send, receive_buf.as_mut_ptr()) }?;
        unsafe { receive_buf.set_len(read) }
        Ok(receive_buf)
    }

    /// Performs an SPI send/receive transaction,
    /// storing the received data in the array pointed to by `receive_buf`.
    ///
    /// # Safety
    ///
    /// `receive_buf` must point to an array with the same length as `to_send`.
    pub unsafe fn transaction_into(
        &mut self,
        to_send: &[u8],
        receive_buf: *mut u8,
    ) -> io::Result<usize> {
        let size = to_send.len();
        io_result(HAL_TransactionSPI(
            self.port,
            to_send.as_ptr(),
            receive_buf,
            size as _,
        ))
    }
}

impl io::Read for Spi {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.read(false, buf)
    }
}

#[cfg(feature = "embedded-hal")]
impl embedded_hal::blocking::spi::Transfer<u8> for Spi {
    type Error = io::Error;

    fn transfer<'w>(&mut self, words: &'w mut [u8]) -> io::Result<&'w [u8]> {
        let ptr = words.as_mut_ptr();
        let size = unsafe { self.transaction_into(words, ptr) }?;
        Ok(&words[..size])
    }
}

/// Automatic SPI transfer engine.
#[derive(Debug)]
pub struct AutoSpi(Spi);

impl AutoSpi {
    /// Initialize automatic SPI transfer engine.
    ///
    /// Only a single engine is available.
    /// This will error if an engine is currently already allocated.
    pub fn new(spi: Spi, buffer_size: i32) -> HalResult<Self> {
        hal_call!(HAL_InitSPIAuto(spi.port, buffer_size))?;
        Ok(Self(spi))
    }

    /// Frees the automatic SPI transfer engine, releasing the underlying `Spi`.
    pub fn stop(self) -> Spi {
        // Spi::drop (HAL_CloseSPI) will ensure the auto SPI is freed if we get dropped.
        let _ = hal_call!(HAL_FreeSPIAuto(self.0.port));
        self.0
    }

    pub fn set_transmit_data(&mut self, to_send: &[u8], zero_size: i32) -> HalResult<()> {
        hal_call!(HAL_SetSPIAutoTransmitData(
            self.0.port,
            to_send.as_ptr(),
            to_send.len() as i32,
            zero_size
        ))
    }

    pub fn start_rate(&mut self, period: time::Duration) -> HalResult<()> {
        hal_call!(HAL_StartSPIAutoRate(self.0.port, period.as_secs_f64()))
    }

    pub fn pause(&mut self) -> HalResult<()> {
        hal_call!(HAL_StopSPIAuto(self.0.port))
    }

    pub fn force_read(&mut self) -> HalResult<()> {
        hal_call!(HAL_ForceSPIAutoRead(self.0.port))
    }

    /**
     * Read data that has been transferred by the automatic SPI transfer engine.
     *
     * Transfers may be made a byte at a time, so it's necessary for the caller
     * to handle cases where an entire transfer has not been completed.
     *
     * Each received data sequence consists of a timestamp followed by the
     * received data bytes, one byte per word (in the least significant byte).
     * The length of each received data sequence is the same as the combined
     * size of the data and `zero_size` set in `set_transmit_data`.
     *
     * Blocks until the buffer is filled or timeout (ms resolution) expires.
     * May be called with an empty buffer to retrieve how many words are available.
     */
    pub fn read_received_data(
        &mut self,
        buffer: &mut [u32],
        timeout: time::Duration,
    ) -> HalResult<i32> {
        hal_call!(HAL_ReadSPIAutoReceivedData(
            self.0.port,
            buffer.as_mut_ptr(),
            buffer.len() as _,
            timeout.as_secs_f64()
        ))
    }

    pub fn dropped_count(&mut self) -> i32 {
        // All this should guarantee we are the auto SPI.
        // If not, something has gone horribly wrong.
        hal_call!(HAL_GetSPIAutoDroppedCount(self.0.port)).unwrap()
    }
}

impl Drop for Spi {
    fn drop(&mut self) {
        unsafe { HAL_CloseSPI(self.port) }
    }
}

/// Convert the return value of HAL SPI read/write/transaction to an io::Result.
fn io_result(rv: i32) -> io::Result<usize> {
    if rv < 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(rv as usize)
    }
}
