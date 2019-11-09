use euclid::{size2, Length, SideOffsets2D, Size2D};
pub struct LogicalPixel;

pub type LogicalSize = Size2D<f32, LogicalPixel>;
pub type LogicalLength = Length<f32, LogicalPixel>;
pub type LogicalSideOffsets = SideOffsets2D<f32, LogicalPixel>;

pub struct Layout {}

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
        Layout {}
    }

    pub fn calc_max_size(&self, opts: &LayoutOptions, parent_size: LogicalSize) -> LogicalSize {
        let mut outer = parent_size;
        if let Some(width) = opts.width {
            outer.width = width.get();
        }
        if let Some(height) = opts.height {
            outer.height = height.get();
        }
        outer - size2(opts.padding.horizontal(), opts.padding.vertical())
    }

    pub fn calc_min_size(&self, opts: &LayoutOptions, child_sizes: &[LogicalSize]) -> LogicalSize {
        let mut width = 0.0f32;
        let mut height = 0.0f32;
        for child in child_sizes {
            width = width.max(child.width);
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
        outer
    }
}

impl Default for Layout {
    fn default() -> Self {
        Self::new()
    }
}
