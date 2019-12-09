pub enum Joined<Left, Right> {
    Both(Left, Right),
    Left(Left),
    Right(Right),
}

pub struct OuterJoin<Left, Right> {
    left: Left,
    right: Right,
}

impl<Left, Right> Iterator for OuterJoin<Left, Right>
where
    Left: Iterator,
    Right: Iterator,
{
    type Item = Joined<Left::Item, Right::Item>;

    fn next(&mut self) -> Option<Self::Item> {
        let left = self.left.next();
        let right = self.right.next();

        match (left, right) {
            (Some(left), Some(right)) => Some(Joined::Both(left, right)),
            (Some(left), None) => Some(Joined::Left(left)),
            (None, Some(right)) => Some(Joined::Right(right)),
            (None, None) => None,
        }
    }
}

#[allow(dead_code)]
pub fn outer_join<Left, Right>(
    left: impl IntoIterator<Item = Left>,
    right: impl IntoIterator<Item = Right>,
) -> impl Iterator<Item = Joined<Left, Right>> {
    let left = left.into_iter();
    let right = right.into_iter();

    OuterJoin { left, right }
}

pub fn outer_join_filter<Left, Right>(
    left: impl IntoIterator<Item = Option<Left>>,
    right: impl IntoIterator<Item = Option<Right>>,
) -> impl Iterator<Item = Joined<Left, Right>> {
    let left = left.into_iter();
    let right = right.into_iter();

    OuterJoin { left, right }.filter_map(|joined| match joined {
        Joined::Both(Some(left), Some(right)) => Some(Joined::Both(left, right)),
        Joined::Both(Some(left), None) | Joined::Left(Some(left)) => Some(Joined::Left(left)),
        Joined::Both(None, Some(right)) | Joined::Right(Some(right)) => Some(Joined::Right(right)),
        Joined::Both(None, None) => None,
        Joined::Left(None) => None,
        Joined::Right(None) => None,
    })
}
