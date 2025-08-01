import os
from pathlib import Path

import arcade
from arcade import load_tilemap

ASSETS_PATH = os.path.join(os.path.dirname(__file__), "assets")


class SmashStageOnlyView(arcade.View):
    def __init__(self, overworld_view):
        super().__init__()
        self.platforms = arcade.SpriteList()
        self.camera = arcade.Camera2D()
        self.overworld_view = overworld_view
        self.timer = 0
        self.max_duration = 2.0
        self.physics_engine = None
        self.setup()
        self.on_show()
        self.held_keys = set()
        self.attack_hitboxes = arcade.SpriteList()
        self.p1_lives_text = arcade.Text(
            "P1 Lives: 100",
            20,
            self.window.height - 40,
            arcade.color.WHITE,
            20,
        )
        self.p2_lives_text = arcade.Text(
            "P2 Lives: 100",
            self.window.width - 160,
            self.window.height - 40,
            arcade.color.WHITE,
            20,
        )

    def setup(self) -> None:
        asset_path = os.path.join(ASSETS_PATH, "sprites/player")
        self.player_list = arcade.SpriteList()
        self.character_1 = Character(asset_path)
        self.character_1.center_x = 200
        self.character_1.center_y = 200
        self.character_2 = Character(asset_path)
        self.character_2.center_x = 600
        self.character_2.center_y = 300
        self.player_list.append(self.character_1)
        self.player_list.append(self.character_2)

    def on_show(self):
        self.tile_map = load_tilemap(
            os.path.join(ASSETS_PATH, "map/smash.tmx"),
            scaling=1.0,
            use_spatial_hash=True,
        )
        self.scene = arcade.Scene.from_tilemap(self.tile_map)
        self.death_fields = self.tile_map.object_lists["death"]
        self.platforms = self.scene["platforms"]
        self.death_field = [field for field in self.death_fields if field.name == "Alive"][0]

        self.physics_engine = arcade.PhysicsEnginePlatformer(
            self.character_1,
            platforms=self.platforms,
        )
        self.physics_engine.enable_multi_jump(2)

        self.physics_engine_2 = arcade.PhysicsEnginePlatformer(
            self.character_2,
            platforms=self.platforms,
        )

    def draw_ui(self):
        self.p1_lives_text.draw()
        self.p2_lives_text.draw()

    def on_draw(self):
        self.clear()
        self.camera.use()
        self.platforms.draw()
        self.player_list.draw()
        self.attack_hitboxes.draw()
        self.draw_ui()

    def on_update(self, delta_time):
        self.timer += delta_time
        self.physics_engine.update()
        self.physics_engine_2.update()
        self.player_list.update()
        self.player_list.update_animation(delta_time)
        if self.physics_engine.can_jump():
            self.character_1.jump_count = 0
        move_speed = self.character_1.MOVE_SPEED
        if arcade.key.LCTRL in self.held_keys:
            move_speed *= 4

        if arcade.key.LEFT in self.held_keys:
            self.character_1.change_x = -move_speed
        elif arcade.key.RIGHT in self.held_keys:
            self.character_1.change_x = move_speed
        else:
            self.character_1.change_x = 0
        hitbox = None
        for hitbox in self.attack_hitboxes:
            hitbox.life_timer -= delta_time
            if hitbox.life_timer <= 0:
                hitbox.remove_from_sprite_lists()
            if hasattr(hitbox, "textures"):
                hitbox.animation_timer += delta_time
                if hitbox.animation_timer > hitbox.frame_duration:
                    hitbox.current_frame = (hitbox.current_frame + 1) % len(
                        hitbox.textures
                    )
                    hitbox.texture = hitbox.textures[hitbox.current_frame]
                    hitbox.animation_timer = 0
        if hitbox is not None:
            for target in self.player_list:
                if target is not hitbox.owner and arcade.check_for_collision(
                    hitbox, target
                ):
                    target.lives = max(0, target.lives - hitbox.damage)

                    target.change_x += hitbox.knockback[0]
                    target.change_y += hitbox.knockback[1]
                    # hitbox.remove_from_sprite_lists()
        self.p1_lives_text.text = f"P1 Lives: {self.character_1.lives}"
        self.p2_lives_text.text = f"P2 Lives: {self.character_2.lives}"

        left_top, right_top, right_bottom, left_bottom = self.death_field.shape
        left = left_top[0]
        right = right_top[0]
        bottom = right_bottom[1]
        top = right_top[1]

        for character in self.player_list:
            x, y = character.center_x, character.center_y
            if not (left <= x <= right and bottom <= y <= top):
                character.lives = 0
                print(
                    f"{'Player 1' if character == self.character_1 else 'Player 2'} viel uit het speelveld!"
                )

        if self.character_1.lives <= 0:
            print("Player 2 wins!")
            self.window.show_view(self.overworld_view)

        elif self.character_2.lives <= 0:
            print("Player 1 wins!")
            self.window.show_view(self.overworld_view)

    def on_key_press(self, key, modifiers):
        self.held_keys.add(key)
        if key == arcade.key.Z:
            # A-knop = tackle (neutrale melee aanval)
            hitbox = self.character_1.create_attack_hitbox(
                direction="neutral", damage=10, knockback=(5, 3), width=40, height=20
            )
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

            # Verschillende effecten per richting
            attack_data = {
                "neutral": dict(damage=8, knockback=(0, 6), width=20, height=20),
                "up": dict(damage=10, knockback=(0, 10), width=20, height=30),
                "down": dict(damage=12, knockback=(0, -8), width=20, height=20),
                "left": dict(damage=9, knockback=(-7, 1), width=30, height=20),
                "right": dict(damage=9, knockback=(7, 1), width=30, height=20),
            }

            props = attack_data[direction]
            hitbox = self.character_1.create_attack_hitbox(direction=direction, **props)
            self.attack_hitboxes.append(hitbox)

        if key == arcade.key.SPACE:
            if self.physics_engine.can_jump():
                self.physics_engine.jump(self.character_1.JUMP_SPEED)

    def on_key_release(self, key, modifiers):
        self.held_keys.discard(key)

        if key in (arcade.key.LEFT, arcade.key.RIGHT):
            self.character_1.change_x = 0


class Character(arcade.Sprite):
    def __init__(self, asset_path):
        super().__init__(scale=0.5)
        self.animations = {
            "down": [
                arcade.load_texture(os.path.join(asset_path, f"player_{i}.png"))
                for i in range(0, 3)
            ],
            "up": [
                arcade.load_texture(os.path.join(asset_path, f"player_{i}.png"))
                for i in range(12, 15)
            ],
            "left": [
                arcade.load_texture(os.path.join(asset_path, f"player_{i}.png"))
                for i in range(4, 8)
            ],
            "right": [
                arcade.load_texture(os.path.join(asset_path, f"player_{i}.png"))
                for i in range(8, 12)
            ],
        }
        self.direction = "down"
        self.current_frame = 0
        self.frame_timer = 0
        self.frame_duration = 0.075
        self.texture = self.animations[self.direction][0]
        self.MOVE_SPEED = 2
        self.JUMP_SPEED = 10
        self.jump_count = 0
        self.max_jumps = 2
        self.lives = 100

    def update_animation(self, delta_time: float = 1 / 60):
        if self.change_x == 0 and self.change_y == 0:
            self.current_frame = 0
            self.texture = self.animations[self.direction][0]
            return

        if abs(self.change_x) > abs(self.change_y):
            if self.change_x > 0:
                self.direction = "right"
            elif self.change_x < 0:
                self.direction = "left"

        self.frame_timer += delta_time
        if self.frame_timer > self.frame_duration:
            self.current_frame = (self.current_frame + 1) % len(
                self.animations[self.direction]
            )
            self.texture = self.animations[self.direction][self.current_frame]
            self.frame_timer = 0

    def update(self, delta_time: float = 1 / 60):
        if abs(self.change_x) > 0.1:
            self.change_x *= 0.85  # Slide to a stop
        else:
            self.change_x = 0

    def create_attack_hitbox(
        self, direction="neutral", damage=10, knockback=(0, 5), width=20, height=20, time_attack:int=1
    ):
        HADOUKEN_FRAMES = [
            arcade.load_texture(
                Path(ASSETS_PATH)
                / f"sprites/pokemon/Charmander/hadukan/hadukan_{i}.png"
            )
            for i in range(1, 11)
        ]
        hitbox = arcade.Sprite()
        hitbox.textures = HADOUKEN_FRAMES
        hitbox.texture = HADOUKEN_FRAMES[0]
        hitbox.width = width
        hitbox.height = height
        hitbox.animation_timer = 0
        hitbox.current_frame = 0
        hitbox.frame_duration = 0.05  # Seconds per frame

        offset_x, offset_y = 0, 0
        if direction == "up":
            offset_y = 40
        elif direction == "down":
            offset_y = -40
        elif direction == "left":
            offset_x = -40
        elif direction == "right":
            offset_x = 40
        elif direction == "neutral":
            offset_x = 30 if self.direction == "right" else -30

        hitbox.center_x = self.center_x + offset_x
        hitbox.center_y = self.center_y + offset_y

        hitbox.life_timer = 0.5
        hitbox.owner = self
        hitbox.damage = damage
        hitbox.knockback = knockback  # (x, y)
        return hitbox


class SmashPhysicsEngine(arcade.PhysicsEnginePlatformer):
    def is_on_platform(self, platform):
        """Check if player is above the platform and falling."""
        return (
            self.player_sprite.bottom >= platform.top
            and self.player_sprite.change_y <= 0
        )

    def _get_new_position(self):
        """
        Override this to ignore collisions with 'soft' platforms unless falling onto them.
        """
        # Only collide with platforms if you're above them and falling
        adjusted_platforms = [
            platform for platform in self.platforms if self.is_on_platform(platform)
        ]
        self.platforms = adjusted_platforms
        return super()._get_new_position()
