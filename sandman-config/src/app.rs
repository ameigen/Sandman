use iced::widget::{button, column, container, row, scrollable, text, text_input, Column};
use iced::{alignment, Element, Length, Padding, Sandbox};
use sandman_share::config::SandmanDirectory;

#[derive(Clone, Debug)]
pub enum Inputs {
    AccessKey,
    Region,
    SecretAccessKey,
    Directory,
    Prefix,
    Bucket,
}

#[derive(Debug, Clone)]
pub enum Message {
    InputChanged(Inputs, String),
    Submitted,
    DeleteItem,
}

pub struct SandmanConfigApp {
    aws_access_key_id: String,
    aws_default_region: String,
    aws_secret_access_key: String,
    directories: Vec<SandmanDirectory>,
    directory_entry: String,
    prefix_entry: String,
    bucket_entry: String,
}

fn on_secret_id_set(value: String) -> Message {
    Message::InputChanged(Inputs::AccessKey, value)
}
fn on_id_set(value: String) -> Message {
    Message::InputChanged(Inputs::SecretAccessKey, value)
}
fn on_region_set(value: String) -> Message {
    Message::InputChanged(Inputs::Region, value)
}

fn on_prefix_set(value: String) -> Message {
    Message::InputChanged(Inputs::Prefix, value)
}
fn on_bucket_set(value: String) -> Message {
    Message::InputChanged(Inputs::Bucket, value)
}
fn on_directory_set(value: String) -> Message {
    Message::InputChanged(Inputs::Directory, value)
}

impl Sandbox for SandmanConfigApp {
    type Message = Message;

    /* Initialize your app */
    fn new() -> SandmanConfigApp {
        Self {
            aws_access_key_id: "AWS_ACCESS_KEY".to_string(),
            aws_default_region: "AWS_REGION".to_string(),
            aws_secret_access_key: "AWS_SECRET_KEY".to_string(),
            directories: vec![],
            directory_entry: "".to_string(),
            prefix_entry: "".to_string(),
            bucket_entry: "".to_string(),
        }
    }

    /**
     * The title of the window. It will show up on the top of your application window.
     */
    fn title(&self) -> String {
        String::from("Sandman Configurator")
    }

    fn update(&mut self, message: Self::Message) {
        println!("{:?}", message);
        match message {
            Message::InputChanged(input, value) => match input {
                Inputs::AccessKey => self.aws_access_key_id = value,
                Inputs::Region => self.aws_default_region = value,
                Inputs::SecretAccessKey => self.aws_secret_access_key = value,
                Inputs::Bucket => self.bucket_entry = value,
                Inputs::Directory => self.directory_entry = value,
                Inputs::Prefix => self.prefix_entry = value,
            },
            Message::Submitted => {
                let dir = SandmanDirectory {
                    directory: self.directory_entry.clone(),
                    prefix: self.prefix_entry.clone(),
                    bucket: self.bucket_entry.clone(),
                };
                self.directories.push(dir);
            }
            Message::DeleteItem => {}
        }
    }

    fn view(&self) -> Element<Self::Message> {
        container(
            column!(
                item_list_view(&self.directories),
                column!(
                    entry_with_title("AWS Region", &self.aws_default_region, on_region_set),
                    entry_with_title("AWS ID", &self.aws_access_key_id, on_id_set),
                    entry_with_title(
                        "AWS Secret ID",
                        &self.aws_secret_access_key,
                        on_secret_id_set
                    ),
                    directory_entry(
                        &self.directory_entry,
                        &self.prefix_entry,
                        &self.bucket_entry
                    ),
                    button("Submit").on_press(Message::Submitted),
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

fn entry_with_title(
    title: &str,
    default: &String,
    set_callback: fn(String) -> Message,
) -> Element<'static, Message> {
    row!(
        text(format!("{}:", title)),
        text_input(default, default).on_input(move |value| set_callback(value)),
    )
    .align_items(iced::Alignment::Start)
    .spacing(30)
    .into()
}

fn directory_entry(
    directory: &String,
    prefix: &String,
    bucket: &String,
) -> Element<'static, Message> {
    container::Container::new(
        row!(
            entry_with_title("Directory", directory, on_directory_set),
            entry_with_title("Prefix", prefix, on_prefix_set),
            entry_with_title("Bucket", bucket, on_bucket_set),
        )
        .align_items(iced::Alignment::Center)
        .spacing(30),
    )
    .into()
}

fn item_list_view(items: &Vec<SandmanDirectory>) -> Element<'static, Message> {
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

fn directory(_index: usize, value: &SandmanDirectory) -> Element<'static, Message> {
    row!(
        text(format!(
            "{} {} {}",
            value.bucket, value.bucket, value.directory
        )),
        button("Delete").on_press(Message::DeleteItem)
    )
    .align_items(iced::Alignment::Center)
    .spacing(30)
    .into()
}
