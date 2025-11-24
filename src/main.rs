use iced::{
    Element, Length, Task, Theme,
    widget::{column, container, horizontal_space, row, text, text_editor},
};
use smol::io;
use std::path::Path;
use std::sync::Arc;

pub fn main() -> iced::Result {
    iced::application("Iced Editor", Editor::update, Editor::view)
        .theme(Editor::theme)
        .run_with(Editor::new)
}

struct Editor {
    content: text_editor::Content,
}

impl Editor {
    fn new() -> (Self, Task<Message>) {
        (
            Self {
                content: text_editor::Content::new(),
            },
            Task::perform(
                load_file(format!("{}/src/main.rs", env!("CARGO_MANIFEST_DIR"))),
                Message::FileLoaded,
            ),
        )
    }
}

#[derive(Debug, Clone)]
enum Message {
    Edit(text_editor::Action),
    FileLoaded(Result<Arc<String>, io::ErrorKind>),
}

impl Editor {
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Edit(action) => {
                self.content.perform(action);
                Task::none()
            }
            Message::FileLoaded(Ok(contents)) => {
                self.content = text_editor::Content::with_text(&contents);
                Task::none()
            }
            Message::FileLoaded(Err(error)) => {
                self.content =
                    text_editor::Content::with_text(&format!("Error loading file: {:?}", error));
                Task::none()
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let input = text_editor(&self.content)
            .height(Length::Fill)
            .on_action(Message::Edit);

        let position = {
            let (line, column) = self.content.cursor_position();

            text(format!("{}:{}", line + 1, column + 1))
        };

        let status_bar = row![horizontal_space(), position];

        container(column![input, status_bar].spacing(10))
            .padding(10)
            .into()
    }

    fn theme(&self) -> Theme {
        Theme::Dark
    }
}

async fn load_file(path: impl AsRef<Path>) -> Result<Arc<String>, io::ErrorKind> {
    smol::fs::read_to_string(path)
        .await
        .map(Arc::new)
        .map_err(|error| error.kind())
}
