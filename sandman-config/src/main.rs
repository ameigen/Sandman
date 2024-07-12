use iced::{
    alignment,
    widget::{button, column, container, row, scrollable, text, Column},
    Element, Length, Sandbox, Settings,
};

// https://leafheap.com/articles/iced-tutorial-version-0-12

#[derive(Debug, Clone)]
enum Message {
    InputValue(String),
    Submitted,
    DeleteItem(usize),
}

struct Directory {
    directory: String,
    prefix: String,
    bucket: String,
}

/**
 * This is your model. It contains all the data needed for your application to work properly.
 * The model can only be updated with the `update` function.
 */
struct SandmanConfiguration {
    aws_access_key_id: String,
    aws_default_region: String,
    aws_secret_access_key: String,
    directories: Vec<Directory>,
}

impl Sandbox for SandmanConfiguration {
    type Message = Message;

    /* Initialize your app */
    fn new() -> SandmanConfiguration {
        Self {
            aws_access_key_id: "AWS_ACCESS_KEY".to_string(),
            aws_default_region: "AWS_REGION".to_string(),
            aws_secret_access_key: "AWS_SECRET_KEY".to_string(),
            directories: vec![],
        }
    }

    /**
     * The title of the window. It will show up on the top of your application window.
     */
    fn title(&self) -> String {
        String::from("Sandman Configurator")
    }

    fn update(&mut self, message: Self::Message) {}

    fn view(&self) -> Element<Self::Message> {
        container(
            column!(
                item_list_view(&self.directories),
                row!(
                    text_input("AWS Region", &self.aws_defualt_region)
                        .on_input(|value| Message::InputValue(value))
                        .on_submit(Message::Submitted),
                    text_input("AWS Secret", &self.aws_secret_access_key)
                        .on_input(|value| Message::InputValue(value))
                        .on_submit(Message::Submitted),
                    text_input("AWS ID", &self.aws_access_key_id)
                        .on_input(|value| Message::InputValue(value))
                        .on_submit(Message::Submitted),
                    button("Submit").on_press(Message::Submitted)
                )
                .spacing(30)
                .padding(Padding::from(30))
            )
            .align_items(iced::Alignment::Center),
        )
        .height(Length::Fill)
        .width(Length::Fill)
        .align_x(alignment::Horizontal::Center)
        .align_y(alignment::Vertical::Center)
        .into()
    }

    fn theme(&self) -> iced::Theme {
        iced::Theme::Dark
    }
}

fn item_list_view(items: &Vec<Directory>) -> Element<'static, Message> {
    let mut column = Column::new()
        .spacing(20)
        .align_items(iced::Alignment::Center)
        .width(Length::Fill);
    for (index, value) in items.into_iter().enumerate() {
        column = column.push(directory(index, value));
    }

    scrollable(container(column))
        .height(250.0)
        .width(300)
        .into()
}

fn directory(index: usize, value: &Directory) -> Element<'static, Message> {
    row!(
        text(format!(
            "{} {} {}",
            value.bucket, value.bucket, value.directory
        )),
        button("Delete").on_press(Message::DeleteItem(index))
    )
    .align_items(iced::Alignment::Center)
    .spacing(30)
    .into()
}

pub fn main() -> iced::Result {
    SandmanConfiguration::run(Settings::default())
}
