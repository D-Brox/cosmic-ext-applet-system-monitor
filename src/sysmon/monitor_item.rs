use crate::applet::Message;
use crate::helpers::collection;
use cosmic::{applet, iced::Pixels, Element, Theme};

pub trait MonitorItem {
    type View;
    type ConfigStruct;

    fn new(c: Self::ConfigStruct, theme: &Theme) -> Self;

    fn view<'a>(
        &'a self,
        context: &'a applet::Context,
        spacing: impl Into<Pixels>,
    ) -> Element<'a, Message> {
        collection(
            context,
            self.view_order()
                .iter()
                .map(|v| self.view_single(v, context)),
            spacing,
        )
    }

    fn view_order(&self) -> &[Self::View];

    fn view_single(&self, chart_view: &Self::View, context: &applet::Context) -> Element<Message>;

    fn resize_queue(&mut self, samples: usize);

    fn update_aspect_ratio(&mut self, aspect_ratio: f32);
}
