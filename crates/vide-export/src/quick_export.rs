use crate::AVFoundationExporter;

pub fn to(output_file: impl ToString) -> AVFoundationExporter {
  let extension = output_file
    .to_string()
    .split('.')
    .last()
    .unwrap_or_else(|| {
      panic!(
        "Vide Quick Export couldn't detect the file extension for {}",
        output_file.to_string()
      )
    })
    .to_string();
  let extension = extension.as_str();

  match extension {
    "mp4" => AVFoundationExporter::new(output_file),
    other => panic!(
      "Vide Quick Export does not support or recognize {} (yet",
      other
    ),
  }
}
