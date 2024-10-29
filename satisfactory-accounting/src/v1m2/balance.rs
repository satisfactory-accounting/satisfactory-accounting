// Copyright 2021 Zachary Stewart
//
//   Licensed under the Apache License, Version 2.0 (the "License");
//   you may not use this file except in compliance with the License.
//   You may obtain a copy of the License at
//
//       http://www.apache.org/licenses/LICENSE-2.0
use std::collections::BTreeMap;
use std::iter::Sum;
use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};

use serde::{Deserialize, Serialize};

use crate::database::ItemId;

/// The balance of a node, including items produced or consumed and power used.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct Balance {
    /// Net power in MW (negative is consumption, positive is production).
    pub power: f32,
    /// Net balance of each item type, in units-per-minute by ID.
    pub balances: BTreeMap<ItemId, f32>,
}

impl Balance {
    /// create a new, empty balance.
    pub fn empty() -> Self {
        Default::default()
    }

    /// Create a balance that only has power usage.
    pub fn power_only(power: f32) -> Self {
        Self {
            power,
            balances: Default::default(),
        }
    }

    /// Create a new balance with the given power and productions.
    pub fn new(power: f32, balances: impl IntoIterator<Item = (ItemId, f32)>) -> Self {
        Self {
            power,
            balances: balances.into_iter().collect(),
        }
    }
}

impl Add for Balance {
    type Output = Self;

    #[inline]
    fn add(mut self, rhs: Self) -> Self::Output {
        self += rhs;
        self
    }
}

impl Add<&Balance> for Balance {
    type Output = Self;

    #[inline]
    fn add(mut self, rhs: &Self) -> Self::Output {
        self += rhs;
        self
    }
}

impl AddAssign for Balance {
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        *self += &rhs;
    }
}

impl AddAssign<&Balance> for Balance {
    fn add_assign(&mut self, rhs: &Self) {
        self.power += rhs.power;
        for (&item, &balance) in &rhs.balances {
            *self.balances.entry(item).or_default() += balance;
        }
    }
}

impl Sub for Balance {
    type Output = Self;

    #[inline]
    fn sub(mut self, rhs: Self) -> Self::Output {
        self -= rhs;
        self
    }
}

impl Sub<&Balance> for Balance {
    type Output = Self;

    #[inline]
    fn sub(mut self, rhs: &Self) -> Self::Output {
        self -= rhs;
        self
    }
}

impl SubAssign for Balance {
    #[inline]
    fn sub_assign(&mut self, rhs: Self) {
        *self -= &rhs;
    }
}

impl SubAssign<&Balance> for Balance {
    fn sub_assign(&mut self, rhs: &Self) {
        self.power -= rhs.power;
        for (&item, &balance) in &rhs.balances {
            *self.balances.entry(item).or_default() -= balance;
        }
    }
}

impl Mul<f32> for Balance {
    type Output = Self;

    #[inline]
    fn mul(mut self, rhs: f32) -> Self::Output {
        self *= rhs;
        self
    }
}

impl Mul<&f32> for Balance {
    type Output = Self;

    #[inline]
    fn mul(self, rhs: &f32) -> Self::Output {
        self * *rhs
    }
}

impl MulAssign<f32> for Balance {
    fn mul_assign(&mut self, rhs: f32) {
        self.power *= rhs;
        for balance in self.balances.values_mut() {
            *balance *= rhs;
        }
    }
}

impl MulAssign<&f32> for Balance {
    #[inline]
    fn mul_assign(&mut self, rhs: &f32) {
        *self *= *rhs;
    }
}

impl Div<f32> for Balance {
    type Output = Self;

    #[inline]
    fn div(mut self, rhs: f32) -> Self::Output {
        self /= rhs;
        self
    }
}

impl Div<&f32> for Balance {
    type Output = Self;

    #[inline]
    fn div(self, rhs: &f32) -> Self::Output {
        self / *rhs
    }
}

impl DivAssign<f32> for Balance {
    fn div_assign(&mut self, rhs: f32) {
        self.power /= rhs;
        for balance in self.balances.values_mut() {
            *balance /= rhs;
        }
    }
}

impl DivAssign<&f32> for Balance {
    #[inline]
    fn div_assign(&mut self, rhs: &f32) {
        *self /= *rhs;
    }
}

impl Neg for Balance {
    type Output = Self;

    fn neg(mut self) -> Self::Output {
        self.power = -self.power;
        for balance in self.balances.values_mut() {
            *balance = -*balance;
        }
        self
    }
}

impl<'a> Sum<&'a Balance> for Balance {
    fn sum<I>(iter: I) -> Self
    where
        I: Iterator<Item = &'a Self>,
    {
        let mut sum = Default::default();
        for balance in iter {
            sum += balance;
        }
        sum
    }
}
