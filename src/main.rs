mod upload;

use std::fmt::{Display, Formatter};
use iced::widget::{column, container, Button, pick_list, row};
use iced::{event, executor, Alignment, Application, Command, Element, Font, Length, Settings, Subscription, Theme, widget};
use rfd::FileDialog;
use std::option::Option;
use std::path::PathBuf;
use iced::widget::text::Shaping::Advanced;


pub fn main() -> iced::Result {
    let setting = Settings {
        window: iced::window::Settings {
            ..Default::default()
        },
        default_font: Font::with_name("Fira Code"),
        ..Settings::default()
    };
    MyApp::run(setting)
}

#[derive(Debug, Clone)]
enum Message {
    FileDropped(Option<PathBuf>),
    OpenImgPressed,
    OnSelectOption(BaseOption),

    OtherEvent(),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BaseOption {
    #[default]
    PicUpload,
    FileMD5,
}

impl Display for BaseOption {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                BaseOption::PicUpload => { "PicUpload" }
                BaseOption::FileMD5 => { "FileMD5" }
            }
        )
    }
}

impl BaseOption {
    const ALL: [BaseOption; 2] = [BaseOption::PicUpload, BaseOption::FileMD5];
    fn hint(&self) -> &str {
        match self {
            BaseOption::PicUpload => { "上传图片到github，将Markdown链接写入剪贴板" }
            BaseOption::FileMD5 => { "计算文件 md5，并生成文件，同时写入剪贴板" }
        }
    }
    fn button_txt(&self) -> &str {
        match self {
            BaseOption::PicUpload => { "Upload Image" }
            BaseOption::FileMD5 => { "Select File" }
        }
    }
}

struct MyApp {
    return_path: Option<String>,
    image_path: Option<PathBuf>,
    selected_option: Option<BaseOption>,
    option_hint: String,
    button_txt: String,
}

fn is_image(path: &PathBuf) -> bool {
    match path.extension() {
        Some(ext) => {
            let ext = ext.to_string_lossy().to_lowercase();
            ext == "jpg" || ext == "jpeg" || ext == "png" || ext == "gif" || ext == "bmp"
        }
        None => false,
    }
}

impl MyApp {
    fn process_path(&mut self, path: Option<PathBuf>) {
        if let Some(path) = &path {
            if let Some(option) = &self.selected_option {
                match option {
                    BaseOption::PicUpload => {
                        if is_image(&path) {
                            self.image_path = Some(PathBuf::from(path));
                            self.return_path = Some("Uploading ... ".to_string());
                            match upload::upload_file_path(&path) {
                                Ok(url) => self.return_path = Some(url),
                                Err(err) => self.return_path = Some(err.to_string()),
                            }
                        } else {
                            self.return_path = Some(format!("Not a image file: {:?}", path));
                        }
                    }
                    BaseOption::FileMD5 => {
                        let md5 = Self::calculate_md5(&path).unwrap();
                        self.return_path = Some(format!("MD5: {}", md5));
                    }
                }
            }
        }
    }

    fn calculate_md5(file_path: &PathBuf) -> Option<String> {
        use std::fs::File;
        use std::io::Read;

        let mut file = File::open(file_path).ok()?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).ok()?;

        let digest = md5::compute(&buffer);
        let str = format!("{:x}", digest);
        // 获取原始文件名并添加_md5.txt后缀
        let file_name = file_path.file_name().unwrap().to_str().unwrap();
        let new_file_name = format!("{}_md5.txt", file_name);

        // 生成新文件路径，直接添加新后缀
        let new_file_path = file_path.with_file_name(new_file_name);

        // 将MD5写入剪贴板
        let _ = clipboard_anywhere::set_clipboard(str.as_str());
        // 保存MD5摘要到新文件
        std::fs::write(&new_file_path, &str).unwrap();
        Some(str)
    }
}

impl Application for MyApp {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: ()) -> (MyApp, Command<Message>) {
        (
            MyApp {
                return_path: None,
                image_path: None,
                selected_option: Some(BaseOption::PicUpload),
                option_hint: "".to_string(),
                button_txt: "Select File".to_string(),
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("文件MD5生成器 & 图片上传工具")
    }
    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::FileDropped(path) => {
                self.process_path(path);
                Command::none()
            }

            Message::OpenImgPressed => Command::perform(
                async move {
                    FileDialog::new()
                        .set_title("选择一张图片")
                        .add_filter("image", &["png", "jpg"])
                        .set_directory("/")
                        .pick_file()
                },
                Message::FileDropped,
            ),
            Message::OtherEvent() => Command::none(),
            Message::OnSelectOption(option) => {
                self.selected_option = Some(option);
                self.option_hint = option.hint().to_string();
                self.button_txt = option.button_txt().to_string();
                Command::none()
            }
        }
    }
    fn view(&self) -> Element<Message> {
        let _label = if let Some(path) = &self.return_path {
            widget::Text::from(path.as_str()).shaping(Advanced)
        } else {
            widget::Text::from("等待文件拖放！").shaping(Advanced)
        };

        let _path_text = if let Some(path) = &self.image_path {
            iced::widget::text(format!("Dropped Image Path: {:?}", path.display()))
        } else {
            let mut content: String = "选择或拖入待上传的图片".to_string();
            match self.selected_option {
                None => {}
                Some(option) => {
                    match option {
                        BaseOption::PicUpload => {}
                        BaseOption::FileMD5 => { content = "选择或拖入需要生成MD5的文件".to_string() }
                    }
                }
            }
            iced::widget::Text::new(content).shaping(Advanced)
        };
        let _btn_open_image = Button::new(
            iced::widget::text(self.button_txt.as_str())
                .horizontal_alignment(iced::alignment::Horizontal::Center)
                .vertical_alignment(iced::alignment::Vertical::Center)
                .size(15),
        )
            .on_press(Message::OpenImgPressed);

        // let combo_box = combo_box(
        //     &self.options,
        //     "选择一个功能",
        //     self.selected_option.as_ref(),
        //     Message::OnSelectOption,
        // )
        //     .width(250)
        //     .on_option_hovered(Message::OptionHovered);

        let _pick_list = pick_list(
            &BaseOption::ALL[..],
            self.selected_option,
            Message::OnSelectOption,
        )
            .placeholder("选择功能")
            .text_shaping(Advanced)
            .width(200);

        let _option_hint = iced::widget::Text::new(self.option_hint.clone())
            .horizontal_alignment(iced::alignment::Horizontal::Center)
            .vertical_alignment(iced::alignment::Vertical::Center)
            .shaping(Advanced);
        let _row = row![
            iced::widget::text("功能选项：").shaping(Advanced),
            _pick_list,
            _option_hint,
        ]
            .spacing(20)
            .width(Length::Fill)
            ;

        let content = column![
            _label,
            _btn_open_image,
            _path_text,
            _row,
        ]
            .width(Length::Fill)
            .align_items(Alignment::Center)
            .spacing(20)
            .padding(20);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }

    fn theme(&self) -> Self::Theme {
        Theme::Dracula
    }

    fn subscription(&self) -> Subscription<Message> {
        event::listen().map(|event| match event {
            iced::Event::Window(iced::window::Id::MAIN, iced::window::Event::FileDropped(stream)) => {
                Message::FileDropped(Some(stream))
            }
            _ => Message::OtherEvent()
        })
        // Subscription::none()
    }
}

