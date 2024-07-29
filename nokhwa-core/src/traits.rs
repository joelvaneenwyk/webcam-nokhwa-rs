/*
 * Copyright 2022 l1npengtul <l1npengtul@protonmail.com> / The Nokhwa Contributors
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

use crate::{
    buffer::Buffer,
    error::NokhwaError,
    types::{
        ApiBackend, CameraControl, CameraFormat, CameraInfo, ControlValueSetter,
        KnownCameraControl, Resolution,
    },
};
use std::{borrow::Cow, collections::HashMap};
use crate::frame_format::FrameFormat;
use crate::types::FrameRate;

pub trait Backend {
    const BACKEND: ApiBackend;
}

/// This trait is for any backend that allows you to grab and take frames from a camera.
/// Many of the backends are **blocking**, if the camera is occupied the library will block while it waits for it to become available.
///
/// **Note**:
/// - Backends, if not provided with a camera format, will be spawned with 640x480@15 FPS, MJPEG [`CameraFormat`].
/// - Behaviour can differ from backend to backend. While the Camera struct abstracts most of this away, if you plan to use the raw backend structs please read the `Quirks` section of each backend.
/// - If you call [`stop_stream()`](CaptureTrait::stop_stream()), you will usually need to call [`open_stream()`](CaptureTrait::open_stream()) to get more frames from the camera.
pub trait CaptureTrait {
    /// Initialize the camera, preparing it for use, with a random format (usually the first one).
    fn init(&mut self) -> Result<(), NokhwaError>;

    /// Returns the current backend used.
    fn backend(&self) -> ApiBackend;

    /// Gets the camera information such as Name and Index as a [`CameraInfo`].
    fn camera_info(&self) -> &CameraInfo;

    /// Forcefully refreshes the stored camera format, bringing it into sync with "reality" (current camera state)
    /// # Errors
    /// If the camera can not get its most recent [`CameraFormat`]. this will error.
    fn refresh_camera_format(&mut self) -> Result<(), NokhwaError>;

    /// Gets the current [`CameraFormat`]. This will force refresh to the current latest if it has changed.
    fn camera_format(&self) -> Option<CameraFormat>;

    /// Will set the current [`CameraFormat`]
    /// This will reset the current stream if used while stream is opened.
    ///
    /// This will also update the cache.
    /// # Errors
    /// If you started the stream and the camera rejects the new camera format, this will return an error.
    fn set_camera_format(&mut self, new_fmt: CameraFormat) -> Result<(), NokhwaError>;

    /// A hashmap of [`Resolution`]s mapped to framerates. Not sorted!
    /// # Errors
    /// This will error if the camera is not queryable or a query operation has failed. Some backends will error this out as a Unsupported Operation ([`UnsupportedOperationError`](NokhwaError::UnsupportedOperationError)).
    fn compatible_list_by_resolution(
        &mut self,
        fourcc: FrameFormat,
    ) -> Result<HashMap<Resolution, Vec<FrameRate>>, NokhwaError>;

    /// Gets the compatible [`CameraFormat`] of the camera
    /// # Errors
    /// If it fails to get, this will error.
    fn compatible_camera_formats(&mut self) -> Result<Vec<CameraFormat>, NokhwaError> {
        let mut compatible_formats = vec![];
        for fourcc in self.compatible_fourcc()? {
            for (resolution, fps_list) in self.compatible_list_by_resolution(fourcc)? {
                for fps in fps_list {
                    compatible_formats.push(CameraFormat::new(resolution, fourcc, fps));
                }
            }
        }

        Ok(compatible_formats)
    }

    /// A Vector of compatible [`FrameFormat`]s. Will only return 2 elements at most.
    /// # Errors
    /// This will error if the camera is not queryable or a query operation has failed. Some backends will error this out as a Unsupported Operation ([`UnsupportedOperationError`](NokhwaError::UnsupportedOperationError)).
    fn compatible_fourcc(&mut self) -> Result<Vec<FrameFormat>, NokhwaError>;

    /// Gets the current camera resolution (See: [`Resolution`], [`CameraFormat`]). This will force refresh to the current latest if it has changed.
    fn resolution(&self) -> Option<Resolution>;

    /// Will set the current [`Resolution`]
    /// This will reset the current stream if used while stream is opened.
    ///
    /// This will also update the cache.
    /// # Errors
    /// If you started the stream and the camera rejects the new resolution, this will return an error.
    fn set_resolution(&mut self, new_res: Resolution) -> Result<(), NokhwaError>;

    /// Gets the current camera framerate (See: [`CameraFormat`]). This will force refresh to the current latest if it has changed.
    fn frame_rate(&self) -> Option<u32>;

    /// Will set the current framerate
    /// This will reset the current stream if used while stream is opened.
    ///
    /// This will also update the cache.
    /// # Errors
    /// If you started the stream and the camera rejects the new framerate, this will return an error.
    fn set_frame_rate(&mut self, new_fps: u32) -> Result<(), NokhwaError>;

    /// Gets the current camera's frame format (See: [`FrameFormat`], [`CameraFormat`]). This will force refresh to the current latest if it has changed.
    fn frame_format(&self) -> FrameFormat;

    /// Will set the current [`FrameFormat`]
    /// This will reset the current stream if used while stream is opened.
    ///
    /// This will also update the cache.
    /// # Errors
    /// If you started the stream and the camera rejects the new frame format, this will return an error.
    fn set_frame_format(&mut self, fourcc: FrameFormat)
        -> Result<(), NokhwaError>;

    /// Gets the value of [`KnownCameraControl`].
    /// # Errors
    /// If the `control` is not supported or there is an error while getting the camera control values (e.g. unexpected value, too high, etc)
    /// this will error.
    fn camera_control(&self, control: KnownCameraControl) -> Result<CameraControl, NokhwaError>;

    /// Gets the current supported list of [`KnownCameraControl`]
    /// # Errors
    /// If the list cannot be collected, this will error. This can be treated as a "nothing supported".
    fn camera_controls(&self) -> Result<Vec<CameraControl>, NokhwaError>;

    /// Sets the control to `control` in the camera.
    /// Usually, the pipeline is calling [`camera_control()`](CaptureTrait::camera_control), getting a camera control that way
    /// then calling [`value()`](CameraControl::value()) to get a [`ControlValueSetter`] and setting the value that way.
    /// # Errors
    /// If the `control` is not supported, the value is invalid (less than min, greater than max, not in step), or there was an error setting the control,
    /// this will error.
    fn set_camera_control(
        &mut self,
        id: KnownCameraControl,
        value: ControlValueSetter,
    ) -> Result<(), NokhwaError>;

    /// Will open the camera stream with set parameters. This will be called internally if you try and call [`frame()`](CaptureTrait::frame()) before you call [`open_stream()`](CaptureTrait::open_stream()).
    /// # Errors
    /// If the specific backend fails to open the camera (e.g. already taken, busy, doesn't exist anymore) this will error.
    fn open_stream(&mut self) -> Result<(), NokhwaError>;

    /// Checks if stream if open. If it is, it will return true.
    fn is_stream_open(&self) -> bool;

    /// Will get a frame from the camera as a [`Buffer`]. Depending on the backend, if you have not called [`open_stream()`](CaptureTrait::open_stream()) before you called this,
    /// it will either return an error.
    /// # Errors
    /// If the backend fails to get the frame (e.g. already taken, busy, doesn't exist anymore), the decoding fails (e.g. MJPEG -> u8), or [`open_stream()`](CaptureTrait::open_stream()) has not been called yet,
    /// this will error.
    fn frame(&mut self) -> Result<Buffer, NokhwaError>;

    /// Will get a frame from the camera **without** any processing applied, meaning you will usually get a frame you need to decode yourself.
    /// # Errors
    /// If the backend fails to get the frame (e.g. already taken, busy, doesn't exist anymore), or [`open_stream()`](CaptureTrait::open_stream()) has not been called yet, this will error.
    fn frame_raw(&mut self) -> Result<Cow<[u8]>, NokhwaError>;

    /// The minimum buffer size needed to write the current frame. If `alpha` is true, it will instead return the minimum size of the buffer with an alpha channel as well.
    /// This assumes that you are decoding to RGB/RGBA for [`FrameFormat::MJPEG`] or [`FrameFormat::YUYV`] and Luma8/LumaA8 for [`FrameFormat::GRAY`]
    #[must_use]
    fn decoded_buffer_size(&self, alpha: bool) -> usize {
        let cfmt = self.camera_format().unwrap();
        let resolution = cfmt.resolution();
        let pxwidth = match cfmt.format() {
            FrameFormat::MJpeg | FrameFormat::Yv12 | FrameFormat::Rgb8 | FrameFormat::Nv12 => 3,
            FrameFormat::Luma8 | FrameFormat::Luma16 => 1,
            FrameFormat::H265 => todo!(),
            FrameFormat::H264 => todo!(),
            FrameFormat::H263 => todo!(),
            FrameFormat::Avc1 => todo!(),
            FrameFormat::Mpeg1 => todo!(),
            FrameFormat::Mpeg2 => todo!(),
            FrameFormat::Mpeg4 => todo!(),
            FrameFormat::XVid => todo!(),
            FrameFormat::VP8 => todo!(),
            FrameFormat::VP9 => todo!(),
            FrameFormat::Yuv422 => todo!(),
            FrameFormat::Uyv422 => todo!(),
            FrameFormat::Nv21 => todo!(),
            FrameFormat::RgbA8 => todo!(),
            FrameFormat::Custom(_) => todo!(),
            FrameFormat::PlatformSpecificCustomFormat(_) => todo!(),
        };
        if alpha {
            return (resolution.width() * resolution.height() * (pxwidth + 1)) as usize;
        }
        (resolution.width() * resolution.height() * pxwidth) as usize
    }

    #[cfg(feature = "wgpu-types")]
    #[cfg_attr(feature = "docs-features", doc(cfg(feature = "wgpu-types")))]
    /// Directly copies a frame to a Wgpu texture. This will automatically convert the frame into a RGBA frame.
    /// # Errors
    /// If the frame cannot be captured or the resolution is 0 on any axis, this will error.
    fn frame_texture<'a>(
        &mut self,
        device: &WgpuDevice,
        queue: &WgpuQueue,
        label: Option<&'a str>,
    ) -> Result<WgpuTexture, NokhwaError> {
        use crate::pixel_format::RgbAFormat;
        use std::num::NonZeroU32;
        let frame = self.frame()?.decode_image::<RgbAFormat>()?;

        let texture_size = Extent3d {
            width: frame.width(),
            height: frame.height(),
            depth_or_array_layers: 1,
        };

        let texture = device.create_texture(&TextureDescriptor {
            label,
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8UnormSrgb,
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
            view_formats: &[],
        });

        let width_nonzero = 4 * frame.width();

        let height_nonzero = frame.height();

        queue.write_texture(
            ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: TextureAspect::All,
            },
            &frame,
            ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(width_nonzero),
                rows_per_image: Some(height_nonzero),
            },
            texture_size,
        );

        Ok(texture)
    }

    /// Will drop the stream.
    /// # Errors
    /// Please check the `Quirks` section of each backend.
    fn stop_stream(&mut self) -> Result<(), NokhwaError>;
}

impl<T> From<T> for Box<dyn CaptureTrait>
where
    T: CaptureTrait + 'static,
{
    fn from(backend: T) -> Self {
        Box::new(backend)
    }
}

#[cfg(feature = "async")]
#[cfg_attr(feature = "async", async_trait::async_trait)]
pub trait AsyncCaptureTrait: CaptureTrait {
    /// Initialize the camera, preparing it for use, with a random format (usually the first one).
    async fn init_async(&mut self) -> Result<(), NokhwaError>;

    /// Forcefully refreshes the stored camera format, bringing it into sync with "reality" (current camera state)
    /// # Errors
    /// If the camera can not get its most recent [`CameraFormat`]. this will error.
    async fn refresh_camera_format_async(&mut self) -> Result<(), NokhwaError>;

    /// Will set the current [`CameraFormat`]
    /// This will reset the current stream if used while stream is opened.
    ///
    /// This will also update the cache.
    /// # Errors
    /// If you started the stream and the camera rejects the new camera format, this will return an error.
    async fn set_camera_format_async(&mut self, new_fmt: CameraFormat) -> Result<(), NokhwaError>;

    /// A hashmap of [`Resolution`]s mapped to framerates. Not sorted!
    /// # Errors
    /// This will error if the camera is not queryable or a query operation has failed. Some backends will error this out as a Unsupported Operation ([`UnsupportedOperationError`](NokhwaError::UnsupportedOperationError)).
    async fn compatible_list_by_resolution_async(
        &mut self,
        fourcc: FrameFormat,
    ) -> Result<HashMap<Resolution, Vec<u32>>, NokhwaError>;

    /// Gets the compatible [`CameraFormat`] of the camera
    /// # Errors
    /// If it fails to get, this will error.
    async fn compatible_camera_formats_async(&mut self) -> Result<Vec<CameraFormat>, NokhwaError>;

    /// A Vector of compatible [`FrameFormat`]s. Will only return 2 elements at most.
    /// # Errors
    /// This will error if the camera is not queryable or a query operation has failed. Some backends will error this out as a Unsupported Operation ([`UnsupportedOperationError`](NokhwaError::UnsupportedOperationError)).
    async fn compatible_fourcc_async(&mut self) -> Result<Vec<FrameFormat>, NokhwaError>;

    /// Will set the current [`Resolution`]
    /// This will reset the current stream if used while stream is opened.
    ///
    /// This will also update the cache.
    /// # Errors
    /// If you started the stream and the camera rejects the new resolution, this will return an error.
    async fn set_resolution_async(&mut self, new_res: Resolution) -> Result<(), NokhwaError>;

    /// Will set the current framerate
    /// This will reset the current stream if used while stream is opened.
    ///
    /// This will also update the cache.
    /// # Errors
    /// If you started the stream and the camera rejects the new framerate, this will return an error.
    async fn set_frame_rate_async(&mut self, new_fps: u32) -> Result<(), NokhwaError>;

    /// Will set the current [`FrameFormat`]
    /// This will reset the current stream if used while stream is opened.
    ///
    /// This will also update the cache.
    /// # Errors
    /// If you started the stream and the camera rejects the new frame format, this will return an error.
    async fn set_frame_format_async(
        &mut self,
        fourcc: FrameFormat,
    ) -> Result<(), NokhwaError>;

    /// Sets the control to `control` in the camera.
    /// Usually, the pipeline is calling [`camera_control()`](CaptureTrait::camera_control), getting a camera control that way
    /// then calling [`value()`](CameraControl::value()) to get a [`ControlValueSetter`] and setting the value that way.
    /// # Errors
    /// If the `control` is not supported, the value is invalid (less than min, greater than max, not in step), or there was an error setting the control,
    /// this will error.
    async fn set_camera_control_async(
        &mut self,
        id: KnownCameraControl,
        value: ControlValueSetter,
    ) -> Result<(), NokhwaError>;

    /// Will open the camera stream with set parameters. This will be called internally if you try and call [`frame()`](CaptureTrait::frame()) before you call [`open_stream()`](CaptureTrait::open_stream()).
    /// # Errors
    /// If the specific backend fails to open the camera (e.g. already taken, busy, doesn't exist anymore) this will error.
    async fn open_stream_async(&mut self) -> Result<(), NokhwaError>;

    /// Will get a frame from the camera as a [`Buffer`]. Depending on the backend, if you have not called [`open_stream()`](CaptureTrait::open_stream()) before you called this,
    /// it will either return an error.
    /// # Errors
    /// If the backend fails to get the frame (e.g. already taken, busy, doesn't exist anymore), the decoding fails (e.g. MJPEG -> u8), or [`open_stream()`](CaptureTrait::open_stream()) has not been called yet,
    /// this will error.
    async fn frame_async(&mut self) -> Result<Buffer, NokhwaError>;

    /// Will get a frame from the camera **without** any processing applied, meaning you will usually get a frame you need to decode yourself.
    /// # Errors
    /// If the backend fails to get the frame (e.g. already taken, busy, doesn't exist anymore), or [`open_stream()`](CaptureTrait::open_stream()) has not been called yet, this will error.
    async fn frame_raw_async(&mut self) -> Result<Cow<[u8]>, NokhwaError>;

    /// Will drop the stream.
    /// # Errors
    /// Please check the `Quirks` section of each backend.
    async fn stop_stream_async(&mut self) -> Result<(), NokhwaError>;
}

#[cfg(feature = "async")]
impl<T> From<T> for Box<dyn AsyncCaptureTrait>
where
    T: AsyncCaptureTrait + 'static,
{
    fn from(backend: T) -> Self {
        Box::new(backend)
    }
}

pub trait OneShot: CaptureTrait {
    fn one_shot(&mut self) -> Result<Buffer, NokhwaError> {
        if self.is_stream_open() {
            self.frame()
        } else {
            self.open_stream()?;
            let frame = self.frame()?;
            self.stop_stream()?;
            Ok(frame)
        }
    }
}

#[cfg(feature = "async")]
#[cfg_attr(feature = "async", async_trait::async_trait)]
pub trait AsyncOneShot: AsyncCaptureTrait {
    async fn one_shot(&mut self) -> Result<Buffer, NokhwaError> {
        if self.is_stream_open() {
            self.frame_async().await
        } else {
            self.open_stream_async().await?;
            let frame = self.frame_async().await?;
            self.stop_stream_async().await?;
            Ok(frame)
        }
    }
}

pub trait VirtualBackendTrait {}

pub trait Distance<T> where T: PartialEq {
    fn distance_from(&self, other: &Self) -> T;
}
