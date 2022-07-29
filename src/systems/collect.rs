use crate::prelude::*;

#[system(for_each)]
#[read_component(WantsToCollect)]
#[read_component(Name)]
#[read_component(Player)]
#[read_component(MagicItem)]
#[read_component(ObfuscatedName)]
pub fn collect(
    entity: &Entity,
    wants_collect: &WantsToCollect,
    ecs: &mut SubWorld,
    commands: &mut CommandBuffer,
    #[resource] gamelog: &mut Gamelog,
    #[resource] dm: &MasterDungeonMap,
) {
    commands.remove_component::<Point>(wants_collect.what);
    commands.add_component(wants_collect.what, Carried(wants_collect.who));
    commands.add_component(wants_collect.who, EquipmentChanged);

    let who = if let Ok(name) = ecs
        .entry_ref(wants_collect.who)
        .unwrap()
        .get_component::<Name>()
    {
        name.0.clone()
    } else if ecs
        .entry_ref(wants_collect.who)
        .unwrap()
        .get_component::<Player>()
        .is_ok()
    {
        "Player".to_string()
    } else {
        "Someone".to_string()
    };
    let what = get_item_display_name(ecs, wants_collect.what, dm);
    gamelog.entries.push(format!("{} picked up {}", who, what));

    commands.remove(*entity);
}
