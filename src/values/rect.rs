use cssparser::*;
use crate::traits::{Parse, ToCss};
use crate::printer::Printer;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Rect<T>(pub T, pub T, pub T, pub T);

impl<T> Rect<T> {
    /// Returns a new `Rect<T>` value.
    pub fn new(first: T, second: T, third: T, fourth: T) -> Self {
        Rect(first, second, third, fourth)
    }
}

impl<T> Rect<T>
where
    T: Clone
{
    /// Returns a rect with all the values equal to `v`.
    pub fn all(v: T) -> Self {
        Rect::new(v.clone(), v.clone(), v.clone(), v)
    }

    /// Parses a new `Rect<T>` value with the given parse function.
    pub fn parse_with<'i, 't, Parse>(
        input: &mut Parser<'i, 't>,
        parse: Parse,
    ) -> Result<Self, ParseError<'i, ()>>
    where
        Parse: Fn(&mut Parser<'i, 't>) -> Result<T, ParseError<'i, ()>>,
    {
        let first = parse(input)?;
        let second = if let Ok(second) = input.try_parse(|i| parse(i)) {
            second
        } else {
            // <first>
            return Ok(Self::new(
                first.clone(),
                first.clone(),
                first.clone(),
                first,
            ));
        };
        let third = if let Ok(third) = input.try_parse(|i| parse(i)) {
            third
        } else {
            // <first> <second>
            return Ok(Self::new(first.clone(), second.clone(), first, second));
        };
        let fourth = if let Ok(fourth) = input.try_parse(|i| parse(i)) {
            fourth
        } else {
            // <first> <second> <third>
            return Ok(Self::new(first, second.clone(), third, second));
        };
        // <first> <second> <third> <fourth>
        Ok(Self::new(first, second, third, fourth))
    }
  }

impl<T> Parse for Rect<T>
where
  T: Clone + PartialEq + Parse
{
  fn parse<'i, 't>(input: &mut Parser<'i, 't>) -> Result<Self, ParseError<'i, ()>> {
    Self::parse_with(input, T::parse)
  }
}

impl<T> ToCss for Rect<T>
where
  T: PartialEq + ToCss
{
    fn to_css<W>(&self, dest: &mut Printer<W>) -> std::fmt::Result where W: std::fmt::Write {
        self.0.to_css(dest)?;
        let same_vertical = self.0 == self.2;
        let same_horizontal = self.1 == self.3;
        if same_vertical && same_horizontal && self.0 == self.1 {
            return Ok(());
        }
        dest.write_str(" ")?;
        self.1.to_css(dest)?;
        if same_vertical && same_horizontal {
            return Ok(());
        }
        dest.write_str(" ")?;
        self.2.to_css(dest)?;
        if same_horizontal {
            return Ok(());
        }
        dest.write_str(" ")?;
        self.3.to_css(dest)
    }
}
