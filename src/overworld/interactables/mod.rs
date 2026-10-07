mod block;
mod lamp;
mod monster;
mod npc;
mod sign;

pub use block::*;
pub use lamp::*;
pub use monster::*;
pub use sign::*;

#[cfg(test)]
pub(crate) mod test_util {
    use bevy::prelude::*;
    use bevy::time::TimeUpdateStrategy;

    pub fn make_app_with_time(step_seconds: f32) -> App {
        let mut app = App::new();
        app.insert_resource(TimeUpdateStrategy::ManualDuration(
            std::time::Duration::from_secs_f32(step_seconds),
        ));
        app.add_plugins(MinimalPlugins);
        app
    }
}
