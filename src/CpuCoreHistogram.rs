pub struct CpuPerCoreHistogram<'a> {
    config: BarConfig,
    container: container::Container<'a, Message, Theme>,
}

impl<'a> std::ops::Deref for CpuPerCoreHistogram<'a> {
    type Target = container::Container<'a, Message, Theme>;

    fn deref(&self) -> &Self::Target {
        &self.container
    }
}

impl<'a> std::ops::DerefMut for CpuPerCoreHistogram<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.container
    }
}

impl<'a> Widget<Message, Theme, Renderer> for CpuPerCoreHistogram<'a> {
    fn children(&self) -> Vec<Tree> {
        self.container.children()
    }
    fn diff(&mut self, tree: &mut Tree) {
        self.container.diff(tree);
    }
    fn size(&self) -> Size<Length> {
        // Size {
        //     height: self.config.height,
        //     width: match self.config.per_cpu_width {
        //         Length::Fixed(float) => Length::Fixed(float * self.cpus.len() as f32),
        //         flex => flex,
        //     },
        // }
        self.container.size()
    }

    fn layout(
        &self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let Size { width, height } =
            <CpuPerCoreHistogram<'_> as Widget<Message, Theme, Renderer>>::size(self);

        layout::atomic(limits, width, height)
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        dbg!(&layout);
        self.container
            .draw(tree, renderer, theme, style, layout, cursor, viewport);
    }
    fn drag_destinations(
        &self,
        state: &Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        dnd_rectangles: &mut core::clipboard::DndDestinationRectangles,
    ) {
        self.container
            .drag_destinations(state, layout, renderer, dnd_rectangles);
    }

    fn state(&self) -> core::widget::tree::State {
        self.container.state()
    }
    fn tag(&self) -> core::widget::tree::Tag {
        self.container.tag()
    }
    fn operate(
        &self,
        state: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn core::widget::Operation,
    ) {
        self.container.operate(state, layout, renderer, operation);
    }
    fn size_hint(&self) -> Size<Length> {
        self.container.size_hint()
    }
    fn mouse_interaction(
        &self,
        state: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.container
            .mouse_interaction(state, layout, cursor, viewport, renderer)
    }
}
