pub mod quick_export;

use std::{panic::AssertUnwindSafe, ptr::NonNull};

use objc2::{rc::Retained, runtime::AnyObject};
use objc2_av_foundation::{
  AVAssetWriter, AVAssetWriterInput, AVFileTypeMPEG4, AVMediaTypeVideo, AVVideoCodecH264,
  AVVideoCodecKey, AVVideoCompressionPropertiesKey, AVVideoHeightKey, AVVideoProfileLevelKey,
  AVVideoWidthKey,
};
use objc2_core_foundation::{CFDictionaryCreate, CFNumber};
use objc2_core_media::{
  kCMTimeInvalid, CMSampleBufferCreateForImageBuffer, CMSampleTimingInfo, CMTime, CMTimeFlags,
  CMVideoFormatDescription, CMVideoFormatDescriptionCreate,
};
use objc2_core_video::{
  kCVPixelBufferHeightKey, kCVPixelBufferPixelFormatTypeKey,
  kCVPixelBufferPoolMinimumBufferCountKey, kCVPixelBufferWidthKey, kCVPixelFormatType_24RGB,
  kCVReturnSuccess, CVPixelBufferGetBaseAddress, CVPixelBufferLockBaseAddress,
  CVPixelBufferLockFlags, CVPixelBufferPool, CVPixelBufferPoolCreate,
  CVPixelBufferPoolCreatePixelBuffer, CVPixelBufferUnlockBaseAddress,
};
use objc2_foundation::{ns_string, NSDictionary, NSNumber, NSString, NSURL};
use vide::io::Export;

pub struct AVFoundationExporter {
  output: String,
  writer: Option<Retained<AVAssetWriter>>,
  writer_input: Option<Retained<AVAssetWriterInput>>,
  format_description: Option<Retained<CMVideoFormatDescription>>,
  pixel_buffer_pool: Option<Retained<CVPixelBufferPool>>,
  current_timestamp: i64,
  ms_per_frame: i64,
  resolution: (usize, usize),
}

impl AVFoundationExporter {
  pub fn new(output: impl ToString) -> Self {
    Self {
      output: format!(
        "{}/{}",
        std::env::current_dir().unwrap().display(),
        output.to_string()
      ),
      writer: None,
      writer_input: None,
      format_description: None,
      pixel_buffer_pool: None,
      current_timestamp: 0,
      ms_per_frame: 0,
      resolution: (1920, 1080),
    }
  }
}

impl Export for AVFoundationExporter {
  fn begin(&mut self, settings: vide::prelude::VideoSettings) {
    let output_path = self.output.clone();
    let resolution = (
      settings.resolution.0 as usize,
      settings.resolution.1 as usize,
    );
    let ms_per_frame = ((1.0 / settings.fps) * 1000000.0) as i64;

    match objc2::exception::catch(AssertUnwindSafe(|| {
      let url = unsafe { NSURL::fileURLWithPath(&NSString::from_str(&output_path)) };

      let writer = unsafe {
        AVAssetWriter::assetWriterWithURL_fileType_error(&url, AVFileTypeMPEG4.unwrap()).unwrap()
      };

      let output_settings = unsafe {
        NSDictionary::<NSString, AnyObject>::from_slices(
          &[
            AVVideoCodecKey.unwrap(),
            AVVideoWidthKey.unwrap(),
            AVVideoHeightKey.unwrap(),
            AVVideoCompressionPropertiesKey.unwrap(),
          ],
          &[
            AVVideoCodecH264.unwrap(),
            &NSString::from_str(&settings.resolution.0.to_string()),
            &NSString::from_str(&settings.resolution.1.to_string()),
            &NSDictionary::from_slices(
              &[AVVideoProfileLevelKey.unwrap()],
              &[ns_string!("H264_Main_AutoLevel")],
            ),
          ],
        )
      };

      let writer_input = unsafe {
        let input = AVAssetWriterInput::assetWriterInputWithMediaType_outputSettings(
          AVMediaTypeVideo.unwrap(),
          Some(&output_settings),
        );
        input.setExpectsMediaDataInRealTime(false);
        input
      };

      let mut format_description_out = std::ptr::null();
      let result = unsafe {
        CMVideoFormatDescriptionCreate(
          None,
          kCVPixelFormatType_24RGB,
          settings.resolution.0 as i32,
          settings.resolution.1 as i32,
          None,
          NonNull::new(&mut format_description_out).unwrap(),
        )
      };
      if result != 0 {
        panic!("Failed to create video format description: {}", result);
      }

      unsafe {
        writer.addInput(&writer_input);
        writer.startWriting();
        writer.startSessionAtSourceTime(CMTime {
          value: 0,
          timescale: 1_000_000,
          flags: CMTimeFlags::Valid,
          epoch: 0,
        });
      }

      let pool_attributes = unsafe {
        let keys = [kCVPixelBufferPoolMinimumBufferCountKey];
        let values = [CFNumber::new_i64(100)];

        CFDictionaryCreate(
          None,
          keys.as_ptr() as *mut *const std::ffi::c_void,
          values.as_ptr() as *mut *const std::ffi::c_void,
          1,
          std::ptr::null(),
          std::ptr::null(),
        )
      };

      let pixel_buffer_attributes = unsafe {
        let keys = [
          kCVPixelBufferWidthKey,
          kCVPixelBufferHeightKey,
          kCVPixelBufferPixelFormatTypeKey,
        ];
        let values = [
          NSNumber::numberWithInt(resolution.0 as i32),
          NSNumber::numberWithInt(resolution.1 as i32),
          NSNumber::numberWithInt(kCVPixelFormatType_24RGB as i32),
        ];

        CFDictionaryCreate(
          None,
          keys.as_ptr() as *mut *const std::ffi::c_void,
          values.as_ptr() as *mut *const std::ffi::c_void,
          3,
          std::ptr::null(),
          std::ptr::null(),
        )
      };

      let mut pool_out = std::ptr::null_mut();
      let result = unsafe {
        CVPixelBufferPoolCreate(
          None,
          pool_attributes.as_deref(),
          pixel_buffer_attributes.as_deref(),
          NonNull::new(&mut pool_out).unwrap(),
        )
      };
      if result != kCVReturnSuccess {
        panic!("Failed to create pixel buffer pool: {:?}", result);
      }

      let format_description =
        unsafe { Retained::from_raw(format_description_out as *mut _).unwrap() };
      let pool = unsafe { Retained::from_raw(pool_out).unwrap() };

      (writer, writer_input, format_description, pool)
    })) {
      Ok((writer, writer_input, format_description, pool)) => {
        self.writer = Some(writer);
        self.writer_input = Some(writer_input);
        self.format_description = Some(format_description);
        self.pixel_buffer_pool = Some(pool);
        self.ms_per_frame = ms_per_frame;
        self.resolution = resolution;
      }
      Err(e) => panic!("Failed to initialize values: {:?}", e),
    }
  }

  fn push_frame(&mut self, _keyframe: bool, frame: &[u8]) {
    let writer_input = self.writer_input.as_ref().unwrap();
    let format_description = self.format_description.as_ref().unwrap();
    let current_timestamp = self.current_timestamp;
    let ms_per_frame = self.ms_per_frame;
    let pool = self.pixel_buffer_pool.as_ref().unwrap();

    match objc2::exception::catch(AssertUnwindSafe(|| {
      let mut pixel_buffer_out = std::ptr::null_mut();
      let result = unsafe {
        CVPixelBufferPoolCreatePixelBuffer(None, pool, NonNull::new(&mut pixel_buffer_out).unwrap())
      };
      if result != kCVReturnSuccess {
        panic!("Failed to create pixel buffer from pool");
      }

      let pixel_buffer = unsafe { Retained::from_raw(pixel_buffer_out).unwrap() };

      let rgb_data = frame
        .chunks(4)
        .flat_map(|p| [p[0], p[1], p[2]])
        .collect::<Vec<u8>>();

      unsafe {
        CVPixelBufferLockBaseAddress(&pixel_buffer, CVPixelBufferLockFlags::empty());
        let pixel_buffer_ptr = CVPixelBufferGetBaseAddress(&pixel_buffer);
        std::ptr::copy_nonoverlapping(rgb_data.as_ptr(), pixel_buffer_ptr.cast(), rgb_data.len());
        CVPixelBufferUnlockBaseAddress(&pixel_buffer, CVPixelBufferLockFlags::empty());
      }

      let mut timing_info = unsafe {
        CMSampleTimingInfo {
          duration: CMTime {
            value: ms_per_frame,
            timescale: 1_000_000,
            flags: CMTimeFlags::Valid,
            epoch: 0,
          },
          presentationTimeStamp: CMTime {
            value: current_timestamp,
            timescale: 1_000_000,
            flags: CMTimeFlags::Valid,
            epoch: 0,
          },
          decodeTimeStamp: kCMTimeInvalid,
        }
      };

      let mut sample_buffer_out = std::ptr::null_mut();
      let result = unsafe {
        CMSampleBufferCreateForImageBuffer(
          None,
          &*pixel_buffer_out,
          true,
          None,
          std::ptr::null_mut(),
          format_description,
          NonNull::new(&mut timing_info).unwrap(),
          NonNull::new(&mut sample_buffer_out).unwrap(),
        )
      };
      if result != 0 {
        panic!("Failed to create CMSampleBuffer: {}", result);
      }

      let sample_buffer = unsafe { Retained::from_raw(sample_buffer_out).unwrap() };

      unsafe {
        writer_input.appendSampleBuffer(&sample_buffer);
      };
    })) {
      Ok(_) => {}
      Err(e) => println!("Failed to push frame: {:?}", e),
    }

    self.current_timestamp += self.ms_per_frame;
  }

  fn end(self) {
    let writer = self.writer.unwrap();
    let writer_input = self.writer_input.unwrap();

    unsafe {
      writer_input.markAsFinished();
      // FIX: Changing this from finishWriting to finishWritingWithCompletionHandler
      // corrupts the file
      writer.finishWriting();
    };
  }
}
