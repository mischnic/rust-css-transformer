use cssparser::*;
use crate::traits::{Parse, ToCss};
use crate::values::{
  angle::Angle,
  percentage::NumberOrPercentage,
  length::{LengthPercentage, Length}
};
use crate::macros::enum_property;
use crate::printer::Printer;
use std::fmt::Write;

/// https://www.w3.org/TR/2019/CR-css-transforms-1-20190214/#propdef-transform
#[derive(Debug, Clone, PartialEq)]
pub struct TransformList(pub Vec<Transform>);

impl Parse for TransformList {
  fn parse<'i, 't>(input: &mut Parser<'i, 't>) -> Result<Self, ParseError<'i, ()>> {
    if input.try_parse(|input| input.expect_ident_matching("none")).is_ok() {
      return Ok(TransformList(vec![]))
    }

    input.skip_whitespace();
    let mut results = vec![Transform::parse(input)?];
    loop {
      input.skip_whitespace();
      if let Ok(item) = input.try_parse(Transform::parse) {
        results.push(item);
      } else {
        return Ok(TransformList(results));
      }
    }
  }
}

impl ToCss for TransformList {
  fn to_css<W>(&self, dest: &mut Printer<W>) -> std::fmt::Result where W: std::fmt::Write {
    if self.0.is_empty() {
      dest.write_str("none")?;
      return Ok(())
    }

    if dest.minify {
      // Combine transforms into a single matrix.
      if let Some(matrix) = self.to_matrix() {
        // Generate based on the original transforms.
        let mut base = String::new();
        self.to_css_base(&mut Printer::new(&mut base, true))?;

        // Decompose the matrix into transform functions if possible.
        // If the resulting length is shorter than the original, use it.
        if let Some(d) = matrix.decompose() {
          let mut decomposed = String::new();
          d.to_css_base(&mut Printer::new(&mut decomposed, true))?;
          if decomposed.len() < base.len() {
            base = decomposed;
          }
        }

        // Also generate a matrix() or matrix3d() representation and compare that.
        let mut mat = String::new();
        if let Some(matrix) = matrix.to_matrix2d() {
          Transform::Matrix(matrix).to_css(&mut Printer::new(&mut mat, true))?
        } else {
          Transform::Matrix3d(matrix).to_css(&mut Printer::new(&mut mat, true))?
        }

        if mat.len() < base.len() {
          dest.write_str(&mat)?;
        } else {
          dest.write_str(&base)?;
        }

        return Ok(())
      }
    }

    self.to_css_base(dest)
  }
}

impl TransformList {
  fn to_css_base<W>(&self, dest: &mut Printer<W>) -> std::fmt::Result where W: std::fmt::Write {
    for item in &self.0 {
      item.to_css(dest)?;
    }
    Ok(())
  }

  pub fn to_matrix(&self) -> Option<Matrix3d<f32>> {
    let mut matrix = Matrix3d::identity();
    for transform in &self.0 {
      if let Some(m) = transform.to_matrix() {
        matrix = m.multiply(&matrix);
      } else {
        return None
      }
    }
    Some(matrix)
  }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Transform {
  Translate(LengthPercentage, LengthPercentage),
  TranslateX(LengthPercentage),
  TranslateY(LengthPercentage),
  TranslateZ(Length),
  Translate3d(LengthPercentage, LengthPercentage, Length),
  Scale(NumberOrPercentage, NumberOrPercentage),
  ScaleX(NumberOrPercentage),
  ScaleY(NumberOrPercentage),
  ScaleZ(NumberOrPercentage),
  Scale3d(NumberOrPercentage, NumberOrPercentage, NumberOrPercentage),
  Rotate(Angle),
  RotateX(Angle),
  RotateY(Angle),
  RotateZ(Angle),
  Rotate3d(f32, f32, f32, Angle),
  Skew(Angle, Angle),
  SkewX(Angle),
  SkewY(Angle),
  Perspective(Length),
  Matrix(Matrix<f32>),
  Matrix3d(Matrix3d<f32>)
}

#[derive(Debug, Clone, PartialEq)]
pub struct Matrix<T> {
  pub a: T, pub b: T, pub c: T,
  pub d: T, pub e: T, pub f: T,
}

impl Matrix<f32> {
  pub fn to_matrix3d(&self) -> Matrix3d<f32> {
    Matrix3d {
      m11: self.a, m12: self.b, m13: 0.0, m14: 0.0,
      m21: self.c, m22: self.d, m23: 0.0, m24: 0.0,
      m31: 0.0,    m32: 0.0,    m33: 1.0, m34: 0.0,
      m41: self.e, m42: self.f, m43: 0.0, m44: 1.0
    }
  }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Matrix3d<T> {
  pub m11: T, pub m12: T, pub m13: T, pub m14: T,
  pub m21: T, pub m22: T, pub m23: T, pub m24: T,
  pub m31: T, pub m32: T, pub m33: T, pub m34: T,
  pub m41: T, pub m42: T, pub m43: T, pub m44: T,
}

/// https://drafts.csswg.org/css-transforms-2/#mathematical-description
impl Matrix3d<f32> {
  pub fn identity() -> Matrix3d<f32> {
    Matrix3d {
      m11: 1.0, m12: 0.0, m13: 0.0, m14: 0.0,
      m21: 0.0, m22: 1.0, m23: 0.0, m24: 0.0,
      m31: 0.0, m32: 0.0, m33: 1.0, m34: 0.0,
      m41: 0.0, m42: 0.0, m43: 0.0, m44: 1.0
    }
  }

  pub fn translate(x: f32, y: f32, z: f32) -> Matrix3d<f32> {
    Matrix3d {
      m11: 1.0, m12: 0.0, m13: 0.0, m14: 0.0,
      m21: 0.0, m22: 1.0, m23: 0.0, m24: 0.0,
      m31: 0.0, m32: 0.0, m33: 1.0, m34: 0.0,
      m41: x,   m42: y,   m43: z,   m44: 1.0
    }
  }

  pub fn scale(x: f32, y: f32, z: f32) -> Matrix3d<f32> {
    Matrix3d {
      m11: x,   m12: 0.0, m13: 0.0, m14: 0.0,
      m21: 0.0, m22: y,   m23: 0.0, m24: 0.0,
      m31: 0.0, m32: 0.0, m33: z,   m34: 0.0,
      m41: 0.0, m42: 0.0, m43: 0.0, m44: 1.0
    }
  }

  pub fn rotate(x: f32, y: f32, z: f32, angle: f32) -> Matrix3d<f32> {
    // Normalize the vector.
    let length = (x * x + y * y + z * z).sqrt();
    if length == 0.0 {
      // A direction vector that cannot be normalized, such as [0,0,0], will cause the rotation to not be applied.
      return Matrix3d::identity()
    }

    let x = x / length;
    let y = y / length;
    let z = z / length;

    let half_angle = angle / 2.0;
    let sin = half_angle.sin();
    let sc = sin * half_angle.cos();
    let sq = sin * sin;
    let m11 = 1.0 - 2.0 * (y * y + z * z) * sq;
    let m12 = 2.0 * (x * y * sq + z * sc);
    let m13 = 2.0 * (x * z * sq - y * sc);
    let m21 = 2.0 * (x * y * sq - z * sc);
    let m22 = 1.0 - 2.0 * (x * x + z * z) * sq;
    let m23 = 2.0 * (y * z * sq + x * sc);
    let m31 = 2.0 * (x * z * sq + y * sc);
    let m32 = 2.0 * (y * z * sq - x * sc);
    let m33 = 1.0 - 2.0 * (x * x + y * y) * sq;
    Matrix3d {
      m11, m12, m13, m14: 0.0,
      m21, m22, m23, m24: 0.0,
      m31, m32, m33, m34: 0.0,
      m41: 0.0, m42: 0.0, m43: 0.0, m44: 1.0
    }
  }

  pub fn skew(a: f32, b: f32) -> Matrix3d<f32> {
    Matrix3d {
      m11: 1.0, m12: b.tan(), m13: 0.0, m14: 0.0,
      m21: a.tan(), m22: 1.0, m23: 0.0, m24: 0.0,
      m31: 0.0, m32: 0.0, m33: 1.0, m34: 0.0,
      m41: 0.0, m42: 0.0, m43: 0.0, m44: 1.0
    }
  }

  pub fn perspective(d: f32) -> Matrix3d<f32> {
    Matrix3d {
      m11: 1.0, m12: 0.0, m13: 0.0, m14: 0.0,
      m21: 0.0, m22: 1.0, m23: 0.0, m24: 0.0,
      m31: 0.0, m32: 0.0, m33: 1.0, m34: -1.0 / d,
      m41: 0.0, m42: 0.0, m43: 0.0, m44: 1.0
    }
  }

  pub fn multiply(&self, other: &Self) -> Self {
    Matrix3d {
      m11: self.m11 * other.m11 + self.m12 * other.m21 +
           self.m13 * other.m31 + self.m14 * other.m41,
      m12: self.m11 * other.m12 + self.m12 * other.m22 +
           self.m13 * other.m32 + self.m14 * other.m42,
      m13: self.m11 * other.m13 + self.m12 * other.m23 +
           self.m13 * other.m33 + self.m14 * other.m43,
      m14: self.m11 * other.m14 + self.m12 * other.m24 +
           self.m13 * other.m34 + self.m14 * other.m44,
      m21: self.m21 * other.m11 + self.m22 * other.m21 +
           self.m23 * other.m31 + self.m24 * other.m41,
      m22: self.m21 * other.m12 + self.m22 * other.m22 +
           self.m23 * other.m32 + self.m24 * other.m42,
      m23: self.m21 * other.m13 + self.m22 * other.m23 +
           self.m23 * other.m33 + self.m24 * other.m43,
      m24: self.m21 * other.m14 + self.m22 * other.m24 +
           self.m23 * other.m34 + self.m24 * other.m44,
      m31: self.m31 * other.m11 + self.m32 * other.m21 +
           self.m33 * other.m31 + self.m34 * other.m41,
      m32: self.m31 * other.m12 + self.m32 * other.m22 +
           self.m33 * other.m32 + self.m34 * other.m42,
      m33: self.m31 * other.m13 + self.m32 * other.m23 +
           self.m33 * other.m33 + self.m34 * other.m43,
      m34: self.m31 * other.m14 + self.m32 * other.m24 +
           self.m33 * other.m34 + self.m34 * other.m44,
      m41: self.m41 * other.m11 + self.m42 * other.m21 +
           self.m43 * other.m31 + self.m44 * other.m41,
      m42: self.m41 * other.m12 + self.m42 * other.m22 +
           self.m43 * other.m32 + self.m44 * other.m42,
      m43: self.m41 * other.m13 + self.m42 * other.m23 +
           self.m43 * other.m33 + self.m44 * other.m43,
      m44: self.m41 * other.m14 + self.m42 * other.m24 +
           self.m43 * other.m34 + self.m44 * other.m44,
    }
  }

  pub fn is_2d(&self) -> bool {
    self.m31 == 0.0 && self.m32 == 0.0 &&
    self.m13 == 0.0 && self.m23 == 0.0 &&
    self.m43 == 0.0 && self.m14 == 0.0 &&
    self.m24 == 0.0 && self.m34 == 0.0 &&
    self.m33 == 1.0 && self.m44 == 1.0
  }

  pub fn to_matrix2d(&self) -> Option<Matrix<f32>> {
    if self.is_2d() {
      return Some(Matrix {
        a: self.m11, b: self.m12,
        c: self.m21, d: self.m22,
        e: self.m41, f: self.m42
      })
    }
    None
  }

  pub fn scale_by_factor(&mut self, scaling_factor: f32) {
    self.m11 *= scaling_factor;
    self.m12 *= scaling_factor;
    self.m13 *= scaling_factor;
    self.m14 *= scaling_factor;
    self.m21 *= scaling_factor;
    self.m22 *= scaling_factor;
    self.m23 *= scaling_factor;
    self.m24 *= scaling_factor;
    self.m31 *= scaling_factor;
    self.m32 *= scaling_factor;
    self.m33 *= scaling_factor;
    self.m34 *= scaling_factor;
    self.m41 *= scaling_factor;
    self.m42 *= scaling_factor;
    self.m43 *= scaling_factor;
    self.m44 *= scaling_factor;
  }

  pub fn determinant(&self) -> f32 {
    self.m14 * self.m23 * self.m32 * self.m41 -
    self.m13 * self.m24 * self.m32 * self.m41 -
    self.m14 * self.m22 * self.m33 * self.m41 +
    self.m12 * self.m24 * self.m33 * self.m41 +
    self.m13 * self.m22 * self.m34 * self.m41 -
    self.m12 * self.m23 * self.m34 * self.m41 -
    self.m14 * self.m23 * self.m31 * self.m42 +
    self.m13 * self.m24 * self.m31 * self.m42 +
    self.m14 * self.m21 * self.m33 * self.m42 -
    self.m11 * self.m24 * self.m33 * self.m42 -
    self.m13 * self.m21 * self.m34 * self.m42 +
    self.m11 * self.m23 * self.m34 * self.m42 +
    self.m14 * self.m22 * self.m31 * self.m43 -
    self.m12 * self.m24 * self.m31 * self.m43 -
    self.m14 * self.m21 * self.m32 * self.m43 +
    self.m11 * self.m24 * self.m32 * self.m43 +
    self.m12 * self.m21 * self.m34 * self.m43 -
    self.m11 * self.m22 * self.m34 * self.m43 -
    self.m13 * self.m22 * self.m31 * self.m44 +
    self.m12 * self.m23 * self.m31 * self.m44 +
    self.m13 * self.m21 * self.m32 * self.m44 -
    self.m11 * self.m23 * self.m32 * self.m44 -
    self.m12 * self.m21 * self.m33 * self.m44 +
    self.m11 * self.m22 * self.m33 * self.m44
  }

  pub fn inverse(&self) -> Option<Matrix3d<f32>> {
    let mut det = self.determinant();
    if det == 0.0 {
      return None;
    }

    det = 1.0 / det;
    Some(Matrix3d {
      m11: det *
      (self.m23 * self.m34 * self.m42 - self.m24 * self.m33 * self.m42 +
        self.m24 * self.m32 * self.m43 - self.m22 * self.m34 * self.m43 -
        self.m23 * self.m32 * self.m44 + self.m22 * self.m33 * self.m44),
      m12: det *
      (self.m14 * self.m33 * self.m42 - self.m13 * self.m34 * self.m42 -
        self.m14 * self.m32 * self.m43 + self.m12 * self.m34 * self.m43 +
        self.m13 * self.m32 * self.m44 - self.m12 * self.m33 * self.m44),
      m13: det *
      (self.m13 * self.m24 * self.m42 - self.m14 * self.m23 * self.m42 +
        self.m14 * self.m22 * self.m43 - self.m12 * self.m24 * self.m43 -
        self.m13 * self.m22 * self.m44 + self.m12 * self.m23 * self.m44),
      m14: det *
      (self.m14 * self.m23 * self.m32 - self.m13 * self.m24 * self.m32 -
        self.m14 * self.m22 * self.m33 + self.m12 * self.m24 * self.m33 +
        self.m13 * self.m22 * self.m34 - self.m12 * self.m23 * self.m34),
      m21: det *
      (self.m24 * self.m33 * self.m41 - self.m23 * self.m34 * self.m41 -
        self.m24 * self.m31 * self.m43 + self.m21 * self.m34 * self.m43 +
        self.m23 * self.m31 * self.m44 - self.m21 * self.m33 * self.m44),
      m22: det *
      (self.m13 * self.m34 * self.m41 - self.m14 * self.m33 * self.m41 +
        self.m14 * self.m31 * self.m43 - self.m11 * self.m34 * self.m43 -
        self.m13 * self.m31 * self.m44 + self.m11 * self.m33 * self.m44),
      m23: det *
      (self.m14 * self.m23 * self.m41 - self.m13 * self.m24 * self.m41 -
        self.m14 * self.m21 * self.m43 + self.m11 * self.m24 * self.m43 +
        self.m13 * self.m21 * self.m44 - self.m11 * self.m23 * self.m44),
      m24: det *
      (self.m13 * self.m24 * self.m31 - self.m14 * self.m23 * self.m31 +
        self.m14 * self.m21 * self.m33 - self.m11 * self.m24 * self.m33 -
        self.m13 * self.m21 * self.m34 + self.m11 * self.m23 * self.m34),
      m31: det *
      (self.m22 * self.m34 * self.m41 - self.m24 * self.m32 * self.m41 +
        self.m24 * self.m31 * self.m42 - self.m21 * self.m34 * self.m42 -
        self.m22 * self.m31 * self.m44 + self.m21 * self.m32 * self.m44),
      m32: det *
      (self.m14 * self.m32 * self.m41 - self.m12 * self.m34 * self.m41 -
        self.m14 * self.m31 * self.m42 + self.m11 * self.m34 * self.m42 +
        self.m12 * self.m31 * self.m44 - self.m11 * self.m32 * self.m44),
      m33: det *
      (self.m12 * self.m24 * self.m41 - self.m14 * self.m22 * self.m41 +
        self.m14 * self.m21 * self.m42 - self.m11 * self.m24 * self.m42 -
        self.m12 * self.m21 * self.m44 + self.m11 * self.m22 * self.m44),
      m34: det *
      (self.m14 * self.m22 * self.m31 - self.m12 * self.m24 * self.m31 -
        self.m14 * self.m21 * self.m32 + self.m11 * self.m24 * self.m32 +
        self.m12 * self.m21 * self.m34 - self.m11 * self.m22 * self.m34),
      m41: det *
      (self.m23 * self.m32 * self.m41 - self.m22 * self.m33 * self.m41 -
        self.m23 * self.m31 * self.m42 + self.m21 * self.m33 * self.m42 +
        self.m22 * self.m31 * self.m43 - self.m21 * self.m32 * self.m43),
      m42: det *
      (self.m12 * self.m33 * self.m41 - self.m13 * self.m32 * self.m41 +
        self.m13 * self.m31 * self.m42 - self.m11 * self.m33 * self.m42 -
        self.m12 * self.m31 * self.m43 + self.m11 * self.m32 * self.m43),
      m43: det *
      (self.m13 * self.m22 * self.m41 - self.m12 * self.m23 * self.m41 -
        self.m13 * self.m21 * self.m42 + self.m11 * self.m23 * self.m42 +
        self.m12 * self.m21 * self.m43 - self.m11 * self.m22 * self.m43),
      m44: det *
      (self.m12 * self.m23 * self.m31 - self.m13 * self.m22 * self.m31 +
        self.m13 * self.m21 * self.m32 - self.m11 * self.m23 * self.m32 -
        self.m12 * self.m21 * self.m33 + self.m11 * self.m22 * self.m33),
    })
  }

  pub fn transpose(&self) -> Self {
    Self {
      m11: self.m11, m12: self.m21, m13: self.m31, m14: self.m41,
      m21: self.m12, m22: self.m22, m23: self.m32, m24: self.m42,
      m31: self.m13, m32: self.m23, m33: self.m33, m34: self.m43,
      m41: self.m14, m42: self.m24, m43: self.m34, m44: self.m44,
    }
  }

  pub fn multiply_vector(&self, pin: &[f32; 4]) -> [f32; 4] {
    [
      pin[0] * self.m11 + pin[1] * self.m21 + pin[2] * self.m31 + pin[3] * self.m41,
      pin[0] * self.m12 + pin[1] * self.m22 + pin[2] * self.m32 + pin[3] * self.m42,
      pin[0] * self.m13 + pin[1] * self.m23 + pin[2] * self.m33 + pin[3] * self.m43,
      pin[0] * self.m14 + pin[1] * self.m24 + pin[2] * self.m34 + pin[3] * self.m44,
    ]
  }

  // https://drafts.csswg.org/css-transforms-2/#decomposing-a-3d-matrix
  pub fn decompose(&self) -> Option<TransformList> {
    // Combine 2 point.
    let combine = |a: [f32; 3], b: [f32; 3], ascl: f32, bscl: f32| {
      [
        (ascl * a[0]) + (bscl * b[0]),
        (ascl * a[1]) + (bscl * b[1]),
        (ascl * a[2]) + (bscl * b[2]),
      ]
    };

    // Dot product.
    let dot = |a: [f32; 3], b: [f32; 3]| a[0] * b[0] + a[1] * b[1] + a[2] * b[2];

    // Cross product.
    let cross = |row1: [f32; 3], row2: [f32; 3]| {
      [
        row1[1] * row2[2] - row1[2] * row2[1],
        row1[2] * row2[0] - row1[0] * row2[2],
        row1[0] * row2[1] - row1[1] * row2[0],
      ]
    };

    if self.m44 == 0.0 {
      return None;
    }

    let scaling_factor = self.m44;

    // Normalize the matrix.
    let mut matrix = self.clone();
    matrix.scale_by_factor(1.0 / scaling_factor);

    // perspective_matrix is used to solve for perspective, but it also provides
    // an easy way to test for singularity of the upper 3x3 component.
    let mut perspective_matrix = matrix.clone();
    perspective_matrix.m14 = 0.0;
    perspective_matrix.m24 = 0.0;
    perspective_matrix.m34 = 0.0;
    perspective_matrix.m44 = 1.0;

    if perspective_matrix.determinant() == 0.0 {
      return None;
    }

    let mut transforms = vec![];

    // First, isolate perspective.
    if matrix.m14 != 0.0 || matrix.m24 != 0.0 || matrix.m34 != 0.0 {
      let right_hand_side: [f32; 4] = [matrix.m14, matrix.m24, matrix.m34, matrix.m44];

      perspective_matrix = perspective_matrix.inverse().unwrap().transpose();
      let perspective = perspective_matrix.multiply_vector(&right_hand_side);
      if perspective[0] == 0.0 && perspective[1] == 0.0 && perspective[3] == 0.0 {
        transforms.push(Transform::Perspective(
          Length::px(-1.0 / perspective[2])
        ))
      } else {
        return None
      }
    }

    // Next take care of translation (easy).
    // let translate = Translate3D(matrix.m41, matrix.m42, matrix.m43);
    if matrix.m41 != 0.0 || matrix.m42 != 0.0 || matrix.m43 != 0.0 {
      transforms.push(Transform::Translate3d(
        LengthPercentage::px(matrix.m41),
        LengthPercentage::px(matrix.m42),
        Length::px(matrix.m43),
      ));
    }

    // Now get scale and shear. 'row' is a 3 element array of 3 component vectors
    let mut row = [
      [ matrix.m11, matrix.m12, matrix.m13 ],
      [ matrix.m21, matrix.m22, matrix.m23 ],
      [ matrix.m31, matrix.m32, matrix.m33 ],
    ];

    // Compute X scale factor and normalize first row.
    let row0len = (row[0][0] * row[0][0] + row[0][1] * row[0][1] + row[0][2] * row[0][2]).sqrt();
    let mut scale_x = row0len;
    row[0] = [
      row[0][0] / row0len,
      row[0][1] / row0len,
      row[0][2] / row0len,
    ];

    // Compute XY shear factor and make 2nd row orthogonal to 1st.
    let mut skew_x = dot(row[0], row[1]);
    row[1] = combine(row[1], row[0], 1.0, -skew_x);

    // Now, compute Y scale and normalize 2nd row.
    let row1len = (row[1][0] * row[1][0] + row[1][1] * row[1][1] + row[1][2] * row[1][2]).sqrt();
    let mut scale_y = row1len;
    row[1] = [
      row[1][0] / row1len,
      row[1][1] / row1len,
      row[1][2] / row1len,
    ];
    skew_x /= scale_y;

    // Compute XZ and YZ shears, orthogonalize 3rd row
    let mut skew_y = dot(row[0], row[2]);
    row[2] = combine(row[2], row[0], 1.0, -skew_y);
    let mut skew_z = dot(row[1], row[2]);
    row[2] = combine(row[2], row[1], 1.0, -skew_z);

    // Next, get Z scale and normalize 3rd row.
    let row2len = (row[2][0] * row[2][0] + row[2][1] * row[2][1] + row[2][2] * row[2][2]).sqrt();
    let mut scale_z = row2len;
    row[2] = [
      row[2][0] / row2len,
      row[2][1] / row2len,
      row[2][2] / row2len,
    ];
    skew_y /= scale_z;
    skew_z /= scale_z;

    if skew_z != 0.0 {
      return None // ???
    }

    // Round to 5 digits of precision, which is what we print.
    macro_rules! round {
      ($var: ident) => {
        $var = ($var * 100000.0).round() / 100000.0;
      };
    }

    round!(skew_x);
    round!(skew_y);
    round!(skew_z);

    if skew_x != 0.0 || skew_y != 0.0 || skew_z != 0.0 {
      transforms.push(Transform::Skew(
        Angle::Rad(skew_x),
        Angle::Rad(skew_y)
      ));
    }

    // At this point, the matrix (in rows) is orthonormal.
    // Check for a coordinate system flip.  If the determinant
    // is -1, then negate the matrix and the scaling factors.
    if dot(row[0], cross(row[1], row[2])) < 0.0 {
      scale_x = -scale_x;
      scale_y = -scale_y;
      scale_z = -scale_z;
      for i in 0..3 {
        row[i][0] *= -1.0;
        row[i][1] *= -1.0;
        row[i][2] *= -1.0;
      }
    }

    round!(scale_x);
    round!(scale_y);
    round!(scale_z);

    if scale_x != 1.0 || scale_y != 1.0 || scale_z != 1.0 {
      transforms.push(Transform::Scale3d(
        NumberOrPercentage::Number(scale_x),
        NumberOrPercentage::Number(scale_y),
        NumberOrPercentage::Number(scale_z)
      ))
    }

    // Now, get the rotations out.
    let mut rotate_x = 0.5 * ((1.0 + row[0][0] - row[1][1] - row[2][2]).max(0.0)).sqrt();
    let mut rotate_y = 0.5 * ((1.0 - row[0][0] + row[1][1] - row[2][2]).max(0.0)).sqrt();
    let mut rotate_z = 0.5 * ((1.0 - row[0][0] - row[1][1] + row[2][2]).max(0.0)).sqrt();
    let rotate_w = 0.5 * ((1.0 + row[0][0] + row[1][1] + row[2][2]).max(0.0)).sqrt();

    if row[2][1] > row[1][2] {
      rotate_x = -rotate_x
    }

    if row[0][2] > row[2][0] {
      rotate_y = -rotate_y
    }

    if row[1][0] > row[0][1] {
      rotate_z = -rotate_z
    }

    let len = (rotate_x * rotate_x + rotate_y * rotate_y + rotate_z * rotate_z).sqrt();
    if len != 0.0 {
      rotate_x /= len;
      rotate_y /= len;
      rotate_z /= len;
    }
    let a = 2.0 * len.atan2(rotate_w);

    // normalize the vector so one of the values is 1
    let max = rotate_x.max(rotate_y).max(rotate_z);
    rotate_x /= max;
    rotate_y /= max;
    rotate_z /= max;

    if a != 0.0 {
      transforms.push(Transform::Rotate3d(rotate_x, rotate_y, rotate_z, Angle::Rad(a)))
    }
    
    if transforms.is_empty() {
      return None
    }

    Some(TransformList(transforms))
  }
}

impl Parse for Transform {
  fn parse<'i, 't>(input: &mut Parser<'i, 't>) -> Result<Self, ParseError<'i, ()>> {
    let function = input.expect_function()?.clone();
    input.parse_nested_block(|input| {
      let location = input.current_source_location();
      match_ignore_ascii_case! { &function,
        "matrix" => {
          let a = f32::parse(input)?;
          input.expect_comma()?;
          let b = f32::parse(input)?;
          input.expect_comma()?;
          let c = f32::parse(input)?;
          input.expect_comma()?;
          let d = f32::parse(input)?;
          input.expect_comma()?;
          let e = f32::parse(input)?;
          input.expect_comma()?;
          let f = f32::parse(input)?;
          Ok(Transform::Matrix(Matrix { a, b, c, d, e, f }))
        },
        "matrix3d" => {
          let m11 = f32::parse(input)?;
          input.expect_comma()?;
          let m12 = f32::parse(input)?;
          input.expect_comma()?;
          let m13 = f32::parse(input)?;
          input.expect_comma()?;
          let m14 = f32::parse(input)?;
          input.expect_comma()?;
          let m21 = f32::parse(input)?;
          input.expect_comma()?;
          let m22 = f32::parse(input)?;
          input.expect_comma()?;
          let m23 = f32::parse(input)?;
          input.expect_comma()?;
          let m24 = f32::parse(input)?;
          input.expect_comma()?;
          let m31 = f32::parse(input)?;
          input.expect_comma()?;
          let m32 = f32::parse(input)?;
          input.expect_comma()?;
          let m33 = f32::parse(input)?;
          input.expect_comma()?;
          let m34 = f32::parse(input)?;
          input.expect_comma()?;
          let m41 = f32::parse(input)?;
          input.expect_comma()?;
          let m42 = f32::parse(input)?;
          input.expect_comma()?;
          let m43 = f32::parse(input)?;
          input.expect_comma()?;
          let m44 = f32::parse(input)?;
          Ok(Transform::Matrix3d(Matrix3d {
            m11, m12, m13, m14,
            m21, m22, m23, m24,
            m31, m32, m33, m34,
            m41, m42, m43, m44
          }))
        },
        "translate" => {
          let x = LengthPercentage::parse(input)?;
          if input.try_parse(|input| input.expect_comma()).is_ok() {
            let y = LengthPercentage::parse(input)?;
            Ok(Transform::Translate(x, y))
          } else {
            Ok(Transform::Translate(x, LengthPercentage::zero()))
          }
        },
        "translatex" => {
          let x = LengthPercentage::parse(input)?;
          Ok(Transform::TranslateX(x))
        },
        "translatey" => {
          let y = LengthPercentage::parse(input)?;
          Ok(Transform::TranslateY(y))
        },
        "translatez" => {
          let z = Length::parse(input)?;
          Ok(Transform::TranslateZ(z))
        },
        "translate3d" => {
          let x = LengthPercentage::parse(input)?;
          input.expect_comma()?;
          let y = LengthPercentage::parse(input)?;
          input.expect_comma()?;
          let z = Length::parse(input)?;
          Ok(Transform::Translate3d(x, y, z))
        },
        "scale" => {
          let x = NumberOrPercentage::parse(input)?;
          if input.try_parse(|input| input.expect_comma()).is_ok() {
            let y = NumberOrPercentage::parse(input)?;
            Ok(Transform::Scale(x, y))
          } else {
            Ok(Transform::Scale(x.clone(), x))
          }
        },
        "scalex" => {
          let x = NumberOrPercentage::parse(input)?;
          Ok(Transform::ScaleX(x))
        },
        "scaley" => {
          let y = NumberOrPercentage::parse(input)?;
          Ok(Transform::ScaleY(y))
        },
        "scalez" => {
          let z = NumberOrPercentage::parse(input)?;
          Ok(Transform::ScaleZ(z))
        },
        "scale3d" => {
          let x = NumberOrPercentage::parse(input)?;
          input.expect_comma()?;
          let y = NumberOrPercentage::parse(input)?;
          input.expect_comma()?;
          let z = NumberOrPercentage::parse(input)?;
          Ok(Transform::Scale3d(x, y, z))
        },
        "rotate" => {
          let angle = Angle::parse(input)?;
          Ok(Transform::Rotate(angle))
        },
        "rotatex" => {
          let angle = Angle::parse(input)?;
          Ok(Transform::RotateX(angle))
        },
        "rotatey" => {
          let angle = Angle::parse(input)?;
          Ok(Transform::RotateY(angle))
        },
        "rotatez" => {
          let angle = Angle::parse(input)?;
          Ok(Transform::RotateZ(angle))
        },
        "rotate3d" => {
          let x = f32::parse(input)?;
          input.expect_comma()?;
          let y = f32::parse(input)?;
          input.expect_comma()?;
          let z = f32::parse(input)?;
          input.expect_comma()?;
          let angle = Angle::parse(input)?;
          Ok(Transform::Rotate3d(x, y, z, angle))
        },
        "skew" => {
          let x = Angle::parse(input)?;
          if input.try_parse(|input| input.expect_comma()).is_ok() {
            let y = Angle::parse(input)?;
            Ok(Transform::Skew(x, y))
          } else {
            Ok(Transform::Skew(x, Angle::Deg(0.0)))
          }
        },
        "skewx" => {
          let angle = Angle::parse(input)?;
          Ok(Transform::SkewX(angle))
        },
        "skewy" => {
          let angle = Angle::parse(input)?;
          Ok(Transform::SkewY(angle))
        },
        "perspective" => {
          let len = Length::parse(input)?;
          Ok(Transform::Perspective(len))
        },
        _ => Err(location.new_unexpected_token_error(
          cssparser::Token::Ident(function.clone())
        ))
      }
    })
  }
}

impl ToCss for Transform {
  fn to_css<W>(&self, dest: &mut Printer<W>) -> std::fmt::Result where W: std::fmt::Write {
    use Transform::*;
    match self {
      Translate(x, y) => {
        if dest.minify && *x == 0.0 && *y != 0.0 {
          dest.write_str("translateY(")?;
          y.to_css(dest)?
        } else {
          dest.write_str("translate(")?;
          x.to_css(dest)?;
          if *y != 0.0 {
            dest.delim(',', false)?;
            y.to_css(dest)?;
          }
        }
        dest.write_char(')')
      }
      TranslateX(x) => {
        dest.write_str(if dest.minify { "translate(" } else { "translateX(" })?;
        x.to_css(dest)?;
        dest.write_char(')')
      }
      TranslateY(y) => {
        dest.write_str("translateY(")?;
        y.to_css(dest)?;
        dest.write_char(')')
      }
      TranslateZ(z) => {
        dest.write_str("translateZ(")?;
        z.to_css(dest)?;
        dest.write_char(')')
      }
      Translate3d(x, y, z) => {
        if dest.minify && *x != 0.0 && *y == 0.0 && *z == 0.0 {
          dest.write_str("translate(")?;
          x.to_css(dest)?;
        } else if dest.minify && *x == 0.0 && *y != 0.0 && *z == 0.0 {
          dest.write_str("translateY(")?;
          y.to_css(dest)?;
        } else if dest.minify && *x == 0.0 && *y == 0.0 && *z != 0.0 {
          dest.write_str("translateZ(")?;
          z.to_css(dest)?;
        } else if dest.minify && *z == 0.0 {
          dest.write_str("translate(")?;
          x.to_css(dest)?;
          dest.delim(',', false)?;
          y.to_css(dest)?;
        } else {
          dest.write_str("translate3d(")?;
          x.to_css(dest)?;
          dest.delim(',', false)?;
          y.to_css(dest)?;
          dest.delim(',', false)?;
          z.to_css(dest)?;
        }
        dest.write_char(')')
      }
      Scale(x, y) => {
        if dest.minify && *x == 1.0 && *y != 1.0 {
          dest.write_str("scaleY(")?;
          y.to_css(dest)?;
        } else if dest.minify && *x != 1.0 && *y == 1.0 {
          dest.write_str("scaleX(")?;
          x.to_css(dest)?;
        } else {
          dest.write_str("scale(")?;
          x.to_css(dest)?;
          if *y != *x {
            dest.delim(',', false)?;
            y.to_css(dest)?;
          }
        }
        dest.write_char(')')
      }
      ScaleX(x) => {
        dest.write_str("scaleX(")?;
        x.to_css(dest)?;
        dest.write_char(')')
      }
      ScaleY(y) => {
        dest.write_str("scaleY(")?;
        y.to_css(dest)?;
        dest.write_char(')')
      }
      ScaleZ(z) => {
        dest.write_str("scaleZ(")?;
        z.to_css(dest)?;
        dest.write_char(')')
      }
      Scale3d(x, y, z) => {
        if dest.minify && *z == 1.0 && *x == *y {
          // scale3d(x, x, 1) => scale(x)
          dest.write_str("scale(")?;
          x.to_css(dest)?;
        } else if dest.minify && *x != 1.0 && *y == 1.0 && *z == 1.0 {
          // scale3d(x, 1, 1) => scaleX(x)
          dest.write_str("scaleX(")?;
          x.to_css(dest)?;
        } else if dest.minify && *x == 1.0 && *y != 1.0 && *z == 1.0 {
           // scale3d(1, y, 1) => scaleY(y)
          dest.write_str("scaleY(")?;
          y.to_css(dest)?;
        } else if dest.minify && *x == 1.0 && *y == 1.0 && *z != 1.0 {
          // scale3d(1, 1, z) => scaleZ(z)
          dest.write_str("scaleZ(")?;
          z.to_css(dest)?;
        } else if dest.minify && *z == 1.0 {
          // scale3d(x, y, 1) => scale(x, y)
          dest.write_str("scale(")?;
          x.to_css(dest)?;
          dest.delim(',', false)?;
          y.to_css(dest)?;
        } else {
          dest.write_str("scale3d(")?;
          x.to_css(dest)?;
          dest.delim(',', false)?;
          y.to_css(dest)?;
          dest.delim(',', false)?;
          z.to_css(dest)?;
        }
        dest.write_char(')')
      }
      Rotate(angle) => {
        dest.write_str("rotate(")?;
        angle.to_css(dest)?;
        dest.write_char(')')
      }
      RotateX(angle) => {
        dest.write_str("rotateX(")?;
        angle.to_css(dest)?;
        dest.write_char(')')
      }
      RotateY(angle) => {
        dest.write_str("rotateY(")?;
        angle.to_css(dest)?;
        dest.write_char(')')
      }
      RotateZ(angle) => {
        dest.write_str(if dest.minify { "rotate(" } else { "rotateZ(" })?;
        angle.to_css(dest)?;
        dest.write_char(')')
      }
      Rotate3d(x, y, z, angle) => {
        if dest.minify && *x == 1.0 && *y == 0.0 && *z == 0.0 {
          // rotate3d(1, 0, 0, a) => rotateX(a)
          dest.write_str("rotateX(")?;
          angle.to_css(dest)?;
        } else if dest.minify && *x == 0.0 && *y == 1.0 && *z == 0.0 {
          // rotate3d(0, 1, 0, a) => rotateY(a)
          dest.write_str("rotateY(")?;
          angle.to_css(dest)?;
        } else if dest.minify && *x == 0.0 && *y == 0.0 && *z == 1.0 {
          // rotate3d(0, 0, 1, a) => rotate(a)
          dest.write_str("rotate(")?;
          angle.to_css(dest)?;
        } else {
          dest.write_str("rotate3d(")?;
          x.to_css(dest)?;
          dest.delim(',', false)?;
          y.to_css(dest)?;
          dest.delim(',', false)?;
          z.to_css(dest)?;
          dest.delim(',', false)?;
          angle.to_css(dest)?;
        }
        dest.write_char(')')
      }
      Skew(x, y) => {
        if dest.minify && x.is_zero() && !y.is_zero() {
          dest.write_str("skewY(")?;
          y.to_css(dest)?
        } else {
          dest.write_str("skew(")?;
          x.to_css(dest)?;
          if !y.is_zero() {
            dest.delim(',', false)?;
            y.to_css(dest)?;
          }
        }
        dest.write_char(')')
      }
      SkewX(angle) => {
        dest.write_str(if dest.minify { "skew(" } else { "skewX(" })?;
        angle.to_css(dest)?;
        dest.write_char(')')
      }
      SkewY(angle) => {
        dest.write_str("skewY(")?;
        angle.to_css(dest)?;
        dest.write_char(')')
      }
      Perspective(len) => {
        dest.write_str("perspective(")?;
        len.to_css(dest)?;
        dest.write_char(')')
      }
      Matrix(super::transform::Matrix { a, b, c, d, e, f }) => {
        dest.write_str("matrix(")?;
        a.to_css(dest)?;
        dest.delim(',', false)?;
        b.to_css(dest)?;
        dest.delim(',', false)?;
        c.to_css(dest)?;
        dest.delim(',', false)?;
        d.to_css(dest)?;
        dest.delim(',', false)?;
        e.to_css(dest)?;
        dest.delim(',', false)?;
        f.to_css(dest)?;
        dest.write_char(')')
      }
      Matrix3d(super::transform::Matrix3d {
        m11, m12, m13, m14,
        m21, m22, m23, m24,
        m31, m32, m33, m34,
        m41, m42, m43, m44
      }) => {
        dest.write_str("matrix3d(")?;
        m11.to_css(dest)?;
        dest.delim(',', false)?;
        m12.to_css(dest)?;
        dest.delim(',', false)?;
        m13.to_css(dest)?;
        dest.delim(',', false)?;
        m14.to_css(dest)?;
        dest.delim(',', false)?;
        m21.to_css(dest)?;
        dest.delim(',', false)?;
        m22.to_css(dest)?;
        dest.delim(',', false)?;
        m23.to_css(dest)?;
        dest.delim(',', false)?;
        m24.to_css(dest)?;
        dest.delim(',', false)?;
        m31.to_css(dest)?;
        dest.delim(',', false)?;
        m32.to_css(dest)?;
        dest.delim(',', false)?;
        m33.to_css(dest)?;
        dest.delim(',', false)?;
        m34.to_css(dest)?;
        dest.delim(',', false)?;
        m41.to_css(dest)?;
        dest.delim(',', false)?;
        m42.to_css(dest)?;
        dest.delim(',', false)?;
        m43.to_css(dest)?;
        dest.delim(',', false)?;
        m44.to_css(dest)?;
        dest.write_char(')')
      }
    }
  }
}

impl Transform {
  pub fn to_matrix(&self) -> Option<Matrix3d<f32>> {
    match &self {
      Transform::Translate(LengthPercentage::Dimension(x), LengthPercentage::Dimension(y)) => {
        if let (Some(x), Some(y)) = (x.to_px(), y.to_px()) {
          return Some(Matrix3d::translate(x, y, 0.0))
        }
      }
      Transform::TranslateX(LengthPercentage::Dimension(x)) => {
        if let Some(x) = x.to_px() {
          return Some(Matrix3d::translate(x, 0.0, 0.0))
        }
      }
      Transform::TranslateY(LengthPercentage::Dimension(y)) => {
        if let Some(y) = y.to_px() {
          return Some(Matrix3d::translate(0.0, y, 0.0))
        }
      }
      Transform::TranslateZ(z) => {
        if let Some(z) = z.to_px() {
          return Some(Matrix3d::translate(0.0, 0.0, z))
        }
      }
      Transform::Translate3d(LengthPercentage::Dimension(x), LengthPercentage::Dimension(y), z) => {
        if let (Some(x), Some(y), Some(z)) = (x.to_px(), y.to_px(), z.to_px()) {
          return Some(Matrix3d::translate(x, y, z))
        }
      }
      Transform::Scale(x, y) => {
        return Some(Matrix3d::scale(x.into(), y.into(), 1.0))
      }
      Transform::ScaleX(x) => {
        return Some(Matrix3d::scale(x.into(), 1.0, 1.0))
      }
      Transform::ScaleY(y) => {
        return Some(Matrix3d::scale(1.0, y.into(), 1.0))
      }
      Transform::ScaleZ(z) => {
        return Some(Matrix3d::scale(1.0, 1.0, z.into()))
      }
      Transform::Scale3d(x, y, z) => {
        return Some(Matrix3d::scale(x.into(), y.into(), z.into()))
      }
      Transform::Rotate(angle) | Transform::RotateZ(angle) => {
        return Some(Matrix3d::rotate(0.0, 0.0, 1.0, angle.to_radians()))
      }
      Transform::RotateX(angle) => {
        return Some(Matrix3d::rotate(1.0, 0.0, 0.0, angle.to_radians()))
      }
      Transform::RotateY(angle) => {
        return Some(Matrix3d::rotate(0.0, 1.0, 0.0, angle.to_radians()))
      }
      Transform::Rotate3d(x, y, z, angle) => {
        return Some(Matrix3d::rotate(*x, *y, *z, angle.to_radians()))
      }
      Transform::Skew(x, y) => {
        return Some(Matrix3d::skew(x.to_radians(), y.to_radians()))
      }
      Transform::SkewX(x) => {
        return Some(Matrix3d::skew(x.to_radians(), 0.0))
      }
      Transform::SkewY(y) => {
        return Some(Matrix3d::skew(0.0, y.to_radians()))
      }
      Transform::Perspective(len) => {
        if let Some(len) = len.to_px() {
          return Some(Matrix3d::perspective(len))
        }
      }
      Transform::Matrix(m) => {
        return Some(m.to_matrix3d())
      }
      Transform::Matrix3d(m) => return Some(m.clone()),
      _ => {}
    }
    None
  }
}

// https://drafts.csswg.org/css-transforms-2/#transform-style-property
enum_property!(TransformStyle,
  ("flat", Flat),
  ("preserve-3d", Preserve3d)
);

// https://drafts.csswg.org/css-transforms-1/#transform-box
enum_property!(TransformBox,
  ("content-box", ContentBox),
  ("border-box", BorderBox),
  ("fill-box", FillBox),
  ("stroke-box", StrokeBox),
  ("view-box", ViewBox)
);

// https://drafts.csswg.org/css-transforms-2/#backface-visibility-property
enum_property!(BackfaceVisibility,
  Visible,
  Hidden
);

/// https://drafts.csswg.org/css-transforms-2/#perspective-property
#[derive(Debug, Clone, PartialEq)]
pub enum Perspective {
  None,
  Length(Length)
}

impl Parse for Perspective {
  fn parse<'i, 't>(input: &mut Parser<'i, 't>) -> Result<Self, ParseError<'i, ()>> {
    if input.try_parse(|input| input.expect_ident_matching("none")).is_ok() {
      return Ok(Perspective::None)
    }

    Ok(Perspective::Length(Length::parse(input)?))
  }
}

impl ToCss for Perspective {
  fn to_css<W>(&self, dest: &mut Printer<W>) -> std::fmt::Result where W: std::fmt::Write {
    match self {
      Perspective::None => dest.write_str("none"),
      Perspective::Length(len) => len.to_css(dest)
    }
  }
}
