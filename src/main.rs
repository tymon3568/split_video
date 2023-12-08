use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;
use walkdir::WalkDir;

fn main() {
    // Lấy thư mục từ tham số dòng lệnh
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <directory>", args[0]);
        std::process::exit(1);
    }

    let directory = &args[1];

    // Duyệt qua tất cả các file trong thư mục
    for entry in WalkDir::new(directory).into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_file() {
            let file_path = entry.path();

            // Lọc các file có đuôi là .mp4 hoặc .webm
            if let Some(ext) = file_path.extension() {
                if ext == "mp4" || ext == "webm" {
                    // Chia nhỏ file video thành các phần nhỏ có độ dài 59s
                    split_video(file_path);
                }
            }
        }
    }
}

fn split_video(file_path: &Path) {
    // Đường dẫn đến thư mục chứa file đã chia nhỏ
    let output_directory = file_path.parent().unwrap().join("output");

    // Tạo thư mục nếu chưa tồn tại
    fs::create_dir_all(&output_directory).expect("Failed to create output directory");

    // Sử dụng ffprobe để lấy thời gian tổng cộng của video
    let ffprobe_output = Command::new("ffprobe")
        .args(&["-v", "error", "-show_entries", "format=duration", "-of", "default=noprint_wrappers=1:nokey=1", file_path.to_str().unwrap()])
        .output();

    let duration: f64 = match ffprobe_output {
        Ok(output) => {
            String::from_utf8_lossy(&output.stdout)
                .trim()
                .parse()
                .expect("Failed to parse video duration")
        }
        Err(err) => {
            eprintln!("Error running ffprobe: {}", err);
            return;
        }
    };

    // Số lần chia nhỏ video, mỗi đoạn có độ dài 59s
    let num_segments = (duration / 59.0).ceil() as usize;

    // Chia nhỏ video tại các điểm thời gian cụ thể
    for i in 0..num_segments {
        let start_time = i as f64 * 59.0;
        let output_file = output_directory.join(format!(
            "{}_segment_{}.mp4",
            file_path.file_stem().unwrap().to_str().unwrap(),
            i
        ));

        let status = Command::new("ffmpeg")
            .args(&[
                "-ss",
                &start_time.to_string(),
                "-i",
                file_path.to_str().unwrap(),
                "-c",
                "copy",
                "-t",
                "59",
                &output_file.to_str().unwrap(),
            ])
            .status();

        match status {
            Ok(exit_status) => {
                if exit_status.success() {
                    println!(
                        "Segment {} of file {} has been split successfully.",
                        i,
                        file_path.display()
                    );
                } else {
                    eprintln!(
                        "Failed to split segment {} of file {}: {}",
                        i,
                        file_path.display(),
                        exit_status
                    );
                }
            }
            Err(err) => {
                eprintln!("Error running ffmpeg: {}", err);
            }
        }
    }
}

// cargo run /your-video-folder-path