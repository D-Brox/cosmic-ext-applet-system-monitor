use cosmic::{applet, Element, Theme};
use plotters_iced::Chart;
use crate::applet::Message;

pub trait MonitorItem<V>: Chart<Message> {
    fn view_as_configured(&self, context: &applet::Context) -> Element<Message>;
    fn view_as(&self, chart_view: V, context: &applet::Context) -> Element<Message>;
}
