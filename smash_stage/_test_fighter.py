import arcade

from smash_stage.attacks import SimpleMelee, FireBreath, BlueFireBreath
from smash_stage.fighter import (
    Character,
    KnockBackDamage,
    NoAttackSet,
    Attacks,
    CharacterAttack,
)
from smash_stage.stage import SmashStage
from utils import SCREEN_WIDTH, SCREEN_HEIGHT, SCREEN_TITLE
from utils import SMASH_MAP_PATH, ASSETS_PATH

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


class TestFighter(arcade.View):

    def __init__(
        self,
        chosen_monster: Character,
    ):
        super().__init__()
        self.camera = arcade.Camera2D()
        self.held_keys = set()
        self.attack_hitboxes = arcade.SpriteList()
        self.timer = 0
        self.character_1 = chosen_monster
        self.stage = SmashStage(SMASH_MAP_PATH)
        self.physics_engine_1 = arcade.PhysicsEnginePlatformer(
            self.character_1, walls=self.stage.platforms, gravity_constant=1
        )
        self.character_1.physics_engine = self.physics_engine_1
        self.physics_engine_1.enable_multi_jump(2)
        self.setup()

    def setup(self):
        self.player_list = arcade.SpriteList()
        self.player_list.extend([self.character_1])

        self.character_1.center_x = SCREEN_WIDTH // 2
        self.character_1.center_y = SCREEN_HEIGHT // 2

        # UI
        self.p1_lives_text = arcade.Text(
            "P1 Lives: 100", 20, self.window.height - 40, arcade.color.WHITE, 20
        )

    def on_show(self):
        pass  # everything handled in setup

    def draw_ui(self):
        self.p1_lives_text.draw()

    def on_draw(self):
        self.clear()
        self.camera.use()
        self.stage.on_draw()
        self.player_list.draw()
        self.attack_hitboxes.draw()
        self.draw_ui()

    def on_update(self, delta_time: float = 1 / 60):
        self.timer += delta_time
        self.character_1.update_jump(delta_time)
        self.character_1.handle_landing(self.stage.platforms)
        self.stage.update()
        self.player_list.update()
        self.player_list.update_animation(delta_time)
        if arcade.key.LEFT in self.held_keys:
            self.character_1.walk("left")
            if arcade.key.LCTRL in self.held_keys:
                self.character_1.run("left")

        elif arcade.key.RIGHT in self.held_keys:
            self.character_1.walk("right")
            if arcade.key.LCTRL in self.held_keys:
                self.character_1.run("right")
        elif not self.held_keys or self.held_keys == [arcade.key.LCTRL]:
            self.character_1.idle()
        else:
            self.character_1.change_x = 0

        # Attacks
        self.attack_hitboxes.update()
        for hitbox in self.attack_hitboxes:
            hitbox.update(delta_time)
            for target in self.player_list:
                target: Character = target
                if (
                    target is not hitbox.owner
                    and arcade.check_for_collision(hitbox, target)
                    and not hitbox.is_hit
                ):
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

        self.physics_engine_1.update()

    def scroll_to_player(self):
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
        # if key == arcade.key.Z:
        #     direction = "normal_neutral"
        #     if arcade.key.UP in self.held_keys:
        #         direction = "normal_up"
        #     elif arcade.key.DOWN in self.held_keys:
        #         direction = "normal_down"
        #     elif arcade.key.LEFT in self.held_keys:
        #         direction = "normal_left"
        #     elif arcade.key.RIGHT in self.held_keys:
        #         direction = "normal_right"
        #
        #     try:
        #         hitbox = self.character_1.perform_attack(direction)
        #     except ValueError:
        #         logger.debug("stil doing a attack")
        #     else:
        #         logger.debug("tacle!")
        #         self.attack_hitboxes.append(hitbox)
        #         self.character_1.animation_state = "attack_2"
        #         self.character_1.current_frame = 0

        if key == arcade.key.X:
            if arcade.key.UP in self.held_keys:
                try:
                    hitbox = self.character_1.attack(self.character_1.attacks.up)
                    self.attack_hitboxes.append(hitbox)
                except (NoAttackSet, AttributeError):
                    pass
            elif arcade.key.DOWN in self.held_keys:
                try:
                    hitbox = self.character_1.attack(self.character_1.attacks.down)
                    self.attack_hitboxes.append(hitbox)
                except (NoAttackSet, AttributeError):
                    pass
            elif arcade.key.LEFT in self.held_keys:
                try:
                    hitbox = self.character_1.attack(self.character_1.attacks.left)
                    self.attack_hitboxes.append(hitbox)
                except (NoAttackSet, AttributeError):
                    pass
            elif arcade.key.RIGHT in self.held_keys:
                try:
                    hitbox = self.character_1.attack(self.character_1.attacks.right)
                    self.attack_hitboxes.append(hitbox)
                except (NoAttackSet, AttributeError):
                    pass
            else:
                try:
                    hitbox = self.character_1.attack(self.character_1.attacks.base)
                    self.attack_hitboxes.append(hitbox)
                except (NoAttackSet, AttributeError):
                    pass
        elif key == arcade.key.SPACE:
            self.character_1.jump()

    def on_key_release(self, key, modifiers):
        self.held_keys.discard(key)
        if key in (arcade.key.LEFT, arcade.key.RIGHT):
            self.character_1.change_x = 0
        if key == arcade.key.SPACE:
            self.character_1.jump_held = False


if __name__ == "__main__":
    attacks = Attacks(
        up=CharacterAttack(
            attack=FireBreath,
            position=(40, -15),
            end_attack_frame=5,
            end_start_up_frame=3,
            animation="attack_heavy_1",
        ),
        down=CharacterAttack(
            attack=BlueFireBreath,
            position=(30, -15),
            end_attack_frame=5,
            end_start_up_frame=3,
            animation="attack_heavy_1",
        ),
        left=None,
        right=None,
        base=CharacterAttack(
            attack=SimpleMelee,
            position=(40, -15),
            animation="attack_2",
            end_attack_frame=5,
            end_start_up_frame=3,
        ),
    )
    window = arcade.Window(SCREEN_WIDTH, SCREEN_HEIGHT, SCREEN_TITLE, resizable=True)
    test_fighter = TestFighter(
        chosen_monster=Character(
            ASSETS_PATH / "sprites" / "pokemon" / "Lightning Mage", "mage"
        )
    )
    test_fighter.character_1.set_attacks(attacks)
    window.show_view(test_fighter)
    arcade.run()
