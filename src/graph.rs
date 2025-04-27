use embedded_graphics::{
    draw_target::DrawTarget,
    pixelcolor::BinaryColor,
    prelude::{Point, Size},
    primitives::Rectangle,
    Drawable,
};
use embedded_layout::prelude::*;
use embedded_plots::{
    axis::Scale,
    curve::{Curve, PlotPoint},
    single_plot::SinglePlot,
};

pub struct MoisturePlot<'a> {
    bounds: Rectangle,
    points: &'a [PlotPoint],
}

impl<'a> MoisturePlot<'a> {
    /// The progress bar has a configurable position and size
    pub fn new(points: &'a [PlotPoint], position: Point, size: Size) -> Self {
        Self {
            bounds: Rectangle::new(position, size),
            points,
        }
    }
}

/// Implementing `View` is required by the layout and alignment operations
/// `View` teaches `embedded-layout` where our object is, how big it is and how to move it.
impl View for MoisturePlot<'_> {
    #[inline]
    fn translate_impl(&mut self, by: Point) {
        // make sure you don't accidentally call `translate`!
        self.bounds.translate_mut(by);
    }

    #[inline]
    fn bounds(&self) -> Rectangle {
        self.bounds
    }
}

/// Need to implement `Drawable` for a _reference_ of our view
impl<'a> Drawable for MoisturePlot<'a> {
    type Color = BinaryColor;
    type Output = ();

    fn draw<D: DrawTarget<Color = BinaryColor>>(&self, display: &mut D) -> Result<(), D::Error> {
        let curve = Curve::from_data(self.points);
        let plot = SinglePlot::new(&curve, Scale::RangeFraction(3), Scale::RangeFraction(2))
            .into_drawable(self.bounds.top_left, self.bounds.bottom_right().unwrap())
            .set_color(BinaryColor::On)
            .set_text_color(BinaryColor::On);

        // Draw views
        plot.draw(display)?;

        Ok(())
    }
}
