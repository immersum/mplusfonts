use embedded_graphics::geometry::{Point, Size};
use embedded_graphics::primitives::Rectangle;

/// Extension trait for rectangles.
pub trait RectangleExt {
    /// Returns the left half of the rectangle, rounding down the resulting width.
    fn left_half(&self) -> Self;

    /// Returns the right half of the rectangle, rounding up the resulting width.
    fn right_half(&self) -> Self;

    /// Returns the rectangle's area that only has pixels to the left of the specified area.
    fn left_of(&self, other: &Self) -> Self;

    /// Returns the rectangle's area that only has pixels to the right of the specified area.
    fn right_of(&self, other: &Self) -> Self;

    /// Returns the rectangle's area that only has pixels above the specified area.
    fn above(&self, other: &Self) -> Self;

    /// Returns the rectangle's area that only has pixels below the specified area.
    fn below(&self, other: &Self) -> Self;

    /// Returns the rectangle with its area extended along the _y_-axis to the specified top and
    /// bottom rows, excluding the bottom row itself.
    fn y_extend(&self, top: i32, bottom: i32) -> Self;

    /// Returns the rectangle with its area reduced along the _y_-axis to the specified top and
    /// bottom rows, excluding the bottom row itself.
    fn y_reduce(&self, top: i32, bottom: i32) -> Self;

    /// Returns the rectangle with its left side indented to the right, making the specified column
    /// its new left side.
    fn indent_to(&self, right: i32) -> Self;

    /// Returns the rectangle with its right side extruded to the right, making the specified column
    /// its new right side.
    fn extrude_to(&self, right: i32) -> Self;
}

impl RectangleExt for Rectangle {
    fn left_half(&self) -> Self {
        let width = self.size.width.checked_div(2).unwrap_or_default();
        let top_left = self.top_left;
        let size = Size::new(width, self.size.height);

        Self { top_left, size }
    }

    fn right_half(&self) -> Self {
        let width = self.size.width.checked_div(2).unwrap_or_default();
        let left = self.top_left.x.saturating_add_unsigned(width);
        let top_left = Point::new(left, self.top_left.y);
        let size = Size::new(width, Default::default());
        let size = self.size.saturating_sub(size);

        Self { top_left, size }
    }

    fn left_of(&self, other: &Self) -> Self {
        let top_left = self.top_left;
        let width = other.top_left.x.saturating_sub(self.top_left.x);
        let width = width.try_into().unwrap_or_default();
        let size = Size::new(width, self.size.height);
        let size = self.size.component_min(size);

        Self { top_left, size }
    }

    fn right_of(&self, other: &Self) -> Self {
        let right = other.top_left.x.saturating_add_unsigned(other.size.width);
        let top_left = Point::new(right, self.top_left.y);
        let top_left = self.top_left.component_max(top_left);
        let width = right.saturating_sub(self.top_left.x);
        let width = width.try_into().unwrap_or_default();
        let size = Size::new(width, Default::default());
        let size = self.size.saturating_sub(size);

        Self { top_left, size }
    }

    fn above(&self, other: &Self) -> Self {
        let top_left = self.top_left;
        let height = other.top_left.y.saturating_sub(self.top_left.y);
        let height = height.try_into().unwrap_or_default();
        let size = Size::new(self.size.width, height);
        let size = self.size.component_min(size);

        Self { top_left, size }
    }

    fn below(&self, other: &Self) -> Self {
        let bottom = other.top_left.y.saturating_add_unsigned(other.size.height);
        let top_left = Point::new(self.top_left.x, bottom);
        let top_left = self.top_left.component_max(top_left);
        let height = bottom.saturating_sub(self.top_left.y);
        let height = height.try_into().unwrap_or_default();
        let size = Size::new(Default::default(), height);
        let size = self.size.saturating_sub(size);

        Self { top_left, size }
    }

    fn y_extend(&self, top: i32, bottom: i32) -> Self {
        let extent = self.top_left.y.saturating_sub(top);
        let extent = extent.try_into().unwrap_or_default();
        let new_size = Size::new(Default::default(), extent);
        let new_size = self.size.saturating_add(new_size);
        let top_left = Point::new(self.top_left.x, top);
        let top_left = self.top_left.component_min(top_left);
        let height = bottom.saturating_sub(top_left.y);
        let height = height.try_into().unwrap_or_default();
        let size = Size::new(Default::default(), height);
        let size = new_size.component_max(size);

        Self { top_left, size }
    }

    fn y_reduce(&self, top: i32, bottom: i32) -> Self {
        let top_left = Point::new(self.top_left.x, top);
        let top_left = self.top_left.component_max(top_left);
        let height = bottom.saturating_sub(top_left.y);
        let height = height.try_into().unwrap_or_default();
        let size = Size::new(self.size.width, height);
        let size = self.size.component_min(size);

        Self { top_left, size }
    }

    fn indent_to(&self, right: i32) -> Self {
        let top_left = Point::new(right, self.top_left.y);
        let top_left = self.top_left.component_max(top_left);
        let width = right.saturating_sub(self.top_left.x);
        let width = width.try_into().unwrap_or_default();
        let size = Size::new(width, Default::default());
        let size = self.size.saturating_sub(size);

        Self { top_left, size }
    }

    fn extrude_to(&self, right: i32) -> Self {
        let top_left = self.top_left;
        let width = right.saturating_sub(self.top_left.x);
        let width = width.try_into().unwrap_or_default();
        let size = Size::new(width, Default::default());
        let size = self.size.component_max(size);

        Self { top_left, size }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! test_left_half {
        (
            $(
                $fn_ident:ident, $self:expr, $expected:expr,
            )*
        ) => {
            $(
                #[test]
                fn $fn_ident() {
                    let result = $self.left_half();
                    assert_eq!(result, $expected);
                }
            )*
        }
    }

    test_left_half! {
        left_half_of_1111_2222_3333_4444,
            Rectangle::new(Point::new(1111, 2222), Size::new(3333, 4444)),
            Rectangle::new(Point::new(1111, 2222), Size::new(3333 / 2, 4444)),

        left_half_of_0_0_0_0,
            Rectangle::new(Point::new(0, 0), Size::new(0, 0)),
            Rectangle::new(Point::new(0, 0), Size::new(0, 0)),

        left_half_of_min_min_max_max,
            Rectangle::new(Point::new(i32::MIN, i32::MIN), Size::new(u32::MAX, u32::MAX)),
            Rectangle::new(Point::new(i32::MIN, i32::MIN), Size::new(u32::MAX / 2, u32::MAX)),

        left_half_of_max_max_max_max,
            Rectangle::new(Point::new(i32::MAX, i32::MAX), Size::new(u32::MAX, u32::MAX)),
            Rectangle::new(Point::new(i32::MAX, i32::MAX), Size::new(u32::MAX / 2, u32::MAX)),
    }

    macro_rules! test_right_half {
        (
            $(
                $fn_ident:ident, $self:expr, $expected:expr,
            )*
        ) => {
            $(
                #[test]
                fn $fn_ident() {
                    let result = $self.right_half();
                    assert_eq!(result, $expected);
                }
            )*
        }
    }

    test_right_half! {
        right_half_of_1111_2222_3333_4444,
            Rectangle::new(Point::new(1111, 2222), Size::new(3333, 4444)),
            Rectangle::new(Point::new(1111 + 3333 / 2, 2222), Size::new(3333 / 2 + 1, 4444)),

        right_half_of_0_0_0_0,
            Rectangle::new(Point::new(0, 0), Size::new(0, 0)),
            Rectangle::new(Point::new(0, 0), Size::new(0, 0)),

        right_half_of_min_min_max_max,
            Rectangle::new(Point::new(i32::MIN, i32::MIN), Size::new(u32::MAX, u32::MAX)),
            Rectangle::new(Point::new(-1, i32::MIN), Size::new(u32::MAX / 2 + 1, u32::MAX)),

        right_half_of_max_max_max_max,
            Rectangle::new(Point::new(i32::MAX, i32::MAX), Size::new(u32::MAX, u32::MAX)),
            Rectangle::new(Point::new(i32::MAX, i32::MAX), Size::new(u32::MAX / 2 + 1, u32::MAX)),
    }

    macro_rules! test_left_of {
        (
            $(
                $fn_ident:ident, $self:expr, $other:expr, $expected:expr,
            )*
        ) => {
            $(
                #[test]
                fn $fn_ident() {
                    let result = $self.left_of(&$other);
                    assert_eq!(result, $expected);
                }
            )*
        }
    }

    test_left_of! {
        left_of_800_for_1111_2222_3333_4444,
            Rectangle::new(Point::new(1111, 2222), Size::new(3333, 4444)),
            Rectangle::new(Point::new(800, 2222), Size::new(3333, 4444)),
            Rectangle::new(Point::new(1111, 2222), Size::new(0, 4444)),

        left_of_1600_for_1111_2222_3333_4444,
            Rectangle::new(Point::new(1111, 2222), Size::new(3333, 4444)),
            Rectangle::new(Point::new(1600, 2222), Size::new(3333, 4444)),
            Rectangle::new(Point::new(1111, 2222), Size::new(1600 - 1111, 4444)),

        left_of_3200_for_1111_2222_3333_4444,
            Rectangle::new(Point::new(1111, 2222), Size::new(3333, 4444)),
            Rectangle::new(Point::new(3200, 2222), Size::new(3333, 4444)),
            Rectangle::new(Point::new(1111, 2222), Size::new(3200 - 1111, 4444)),

        left_of_6400_for_1111_2222_3333_4444,
            Rectangle::new(Point::new(1111, 2222), Size::new(3333, 4444)),
            Rectangle::new(Point::new(6400, 2222), Size::new(3333, 4444)),
            Rectangle::new(Point::new(1111, 2222), Size::new(3333, 4444)),

        left_of_0_for_0_0_0_0,
            Rectangle::new(Point::new(0, 0), Size::new(0, 0)),
            Rectangle::new(Point::new(0, 0), Size::new(0, 0)),
            Rectangle::new(Point::new(0, 0), Size::new(0, 0)),

        left_of_minus_1_for_min_min_max_max,
            Rectangle::new(Point::new(i32::MIN, i32::MIN), Size::new(u32::MAX, u32::MAX)),
            Rectangle::new(Point::new(-1, i32::MIN), Size::new(u32::MAX, u32::MAX)),
            Rectangle::new(Point::new(i32::MIN, i32::MIN), Size::new(u32::MAX / 2, u32::MAX)),

        left_of_minus_1_for_max_max_max_max,
            Rectangle::new(Point::new(i32::MAX, i32::MAX), Size::new(u32::MAX, u32::MAX)),
            Rectangle::new(Point::new(-1, i32::MIN), Size::new(u32::MAX, u32::MAX)),
            Rectangle::new(Point::new(i32::MAX, i32::MAX), Size::new(0, u32::MAX)),
    }

    macro_rules! test_right_of {
        (
            $(
                $fn_ident:ident, $self:expr, $other:expr, $expected:expr,
            )*
        ) => {
            $(
                #[test]
                fn $fn_ident() {
                    let result = $self.right_of(&$other);
                    assert_eq!(result, $expected);
                }
            )*
        }
    }

    test_right_of! {
        right_of_800_for_1111_2222_3333_4444,
            Rectangle::new(Point::new(1111, 2222), Size::new(3333, 4444)),
            Rectangle::new(Point::new(0, 2222), Size::new(800, 4444)),
            Rectangle::new(Point::new(1111, 2222), Size::new(3333, 4444)),

        right_of_1600_for_1111_2222_3333_4444,
            Rectangle::new(Point::new(1111, 2222), Size::new(3333, 4444)),
            Rectangle::new(Point::new(0, 2222), Size::new(1600, 4444)),
            Rectangle::new(Point::new(1600, 2222), Size::new(3333 + 1111 - 1600, 4444)),

        right_of_3200_for_1111_2222_3333_4444,
            Rectangle::new(Point::new(1111, 2222), Size::new(3333, 4444)),
            Rectangle::new(Point::new(0, 2222), Size::new(3200, 4444)),
            Rectangle::new(Point::new(3200, 2222), Size::new(3333 + 1111 - 3200, 4444)),

        right_of_6400_for_1111_2222_3333_4444,
            Rectangle::new(Point::new(1111, 2222), Size::new(3333, 4444)),
            Rectangle::new(Point::new(0, 2222), Size::new(6400, 4444)),
            Rectangle::new(Point::new(6400, 2222), Size::new(0, 4444)),

        right_of_0_for_0_0_0_0,
            Rectangle::new(Point::new(0, 0), Size::new(0, 0)),
            Rectangle::new(Point::new(0, 0), Size::new(0, 0)),
            Rectangle::new(Point::new(0, 0), Size::new(0, 0)),

        right_of_minus_1_for_min_min_max_max,
            Rectangle::new(Point::new(i32::MIN, i32::MIN), Size::new(u32::MAX, u32::MAX)),
            Rectangle::new(Point::new(i32::MIN, i32::MIN), Size::new(u32::MAX / 2, u32::MAX)),
            Rectangle::new(Point::new(-1, i32::MIN), Size::new(u32::MAX / 2 + 1, u32::MAX)),

        right_of_minus_1_for_max_max_max_max,
            Rectangle::new(Point::new(i32::MAX, i32::MAX), Size::new(u32::MAX, u32::MAX)),
            Rectangle::new(Point::new(i32::MIN, i32::MIN), Size::new(u32::MAX / 2, u32::MAX)),
            Rectangle::new(Point::new(i32::MAX, i32::MAX), Size::new(u32::MAX, u32::MAX)),
    }

    macro_rules! test_above {
        (
            $(
                $fn_ident:ident, $self:expr, $other:expr, $expected:expr,
            )*
        ) => {
            $(
                #[test]
                fn $fn_ident() {
                    let result = $self.above(&$other);
                    assert_eq!(result, $expected);
                }
            )*
        }
    }

    test_above! {
        above_800_for_1111_2222_3333_4444,
            Rectangle::new(Point::new(1111, 2222), Size::new(3333, 4444)),
            Rectangle::new(Point::new(2222, 800), Size::new(3333, 4444)),
            Rectangle::new(Point::new(1111, 2222), Size::new(3333, 0)),

        above_1600_for_1111_2222_3333_4444,
            Rectangle::new(Point::new(1111, 2222), Size::new(3333, 4444)),
            Rectangle::new(Point::new(2222, 1600), Size::new(3333, 4444)),
            Rectangle::new(Point::new(1111, 2222), Size::new(3333, 0)),

        above_3200_for_1111_2222_3333_4444,
            Rectangle::new(Point::new(1111, 2222), Size::new(3333, 4444)),
            Rectangle::new(Point::new(1111, 3200), Size::new(3333, 4444)),
            Rectangle::new(Point::new(1111, 2222), Size::new(3333, 3200 - 2222)),

        above_6400_for_1111_2222_3333_4444,
            Rectangle::new(Point::new(1111, 2222), Size::new(3333, 4444)),
            Rectangle::new(Point::new(1111, 6400), Size::new(3333, 4444)),
            Rectangle::new(Point::new(1111, 2222), Size::new(3333, 6400 - 2222)),

        above_0_for_0_0_0_0,
            Rectangle::new(Point::new(0, 0), Size::new(0, 0)),
            Rectangle::new(Point::new(0, 0), Size::new(0, 0)),
            Rectangle::new(Point::new(0, 0), Size::new(0, 0)),

        above_minus_1_for_min_min_max_max,
            Rectangle::new(Point::new(i32::MIN, i32::MIN), Size::new(u32::MAX, u32::MAX)),
            Rectangle::new(Point::new(i32::MIN, -1), Size::new(u32::MAX, u32::MAX)),
            Rectangle::new(Point::new(i32::MIN, i32::MIN), Size::new(u32::MAX, u32::MAX / 2)),

        above_minus_1_for_max_max_max_max,
            Rectangle::new(Point::new(i32::MAX, i32::MAX), Size::new(u32::MAX, u32::MAX)),
            Rectangle::new(Point::new(i32::MIN, -1), Size::new(u32::MAX, u32::MAX)),
            Rectangle::new(Point::new(i32::MAX, i32::MAX), Size::new(u32::MAX, 0)),
    }

    macro_rules! test_below {
        (
            $(
                $fn_ident:ident, $self:expr, $other:expr, $expected:expr,
            )*
        ) => {
            $(
                #[test]
                fn $fn_ident() {
                    let result = $self.below(&$other);
                    assert_eq!(result, $expected);
                }
            )*
        }
    }

    test_below! {
        below_800_for_1111_2222_3333_4444,
            Rectangle::new(Point::new(1111, 2222), Size::new(3333, 4444)),
            Rectangle::new(Point::new(1111, 0), Size::new(3333, 800)),
            Rectangle::new(Point::new(1111, 2222), Size::new(3333, 4444)),

        below_1600_for_1111_2222_3333_4444,
            Rectangle::new(Point::new(1111, 2222), Size::new(3333, 4444)),
            Rectangle::new(Point::new(1111, 0), Size::new(3333, 1600)),
            Rectangle::new(Point::new(1111, 2222), Size::new(3333, 4444)),

        below_3200_for_1111_2222_3333_4444,
            Rectangle::new(Point::new(1111, 2222), Size::new(3333, 4444)),
            Rectangle::new(Point::new(1111, 0), Size::new(3333, 3200)),
            Rectangle::new(Point::new(1111, 3200), Size::new(3333, 4444 + 2222 - 3200)),

        below_6400_for_1111_2222_3333_4444,
            Rectangle::new(Point::new(1111, 2222), Size::new(3333, 4444)),
            Rectangle::new(Point::new(1111, 0), Size::new(3333, 6400)),
            Rectangle::new(Point::new(1111, 6400), Size::new(3333, 4444 + 2222 - 6400)),

        below_0_for_0_0_0_0,
            Rectangle::new(Point::new(0, 0), Size::new(0, 0)),
            Rectangle::new(Point::new(0, 0), Size::new(0, 0)),
            Rectangle::new(Point::new(0, 0), Size::new(0, 0)),

        below_minus_1_for_min_min_max_max,
            Rectangle::new(Point::new(i32::MIN, i32::MIN), Size::new(u32::MAX, u32::MAX)),
            Rectangle::new(Point::new(i32::MIN, i32::MIN), Size::new(u32::MAX, u32::MAX / 2)),
            Rectangle::new(Point::new(i32::MIN, -1), Size::new(u32::MAX, u32::MAX / 2 + 1)),

        below_minus_1_for_max_max_max_max,
            Rectangle::new(Point::new(i32::MAX, i32::MAX), Size::new(u32::MAX, u32::MAX)),
            Rectangle::new(Point::new(i32::MIN, i32::MIN), Size::new(u32::MAX, u32::MAX / 2)),
            Rectangle::new(Point::new(i32::MAX, i32::MAX), Size::new(u32::MAX, u32::MAX)),
    }

    macro_rules! test_y_extend {
        (
            $(
                $fn_ident:ident, $self:expr, $top:expr, $bottom:expr, $expected:expr,
            )*
        ) => {
            $(
                #[test]
                fn $fn_ident() {
                    let result = $self.y_extend($top, $bottom);
                    assert_eq!(result, $expected);
                }
            )*
        }
    }

    test_y_extend! {
        y_extend_top_to_0_for_1111_2222_3333_4444,
            Rectangle::new(Point::new(1111, 2222), Size::new(3333, 4444)),
            0, 4444 + 2222,
            Rectangle::new(Point::new(1111, 0), Size::new(3333, 4444 + 2222)),

        y_extend_top_to_1600_for_1111_2222_3333_4444,
            Rectangle::new(Point::new(1111, 2222), Size::new(3333, 4444)),
            1600, 4444 + 2222 - 1600,
            Rectangle::new(Point::new(1111, 1600), Size::new(3333, 4444 + 2222 - 1600)),

        y_extend_bottom_to_3200_for_1111_2222_3333_4444,
            Rectangle::new(Point::new(1111, 2222), Size::new(3333, 4444)),
            2222, 3200,
            Rectangle::new(Point::new(1111, 2222), Size::new(3333, 4444)),

        y_extend_bottom_to_6400_for_1111_2222_3333_4444,
            Rectangle::new(Point::new(1111, 2222), Size::new(3333, 4444)),
            2222, 6400,
            Rectangle::new(Point::new(1111, 2222), Size::new(3333, 4444)),

        y_extend_bottom_to_12800_for_1111_2222_3333_4444,
            Rectangle::new(Point::new(1111, 2222), Size::new(3333, 4444)),
            2222, 12800,
            Rectangle::new(Point::new(1111, 2222), Size::new(3333, 12800 - 2222)),

        y_extend_top_to_0_and_bottom_to_25600_for_1111_2222_3333_4444,
            Rectangle::new(Point::new(1111, 2222), Size::new(3333, 4444)),
            0, 25600,
            Rectangle::new(Point::new(1111, 0), Size::new(3333, 25600)),

        y_extend_top_to_25600_and_bottom_to_0_for_1111_2222_3333_4444,
            Rectangle::new(Point::new(1111, 2222), Size::new(3333, 4444)),
            25600, 0,
            Rectangle::new(Point::new(1111, 2222), Size::new(3333, 4444)),

        y_extend_top_to_half_min_and_bottom_to_half_max_for_1111_2222_3333_4444,
            Rectangle::new(Point::new(1111, 2222), Size::new(3333, 4444)),
            i32::MIN / 2, i32::MAX / 2,
            Rectangle::new(Point::new(1111, i32::MIN / 2), Size::new(3333, u32::MAX / 2)),

        y_extend_top_to_0_and_bottom_to_max_for_1111_2222_3333_4444,
            Rectangle::new(Point::new(1111, 2222), Size::new(3333, 4444)),
            0, i32::MAX,
            Rectangle::new(Point::new(1111, 0), Size::new(3333, u32::MAX / 2)),

        y_extend_top_to_max_and_bottom_to_max_for_1111_2222_3333_4444,
            Rectangle::new(Point::new(1111, 2222), Size::new(3333, 4444)),
            i32::MAX, i32::MAX,
            Rectangle::new(Point::new(1111, 2222), Size::new(3333, u32::MAX / 2 - 2222)),
    }

    macro_rules! test_y_reduce {
        (
            $(
                $fn_ident:ident, $self:expr, $top:expr, $bottom:expr, $expected:expr,
            )*
        ) => {
            $(
                #[test]
                fn $fn_ident() {
                    let result = $self.y_reduce($top, $bottom);
                    assert_eq!(result, $expected);
                }
            )*
        }
    }

    test_y_reduce! {
        y_reduce_top_to_0_for_1111_2222_3333_4444,
            Rectangle::new(Point::new(1111, 2222), Size::new(3333, 4444)),
            0, 4444 + 2222,
            Rectangle::new(Point::new(1111, 2222), Size::new(3333, 4444)),

        y_reduce_top_to_1600_for_1111_2222_3333_4444,
            Rectangle::new(Point::new(1111, 2222), Size::new(3333, 4444)),
            1600, 4444 + 2222,
            Rectangle::new(Point::new(1111, 2222), Size::new(3333, 4444)),

        y_reduce_top_to_3200_for_1111_2222_3333_4444,
            Rectangle::new(Point::new(1111, 2222), Size::new(3333, 4444)),
            3200, 4444 + 2222,
            Rectangle::new(Point::new(1111, 3200), Size::new(3333, 4444 + 2222 - 3200)),

        y_reduce_top_to_6400_for_1111_2222_3333_4444,
            Rectangle::new(Point::new(1111, 2222), Size::new(3333, 4444)),
            6400, 4444 + 2222,
            Rectangle::new(Point::new(1111, 6400), Size::new(3333, 4444 + 2222 - 6400)),

        y_reduce_bottom_to_3200_for_1111_2222_3333_4444,
            Rectangle::new(Point::new(1111, 2222), Size::new(3333, 4444)),
            2222, 3200,
            Rectangle::new(Point::new(1111, 2222), Size::new(3333, 3200 - 2222)),

        y_reduce_bottom_to_6400_for_1111_2222_3333_4444,
            Rectangle::new(Point::new(1111, 2222), Size::new(3333, 4444)),
            2222, 6400,
            Rectangle::new(Point::new(1111, 2222), Size::new(3333, 6400 - 2222)),

        y_reduce_bottom_to_12800_for_1111_2222_3333_4444,
            Rectangle::new(Point::new(1111, 2222), Size::new(3333, 4444)),
            2222, 12800,
            Rectangle::new(Point::new(1111, 2222), Size::new(3333, 4444)),

        y_reduce_top_to_0_and_bottom_to_25600_for_1111_2222_3333_4444,
            Rectangle::new(Point::new(1111, 2222), Size::new(3333, 4444)),
            0, 25600,
            Rectangle::new(Point::new(1111, 2222), Size::new(3333, 4444)),

        y_reduce_top_to_25600_and_bottom_to_0_for_1111_2222_3333_4444,
            Rectangle::new(Point::new(1111, 2222), Size::new(3333, 4444)),
            25600, 0,
            Rectangle::new(Point::new(1111, 25600), Size::new(3333, 0)),

        y_reduce_top_to_max_and_bottom_to_max_for_1111_2222_3333_4444,
            Rectangle::new(Point::new(1111, 2222), Size::new(3333, 4444)),
            i32::MAX, i32::MAX,
            Rectangle::new(Point::new(1111, i32::MAX), Size::new(3333, 0)),
    }

    macro_rules! test_indent_to {
        (
            $(
                $fn_ident:ident, $self:expr, $right:expr, $expected:expr,
            )*
        ) => {
            $(
                #[test]
                fn $fn_ident() {
                    let result = $self.indent_to($right);
                    assert_eq!(result, $expected);
                }
            )*
        }
    }

    test_indent_to! {
        indent_to_800_for_1111_2222_3333_4444,
            Rectangle::new(Point::new(1111, 2222), Size::new(3333, 4444)),
            800,
            Rectangle::new(Point::new(1111, 2222), Size::new(3333, 4444)),

        indent_to_1600_for_1111_2222_3333_4444,
            Rectangle::new(Point::new(1111, 2222), Size::new(3333, 4444)),
            1600,
            Rectangle::new(Point::new(1600, 2222), Size::new(3333 + 1111 - 1600, 4444)),

        indent_to_3200_for_1111_2222_3333_4444,
            Rectangle::new(Point::new(1111, 2222), Size::new(3333, 4444)),
            3200,
            Rectangle::new(Point::new(3200, 2222), Size::new(3333 + 1111 - 3200, 4444)),

        indent_to_6400_for_1111_2222_3333_4444,
            Rectangle::new(Point::new(1111, 2222), Size::new(3333, 4444)),
            6400,
            Rectangle::new(Point::new(6400, 2222), Size::new(0, 4444)),

        indent_to_0_for_0_0_0_0,
            Rectangle::new(Point::new(0, 0), Size::new(0, 0)),
            0,
            Rectangle::new(Point::new(0, 0), Size::new(0, 0)),

        indent_to_minus_1_for_min_min_max_max,
            Rectangle::new(Point::new(i32::MIN, i32::MIN), Size::new(u32::MAX, u32::MAX)),
            -1,
            Rectangle::new(Point::new(-1, i32::MIN), Size::new(u32::MAX / 2 + 1, u32::MAX)),

        indent_to_minus_1_for_max_max_max_max,
            Rectangle::new(Point::new(i32::MAX, i32::MAX), Size::new(u32::MAX, u32::MAX)),
            -1,
            Rectangle::new(Point::new(i32::MAX, i32::MAX), Size::new(u32::MAX, u32::MAX)),

    }

    macro_rules! test_extrude_to {
        (
            $(
                $fn_ident:ident, $self:expr, $right:expr, $expected:expr,
            )*
        ) => {
            $(
                #[test]
                fn $fn_ident() {
                    let result = $self.extrude_to($right);
                    assert_eq!(result, $expected);
                }
            )*
        }
    }

    test_extrude_to! {
        extrude_to_800_for_1111_2222_3333_4444,
            Rectangle::new(Point::new(1111, 2222), Size::new(3333, 4444)),
            800,
            Rectangle::new(Point::new(1111, 2222), Size::new(3333, 4444)),

        extrude_to_1600_for_1111_2222_3333_4444,
            Rectangle::new(Point::new(1111, 2222), Size::new(3333, 4444)),
            1600,
            Rectangle::new(Point::new(1111, 2222), Size::new(3333, 4444)),

        extrude_to_3200_for_1111_2222_3333_4444,
            Rectangle::new(Point::new(1111, 2222), Size::new(3333, 4444)),
            3200,
            Rectangle::new(Point::new(1111, 2222), Size::new(3333, 4444)),

        extrude_to_6400_for_1111_2222_3333_4444,
            Rectangle::new(Point::new(1111, 2222), Size::new(3333, 4444)),
            6400,
            Rectangle::new(Point::new(1111, 2222), Size::new(6400 - 1111, 4444)),

        extrude_to_max_for_0_0_0_0,
            Rectangle::new(Point::new(0, 0), Size::new(0, 0)),
            i32::MAX,
            Rectangle::new(Point::new(0, 0), Size::new(u32::MAX / 2, 0)),

        extrude_to_max_for_min_min_max_max,
            Rectangle::new(Point::new(i32::MIN, i32::MIN), Size::new(u32::MAX, u32::MAX)),
            i32::MAX,
            Rectangle::new(Point::new(i32::MIN, i32::MIN), Size::new(u32::MAX, u32::MAX)),

        extrude_to_max_for_max_max_max_max,
            Rectangle::new(Point::new(i32::MAX, i32::MAX), Size::new(u32::MAX, u32::MAX)),
            i32::MAX,
            Rectangle::new(Point::new(i32::MAX, i32::MAX), Size::new(u32::MAX, u32::MAX)),

    }
}
