use std::env;
use std::process::{Command, Stdio, ExitCode};

// add a comment
use std::fs::File;
use std::io::{BufRead, BufReader};

pub fn file_reader(file_loc: &str) -> BufReader<File> {
    let file = File::open(file_loc).unwrap();
    let reader = BufReader::new(file);
    reader
}

pub fn get_files_lines(file_loc: &str) -> Vec<String> {
    let file_r: BufReader<_> = file_reader(file_loc);
    file_r
        .lines()
        .map(|l| l.unwrap())
        .filter(|s| s != &"")
        .collect()
}

fn main() -> ExitCode {

    let cut_file = env::args().nth(1).expect("Please enter cut file.");
    let downld_fmt = env::args().nth(2).unwrap_or_else(|| String::from("251"));
    let split_by_chapters = 
        match env::args().nth(3).unwrap_or_else(|| String::from("No")).as_ref() {
            "Yes" => true,
            "No" =>  false,
            _ =>  false 
        };

    let lines = get_files_lines(&cut_file);
    let youtube_link = &lines[0];
    let dwnld_out = &lines[1];
    // second line contains boiler plate which is: | Date | Event
    let boilerplate = &lines[2]; 
    // third line contains artist
    let artist = &lines[3]; 
    // every other line follows the format: start, end, title
    let data = &lines[4..];
    // So each track has the following name: title | Date | Event - artist
    //
    println!("Starting yt-dlp...");
    let mut command = Command::new("yt-dlp");
    command.args([
        format!("{}", youtube_link), 
        "-f".to_string(),
        downld_fmt.to_string(),
        "-x".to_string(),
        "--audio-format=vorbis".to_string(),
        ]);
    if split_by_chapters {
        command.args([
           "--split-chapters".to_string(),
            "-o".to_string(),
            format!("{}:%(section_number)s %(section_title)s {}.%(ext)s", dwnld_out, boilerplate),
        ]);
        } else {
            command.args([
                "-o".to_string(),
                format!("{}.%(ext)s", dwnld_out),
            ]);
        }
    let output = command
                    .stdout(Stdio::inherit())
                    .output()
                    .expect("failed to execute process");

    if !output.status.success() {
        println!("stderr: {}", String::from_utf8_lossy(&output.stderr));
        return ExitCode::FAILURE;
    }

    if !split_by_chapters {
        let mut ffmpeg_command = Command::new("ffmpeg");
        let ffmpeg_out = ffmpeg_command.args([
                                    "-hide_banner",
                                    "-v",
                                    "warning",
                                    "-stats", 
                                    "-i",
                                    format!("{}.ogg", dwnld_out).as_str()
                                    ]);
        for data_line in data.iter() {
            let pieces: Vec<String> = data_line.split(" ").map(|s| s.to_string()).into_iter().collect();
            let start = &pieces[0];
            let end = &pieces[1];
            let title = &pieces[2..].join(" ");

            ffmpeg_out.args([
                "-c",
                "copy",
                "-ss",
                start.as_str(),
                "-to",
                end.as_str(),
                format!("{} | {} - {}.ogg", title, boilerplate, artist).as_str(),
            ]);

        }

        println!("Starting ffmpeg...");
        
        let executed = ffmpeg_out
            .stdout(Stdio::inherit())
            .output()
            .expect("failed to execute process");

        if !executed.status.success() {
        println!("stderr: {}", String::from_utf8_lossy(&executed.stderr));
            return ExitCode::FAILURE;
        }
    }
    ExitCode::from(0)
}
