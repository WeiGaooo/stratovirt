// Copyright (c) 2023 Huawei Technologies Co.,Ltd. All rights reserved.
//
// StratoVirt is licensed under Mulan PSL v2.
// You can use this software according to the terms and conditions of the Mulan
// PSL v2.
// You may obtain a copy of Mulan PSL v2 at:
//         http://license.coscl.org.cn/MulanPSL2
// THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY
// KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO
// NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
// See the Mulan PSL v2 for more details.

//! The abstract layer that connects different frontend & backend camera devices.
//! Backend devices, such as v4l2, usb, or demo device, etc., shall implement trait CameraHostdevOps.

pub mod demo;
pub mod v4l2;

use std::sync::Arc;

use anyhow::{bail, Result};

use util::aio::Iovec;

/// Frame interval in 100ns units.
pub const INTERVALS_PER_SEC: u32 = 10_000_000;

#[allow(dead_code)]
#[derive(Default)]
pub struct CamFmt {
    // Basic 3 configurations: frame size, format, frame frequency.
    basic_fmt: CamBasicFmt,
    // Processing Unit Configuration: brightness, hue, etc.
    pu_fmt: CamPUFmt,
    // Camera Terminal Configuration: focus, exposure time, iris, etc.
    lens_fmt: CamLensFmt,
}

impl CamFmt {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }
}

#[derive(Default, Debug)]
pub struct CamBasicFmt {
    width: u32,
    height: u32,
    fps: u32,
    fmttype: FmtType,
}

impl CamBasicFmt {
    pub fn get_frame_intervals(&self) -> Result<u32> {
        if self.fps == 0 {
            bail!("Invalid fps!");
        }
        Ok(INTERVALS_PER_SEC / self.fps)
    }
}

#[allow(dead_code)]
#[derive(Default)]
pub struct CamPUFmt {
    bright: u64,
    contrast: u64,
    hue: u64,
    saturatio: u64,
    // TODO: to be extended.
}

#[allow(dead_code)]
#[derive(Default)]
pub struct CamLensFmt {
    focus: u64,
    zoom: u64,
    // TODO: to be extended.
}

#[derive(Debug)]
pub enum FmtType {
    Uncompressed = 0,
    Mjpg,
}

impl Default for FmtType {
    fn default() -> Self {
        FmtType::Uncompressed
    }
}

#[derive(Debug)]
pub struct CameraFrame {
    pub width: u32,
    pub height: u32,
    pub interval: u32,
}

pub struct CameraFormatList {
    pub format: FmtType,
    pub frame: Vec<CameraFrame>,
}

/// Callback function which is called when frame data is coming.
pub type CameraNotifyCallback = Arc<dyn Fn() + Send + Sync>;

/// Callback function which is called when backend is broken.
pub type CameraBrokenCallback = Arc<dyn Fn() + Send + Sync>;

pub trait CameraHostdevOps: Send + Sync {
    fn init(&self) -> Result<()>;
    fn is_camera(&self) -> Result<bool>;
    fn get_fmt(&self) -> Result<()>;
    /// Set a specific format.
    fn set_fmt(&mut self, fmt: &CamBasicFmt) -> Result<()>;
    fn set_ctl(&self) -> Result<()>;

    // Turn stream on to start to receive frame buffer.
    fn video_stream_on(&mut self) -> Result<()>;

    // Turn stream off to end receiving frame buffer.
    fn video_stream_off(&mut self) -> Result<()>;

    /// List all formats supported by backend.
    fn list_format(&mut self) -> Result<Vec<CameraFormatList>>;

    /// Reset the device.
    fn reset(&mut self);

    /// Get the total size of current frame.
    fn get_frame_size(&self) -> usize;

    /// Copy frame data to iovecs.
    fn get_frame(&self, iovecs: &[Iovec], frame_offset: usize, len: usize) -> Result<usize>;

    /// Get next frame when current frame is read complete.
    fn next_frame(&mut self) -> Result<()>;

    /// Register notify callback which is called when data is coming.
    fn register_notify_cb(&mut self, cb: CameraNotifyCallback);

    /// Register broken callback which is called when backend is broken.
    fn register_broken_cb(&mut self, cb: CameraBrokenCallback);
}

pub fn get_format_by_index(format_index: u8, frame_index: u8) -> Result<CamBasicFmt> {
    let fmttype = if format_index == 1 {
        FmtType::Mjpg
    } else if format_index == 2 {
        FmtType::Uncompressed
    } else {
        bail!("Invalid format index {}", format_index);
    };

    let width_height_list = [(960, 540), (1280, 720), (640, 480)];
    let fps_list = [5, 10, 30];
    if width_height_list.get((frame_index - 1) as usize).is_none() {
        bail!("Invalid frame index {}", frame_index);
    }
    let fps = if format_index == 1 {
        30
    } else {
        fps_list[(frame_index - 1) as usize]
    };

    Ok(CamBasicFmt {
        width: width_height_list[(frame_index - 1) as usize].0,
        height: width_height_list[(frame_index - 1) as usize].1,
        fmttype,
        fps,
    })
}
