#[derive(Debug, Clone, Copy)]
pub struct BatterySnapshot {
    pub percent: Option<f32>,
    pub charging: Option<bool>,
}

pub fn poll_battery() -> BatterySnapshot {
    let Ok(manager) = battery::Manager::new() else {
        return BatterySnapshot {
            percent: None,
            charging: None,
        };
    };

    let Ok(mut batteries) = manager.batteries() else {
        return BatterySnapshot {
            percent: None,
            charging: None,
        };
    };

    if let Some(Ok(battery)) = batteries.next() {
        let percent = battery.state_of_charge().get::<battery::units::ratio::percent>();
        let charging = matches!(
            battery.state(),
            battery::State::Charging | battery::State::Full
        );

        BatterySnapshot {
            percent: Some(percent as f32),
            charging: Some(charging),
        }
    } else {
        BatterySnapshot {
            percent: None,
            charging: None,
        }
    }
}
