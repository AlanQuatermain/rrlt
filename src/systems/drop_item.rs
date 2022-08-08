use crate::prelude::*;

#[system(for_each)]
#[read_component(Player)]
#[read_component(Point)]
#[read_component(Item)]
#[read_component(Name)]
#[read_component(WantsToDrop)]
pub fn drop_item(
    entity: &Entity,
    want_drop: &WantsToDrop,
    ecs: &SubWorld,
    #[resource] dm: &MasterDungeonMap,
    commands: &mut CommandBuffer,
) {
    if let Ok(who) = ecs.entry_ref(want_drop.who) {
        if let Ok(pos) = who.get_component::<Point>() {
            commands.add_component(want_drop.what, pos.clone());
            commands.remove_component::<Carried>(want_drop.what);
            commands.add_component(want_drop.who, EquipmentChanged);
        }

        let item_name = get_item_display_name(ecs, want_drop.what, dm);
        if who.get_component::<Player>().is_ok() {
            crate::gamelog::Logger::new()
                .append("You dropped the")
                .item_name(&item_name)
                .log();
        }
    }
    commands.remove(*entity);
}
