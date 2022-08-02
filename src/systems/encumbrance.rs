use crate::prelude::*;

#[system(for_each)]
#[read_component(EquipmentChanged)]
#[read_component(Carried)]
#[read_component(Equipped)]
#[write_component(Pools)]
#[read_component(Item)]
#[write_component(Attributes)]
#[read_component(AttributeBonus)]
#[read_component(StatusEffect)]
#[filter(component::<EquipmentChanged>())]
pub fn encumbrance(
    ecs: &SubWorld,
    entity: &Entity,
    stats: &mut Pools,
    attrs: &mut Attributes,
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

    // Gather the combined attributes of all equipped attr-bonus items
    let bonus_totals: AttributeBonus = <(&Equipped, &AttributeBonus)>::query()
        .iter(ecs)
        .filter(|(e, _)| e.owner == *entity)
        .map(|(_, b)| *b)
        .sum();

    // Total up the carried items
    let carried_totals: (f32, f32) = <(&Item, &Carried)>::query()
        .filter(!component::<Equipped>())
        .iter(ecs)
        .filter(|(_, e)| e.0 == *entity)
        .map(|(i, _)| (i.weight_lbs, i.initiative_penalty))
        .reduce(|total, value| (total.0 + value.0, total.1 + value.1))
        .unwrap_or((0.0, 0.0));

    // Total up status effect modifiers
    let effect_totals: AttributeBonus = <(&StatusEffect, &AttributeBonus)>::query()
        .iter(ecs)
        .filter(|(e, _)| e.target == *entity)
        .map(|(_, b)| *b)
        .sum();

    stats.total_weight = equipped_totals.0 + carried_totals.0;
    stats.total_initiative_penalty = equipped_totals.1 + carried_totals.1;

    *attrs = attrs.clone() + (bonus_totals + effect_totals);

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
