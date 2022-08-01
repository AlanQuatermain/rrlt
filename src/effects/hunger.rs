use crate::prelude::*;

pub fn well_fed(ecs: &mut SubWorld, target: Entity) {
    if let Ok(mut entry) = ecs.entry_mut(target) {
        if let Ok(hclock) = entry.get_component_mut::<HungerClock>() {
            hclock.state = HungerState::WellFed;
            hclock.duration = 20;
        }
    }
}
