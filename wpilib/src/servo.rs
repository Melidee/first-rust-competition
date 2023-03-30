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
    fn new(channel: i32) -> Servo {
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

    fn set(&mut self, value: f64) {
        self.pwm.set_position(value).unwrap();
    }

    fn get(&self) -> f64 {
        self.pwm.position().unwrap()
    }

    fn set_offline(&mut self) {
        self.pwm.set_raw(0).unwrap();
    }

    fn set_angle(&mut self, new_angle_degrees: f64) {
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
    }

    fn angle(&self) -> f64 {
        self.get().unwrap() * get_servo_angle_range() + k_min_servo_angle
    }

    fn max_angle(&self) -> f64 {
        k_max_servo_angle
    }

    fn min_angle(&self) -> f64 {
        k_min_servo_angle
    }

    fn get_servo_angle_range(&self) -> HalResult<f64> {
        self.k_max_servo_angle - self.k_min_servo_angle
    }
}
