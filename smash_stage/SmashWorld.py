import math

import arcade
from smash_stage.fighter import Character
from smash_stage.stage import SmashStage
from utils import PLAYER_PATH, SMASH_MAP_PATH


class SmashWorld(arcade.View):
    def __init__(self, overworld_view):
        super().__init__()
        self.overworld_view = overworld_view
        self.camera = arcade.Camera2D()
        self.held_keys = set()
        self.attack_hitboxes = arcade.SpriteList()
        self.timer = 0
        self.character_1 = Character(PLAYER_PATH,"hero")
        self.character_2 = Character(PLAYER_PATH,"monster")
        self.stage = SmashStage(
            SMASH_MAP_PATH
        )
        self.physics_engine_1 = arcade.PhysicsEnginePlatformer(self.character_1, platforms=self.stage.platforms,gravity_constant=0.3)
        self.physics_engine_2 = arcade.PhysicsEnginePlatformer(self.character_2, platforms=self.stage.platforms,gravity_constant=0.3)
        self.physics_engine_1.enable_multi_jump(2)
        self.setup()

    def setup(self):
        self.player_list = arcade.SpriteList()
        self.player_list.extend([self.character_1, self.character_2])

        self.character_1.center_x = self.stage.spawn_points["player1"].shape[0]
        self.character_1.center_y = self.stage.spawn_points["player1"].shape[1]
        self.character_2.center_x = self.stage.spawn_points["player2"].shape[0]
        self.character_2.center_y = self.stage.spawn_points["player2"].shape[1]

        # UI
        self.p1_lives_text = arcade.Text("P1 Lives: 100", 20, self.window.height - 40, arcade.color.WHITE, 20)
        self.p2_lives_text = arcade.Text("P2 Lives: 100", self.window.width - 160, self.window.height - 40, arcade.color.WHITE, 20)

    def on_show(self):
        pass  # everything handled in setup

    def draw_ui(self):
        self.p1_lives_text.draw()
        self.p2_lives_text.draw()

    def on_draw(self):
        self.clear()
        self.camera.use()
        self.stage.on_draw()
        self.attack_hitboxes.draw()
        self.player_list.draw()
        self.draw_ui()

    def on_update(self, delta_time: float = 1 / 60):
        self.timer += delta_time
        self.stage.update()
        self.player_list.update()
        self.player_list.update_animation(delta_time)

        if self.physics_engine_1.can_jump():
            self.character_1.jump_count = 0

        # Movement
        move_speed = self.character_1.MOVE_SPEED * (4 if arcade.key.LCTRL in self.held_keys else 1)
        if arcade.key.LEFT in self.held_keys:
            self.character_1.change_x = -move_speed
        elif arcade.key.RIGHT in self.held_keys:
            self.character_1.change_x = move_speed
        else:
            self.character_1.change_x = 0

        # Attacks
        for hitbox in self.attack_hitboxes:
            hitbox.update(delta_time)
            for target in self.player_list:
                if target is not hitbox.owner and arcade.check_for_collision(hitbox, target) and not target.is_hit:
                    target.lives = max(0, target.lives - hitbox.damage)

                    dir_dx = hitbox.knockback_direction.x_direction
                    dir_dy = hitbox.knockback_direction.y_direction
                    dir_dx *= hitbox.facing
                    expl_dx = target.center_x - hitbox.center_x
                    expl_dy = target.center_y - hitbox.center_y
                    length = math.hypot(expl_dx, expl_dy)
                    if length != 0:
                        expl_dx /= length
                        expl_dy /= length
                    else:
                        expl_dx, expl_dy = 0, 0

                    target.change_x += dir_dx * hitbox.knockback_strength + expl_dx * hitbox.explosion_knock_back
                    target.change_y += dir_dy * hitbox.knockback_strength + expl_dy * hitbox.explosion_knock_back

                    target.is_hit = True

        self.p1_lives_text.text = f"P1 Lives: {self.character_1.lives}"
        self.p2_lives_text.text = f"P2 Lives: {self.character_2.lives}"

        self.stage.check_death_zones(self.player_list)

        self.physics_engine_1.update()
        self.physics_engine_2.update()
        if self.character_1.lives <= 0:
            print(f"{self.character_2.name} wins!")
            self.window.show_view(self.overworld_view)
        elif self.character_2.lives <= 0:
            print(f"{self.character_1.name} wins!")
            self.window.show_view(self.overworld_view)

    def on_key_press(self, key, modifiers):
        self.held_keys.add(key)
        if key == arcade.key.Z:
            hitbox = self.character_1.perform_attack("tackle", "neutral")
            if hitbox and hitbox not in self.attack_hitboxes:
                self.attack_hitboxes.append(hitbox)
        elif key == arcade.key.X:
            direction = "neutral"
            if arcade.key.UP in self.held_keys:
                direction = "up"
            elif arcade.key.DOWN in self.held_keys:
                direction = "down"
            elif arcade.key.LEFT in self.held_keys:
                direction = "left"
            elif arcade.key.RIGHT in self.held_keys:
                direction = "right"

            hitbox = self.character_1.perform_attack(direction, direction)
            if hitbox and hitbox not in self.attack_hitboxes:
                self.attack_hitboxes.append(hitbox)

        elif key == arcade.key.SPACE:
            if self.physics_engine_1.can_jump():
                self.physics_engine_1.jump(self.character_1.JUMP_SPEED)

    def on_key_release(self, key, modifiers):
        self.held_keys.discard(key)
        if key in (arcade.key.LEFT, arcade.key.RIGHT):
            self.character_1.change_x = 0
