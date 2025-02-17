use crate::values::length::*;
use crate::values::size::Size2D;
use cssparser::*;
use crate::traits::{Parse, ToCss, PropertyHandler};
use super::prefixes::{Feature, Browsers};
use crate::properties::{Property, VendorPrefix};
use crate::values::rect::Rect;
use crate::printer::Printer;

#[derive(Debug, Clone, PartialEq)]
pub struct BorderRadius {
  top_left: Size2D<LengthPercentage>,
  top_right: Size2D<LengthPercentage>,
  bottom_left: Size2D<LengthPercentage>,
  bottom_right: Size2D<LengthPercentage>
}

impl Parse for BorderRadius {
  fn parse<'i, 't>(input: &mut Parser<'i, 't>) -> Result<Self, ParseError<'i, ()>> {
    let widths: Rect<LengthPercentage> = Rect::parse(input)?;
    let heights = if input.try_parse(|input| input.expect_delim('/')).is_ok() {
      Rect::parse(input)?
    } else {
      widths.clone()
    };

    Ok(BorderRadius {
      top_left: Size2D(widths.0, heights.0),
      top_right: Size2D(widths.1, heights.1),
      bottom_left: Size2D(widths.2, heights.2),
      bottom_right: Size2D(widths.3, heights.3)
    })
  }
}

impl ToCss for BorderRadius {
  fn to_css<W>(&self, dest: &mut Printer<W>) -> std::fmt::Result where W: std::fmt::Write {
    let widths = Rect::new(&self.top_left.0, &self.top_right.0, &self.bottom_left.0, &self.bottom_right.0);
    let heights = Rect::new(&self.top_left.1, &self.top_right.1, &self.bottom_left.1, &self.bottom_right.1);

    widths.to_css(dest)?;
    if widths != heights {
      dest.delim('/', true)?;
      heights.to_css(dest)?;
    }

    Ok(())
  }
}

#[derive(Default, Debug)]
pub struct BorderRadiusHandler {
  targets: Option<Browsers>,
  top_left: Option<(Size2D<LengthPercentage>, VendorPrefix)>,
  top_right: Option<(Size2D<LengthPercentage>, VendorPrefix)>,
  bottom_left: Option<(Size2D<LengthPercentage>, VendorPrefix)>,
  bottom_right: Option<(Size2D<LengthPercentage>, VendorPrefix)>,
  logical: Vec<Property>,
  decls: Vec<Property>
}

impl BorderRadiusHandler {
  pub fn new(targets: Option<Browsers>) -> BorderRadiusHandler {
    BorderRadiusHandler {
      targets,
      ..BorderRadiusHandler::default()
    }
  }
}

impl PropertyHandler for BorderRadiusHandler {
  fn handle_property(&mut self, property: &Property) -> bool {
    use Property::*;

    macro_rules! property {
      ($prop: ident, $val: expr, $vp: ident) => {{
        // If two vendor prefixes for the same property have different
        // values, we need to flush what we have immediately to preserve order.
        if let Some((val, prefixes)) = &self.$prop {
          if val != $val && !prefixes.contains(*$vp) {
            self.flush();
          }
        }

        // Otherwise, update the value and add the prefix.
        if let Some((val, prefixes)) = &mut self.$prop {
          *val = $val.clone();
          *prefixes |= *$vp;
        } else {
          self.$prop = Some(($val.clone(), *$vp))
        }
      }};
    }

    match property {
      BorderTopLeftRadius(val, vp) => property!(top_left, val, vp),
      BorderTopRightRadius(val, vp) => property!(top_right, val, vp),
      BorderBottomLeftRadius(val, vp) => property!(bottom_left, val, vp),
      BorderBottomRightRadius(val, vp) => property!(bottom_right, val, vp),
      BorderStartStartRadius(_) | BorderStartEndRadius(_) | BorderEndStartRadius(_) | BorderEndEndRadius(_) => {
        self.flush();
        self.logical.push(property.clone());
      }
      BorderRadius(val, vp) => {
        self.logical.clear();
        property!(top_left, &val.top_left, vp);
        property!(top_right, &val.top_right, vp);
        property!(bottom_left, &val.bottom_left, vp);
        property!(bottom_right, &val.bottom_right, vp);
      }
      _ => return false
    }

    true
  }

  fn finalize(&mut self) -> Vec<Property> {
    self.flush();
    std::mem::take(&mut self.decls)
  }
}

impl BorderRadiusHandler {
  fn flush(&mut self) {
    let mut top_left = std::mem::take(&mut self.top_left);
    let mut top_right = std::mem::take(&mut self.top_right);
    let mut bottom_left = std::mem::take(&mut self.bottom_left);
    let mut bottom_right = std::mem::take(&mut self.bottom_right);

    self.decls.extend(self.logical.drain(..));

    if let (Some((top_left, tl_prefix)), Some((top_right, tr_prefix)), Some((bottom_left, bl_prefix)), Some((bottom_right, br_prefix))) = (&mut top_left, &mut top_right, &mut bottom_left, &mut bottom_right) {
      let intersection = *tl_prefix & *tr_prefix & *bl_prefix & *br_prefix;
      if !intersection.is_empty() {
        let mut prefix = intersection;
        if prefix.contains(VendorPrefix::None) {
          if let Some(targets) = self.targets {
            prefix = Feature::BorderRadius.prefixes_for(targets)
          }
        }
        self.decls.push(Property::BorderRadius(BorderRadius {
          top_left: top_left.clone(),
          top_right: top_right.clone(),
          bottom_left: bottom_left.clone(),
          bottom_right: bottom_right.clone(),
        }, prefix));
        tl_prefix.remove(intersection);
        tr_prefix.remove(intersection);
        bl_prefix.remove(intersection);
        br_prefix.remove(intersection);
      }
    }

    macro_rules! single_property {
      ($prop: ident, $key: ident) => {
        if let Some((val, mut vp)) = $key {
          if !vp.is_empty() {
            if vp.contains(VendorPrefix::None) {
              if let Some(targets) = self.targets {
                vp = Feature::$prop.prefixes_for(targets)
              }
            }
            self.decls.push(Property::$prop(val, vp))
          }
        }
      };
    }

    single_property!(BorderTopLeftRadius, top_left);
    single_property!(BorderTopRightRadius, top_right);
    single_property!(BorderBottomLeftRadius, bottom_left);
    single_property!(BorderBottomRightRadius, bottom_right);
  }
}
