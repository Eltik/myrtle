use std::hash::{Hash, Hasher};
use std::ops::{Add, Div, Index, IndexMut, Mul, Sub};

/// Epsilon value for floating-point comparisons
const K_EPSILON: f32 = 0.00001;

/// A 2D vector with x and y components
///
/// Commonly used in Unity for 2D positions, directions, and sizes.
#[derive(Debug, Clone, Copy, serde::Deserialize, serde::Serialize, Default)]
pub struct Vector2 {
    pub x: f32,
    pub y: f32,
}

impl Vector2 {
    /// Creates a new Vector2
    pub fn new(x: f32, y: f32) -> Self {
        Vector2 { x, y }
    }

    /// Gets component by index (0=x, 1=y)
    pub fn get(&self, index: usize) -> f32 {
        match index {
            0 => self.x,
            1 => self.y,
            _ => panic!("Index out of range for Vector2"),
        }
    }

    /// Sets component by index (0=x, 1=y)
    pub fn set(&mut self, index: usize, value: f32) {
        match index {
            0 => self.x = value,
            1 => self.y = value,
            _ => panic!("Index out of range for Vector2"),
        }
    }

    /// Returns the squared length (magnitude) of the vector
    pub fn length_squared(&self) -> f32 {
        self.x * self.x + self.y * self.y
    }

    /// Returns the length (magnitude) of the vector
    pub fn length(&self) -> f32 {
        self.length_squared().sqrt()
    }

    /// Normalizes the vector to unit length (mutates in place)
    pub fn normalize(&mut self) {
        let len = self.length();
        if len > K_EPSILON {
            let inv_norm = 1.0 / len;
            self.x *= inv_norm;
            self.y *= inv_norm;
        } else {
            self.x = 0.0;
            self.y = 0.0;
        }
    }

    /// Returns a Vector2 with all components set to zero
    pub fn zero() -> Vector2 {
        Vector2::new(0.0, 0.0)
    }

    /// Returns a Vector2 with all components set to one
    pub fn one() -> Vector2 {
        Vector2::new(1.0, 1.0)
    }
}

impl Index<usize> for Vector2 {
    type Output = f32;

    fn index(&self, index: usize) -> &Self::Output {
        match index {
            0 => &self.x,
            1 => &self.y,
            _ => panic!("Index out of range for Vector2"),
        }
    }
}

impl IndexMut<usize> for Vector2 {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        match index {
            0 => &mut self.x,
            1 => &mut self.y,
            _ => panic!("Index out of range for Vector2"),
        }
    }
}

// Vector2 + Vector2
impl Add for Vector2 {
    type Output = Vector2;

    fn add(self, other: Vector2) -> Vector2 {
        Vector2::new(self.x + other.x, self.y + other.y)
    }
}

// Vector2 - Vector2
impl Sub for Vector2 {
    type Output = Vector2;

    fn sub(self, other: Vector2) -> Vector2 {
        Vector2::new(self.x - other.x, self.y - other.y)
    }
}

// Vector2 * f32 (scalar multiplication)
impl Mul<f32> for Vector2 {
    type Output = Vector2;

    fn mul(self, scalar: f32) -> Vector2 {
        Vector2::new(self.x * scalar, self.y * scalar)
    }
}

// Vector2 / f32 (scalar division)
impl Div<f32> for Vector2 {
    type Output = Vector2;

    fn div(self, scalar: f32) -> Vector2 {
        Vector2::new(self.x / scalar, self.y / scalar)
    }
}

impl PartialEq for Vector2 {
    fn eq(&self, other: &Self) -> bool {
        let diff = *self - *other;
        diff.length_squared() < K_EPSILON * K_EPSILON
    }
}

impl Hash for Vector2 {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // Hash the bit representation of floats
        self.x.to_bits().hash(state);
        (self.y.to_bits() << 2).hash(state);
    }
}

/// A 3D vector with x, y, and z components
///
/// Commonly used in Unity for 3D positions, directions, and scales.
#[derive(Debug, Clone, Copy, serde::Deserialize, serde::Serialize, Default)]
pub struct Vector3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vector3 {
    /// Creates a new Vector3
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Vector3 { x, y, z }
    }

    /// Gets component by index (0=x, 1=y, 2=z)
    pub fn get(&self, index: usize) -> f32 {
        match index {
            0 => self.x,
            1 => self.y,
            2 => self.z,
            _ => panic!("Index out of range for Vector3"),
        }
    }

    /// Sets component by index (0=x, 1=y, 2=z)
    pub fn set(&mut self, index: usize, value: f32) {
        match index {
            0 => self.x = value,
            1 => self.y = value,
            2 => self.z = value,
            _ => panic!("Index out of range for Vector3"),
        }
    }

    /// Returns the squared length (magnitude) of the vector
    ///
    /// This is faster than length() as it avoids the sqrt operation.
    /// Useful for comparing distances without needing the actual length.
    pub fn length_squared(&self) -> f32 {
        self.x * self.x + self.y * self.y + self.z * self.z
    }

    /// Returns the length (magnitude) of the vector
    ///
    /// Calculates sqrt(x² + y² + z²)
    pub fn length(&self) -> f32 {
        self.length_squared().sqrt()
    }

    /// Normalizes the vector to unit length (mutates in place)
    ///
    /// If the length is very small (< K_EPSILON), sets all components to 0.
    pub fn normalize(&mut self) {
        let len = self.length();
        if len > K_EPSILON {
            let inv_norm = 1.0 / len;
            self.x *= inv_norm;
            self.y *= inv_norm;
            self.z *= inv_norm;
        } else {
            self.x = 0.0;
            self.y = 0.0;
            self.z = 0.0;
        }
    }

    /// Returns a Vector3 with all components set to zero
    pub fn zero() -> Vector3 {
        Vector3::new(0.0, 0.0, 0.0)
    }

    /// Returns a Vector3 with all components set to one
    pub fn one() -> Vector3 {
        Vector3::new(1.0, 1.0, 1.0)
    }

    /// Converts this Vector3 to a Vector2 (drops the z component)
    pub fn to_vector2(&self) -> Vector2 {
        Vector2::new(self.x, self.y)
    }

    /// Converts this Vector3 to a Vector4 (sets w component to 0.0)
    pub fn to_vector4(&self) -> Vector4 {
        Vector4::new(self.x, self.y, self.z, 0.0)
    }
}

impl Index<usize> for Vector3 {
    type Output = f32;

    fn index(&self, index: usize) -> &Self::Output {
        match index {
            0 => &self.x,
            1 => &self.y,
            2 => &self.z,
            _ => panic!("Index out of range for Vector3"),
        }
    }
}

impl IndexMut<usize> for Vector3 {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        match index {
            0 => &mut self.x,
            1 => &mut self.y,
            2 => &mut self.z,
            _ => panic!("Index out of range for Vector3"),
        }
    }
}

// Vector3 + Vector3
impl Add for Vector3 {
    type Output = Vector3;

    fn add(self, other: Vector3) -> Vector3 {
        Vector3::new(self.x + other.x, self.y + other.y, self.z + other.z)
    }
}

// Vector3 - Vector3
impl Sub for Vector3 {
    type Output = Vector3;

    fn sub(self, other: Vector3) -> Vector3 {
        Vector3::new(self.x - other.x, self.y - other.y, self.z - other.z)
    }
}

// Vector3 * f32 (scalar multiplication)
impl Mul<f32> for Vector3 {
    type Output = Vector3;

    fn mul(self, scalar: f32) -> Vector3 {
        Vector3::new(self.x * scalar, self.y * scalar, self.z * scalar)
    }
}

// Vector3 / f32 (scalar division)
impl Div<f32> for Vector3 {
    type Output = Vector3;

    fn div(self, scalar: f32) -> Vector3 {
        Vector3::new(self.x / scalar, self.y / scalar, self.z / scalar)
    }
}

impl PartialEq for Vector3 {
    fn eq(&self, other: &Self) -> bool {
        let diff = *self - *other;
        diff.length_squared() < K_EPSILON * K_EPSILON
    }
}

impl Hash for Vector3 {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // Hash the bit representation of floats
        self.x.to_bits().hash(state);
        (self.y.to_bits() << 2).hash(state);
        (self.z.to_bits() >> 2).hash(state);
    }
}

/// A 4D vector with x, y, z, and w components
///
/// Commonly used in Unity for homogeneous coordinates and shader parameters.
#[derive(Debug, Clone, Copy, serde::Deserialize, serde::Serialize, Default)]
pub struct Vector4 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl Vector4 {
    /// Creates a new Vector4
    pub fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
        Vector4 { x, y, z, w }
    }

    /// Gets the component at the specified index (0=x, 1=y, 2=z, 3=w)
    pub fn get(&self, index: usize) -> f32 {
        match index {
            0 => self.x,
            1 => self.y,
            2 => self.z,
            3 => self.w,
            _ => panic!("Index out of range for Vector4"),
        }
    }

    /// Sets the component at the specified index (0=x, 1=y, 2=z, 3=w)
    pub fn set(&mut self, index: usize, value: f32) {
        match index {
            0 => self.x = value,
            1 => self.y = value,
            2 => self.z = value,
            3 => self.w = value,
            _ => panic!("Index out of range for Vector4"),
        }
    }

    /// Returns the squared length of this vector
    pub fn length_squared(&self) -> f32 {
        self.x * self.x + self.y * self.y + self.z * self.z + self.w * self.w
    }

    /// Returns the length of this vector
    pub fn length(&self) -> f32 {
        self.length_squared().sqrt()
    }

    /// Normalizes this vector to unit length.
    /// If the length is very small (< K_EPSILON), sets all components to 0.
    pub fn normalize(&mut self) {
        let len = self.length();
        if len > K_EPSILON {
            let inv_norm = 1.0 / len;
            self.x *= inv_norm;
            self.y *= inv_norm;
            self.z *= inv_norm;
            self.w *= inv_norm;
        } else {
            self.x = 0.0;
            self.y = 0.0;
            self.z = 0.0;
            self.w = 0.0;
        }
    }

    /// Returns a Vector4 with all components set to zero
    pub fn zero() -> Vector4 {
        Vector4::new(0.0, 0.0, 0.0, 0.0)
    }

    /// Returns a Vector4 with all components set to one
    pub fn one() -> Vector4 {
        Vector4::new(1.0, 1.0, 1.0, 1.0)
    }

    /// Converts this Vector4 to a Vector3 (drops the w component)
    pub fn to_vector3(&self) -> Vector3 {
        Vector3::new(self.x, self.y, self.z)
    }
}

impl Index<usize> for Vector4 {
    type Output = f32;

    fn index(&self, index: usize) -> &Self::Output {
        match index {
            0 => &self.x,
            1 => &self.y,
            2 => &self.z,
            3 => &self.w,
            _ => panic!("Index out of range for Vector4"),
        }
    }
}

impl IndexMut<usize> for Vector4 {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        match index {
            0 => &mut self.x,
            1 => &mut self.y,
            2 => &mut self.z,
            3 => &mut self.w,
            _ => panic!("Index out of range for Vector4"),
        }
    }
}

impl Add for Vector4 {
    type Output = Vector4;

    fn add(self, other: Vector4) -> Vector4 {
        Vector4::new(
            self.x + other.x,
            self.y + other.y,
            self.z + other.z,
            self.w + other.w,
        )
    }
}

impl Sub for Vector4 {
    type Output = Vector4;

    fn sub(self, other: Vector4) -> Vector4 {
        Vector4::new(
            self.x - other.x,
            self.y - other.y,
            self.z - other.z,
            self.w - other.w,
        )
    }
}

impl Mul<f32> for Vector4 {
    type Output = Vector4;

    fn mul(self, scalar: f32) -> Vector4 {
        Vector4::new(
            self.x * scalar,
            self.y * scalar,
            self.z * scalar,
            self.w * scalar,
        )
    }
}

impl Div<f32> for Vector4 {
    type Output = Vector4;

    fn div(self, scalar: f32) -> Vector4 {
        Vector4::new(
            self.x / scalar,
            self.y / scalar,
            self.z / scalar,
            self.w / scalar,
        )
    }
}

impl PartialEq for Vector4 {
    fn eq(&self, other: &Self) -> bool {
        let diff = *self - *other;
        diff.length_squared() < K_EPSILON * K_EPSILON
    }
}

impl Hash for Vector4 {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.x.to_bits().hash(state);
        (self.y.to_bits() << 2).hash(state);
        (self.z.to_bits() >> 2).hash(state);
        (self.w.to_bits() >> 1).hash(state);
    }
}

/// A quaternion representing a rotation in 3D space
///
/// Quaternions are used in Unity to represent rotations without gimbal lock.
/// Components are x, y, z (imaginary parts) and w (real part).
#[derive(Debug, Clone, Copy, serde::Deserialize, serde::Serialize)]
pub struct Quaternion {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl Default for Quaternion {
    /// Creates a default Quaternion (identity: w=1.0)
    /// Matches Python's UnityPy.math.Quaternion default behavior
    fn default() -> Self {
        Quaternion {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            w: 1.0, // Identity quaternion
        }
    }
}

impl Quaternion {
    /// Creates a new Quaternion
    /// Default w value is 1.0 (identity quaternion)
    pub fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
        Quaternion { x, y, z, w }
    }

    /// Gets component by index (0=x, 1=y, 2=z, 3=w)
    pub fn get(&self, index: usize) -> f32 {
        match index {
            0 => self.x,
            1 => self.y,
            2 => self.z,
            3 => self.w,
            _ => panic!("Index out of range for Quaternion"),
        }
    }

    /// Sets component by index (0=x, 1=y, 2=z, 3=w)
    pub fn set(&mut self, index: usize, value: f32) {
        match index {
            0 => self.x = value,
            1 => self.y = value,
            2 => self.z = value,
            3 => self.w = value,
            _ => panic!("Index out of range for Quaternion"),
        }
    }
}

impl Index<usize> for Quaternion {
    type Output = f32;

    fn index(&self, index: usize) -> &Self::Output {
        match index {
            0 => &self.x,
            1 => &self.y,
            2 => &self.z,
            3 => &self.w,
            _ => panic!("Index out of range for Quaternion"),
        }
    }
}

impl IndexMut<usize> for Quaternion {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        match index {
            0 => &mut self.x,
            1 => &mut self.y,
            2 => &mut self.z,
            3 => &mut self.w,
            _ => panic!("Index out of range for Quaternion"),
        }
    }
}

impl PartialEq for Quaternion {
    fn eq(&self, other: &Self) -> bool {
        self.x == other.x && self.y == other.y && self.z == other.z && self.w == other.w
    }
}

/// A rectangle defined by position (x, y) and size (width, height)
///
/// Commonly used in Unity for UI elements and 2D bounds.
#[derive(Debug, Clone, Copy, PartialEq, serde::Deserialize, serde::Serialize, Default)]
pub struct Rectangle {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Rectangle {
    /// Creates a new Rectangle
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Rectangle {
            x,
            y,
            width,
            height,
        }
    }

    /// Returns the x coordinate (left edge)
    pub fn left(&self) -> f32 {
        self.x
    }

    /// Returns the y coordinate (top edge)
    pub fn top(&self) -> f32 {
        self.y
    }

    /// Returns the right edge (x + width)
    pub fn right(&self) -> f32 {
        self.x + self.width
    }

    /// Returns the bottom edge (y + height)
    pub fn bottom(&self) -> f32 {
        self.y + self.height
    }

    /// Returns the size as (width, height)
    pub fn size(&self) -> (f32, f32) {
        (self.width, self.height)
    }

    /// Returns the location as (x, y)
    pub fn location(&self) -> (f32, f32) {
        (self.x, self.y)
    }

    /// Returns a new Rectangle with all components rounded to nearest integer
    pub fn round(&self) -> Rectangle {
        Rectangle::new(
            self.x.round(),
            self.y.round(),
            self.width.round(),
            self.height.round(),
        )
    }
}

/// An RGBA color with floating-point components
///
/// Each component (r, g, b, a) is in the range [0.0, 1.0].
/// Used in Unity for colors with alpha transparency.
#[derive(Debug, Clone, Copy, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Default for Color {
    /// Creates a default Color with alpha = 0.0 (transparent)
    /// Matches Python's UnityPy.math.Color default behavior
    fn default() -> Self {
        Color {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 0.0, // Python default: transparent
        }
    }
}

impl Color {
    /// Creates a new Color
    ///
    /// # Arguments
    ///
    /// * `r` - Red component (0.0 to 1.0)
    /// * `g` - Green component (0.0 to 1.0)
    /// * `b` - Blue component (0.0 to 1.0)
    /// * `a` - Alpha component (0.0 to 1.0)
    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Color { r, g, b, a }
    }

    /// Converts this Color to a Vector4 (r, g, b, a -> x, y, z, w)
    pub fn to_vector4(&self) -> Vector4 {
        Vector4::new(self.r, self.g, self.b, self.a)
    }
}

// Color + Color (component-wise addition)
impl Add for Color {
    type Output = Color;

    fn add(self, other: Color) -> Color {
        Color::new(
            self.r + other.r,
            self.g + other.g,
            self.b + other.b,
            self.a + other.a,
        )
    }
}

// Color - Color (component-wise subtraction)
impl Sub for Color {
    type Output = Color;

    fn sub(self, other: Color) -> Color {
        Color::new(
            self.r - other.r,
            self.g - other.g,
            self.b - other.b,
            self.a - other.a,
        )
    }
}

// Color * Color (component-wise multiplication)
impl Mul<Color> for Color {
    type Output = Color;

    fn mul(self, other: Color) -> Color {
        Color::new(
            self.r * other.r,
            self.g * other.g,
            self.b * other.b,
            self.a * other.a,
        )
    }
}

// Color * f32 (scalar multiplication)
impl Mul<f32> for Color {
    type Output = Color;

    fn mul(self, scalar: f32) -> Color {
        Color::new(
            self.r * scalar,
            self.g * scalar,
            self.b * scalar,
            self.a * scalar,
        )
    }
}

// Color / Color (component-wise division)
impl Div<Color> for Color {
    type Output = Color;

    fn div(self, other: Color) -> Color {
        Color::new(
            self.r / other.r,
            self.g / other.g,
            self.b / other.b,
            self.a / other.a,
        )
    }
}

// Color / f32 (scalar division)
impl Div<f32> for Color {
    type Output = Color;

    fn div(self, scalar: f32) -> Color {
        Color::new(
            self.r / scalar,
            self.g / scalar,
            self.b / scalar,
            self.a / scalar,
        )
    }
}

/// A 4x4 matrix used for transformations in 3D space
///
/// The matrix is stored in column-major order (16 floats).
/// Used in Unity for transformations, projections, and view matrices.
#[derive(Debug, Clone, Copy, serde::Deserialize, serde::Serialize, Default)]
pub struct Matrix4x4 {
    /// The 16 matrix elements in column-major order
    pub data: [f32; 16],
}

impl Matrix4x4 {
    /// Creates a new Matrix4x4 from 16 floats in column-major order
    pub fn new(data: [f32; 16]) -> Self {
        Matrix4x4 { data }
    }

    /// Gets element at linear index (0-15)
    ///
    /// Matrix is stored in column-major order:
    /// Index layout: [M00, M10, M20, M30, M01, M11, M21, M31, ...]
    pub fn get(&self, index: usize) -> f32 {
        self.data[index]
    }

    /// Sets element at linear index (0-15)
    pub fn set(&mut self, index: usize, value: f32) {
        self.data[index] = value;
    }

    /// Gets element at row and column (both 0-3)
    ///
    /// Column-major order: index = row + col * 4
    pub fn get_rc(&self, row: usize, col: usize) -> f32 {
        if row >= 4 || col >= 4 {
            panic!("Row and column indices must be in range [0, 3]");
        }
        self.data[row + col * 4]
    }

    /// Sets element at row and column (both 0-3)
    pub fn set_rc(&mut self, row: usize, col: usize, value: f32) {
        if row >= 4 || col >= 4 {
            panic!("Row and column indices must be in range [0, 3]");
        }
        self.data[row + col * 4] = value;
    }

    // Column 0 accessors (M00, M10, M20, M30)
    pub fn m00(&self) -> f32 {
        self.data[0]
    }
    pub fn set_m00(&mut self, value: f32) {
        self.data[0] = value;
    }

    pub fn m10(&self) -> f32 {
        self.data[1]
    }
    pub fn set_m10(&mut self, value: f32) {
        self.data[1] = value;
    }

    pub fn m20(&self) -> f32 {
        self.data[2]
    }
    pub fn set_m20(&mut self, value: f32) {
        self.data[2] = value;
    }

    pub fn m30(&self) -> f32 {
        self.data[3]
    }
    pub fn set_m30(&mut self, value: f32) {
        self.data[3] = value;
    }

    // Column 1 accessors (M01, M11, M21, M31)
    pub fn m01(&self) -> f32 {
        self.data[4]
    }
    pub fn set_m01(&mut self, value: f32) {
        self.data[4] = value;
    }

    pub fn m11(&self) -> f32 {
        self.data[5]
    }
    pub fn set_m11(&mut self, value: f32) {
        self.data[5] = value;
    }

    pub fn m21(&self) -> f32 {
        self.data[6]
    }
    pub fn set_m21(&mut self, value: f32) {
        self.data[6] = value;
    }

    pub fn m31(&self) -> f32 {
        self.data[7]
    }
    pub fn set_m31(&mut self, value: f32) {
        self.data[7] = value;
    }

    // Column 2 accessors (M02, M12, M22, M32)
    pub fn m02(&self) -> f32 {
        self.data[8]
    }
    pub fn set_m02(&mut self, value: f32) {
        self.data[8] = value;
    }

    pub fn m12(&self) -> f32 {
        self.data[9]
    }
    pub fn set_m12(&mut self, value: f32) {
        self.data[9] = value;
    }

    pub fn m22(&self) -> f32 {
        self.data[10]
    }
    pub fn set_m22(&mut self, value: f32) {
        self.data[10] = value;
    }

    pub fn m32(&self) -> f32 {
        self.data[11]
    }
    pub fn set_m32(&mut self, value: f32) {
        self.data[11] = value;
    }

    // Column 3 accessors (M03, M13, M23, M33)
    pub fn m03(&self) -> f32 {
        self.data[12]
    }
    pub fn set_m03(&mut self, value: f32) {
        self.data[12] = value;
    }

    pub fn m13(&self) -> f32 {
        self.data[13]
    }
    pub fn set_m13(&mut self, value: f32) {
        self.data[13] = value;
    }

    pub fn m23(&self) -> f32 {
        self.data[14]
    }
    pub fn set_m23(&mut self, value: f32) {
        self.data[14] = value;
    }

    pub fn m33(&self) -> f32 {
        self.data[15]
    }
    pub fn set_m33(&mut self, value: f32) {
        self.data[15] = value;
    }

    /// Creates a scaling matrix from a Vector3
    ///
    /// The resulting matrix scales by vector.x, vector.y, and vector.z along each axis.
    /// Layout: [X, 0, 0, 0,  0, Y, 0, 0,  0, 0, Z, 0,  0, 0, 0, 1]
    pub fn scale(vector: Vector3) -> Matrix4x4 {
        Matrix4x4::new([
            vector.x, 0.0, 0.0, 0.0, 0.0, vector.y, 0.0, 0.0, 0.0, 0.0, vector.z, 0.0, 0.0, 0.0,
            0.0, 1.0,
        ])
    }
}

impl Index<usize> for Matrix4x4 {
    type Output = f32;

    fn index(&self, index: usize) -> &Self::Output {
        &self.data[index]
    }
}

impl IndexMut<usize> for Matrix4x4 {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.data[index]
    }
}

impl Mul for Matrix4x4 {
    type Output = Matrix4x4;

    fn mul(self, rhs: Matrix4x4) -> Matrix4x4 {
        let mut result = Matrix4x4::new([0.0; 16]);

        // Row 0
        result.set_m00(
            self.m00() * rhs.m00()
                + self.m01() * rhs.m10()
                + self.m02() * rhs.m20()
                + self.m03() * rhs.m30(),
        );
        result.set_m01(
            self.m00() * rhs.m01()
                + self.m01() * rhs.m11()
                + self.m02() * rhs.m21()
                + self.m03() * rhs.m31(),
        );
        result.set_m02(
            self.m00() * rhs.m02()
                + self.m01() * rhs.m12()
                + self.m02() * rhs.m22()
                + self.m03() * rhs.m32(),
        );
        result.set_m03(
            self.m00() * rhs.m03()
                + self.m01() * rhs.m13()
                + self.m02() * rhs.m23()
                + self.m03() * rhs.m33(),
        );

        // Row 1
        result.set_m10(
            self.m10() * rhs.m00()
                + self.m11() * rhs.m10()
                + self.m12() * rhs.m20()
                + self.m13() * rhs.m30(),
        );
        result.set_m11(
            self.m10() * rhs.m01()
                + self.m11() * rhs.m11()
                + self.m12() * rhs.m21()
                + self.m13() * rhs.m31(),
        );
        result.set_m12(
            self.m10() * rhs.m02()
                + self.m11() * rhs.m12()
                + self.m12() * rhs.m22()
                + self.m13() * rhs.m32(),
        );
        result.set_m13(
            self.m10() * rhs.m03()
                + self.m11() * rhs.m13()
                + self.m12() * rhs.m23()
                + self.m13() * rhs.m33(),
        );

        // Row 2
        result.set_m20(
            self.m20() * rhs.m00()
                + self.m21() * rhs.m10()
                + self.m22() * rhs.m20()
                + self.m23() * rhs.m30(),
        );
        result.set_m21(
            self.m20() * rhs.m01()
                + self.m21() * rhs.m11()
                + self.m22() * rhs.m21()
                + self.m23() * rhs.m31(),
        );
        result.set_m22(
            self.m20() * rhs.m02()
                + self.m21() * rhs.m12()
                + self.m22() * rhs.m22()
                + self.m23() * rhs.m32(),
        );
        result.set_m23(
            self.m20() * rhs.m03()
                + self.m21() * rhs.m13()
                + self.m22() * rhs.m23()
                + self.m23() * rhs.m33(),
        );

        // Row 3
        result.set_m30(
            self.m30() * rhs.m00()
                + self.m31() * rhs.m10()
                + self.m32() * rhs.m20()
                + self.m33() * rhs.m30(),
        );
        result.set_m31(
            self.m30() * rhs.m01()
                + self.m31() * rhs.m11()
                + self.m32() * rhs.m21()
                + self.m33() * rhs.m31(),
        );
        result.set_m32(
            self.m30() * rhs.m02()
                + self.m31() * rhs.m12()
                + self.m32() * rhs.m22()
                + self.m33() * rhs.m32(),
        );
        result.set_m33(
            self.m30() * rhs.m03()
                + self.m31() * rhs.m13()
                + self.m32() * rhs.m23()
                + self.m33() * rhs.m33(),
        );

        result
    }
}

impl PartialEq for Matrix4x4 {
    fn eq(&self, other: &Self) -> bool {
        const EPSILON: f32 = 1e-6;

        for i in 0..16 {
            if (self.data[i] - other.data[i]).abs() >= EPSILON {
                return false;
            }
        }

        true
    }
}

#[derive(Debug, Clone, Copy, PartialEq, serde::Deserialize, serde::Serialize, Default)]
pub struct Matrix3x4 {
    pub e00: f32,
    pub e01: f32,
    pub e02: f32,
    pub e03: f32,
    pub e10: f32,
    pub e11: f32,
    pub e12: f32,
    pub e13: f32,
    pub e20: f32,
    pub e21: f32,
    pub e22: f32,
    pub e23: f32,
}
