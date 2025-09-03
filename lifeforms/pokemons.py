import random
from pathlib import Path

import arcade
from arcade.hitbox import HitBox
from pydantic import BaseModel


class OverWorldMonsterSprites(BaseModel):
    up: Path
    down: Path
    left: Path
    right: Path
    dead: Path
    banner: Path


class WildPokemon(arcade.Sprite):

    def __init__(
        self,
        image_path: OverWorldMonsterSprites,
        name: str,
        scale=2.0,
    ):
        self.alive = True
        self.sprite_file_location = image_path.banner
        self.name = name
        self.animations = {
            "down": arcade.load_spritesheet(image_path.down).get_texture_grid(
                (32, 32), 3, 3
            ),
            "up": arcade.load_spritesheet(image_path.up).get_texture_grid(
                (32, 32), 3, 3
            ),
            "left": arcade.load_spritesheet(image_path.left).get_texture_grid(
                (32, 32), 3, 3
            ),
            "right": arcade.load_spritesheet(image_path.right).get_texture_grid(
                (32, 32), 3, 3
            ),
            "dead": arcade.load_spritesheet(image_path.dead).get_texture_grid(
                (32, 32), 3, 3
            ),
        }
        super().__init__(self.animations["down"][0], scale)
        self.direction = "down"
        self.texture = self.animations[self.direction][0]
        self.direction_timer = 0
        self.change_interval = 0.6
        self.current_frame = 0
        self.frame_timer = 0
        self.frame_duration = 0.05
        self._hit_box = HitBox([
            (-4, -20),
            (4, -20),
            (4, -4),
            (-4, -4),
        ])

    def update_animation(self, delta_time: float = 1 / 60):
        if self.change_x == 0 and self.change_y == 0:
            self.current_frame = 0
            self.texture = self.animations[self.direction][0]
            return

        if abs(self.change_x) > abs(self.change_y):
            self.direction = "right" if self.change_x > 0 else "left"
        else:
            self.direction = "up" if self.change_y > 0 else "down"

        self.frame_timer += delta_time
        if self.frame_timer > self.frame_duration:
            self.current_frame = (self.current_frame + 1) % len(self.animations[self.direction])
            self.texture = self.animations[self.direction][self.current_frame]
            self.frame_timer = 0

    def update(self, delta_time: float):
        if not self.alive:
            self.direction = "dead"
            self.update_animation(delta_time)
            return
        self.direction_timer += delta_time
        if self.direction_timer >= self.change_interval:
            self.direction_timer = 0
            direction = random.choice(['up', 'down', 'left', 'right', 'idle'])
            speed = 1
            if direction == 'up':
                self.change_y = speed
                self.change_x = 0
            elif direction == 'down':
                self.change_y = -speed
                self.change_x = 0
            elif direction == 'left':
                self.change_x = -speed
                self.change_y = 0
            elif direction == 'right':
                self.change_x = speed
                self.change_y = 0
            else:
                self.change_x = 0
                self.change_y = 0

        self.center_x += self.change_x
        self.center_y += self.change_y

        self.overlay_visible = False

        # make somthing to make a movement trough grass
