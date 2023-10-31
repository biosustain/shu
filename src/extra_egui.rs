use bevy_egui::egui::{Link, Widget, WidgetText};

/// Clickable hyperlink, same as [`bevy_egui::egui::Hyperlink`] but it always
/// opens the url in a new tab.
#[must_use = "You should put this widget in an ui with `ui.add(widget);`"]
pub struct NewTabHyperlink {
    url: &'static str,
    text: WidgetText,
}

impl NewTabHyperlink {
    pub fn from_label_and_url(text: impl Into<WidgetText>, url: &'static str) -> Self {
        Self {
            url,
            text: text.into(),
        }
    }
}
impl Widget for NewTabHyperlink {
    fn ui(self, ui: &mut bevy_egui::egui::Ui) -> bevy_egui::egui::Response {
        let Self { url, text } = self;

        let response = ui.add(Link::new(text));
        if response.clicked() | response.middle_clicked() {
            ui.ctx().output_mut(|o| {
                o.open_url = Some(bevy_egui::egui::output::OpenUrl {
                    url: url.to_string(),
                    new_tab: true,
                });
            });
        }
        response.on_hover_text(url)
    }
}
