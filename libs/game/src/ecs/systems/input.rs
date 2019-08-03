use amethyst::{
    core::{math::Point2, Transform},
    ecs::{Join, ReadExpect, ReadStorage, System, WriteStorage},
    input::{InputHandler, StringBindings},
    renderer::Camera,
    window::ScreenDimensions,
    winit::MouseButton,
};

use ha_core::math::Vector2;

use crate::{
    actions::player::{PlayerCastAction, PlayerLookAction, PlayerWalkAction},
    ecs::{
        components::{PlayerActions, WorldPosition},
        resources::GameEngineState,
    },
};

#[derive(Default)]
pub struct InputSystem;

impl<'s> System<'s> for InputSystem {
    type SystemData = (
        ReadExpect<'s, InputHandler<StringBindings>>,
        ReadExpect<'s, ScreenDimensions>,
        ReadExpect<'s, GameEngineState>,
        ReadStorage<'s, Camera>,
        ReadStorage<'s, Transform>,
        ReadStorage<'s, WorldPosition>,
        WriteStorage<'s, PlayerActions>,
    );

    fn run(
        &mut self,
        (
            input,
            screen_dimensions,
            game_state,
            cameras,
            transforms,
            world_positions,
            mut player_actions,
        ): Self::SystemData,
    ) {
        if let GameEngineState::Playing = *game_state {
        } else {
            return;
        }

        let (player_actions, player_position) = (&mut player_actions, &world_positions)
            .join()
            .next()
            .unwrap();
        self.process_mouse_input(
            &screen_dimensions,
            &*input,
            &cameras,
            &transforms,
            &mut *player_actions,
            **player_position,
        );
        self.process_keyboard_input(&*input, &mut *player_actions);
    }
}

impl InputSystem {
    fn process_mouse_input(
        &mut self,
        screen_dimensions: &ScreenDimensions,
        input: &InputHandler<StringBindings>,
        cameras: &ReadStorage<'_, Camera>,
        transforms: &ReadStorage<'_, Transform>,
        player_actions: &mut PlayerActions,
        player_position: Vector2,
    ) {
        let mouse_world_position = {
            let mouse_position = input.mouse_position();
            if mouse_position.is_none() {
                return;
            }
            let (mouse_x, mouse_y) = mouse_position.unwrap();

            let components = (cameras, transforms).join().next();
            if components.is_none() {
                return;
            }
            let (camera, camera_transform) = components.unwrap();

            let position = camera.projection().screen_to_world(
                Point2::new(mouse_x as f32, mouse_y as f32),
                screen_dimensions.diagonal(),
                camera_transform,
            );
            Vector2::new(position.x, position.y)
        };

        player_actions.look_actions.push(PlayerLookAction {
            direction: mouse_world_position - player_position,
        });

        if input.mouse_button_is_down(MouseButton::Left) {
            player_actions.cast_actions.push(PlayerCastAction {
                cast_position: player_position,
                target_position: mouse_world_position,
            });
        }
    }

    fn process_keyboard_input(
        &mut self,
        input: &InputHandler<StringBindings>,
        player_actions: &mut PlayerActions,
    ) {
        if let (Some(x), Some(y)) = (input.axis_value("horizontal"), input.axis_value("vertical")) {
            player_actions.walk_actions.push(PlayerWalkAction {
                direction: Vector2::new(x, y),
            });
        }
    }
}