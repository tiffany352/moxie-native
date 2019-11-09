use euclid::{point2, size2, Length, Point2D, SideOffsets2D, Size2D};
pub struct LogicalPixel;

pub type LogicalPoint = Point2D<f32, LogicalPixel>;
pub type LogicalSize = Size2D<f32, LogicalPixel>;
pub type LogicalLength = Length<f32, LogicalPixel>;
pub type LogicalSideOffsets = SideOffsets2D<f32, LogicalPixel>;

pub struct Layout {
    max_size: LogicalSize,
    min_size: LogicalSize,
    child_positions: Vec<LogicalPoint>,
}

pub struct LayoutOptions {
    pub padding: LogicalSideOffsets,
    pub width: Option<LogicalLength>,
    pub height: Option<LogicalLength>,
}

impl Default for LayoutOptions {
    fn default() -> Self {
        LayoutOptions {
            padding: LogicalSideOffsets::new_all_same(0.0f32),
            width: None,
            height: None,
        }
    }
}

impl Layout {
    pub fn new() -> Layout {
        Layout {
            max_size: LogicalSize::new(0.0, 0.0),
            min_size: LogicalSize::new(0.0, 0.0),
            child_positions: vec![],
        }
    }

    pub fn calc_max_size(&mut self, opts: &LayoutOptions, parent_size: LogicalSize) -> LogicalSize {
        let mut outer = parent_size;
        if let Some(width) = opts.width {
            outer.width = width.get();
        }
        if let Some(height) = opts.height {
            outer.height = height.get();
        }
        self.max_size = outer - size2(opts.padding.horizontal(), opts.padding.vertical());
        self.max_size
    }

    pub fn calc_min_size(
        &mut self,
        opts: &LayoutOptions,
        child_sizes: &[LogicalSize],
    ) -> LogicalSize {
        let mut width = 0.0f32;
        let mut height = 0.0f32;
        for child in child_sizes {
            width = width.max(child.width);
            self.child_positions
                .push(point2(opts.padding.left, height + opts.padding.top));
            height += child.height;
        }
        let mut outer =
            size2(width, height) + size2(opts.padding.horizontal(), opts.padding.vertical());
        if let Some(width) = opts.width {
            outer.width = width.get();
        }
        if let Some(height) = opts.height {
            outer.height = height.get();
        }
        self.min_size = outer;
        outer
    }

    pub fn size(&self) -> LogicalSize {
        self.min_size
    }

    pub fn child_positions(&self) -> &[LogicalPoint] {
        &self.child_positions[..]
    }
}

impl Default for Layout {
    fn default() -> Self {
        Self::new()
    }
}
