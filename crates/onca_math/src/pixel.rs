use core::ops;
use crate::{Numeric, NumericCast, Vec2};

/// Physical pixel size
#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub struct PhysicalSize<T: Numeric> {
    pub width  : T,
    pub height : T,
}

impl<T: Numeric> PhysicalSize<T> {
    /// Create a new physical pixel size
    pub fn new(width: T, height: T) -> Self {
        PhysicalSize { width, height }
    }

    /// Convert to a logical size using the given scale
    pub fn to_logical(self, scale: T) -> LogicalSize<T> {
        LogicalSize { width: self.width * scale, height: self.height * scale  }
    }

    /// Cast to a given type
    pub fn cast<U: Numeric>(self) -> PhysicalSize<U>
    where
        T : NumericCast<U>
    {
        PhysicalSize { width: self.width.cast(), height: self.height.cast() }
    }
}

impl<T: Numeric> ops::Add for PhysicalSize<T> {
    type Output = PhysicalSize<T>;

    fn add(self, rhs: Self) -> Self::Output {
        PhysicalSize { width: self.width + rhs.width, height: self.height + rhs.height }
    }
}

impl<T: Numeric> ops::AddAssign for PhysicalSize<T> {
    fn add_assign(&mut self, rhs: Self) {
        self.width  += rhs.width;
        self.height += rhs.height;
    }
}

impl<T: Numeric> ops::Sub for PhysicalSize<T> {
    type Output = PhysicalSize<T>;

    fn sub(self, rhs: Self) -> Self::Output {
        PhysicalSize { width: self.width - rhs.width, height: self.height - rhs.height }
    }
}

impl<T: Numeric> ops::SubAssign for PhysicalSize<T> {
    fn sub_assign(&mut self, rhs: Self) {
        self.width  -= rhs.width;
        self.height -= rhs.height;
    }
}

impl<T: Numeric> From<(T, T)> for PhysicalSize<T> {
    fn from(value: (T, T)) -> Self {
        Self { width: value.0, height: value.1 }
    }
}

impl<T: Numeric> From<[T; 2]> for PhysicalSize<T> {
    fn from(value: [T; 2]) -> Self {
        Self { width: value[0], height: value[1] }
    }
}

impl<T: Numeric> From<Vec2<T>> for PhysicalSize<T> {
    fn from(value: Vec2<T>) -> Self {
        Self { width: value.x, height: value.y }
    }
}

impl<T: Numeric> From<PhysicalSize<T>> for (T, T) {
    fn from(value: PhysicalSize<T>) -> Self {
        (value.width, value.height)
    }
}

impl<T: Numeric> From<PhysicalSize<T>> for [T; 2] {
    fn from(value: PhysicalSize<T>) -> Self {
        [value.width, value.height]
    }
}

impl<T: Numeric> From<PhysicalSize<T>> for Vec2<T> {
    fn from(value: PhysicalSize<T>) -> Self {
        Vec2 { x: value.width, y: value.height }
    }
}

//==============================================================================================================================

/// Logical pixel size
#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub struct LogicalSize<T: Numeric> {
    pub width  : T,
    pub height : T,
}

impl<T: Numeric> LogicalSize<T> {
    /// Create a new physical pixel size
    pub fn new(width: T, height: T) -> Self {
        LogicalSize { width, height }
    }
    
    /// Convert to a physical size using the given scale
    pub fn to_physical(self, scale: T) -> PhysicalSize<T> {
        PhysicalSize { width: self.width * scale, height: self.height * scale  }
    }

    /// Cast to a given type
    pub fn cast<U: Numeric>(self) -> LogicalSize<U>
    where
        T : NumericCast<U>
    {
        LogicalSize { width: self.width.cast(), height: self.height.cast() }
    }
}

impl<T: Numeric> ops::Add for LogicalSize<T> {
    type Output = LogicalSize<T>;

    fn add(self, rhs: Self) -> Self::Output {
        LogicalSize { width: self.width + rhs.width, height: self.height + rhs.height }
    }
}

impl<T: Numeric> ops::AddAssign for LogicalSize<T> {
    fn add_assign(&mut self, rhs: Self) {
        self.width  += rhs.width;
        self.height += rhs.height;
    }
}

impl<T: Numeric> ops::Sub for LogicalSize<T> {
    type Output = LogicalSize<T>;

    fn sub(self, rhs: Self) -> Self::Output {
        LogicalSize { width: self.width - rhs.width, height: self.height - rhs.height }
    }
}

impl<T: Numeric> ops::SubAssign for LogicalSize<T> {
    fn sub_assign(&mut self, rhs: Self) {
        self.width  -= rhs.width;
        self.height -= rhs.height;
    }
}

impl<T: Numeric> From<(T, T)> for LogicalSize<T> {
    fn from(value: (T, T)) -> Self {
        Self { width: value.0, height: value.1 }
    }
}

impl<T: Numeric> From<[T; 2]> for LogicalSize<T> {
    fn from(value: [T; 2]) -> Self {
        Self { width: value[0], height: value[1] }
    }
}

impl<T: Numeric> From<Vec2<T>> for LogicalSize<T> {
    fn from(value: Vec2<T>) -> Self {
        Self { width: value.x, height: value.y }
    }
}

impl<T: Numeric> From<LogicalSize<T>> for (T, T) {
    fn from(value: LogicalSize<T>) -> Self {
        (value.width, value.height)
    }
}

impl<T: Numeric> From<LogicalSize<T>> for [T; 2] {
    fn from(value: LogicalSize<T>) -> Self {
        [value.width, value.height]
    }
}

impl<T: Numeric> From<LogicalSize<T>> for Vec2<T> {
    fn from(value: LogicalSize<T>) -> Self {
        Vec2 { x: value.width, y: value.height }
    }
}

//==============================================================================================================================

/// Pixel size
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Size<P: Numeric, L: Numeric> {
    Physical(PhysicalSize<P>),
    Logical(LogicalSize<L>),
}

impl<P: Numeric, L: Numeric> Size<P, L> {
    /// Get the size in physical pixels.
    pub fn to_physical(self, log_to_phys_scale: L) -> PhysicalSize<P>
    where
        L : NumericCast<P>
    {
        match self {
            Size::Physical(phys) => phys,
            Size::Logical(log) => log.to_physical(log_to_phys_scale).cast(),
        }
    }

    /// Get the size in logical pixels.
    pub fn to_logical(self, phys_to_log_scale: L) -> LogicalSize<L>
    where
        P : NumericCast<L>
    {
        match self {
            Size::Physical(phys) => phys.cast().to_logical(phys_to_log_scale),
            Size::Logical(log) => log,
        }
    }
}

impl<P: Numeric, L: Numeric> From<PhysicalSize<P>> for Size<P, L> {
    fn from(value: PhysicalSize<P>) -> Self {
        Self::Physical(value)
    }
}

impl<P: Numeric, L: Numeric> From<LogicalSize<L>> for Size<P, L> {
    fn from(value: LogicalSize<L>) -> Self {
        Self::Logical(value)
    }
}

//==============================================================================================================================

/// Physical pixel size
#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub struct PhysicalPosition<T: Numeric> {
    pub x : T,
    pub y : T,
}

impl<T: Numeric> PhysicalPosition<T> {
    /// Create a new physical pixel size
    pub fn new(x: T, y: T) -> Self {
        PhysicalPosition { x, y }
    }

    /// Convert to a logical size using the given scale
    pub fn to_logical(self, scale: T) -> LogicalPosition<T> {
        LogicalPosition { x: self.x * scale, y: self.y * scale  }
    }

    /// Cast to a given type
    pub fn cast<U: Numeric>(self) -> PhysicalPosition<U>
    where
        T : NumericCast<U>
    {
        PhysicalPosition { x: self.x.cast(), y: self.y.cast() }
    }
}

impl<T: Numeric> ops::Add for PhysicalPosition<T> {
    type Output = PhysicalPosition<T>;

    fn add(self, rhs: Self) -> Self::Output {
        PhysicalPosition { x: self.x + rhs.x, y: self.y + rhs.y }
    }
}

impl<T: Numeric> ops::AddAssign for PhysicalPosition<T> {
    fn add_assign(&mut self, rhs: Self) {
        self.x  += rhs.x;
        self.y += rhs.y;
    }
}

impl<T: Numeric> ops::Sub for PhysicalPosition<T> {
    type Output = PhysicalPosition<T>;

    fn sub(self, rhs: Self) -> Self::Output {
        PhysicalPosition { x: self.x - rhs.x, y: self.y - rhs.y }
    }
}

impl<T: Numeric> ops::SubAssign for PhysicalPosition<T> {
    fn sub_assign(&mut self, rhs: Self) {
        self.x  -= rhs.x;
        self.y -= rhs.y;
    }
}

impl<T: Numeric> ops::Add<PhysicalSize<T>> for PhysicalPosition<T> {
    type Output = PhysicalPosition<T>;

    fn add(self, rhs: PhysicalSize<T>) -> Self::Output {
        PhysicalPosition { x: self.x + rhs.width, y: self.y + rhs.width }
    }
}

impl<T: Numeric> ops::AddAssign<PhysicalSize<T>> for PhysicalPosition<T> {
    fn add_assign(&mut self, rhs: PhysicalSize<T>) {
        self.x  += rhs.width;
        self.y += rhs.height;
    }
}

impl<T: Numeric> ops::Sub<PhysicalSize<T>> for PhysicalPosition<T> {
    type Output = PhysicalPosition<T>;

    fn sub(self, rhs: PhysicalSize<T>) -> Self::Output {
        PhysicalPosition { x: self.x - rhs.width, y: self.y - rhs.height }
    }
}

impl<T: Numeric> ops::SubAssign<PhysicalSize<T>> for PhysicalPosition<T> {
    fn sub_assign(&mut self, rhs: PhysicalSize<T>) {
        self.x  -= rhs.width;
        self.y -= rhs.height;
    }
}

impl<T: Numeric> From<(T, T)> for PhysicalPosition<T> {
    fn from(value: (T, T)) -> Self {
        Self { x: value.0, y: value.1 }
    }
}

impl<T: Numeric> From<[T; 2]> for PhysicalPosition<T> {
    fn from(value: [T; 2]) -> Self {
        Self { x: value[0], y: value[1] }
    }
}

impl<T: Numeric> From<Vec2<T>> for PhysicalPosition<T> {
    fn from(value: Vec2<T>) -> Self {
        Self { x: value.x, y: value.y }
    }
}

impl<T: Numeric> From<PhysicalPosition<T>> for (T, T) {
    fn from(value: PhysicalPosition<T>) -> Self {
        (value.x, value.y)
    }
}

impl<T: Numeric> From<PhysicalPosition<T>> for [T; 2] {
    fn from(value: PhysicalPosition<T>) -> Self {
        [value.x, value.y]
    }
}

impl<T: Numeric> From<PhysicalPosition<T>> for Vec2<T> {
    fn from(value: PhysicalPosition<T>) -> Self {
        Vec2 { x: value.x, y: value.y }
    }
}

//==============================================================================================================================

/// Logical pixel size
#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub struct LogicalPosition<T: Numeric> {
    pub x  : T,
    pub y : T,
}

impl<T: Numeric> LogicalPosition<T> {
    /// Create a new physical pixel size
    pub fn new(x: T, y: T) -> Self {
        LogicalPosition { x, y }
    }
    
    /// Convert to a physical size using the given scale
    pub fn to_physical(self, scale: T) -> PhysicalPosition<T> {
        PhysicalPosition { x: self.x * scale, y: self.y * scale  }
    }

    /// Cast to a given type
    pub fn cast<U: Numeric>(self) -> LogicalPosition<U>
    where
        T : NumericCast<U>
    {
        LogicalPosition { x: self.x.cast(), y: self.y.cast() }
    }
}

impl<T: Numeric> ops::Add for LogicalPosition<T> {
    type Output = LogicalPosition<T>;

    fn add(self, rhs: Self) -> Self::Output {
        LogicalPosition { x: self.x + rhs.x, y: self.y + rhs.y }
    }
}

impl<T: Numeric> ops::AddAssign for LogicalPosition<T> {
    fn add_assign(&mut self, rhs: Self) {
        self.x  += rhs.x;
        self.y += rhs.y;
    }
}

impl<T: Numeric> ops::Sub for LogicalPosition<T> {
    type Output = LogicalPosition<T>;

    fn sub(self, rhs: Self) -> Self::Output {
        LogicalPosition { x: self.x - rhs.x, y: self.y - rhs.y }
    }
}

impl<T: Numeric> ops::SubAssign for LogicalPosition<T> {
    fn sub_assign(&mut self, rhs: Self) {
        self.x  -= rhs.x;
        self.y -= rhs.y;
    }
}

impl<T: Numeric> ops::Add<LogicalSize<T>> for LogicalPosition<T> {
    type Output = LogicalPosition<T>;

    fn add(self, rhs: LogicalSize<T>) -> Self::Output {
        LogicalPosition { x: self.x + rhs.width, y: self.y + rhs.width }
    }
}

impl<T: Numeric> ops::AddAssign<LogicalSize<T>> for LogicalPosition<T> {
    fn add_assign(&mut self, rhs: LogicalSize<T>) {
        self.x  += rhs.width;
        self.y += rhs.height;
    }
}

impl<T: Numeric> ops::Sub<LogicalSize<T>> for LogicalPosition<T> {
    type Output = LogicalPosition<T>;

    fn sub(self, rhs: LogicalSize<T>) -> Self::Output {
        LogicalPosition { x: self.x - rhs.width, y: self.y - rhs.height }
    }
}

impl<T: Numeric> ops::SubAssign<LogicalSize<T>> for LogicalPosition<T> {
    fn sub_assign(&mut self, rhs: LogicalSize<T>) {
        self.x  -= rhs.width;
        self.y -= rhs.height;
    }
}

impl<T: Numeric> From<(T, T)> for LogicalPosition<T> {
    fn from(value: (T, T)) -> Self {
        Self { x: value.0, y: value.1 }
    }
}

impl<T: Numeric> From<[T; 2]> for LogicalPosition<T> {
    fn from(value: [T; 2]) -> Self {
        Self { x: value[0], y: value[1] }
    }
}

impl<T: Numeric> From<Vec2<T>> for LogicalPosition<T> {
    fn from(value: Vec2<T>) -> Self {
        Self { x: value.x, y: value.y }
    }
}

impl<T: Numeric> From<LogicalPosition<T>> for (T, T) {
    fn from(value: LogicalPosition<T>) -> Self {
        (value.x, value.y)
    }
}

impl<T: Numeric> From<LogicalPosition<T>> for [T; 2] {
    fn from(value: LogicalPosition<T>) -> Self {
        [value.x, value.y]
    }
}

impl<T: Numeric> From<LogicalPosition<T>> for Vec2<T> {
    fn from(value: LogicalPosition<T>) -> Self {
        Vec2 { x: value.x, y: value.y }
    }
}

//==============================================================================================================================

/// Pixel position
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Position<P: Numeric, L: Numeric> {
    Physical(PhysicalPosition<P>),
    Logical(LogicalPosition<L>),
}

impl<P: Numeric, L: Numeric> Position<P, L> {
    /// Get the position in physical pixels.
    pub fn to_physical(self, log_to_phys_scale: L) -> PhysicalPosition<P>
    where
        L : NumericCast<P>
    {
        match self {
            Position::Physical(phys) => phys,
            Position::Logical(log) => log.to_physical(log_to_phys_scale).cast(),
        }
    }

    /// Get the position in logical pixels.
    pub fn to_logical(self, phys_to_log_scale: L) -> LogicalPosition<L>
    where
        P : NumericCast<L>
    {
        match self {
            Position::Physical(phys) => phys.cast().to_logical(phys_to_log_scale),
            Position::Logical(log) => log,
        }
    }
}

impl<P: Numeric, L: Numeric> From<PhysicalPosition<P>> for Position<P, L> {
    fn from(value: PhysicalPosition<P>) -> Self {
        Self::Physical(value)
    }
}

impl<P: Numeric, L: Numeric> From<LogicalPosition<L>> for Position<P, L> {
    fn from(value: LogicalPosition<L>) -> Self {
        Self::Logical(value)
    }
}