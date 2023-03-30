use wpilib_sys::*;
use PWM::{PeriodMultiplier, PWM};

pub struct Servo {
    pwm: PWM,
    k_max_servo_angle: f64,
    k_min_servo_angle: f64,
    k_default_max_servo_pwm: f64,
    k_default_min_servo_pwm: f64,
}

impl Servo {
    pub fn new(channel: i32) -> Servo {
        let k_max_servo_angle = 180.0;
        let k_min_servo_angle = 0.0;
        let k_default_max_servo_pwm = 2.4;
        let k_default_min_servo_pwm = 0.6;
        let pwm = PWM::new(channel).unwrap();
        pwm.set_bounds(
            k_default_max_servo_pwm,
            0.0,
            0.0,
            0.0,
            k_default_min_servo_pwm,
        );
        pwm.set_period_multiplier(PeriodMultiplier::Multiplier4x);
        Servo {
            pwm,
            k_max_servo_angle,
            k_min_servo_angle,
            k_default_max_servo_pwm,
            k_default_min_servo_pwm,
        }
    }

    pub fn set(&mut self, value: f64) -> HalResult<()> {
        self.pwm.set_position(value)
    }

    pub fn get(&self) -> HalResult<f64> {
        self.pwm.position()
    }

    pub fn set_offline(&mut self) -> HalResult<()> {
        self.pwm.set_raw(0)
    }

    pub fn set_angle(&mut self, new_angle_degrees: f64) -> HalResult<()> {
        let angle = if new_angle_degrees < self.k_min_servo_angle {
            self.k_min_servo_angle
        } else if new_angle_degrees > self.k_max_servo_angle {
            self.k_max_servo_angle
        } else {
            new_angle_degrees
        };
        self.pwm
            .set_position((angle - self.k_min_servo_angle) / self.get_servo_angle_range())
            .unwrap();
        Ok(())
    }

    pub fn angle(&self) -> HalResult<f64> {
        Ok(self.get()? * get_servo_angle_range() + k_min_servo_angle)
    }

    pub fn max_angle(&self) -> f64 {
        k_max_servo_angle
    }

    pub fn min_angle(&self) -> f64 {
        k_min_servo_angle
    }

    pub fn get_servo_angle_range(&self) -> f64 {
        self.k_max_servo_angle - self.k_min_servo_angle
    }

    pub fn set_raw(&mut self, value: i32) -> HalResult<()> {
        self.pwm.set_raw(value)
    }

    pub fn raw(&self) -> HalResult<i32> {
        self.pwm.raw()
    }
    
    pub fn set_speed(&mut self, speed: f64) -> HalResult<()> {
        self.pwm.set_speed(speed)
    }

    pub fn speed(&self)-> HalResult<f64> {
        self.pwm.raw()
    }

    pub fn set_disabled(&mut self) -> HalResult<()> {
        self.pwm.set_disabled()
    }

    pub fn set_period_multiplier(&mut self, mult: PeriodMultiplier) -> HalResult<()> {
        self.pwm.set_period_multiplier()
    }

    pub fn set_zero_latch(&mut self) -> HalResult<()> {
        self.pwm.set_zero_latch()
    }

    pub fn enable_deadband_elimination(&mut self, eliminate_deadband: bool) -> HalResult<()> {
        self.pwm.enable_deadband_elimination()
    }

    pub fn set_bounds(
        &mut self,
        max: f64,
        deadband_max: f64,
        center: f64,
        deadband_min: f64,
        min: f64,
    ) -> HalResult<()> {
        self.pwm.set_bounds(max, deadband_max, center, deadband_min, min)
    }

    pub fn set_raw_bounds(
        &mut self,
        max: i32,
        deadband_max: i32,
        center: i32,
        deadband_min: i32,
        min: i32,
    ) -> HalResult<()> {
        self.pwm.set_raw_bounds(max, deadband_max, center, deadband_min, min)
    }

    pub fn raw_bounds(
        &self,
        max: &mut i32,
        deadband_max: &mut i32,
        center: &mut i32,
        deadband_min: &mut i32,
        min: &mut i32,
    ) -> HalResult<()> {
        self.pwm.raw_bounds(max, deadband_max, center, deadband_min, min)
    }

    pub fn channel(&self) -> i32 {
        self.pwm.channel()
    }
}
