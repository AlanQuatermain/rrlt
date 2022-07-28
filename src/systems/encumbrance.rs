use crate::prelude::*;

#[system(for_each)]
#[read_component(EquipmentChanged)]
#[read_component(Carried)]
#[read_component(Equipped)]
#[write_component(Pools)]
#[read_component(Item)]
#[read_component(Attributes)]
#[filter(component::<EquipmentChanged>())]
pub fn encumbrance(
    ecs: &SubWorld,
    entity: &Entity,
    stats: &mut Pools,
    attrs: &Attributes,
    #[resource] gamelog: &mut Gamelog,
    commands: &mut CommandBuffer,
) {
    commands.remove_component::<EquipmentChanged>(*entity);

    // Total up the equipped items
    let equipped_totals: (f32, f32) = <(&Item, &Equipped)>::query()
        .iter(ecs)
        .filter(|(_, e)| e.owner == *entity)
        .map(|(i, _)| (i.weight_lbs, i.initiative_penalty))
        .reduce(|total, value| (total.0 + value.0, total.1 + value.1))
        .unwrap_or((0.0, 0.0));

    // Total up the carried items
    let carried_totals: (f32, f32) = <(&Item, &Carried)>::query()
        .iter(ecs)
        .filter(|(_, e)| e.0 == *entity)
        .map(|(i, _)| (i.weight_lbs, i.initiative_penalty))
        .reduce(|total, value| (total.0 + value.0, total.1 + value.1))
        .unwrap_or((0.0, 0.0));

    stats.total_weight = equipped_totals.0 + carried_totals.0;
    stats.total_initiative_penalty = equipped_totals.1 + carried_totals.1;

    let capacity = (attrs.might.base + attrs.might.modifiers) * 15;
    if stats.total_weight > capacity as f32 {
        // Overburdened
        stats.total_initiative_penalty += 4.0;
        if ecs
            .entry_ref(*entity)
            .unwrap()
            .get_component::<Player>()
            .is_ok()
        {
            gamelog
                .entries
                .push("You are overburdened, and suffering from fatigue.".to_string());
        }
    }
}
