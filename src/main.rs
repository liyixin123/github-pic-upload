mod upload;

use iced::widget::{column, container, text, Button, Text};
use iced::{
    event, executor, Alignment, Application, Command, Element, Font, Length, Settings,
    Subscription, Theme,
};
use rfd::FileDialog;
use std::option::Option;
use std::path::PathBuf;

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
    OtherEvent(),
}

struct MyApp {
    return_path: Option<String>,
    image_path: Option<PathBuf>,
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
        }
    }
    fn view(&self) -> Element<Message> {
        let _label: Text = if let Some(path) = &self.return_path {
            text(path)
        } else {
            text("Wait for drop file!")
        };
        let _path_text = if let Some(path) = &self.image_path {
            text(format!("Dropped Image Path: {:?}", path.display()))
        } else {
            text("Drop an image here")
        };
        let btn_open_image = Button::new(
            text("Upload Image")
                .horizontal_alignment(iced::alignment::Horizontal::Center)
                .vertical_alignment(iced::alignment::Vertical::Center)
                .size(15),
        )
        .on_press(Message::OpenImgPressed);

        let content = column![btn_open_image, _label, _path_text,]
            .width(Length::Fill)
            .align_items(Alignment::Center)
            .spacing(10)
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
