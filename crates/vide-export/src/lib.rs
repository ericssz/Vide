pub mod quick_export;

use std::{panic::AssertUnwindSafe, ptr::NonNull};

use objc2::{rc::Retained, runtime::AnyObject};
use objc2_av_foundation::{
  AVAssetWriter, AVAssetWriterInput, AVFileTypeMPEG4, AVMediaTypeVideo, AVVideoCodecH264,
  AVVideoCodecKey, AVVideoCompressionPropertiesKey, AVVideoHeightKey, AVVideoProfileLevelKey,
  AVVideoWidthKey,
};
use objc2_core_media::{
  kCMTimeInvalid, CMSampleBufferCreateForImageBuffer, CMSampleTimingInfo, CMTime, CMTimeFlags,
  CMVideoFormatDescription, CMVideoFormatDescriptionCreate,
};
use objc2_core_video::{
  kCVPixelFormatType_24RGB, kCVReturnSuccess, CVPixelBuffer, CVPixelBufferCreate,
  CVPixelBufferGetBaseAddress, CVPixelBufferLockBaseAddress, CVPixelBufferLockFlags,
  CVPixelBufferUnlockBaseAddress,
};
use objc2_foundation::{ns_string, NSDictionary, NSString, NSURL};
use vide::io::Export;

pub struct AVFoundationExporter {
  output: String,
  writer: Option<Retained<AVAssetWriter>>,
  writer_input: Option<Retained<AVAssetWriterInput>>,
  format_description: Option<&'static CMVideoFormatDescription>,
  pixel_buffer: Option<&'static CVPixelBuffer>,
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
      pixel_buffer: None,
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

      let video_settings = unsafe {
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
          Some(&video_settings),
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

      let mut pixel_buffer_out = std::ptr::null_mut();
      let result = unsafe {
        CVPixelBufferCreate(
          None,
          resolution.0,
          resolution.1,
          kCVPixelFormatType_24RGB,
          None,
          NonNull::new(&mut pixel_buffer_out).unwrap(),
        )
      };
      if result != kCVReturnSuccess {
        panic!("Failed to create CVPixelBuffer: {}", result);
      }

      (
        writer,
        writer_input,
        format_description_out,
        pixel_buffer_out,
      )
    })) {
      Ok((writer, writer_input, format_description, pixel_buffer_out)) => {
        self.writer = Some(writer);
        self.writer_input = Some(writer_input);
        self.format_description = unsafe { Some(&*format_description) };
        self.pixel_buffer = unsafe { Some(&*pixel_buffer_out) };
        self.ms_per_frame = ms_per_frame;
        self.resolution = resolution;
      }
      Err(e) => panic!("Failed to initialize values: {:?}", e),
    }
  }

  fn push_frame(&mut self, _keyframe: bool, frame: &[u8]) {
    let writer_input = self.writer_input.as_ref().unwrap();
    let format_description = self.format_description.unwrap();
    let current_timestamp = self.current_timestamp;
    let ms_per_frame = self.ms_per_frame;
    let resolution = self.resolution;
    let pixel_buffer = self.pixel_buffer.unwrap();

    match objc2::exception::catch(AssertUnwindSafe(|| {
      let rgb_data = frame
        .chunks(4)
        .flat_map(|p| [p[0], p[1], p[2]])
        .collect::<Vec<u8>>();

      unsafe {
        let pixel_buffer = &*pixel_buffer;
        CVPixelBufferLockBaseAddress(pixel_buffer, CVPixelBufferLockFlags::empty());
        let pixel_buffer_ptr = CVPixelBufferGetBaseAddress(pixel_buffer);
        std::ptr::copy_nonoverlapping(rgb_data.as_ptr(), pixel_buffer_ptr.cast(), rgb_data.len());
        CVPixelBufferUnlockBaseAddress(pixel_buffer, CVPixelBufferLockFlags::empty());
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
          &*pixel_buffer,
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

      unsafe {
        writer_input.appendSampleBuffer(&*sample_buffer_out);
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
    };

    // FIX: Changing this from finishWriting to finishWritingWithCompletionHandler
    // corrupts the file
    match objc2::exception::catch(AssertUnwindSafe(|| unsafe {
      writer.finishWriting();
    })) {
      Ok(_) => {}
      Err(e) => panic!("Failed to end: {:?}", e),
    }
  }
}
