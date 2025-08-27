import arcade
from loguru import logger

from lifeforms.pokemons import WildPokemon
from smash_stage.fighter import Character, KnockBackDamage, EnemyAI
from smash_stage.stage import SmashStage
from utils import SMASH_MAP_PATH, ASSETS_PATH
from utils import SCREEN_WIDTH, SCREEN_HEIGHT, SCREEN_TITLE

VIEWPORT_MARGIN = 200

HORIZONTAL_BOUNDARY = SCREEN_WIDTH / 2.0 - VIEWPORT_MARGIN

VERTICAL_BOUNDARY = SCREEN_HEIGHT / 2.0 - VIEWPORT_MARGIN

# If the player moves further than this boundary away from the camera we use a

# constraint to move the camera
CAMERA_SPEED = 0.1
CAMERA_BOUNDARY = arcade.LRBT(

    -HORIZONTAL_BOUNDARY,

      HORIZONTAL_BOUNDARY,

      -VERTICAL_BOUNDARY,

      VERTICAL_BOUNDARY,

)

class SmashWorld(arcade.View):
    def __init__(self, overworld_view,wild_monster:WildPokemon):
        super().__init__()
        self.wild_monster = wild_monster
        self.overworld_view = overworld_view
        self.camera = arcade.Camera2D()
        self.held_keys = set()
        self.attack_hitboxes = arcade.SpriteList()
        self.timer = 0
        self.character_1 = Character(
            ASSETS_PATH / "sprites" / "pokemon" / "Lightning Mage", "mage"
        )
        self.character_2 = Character(
            ASSETS_PATH / "sprites" / "pokemon" / "Lightning Mage", "mage"
        )
        self.stage = SmashStage(SMASH_MAP_PATH)
        self.physics_engine_1 = arcade.PhysicsEnginePlatformer(
            self.character_1, platforms=self.stage.platforms, gravity_constant=1
        )
        self.physics_engine_2 = arcade.PhysicsEnginePlatformer(
            self.character_2, platforms=self.stage.platforms, gravity_constant=1
        )
        self.physics_engine_1.enable_multi_jump(1)
        self.enemy_ai = EnemyAI(self.character_2, self.character_1, self.stage)
        self.setup()

    def setup(self):
        self.player_list = arcade.SpriteList()
        self.player_list.extend([self.character_1, self.character_2])

        self.character_1.center_x = self.stage.spawn_points["player1"].shape[0]
        self.character_1.center_y = self.stage.spawn_points["player1"].shape[1] + 128
        self.character_2.center_x = self.stage.spawn_points["player2"].shape[0]
        self.character_2.center_y = self.stage.spawn_points["player2"].shape[1] + 128

        # UI
        self.p1_lives_text = arcade.Text(
            "P1 Lives: 100", 20, self.window.height - 40, arcade.color.WHITE, 20
        )
        self.p2_lives_text = arcade.Text(
            "P2 Lives: 100",
            self.window.width - 160,
            self.window.height - 40,
            arcade.color.WHITE,
            20,
        )

    def on_show(self):
        pass  # everything handled in setup

    def draw_ui(self):
        self.p1_lives_text.draw()
        self.p2_lives_text.draw()

    def on_draw(self):
        self.clear()
        self.camera.use()
        self.stage.on_draw()
        self.player_list.draw()
        self.attack_hitboxes.draw()
        self.draw_ui()

    def on_update(self, delta_time: float = 1 / 60):
        self.timer += delta_time
        self.stage.update()
        self.player_list.update()
        self.player_list.update_animation(delta_time)

        if self.physics_engine_1.can_jump():
            self.character_1.jump_count = 0

        # Movement
        move_speed = self.character_1.MOVE_SPEED

        if arcade.key.LEFT in self.held_keys:
            self.character_1.change_x = -move_speed
            self.character_1.reverse = True
            self.character_1.animation_state = "walk"
            if arcade.key.LCTRL in self.held_keys:
                self.character_1.change_x = -move_speed * 2
                self.character_1.reverse = True
                self.character_1.animation_state = "run"

        elif arcade.key.RIGHT in self.held_keys:
            self.character_1.change_x = move_speed
            self.character_1.reverse = False
            self.character_1.animation_state = "walk"
            if arcade.key.LCTRL in self.held_keys:
                self.character_1.change_x = move_speed * 2
                self.character_1.reverse = False
                self.character_1.animation_state = "run"
        elif not self.held_keys or self.held_keys == [arcade.key.LCTRL]:
            self.character_1.animation_state = "idle"
        # TODO: move all animation_state setters to fighter class.
        else:
            self.character_1.change_x = 0

        # Attacks
        self.attack_hitboxes.update()
        for hitbox in self.attack_hitboxes:
            hitbox.update(delta_time)
            for target in self.player_list:
                target: Character = target
                if target is not hitbox.owner and arcade.check_for_collision(
                    hitbox, target
                ) and not hitbox.is_hit:
                    target.take_hit(
                        KnockBackDamage(
                            x_position=hitbox.knockback_direction.x_direction,
                            y_position=hitbox.knockback_direction.y_direction,
                            knockback=hitbox.knockback_strength,
                            damage=hitbox.damage,
                        )
                    )
                    hitbox.is_hit = True

        self.p1_lives_text.text = f"P1 Lives: {self.character_1.lives}"
        self.p2_lives_text.text = f"P2 Lives: {self.character_2.lives}"

        if self.stage.check_death_zones(self.player_list):
            if self.character_1.lives <= 0:
                self.overworld_view.player_lost()
            elif self.character_2.lives <= 0:
                self.overworld_view.enemy_out_of_bounds(self.wild_monster)

        self.physics_engine_1.update()
        self.physics_engine_2.update()
        if self.character_1.lives <= 0:
            self.overworld_view.player_lost()
        elif self.character_2.lives <= 0:
            self.overworld_view.enemy_defeated(self.wild_monster)
        self.enemy_ai.update(delta_time)
        self.scroll_to_player()

    def scroll_to_player(self):

        # --- Manage Scrolling ---

        new_position = arcade.camera.grips.constrain_boundary_xy(
            self.camera.view_data, CAMERA_BOUNDARY, self.character_1.position
        )

        self.camera.position = arcade.math.lerp_2d(
            self.camera.position,
            (new_position[0], new_position[1]),
            CAMERA_SPEED,
        )

    def on_key_press(self, key, modifiers):
        self.held_keys.add(key)
        if key == arcade.key.Z:
            direction = "normal_neutral"
            if arcade.key.UP in self.held_keys:
                direction = "normal_up"
            elif arcade.key.DOWN in self.held_keys:
                direction = "normal_down"
            elif arcade.key.LEFT in self.held_keys:
                direction = "normal_left"
            elif arcade.key.RIGHT in self.held_keys:
                direction = "normal_right"

            try:
                hitbox = self.character_1.perform_attack(direction)
            except ValueError:
                logger.debug("stil doing a attack")
            else:
                logger.debug("tacle!")
                self.attack_hitboxes.append(hitbox)
                self.character_1.animation_state = "attack_2"
                self.character_1.current_frame = 0

        elif key == arcade.key.X:
            direction = "special_neutral"
            if arcade.key.UP in self.held_keys:
                direction = "special_up"
            elif arcade.key.DOWN in self.held_keys:
                direction = "special_down"
            elif arcade.key.LEFT in self.held_keys:
                direction = "special_left"
            elif arcade.key.RIGHT in self.held_keys:
                direction = "special_right"

            try:
                hitbox = self.character_1.perform_attack(direction)
            except ValueError:
                logger.debug("stil doing a attackx")
            else:
                logger.debug("kamehama")
                self.attack_hitboxes.append(hitbox)
                self.character_1.animation_state = "attack_heavy_1"
                self.character_1.current_frame = 0

        elif key == arcade.key.SPACE:
            if self.physics_engine_1.can_jump():
                self.character_1.current_frame = 0
                self.character_1.animation_state = "jump"

                self.physics_engine_1.jump(self.character_1.JUMP_SPEED)

    def on_key_release(self, key, modifiers):
        self.held_keys.discard(key)
        if key in (arcade.key.LEFT, arcade.key.RIGHT):
            self.character_1.change_x = 0
if __name__ == "__main__":
    window = arcade.Window(SCREEN_WIDTH, SCREEN_HEIGHT, SCREEN_TITLE)
    window.show_view(SmashWorld(None))
    arcade.run()
