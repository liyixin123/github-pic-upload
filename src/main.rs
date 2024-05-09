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
    DraggedImage(iced::Event),
    ImageDropped(Option<PathBuf>),
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
}

struct MyApp {
    return_path: Option<String>,
    image_path: Option<PathBuf>,
    selected_option: Option<BaseOption>,
    option_hint: String,
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
                selected_option: None,
                option_hint: "".to_string(),
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("拖入图片，成功后直接粘贴即可")
    }
    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::ImageDropped(path) => {
                if let Some(path) = &path {
                    self.image_path = Some(PathBuf::from(path));
                    self.return_path = Some("Uploading ... ".to_string());
                    match upload::upload_file_path(&path) {
                        Ok(url) => self.return_path = Some(url),
                        Err(err) => self.return_path = Some(err.to_string()),
                    }
                }
                Command::none()
            }
            Message::DraggedImage(event) => {
                println!("dragged Image");
                Command::perform(handle_dragged_image(event), Message::ImageDropped)
            }
            Message::OpenImgPressed => Command::perform(
                async move {
                    FileDialog::new()
                        .set_title("选择一张图片")
                        .add_filter("image", &["png", "jpg"])
                        .set_directory("/")
                        .pick_file()
                },
                Message::ImageDropped,
            ),
            Message::OtherEvent() => Command::none(),
            Message::OnSelectOption(option) => {
                self.selected_option = Some(option);
                self.option_hint = option.hint().to_string();
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
            iced::widget::text("Upload Image")
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

    fn subscription(&self) -> Subscription<Message> {
        event::listen().map(|event| match event {
            iced::Event::Window(iced::window::Id::MAIN, iced::window::Event::FileDropped(_)) => {
                Message::DraggedImage(event)
            }
            _ => Message::OtherEvent(),
        })
        // Subscription::none()
    }

    fn theme(&self) -> Self::Theme {
        Theme::Dracula
    }
}

//
async fn handle_dragged_image(event: iced::Event) -> Option<PathBuf> {
    if let iced::Event::Window(iced::window::Id::MAIN, iced::window::Event::FileDropped(file)) =
        event
    {
        Some(file)
    } else {
        None
    }
}
