use iced::highlighter;
use iced::theme;
use iced::{
    Element, Font, Length, Task, Theme,
    widget::{button, column, container, horizontal_space, row, text, text_editor, tooltip},
};
use smol::io;
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub fn main() -> iced::Result {
    iced::application("Iced Editor", Editor::update, Editor::view)
        .theme(Editor::theme)
        .default_font(Font::MONOSPACE)
        .font(include_bytes!("../fonts/editor-icons.ttf").as_slice())
        .run_with(Editor::new)
}

struct Editor {
    path: Option<PathBuf>,
    content: text_editor::Content,
    error: Option<Error>,
}

impl Editor {
    fn new() -> (Self, Task<Message>) {
        (
            Self {
                path: None,
                content: text_editor::Content::new(),
                error: None,
            },
            Task::perform(load_file(default_file()), Message::FileOpened),
        )
    }
}

#[derive(Debug, Clone)]
enum Message {
    Edit(text_editor::Action),
    New,
    Open,
    FileOpened(Result<(PathBuf, Arc<String>), Error>),
    Save,
    FileSaved(Result<PathBuf, Error>),
}

impl Editor {
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Edit(action) => {
                self.content.perform(action);
                self.error = None;
                Task::none()
            }
            Message::New => {
                self.path = None;
                self.content = text_editor::Content::new();
                Task::none()
            }
            Message::Open => Task::perform(pick_file(), Message::FileOpened),
            Message::Save => {
                let text = self.content.text();

                Task::perform(save_file(self.path.clone(), text), Message::FileSaved)
            }
            Message::FileSaved(Ok((path))) => {
                self.path = Some(path);
                Task::none()
            }
            Message::FileSaved(Err(error)) => {
                self.error = Some(error);
                Task::none()
            }
            Message::FileOpened(Ok((path, contents))) => {
                self.path = Some(path);
                self.content = text_editor::Content::with_text(&contents);
                Task::none()
            }
            Message::FileOpened(Err(error)) => {
                self.error = Some(error);
                Task::none()
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let controls = row![
            action(new_icon(), "New file", Message::New),
            action(open_icon(), "Open file", Message::Open),
            action(save_icon(), "Save file", Message::Save)
        ]
        .spacing(10);

        let input = text_editor(&self.content)
            .height(Length::Fill)
            .highlight(
                self.path
                    .as_ref()
                    .and_then(|path| path.extension()?.to_str())
                    .unwrap_or("rs"),
                highlighter::Theme::SolarizedDark,
            )
            .on_action(Message::Edit);

        let position = {
            let (line, column) = self.content.cursor_position();

            text(format!("{}:{}", line + 1, column + 1))
        };

        let status = if let Some(Error::IOFailed(error)) = self.error.as_ref() {
            text(error.to_string())
        } else {
            match self.path.as_deref().and_then(Path::to_str) {
                Some(path) => text(path).size(14),
                None => text("New file"),
            }
        };

        let status_bar = row![status, horizontal_space(), position];

        container(column![controls, input, status_bar].spacing(10))
            .padding(10)
            .into()
    }

    fn theme(&self) -> Theme {
        Theme::Dark
    }
}

async fn pick_file() -> Result<(PathBuf, Arc<String>), Error> {
    let handle = rfd::AsyncFileDialog::new()
        .set_title("Choose a text file...")
        .pick_file()
        .await
        .ok_or(Error::DialogClosed)?;

    load_file(handle.path().to_owned()).await
}

async fn load_file(path: PathBuf) -> Result<(PathBuf, Arc<String>), Error> {
    let contents = smol::fs::read_to_string(&path)
        .await
        .map(Arc::new)
        .map_err(|error| error.kind())
        .map_err(Error::IOFailed)?;

    Ok((path, contents))
}

async fn save_file(path: Option<PathBuf>, text: String) -> Result<PathBuf, Error> {
    let path = if let Some(path) = path {
        path
    } else {
        rfd::AsyncFileDialog::new()
            .set_title("Choose a file name...")
            .save_file()
            .await
            .ok_or(Error::DialogClosed)
            .map(|handle| handle.path().to_owned())?
    };

    smol::fs::write(&path, text)
        .await
        .map_err(|error| Error::IOFailed(error.kind()))?;

    Ok(path)
}

fn default_file() -> PathBuf {
    PathBuf::from(format!("{}/src/main.rs", env!("CARGO_MANIFEST_DIR")))
}

fn new_icon<'a>() -> Element<'a, Message> {
    icon('\u{E800}')
}

fn save_icon<'a>() -> Element<'a, Message> {
    icon('\u{E801}')
}

fn open_icon<'a>() -> Element<'a, Message> {
    icon('\u{F115}')
}

fn icon<'a>(codepoint: char) -> Element<'a, Message> {
    const ICON_FONT: Font = Font::with_name("editor-icons");
    text(codepoint).font(ICON_FONT).into()
}

fn action<'a>(
    content: Element<'a, Message>,
    label: &'a str,
    on_press: Message,
) -> Element<'a, Message> {
    tooltip(
        button(container(content).center_x(30))
            .on_press(on_press)
            .padding([5, 10]),
        label,
        tooltip::Position::FollowCursor,
    )
    .style(|theme| container::Style {
        background: Some(theme.palette().background.into()),
        ..Default::default()
    })
    .into()
}

#[derive(Debug, Clone)]
enum Error {
    DialogClosed,
    IOFailed(io::ErrorKind),
}
