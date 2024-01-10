mod upload;

use std::path::PathBuf;
use std::option::Option;
use iced::{executor, Command, Element, Settings, Theme, Alignment, Application, Length, Subscription};
use iced::widget::{column,  text, container, Text};


pub fn main() -> iced::Result {
    MyApp::run(Settings::default())
}

#[derive(Debug, Clone)]
enum Message {
    DraggedImage(iced::Event),
    ImageDropped(Option<PathBuf>),
    OtherEvent(iced::Event),
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
                        Ok(url) => { self.return_path = Some(url) }
                        Err(err) => { self.return_path = Some(err.to_string()) }
                    }
                }

                Command::none()
            }
            Message::DraggedImage(event) => {
                println!("dragged Image");
                Command::perform(handle_dragged_image(event), Message::ImageDropped)
            }
            _ => { Command::none() }
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
        let content = column![
            _label,
            _path_text,
        ]
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

    // how to fix it
    fn subscription(&self) -> Subscription<Message> {
        iced::subscription::events().map(|event| match event {
            iced::Event::Window(iced::window::Event::FileDropped(_)) => Message::DraggedImage(event),
            _ => { Message::OtherEvent(event) }
        })
        // Subscription::none()
    }
}

// comment this function

async fn handle_dragged_image(event: iced::Event) -> Option<PathBuf> {
    if let iced::Event::Window(iced::window::Event::FileDropped(file)) = event {
        Some(file)
    } else { None }
}