use std::io::stdin;
use std::path::Path;
use std::time::SystemTime;

#[cfg(target_os = "macos")]
use yas_scanner::common::utils::get_pid_and_ui;
use yas_scanner::common::{utils, UI};
use yas_scanner::common::{PixelRect, RawImage};
use yas_scanner::expo::good::GOODFormat;
use yas_scanner::expo::mingyu_lab::MingyuLabFormat;
use yas_scanner::expo::mona_uranai::MonaFormat;

use yas_scanner::inference::pre_process::image_to_raw;
use yas_scanner::info::info;
use yas_scanner::scanner::yas_scanner::{YasScanner, YasScannerConfig};

use clap::{App, Arg};
use env_logger::Builder;
use screenshots::image;
use screenshots::image::imageops::grayscale;

use log::{info, warn, LevelFilter};

fn open_local(path: String) -> RawImage {
    let img = image::open(path).unwrap();
    let img = grayscale(&img);
    let raw_img = image_to_raw(img);

    raw_img
}

fn main() {
    Builder::new().filter_level(LevelFilter::Info).init();

    #[cfg(windows)]
    if !utils::is_admin() {
        utils::error_and_quit("请以管理员身份运行该程序")
    }

    if let Some(v) = utils::check_update() {
        warn!("检测到新版本，请手动更新：{}", v);
    }

    let matches = App::new("YAS - 原神圣遗物导出器")
        .version(utils::VERSION)
        .author("wormtql <584130248@qq.com>")
        .about("Genshin Impact Artifact Exporter")
        .arg(
            Arg::with_name("max-row")
                .long("max-row")
                .takes_value(true)
                .help("最大扫描行数"),
        )
        .arg(
            Arg::with_name("dump")
                .long("dump")
                .required(false)
                .takes_value(false)
                .help("输出模型预测结果、二值化图像和灰度图像，debug专用"),
        )
        .arg(
            Arg::with_name("capture-only")
                .long("capture-only")
                .required(false)
                .takes_value(false)
                .help("只保存截图，不进行扫描，debug专用"),
        )
        .arg(
            Arg::with_name("min-star")
                .long("min-star")
                .takes_value(true)
                .help("最小星级"),
        )
        .arg(
            Arg::with_name("min-level")
                .long("min-level")
                .takes_value(true)
                .help("最小等级"),
        )
        .arg(
            Arg::with_name("max-wait-switch-artifact")
                .long("max-wait-switch-artifact")
                .takes_value(true)
                .help("切换圣遗物最大等待时间(ms)"),
        )
        .arg(
            Arg::with_name("output-dir")
                .long("output-dir")
                .short("o")
                .takes_value(true)
                .help("输出目录")
                .default_value("."),
        )
        .arg(
            Arg::with_name("scroll-stop")
                .long("scroll-stop")
                .takes_value(true)
                .help("翻页时滚轮停顿时间（ms）（翻页不正确可以考虑加大该选项，默认为80）"),
        )
        .arg(
            Arg::with_name("number")
                .long("number")
                .takes_value(true)
                .help("指定圣遗物数量（在自动识别数量不准确时使用）"),
        )
        .arg(
            Arg::with_name("verbose")
                .long("verbose")
                .help("显示详细信息"),
        )
        .arg(
            Arg::with_name("offset-x")
                .long("offset-x")
                .takes_value(true)
                .help("人为指定横坐标偏移（截图有偏移时可用该选项校正）"),
        )
        .arg(
            Arg::with_name("offset-y")
                .long("offset-y")
                .takes_value(true)
                .help("人为指定纵坐标偏移（截图有偏移时可用该选项校正）"),
        )
        .arg(
            Arg::with_name("output-format")
                .long("output-format")
                .short("f")
                .takes_value(true)
                .help("输出格式")
                .possible_values(&["mona", "mingyulab", "good"])
                .default_value("mona"),
        )
        .arg(
            Arg::with_name("cloud-wait-switch-artifact")
                .long("cloud-wait-switch-artifact")
                .takes_value(true)
                .help("指定云·原神切换圣遗物等待时间(ms)"),
        )
        .get_matches();
    let config = YasScannerConfig::from_match(&matches);

    let rect: PixelRect;
    let is_cloud: bool;
    let ui: UI;

    #[cfg(windows)]
    {
        use winapi::um::winuser::{SetForegroundWindow, ShowWindow, SW_RESTORE};
        // use winapi::um::shellscalingapi::{SetProcessDpiAwareness, PROCESS_PER_MONITOR_DPI_AWARE};

        utils::set_dpi_awareness();

        let hwnd;

        (hwnd, is_cloud) = utils::find_window_local("原神")
            .or_else(|_| utils::find_window_local("Genshin Impact"))
            .map(|hwnd| (hwnd, false))
            .unwrap_or_else(|_| {
                let Ok(hwnd) = utils::find_window_cloud() else {
                    utils::error_and_quit("未找到原神窗口，请确认原神已经开启")
                };
                (hwnd, true)
            });

        unsafe {
            ShowWindow(hwnd, SW_RESTORE);
        }
        // utils::sleep(1000);
        unsafe {
            SetForegroundWindow(hwnd);
        }
        utils::sleep(1000);

        rect = utils::get_client_rect(hwnd).unwrap();
        ui = UI::Desktop;
    }

    #[cfg(all(target_os = "linux"))]
    {
        rect = PixelRect {
            left: 0,
            top: 0,
            width: 1920,
            height: 1080,
        };
        is_cloud = false; // todo: detect cloud genshin by title
        ui = UI::Desktop;
    }

    #[cfg(target_os = "macos")]
    {
        let (pid, ui_) = get_pid_and_ui();
        let window_title: String;
        (rect, window_title) = unsafe { utils::find_window_by_pid(pid).unwrap() };
        info!("Found genshin pid:{}, window name:{}", pid, window_title);
        is_cloud = false; // todo: detect cloud genshin by title
        ui = ui_;
    }

    // rect.scale(1.25);
    info!(
        "left = {}, top = {}, width = {}, height = {}",
        rect.left, rect.top, rect.width, rect.height
    );

    let mut info: info::ScanInfo;

    // desktop ui or mobile ui
    match ui {
        UI::Desktop => {
            info!("desktop ui");
            info = info::ScanInfo::from_pc(
                rect.width as u32,
                rect.height as u32,
                rect.left,
                rect.top,
            );
        },
        UI::Mobile => {
            info!("mobile ui");
            info = info::ScanInfo::from_mobile(
                rect.width as u32,
                rect.height as u32,
                rect.left,
                rect.top,
            );
        },
    }

    let offset_x = matches
        .value_of("offset-x")
        .unwrap_or("0")
        .parse::<i32>()
        .unwrap();
    let offset_y = matches
        .value_of("offset-y")
        .unwrap_or("0")
        .parse::<i32>()
        .unwrap();
    info.left += offset_x;
    info.top += offset_y;

    let mut scanner = YasScanner::new(info.clone(), config, is_cloud);

    let now = SystemTime::now();
    #[cfg(target_os = "macos")]
    {
        info!("初始化完成，请切换到原神窗口，yas将在10s后开始扫描圣遗物");
        utils::sleep(10000);
    }
    let results = scanner.start();
    let t = now.elapsed().unwrap().as_secs_f64();
    info!("time: {}s", t);

    let output_dir = Path::new(matches.value_of("output-dir").unwrap());
    match matches.value_of("output-format") {
        Some("mona") => {
            let output_filename = output_dir.join("mona.json");
            let mona = MonaFormat::new(&results);
            mona.save(String::from(output_filename.to_str().unwrap()));
        },
        Some("mingyulab") => {
            let output_filename = output_dir.join("mingyulab.json");
            let mingyulab = MingyuLabFormat::new(&results);
            mingyulab.save(String::from(output_filename.to_str().unwrap()));
        },
        Some("good") => {
            let output_filename = output_dir.join("good.json");
            let good = GOODFormat::new(&results);
            good.save(String::from(output_filename.to_str().unwrap()));
        },
        _ => unreachable!(),
    }
    // let info = info;
    // let img = info.art_count_position.capture_relative(&info).unwrap();

    // let mut inference = CRNNModel::new(String::from("model_training.onnx"), String::from("index_2_word.json"));
    // let s = inference.inference_string(&img);
    // println!("{}", s);
    info!("识别结束，请按Enter退出");
    let mut s = String::new();
    stdin().read_line(&mut s).unwrap();
}
