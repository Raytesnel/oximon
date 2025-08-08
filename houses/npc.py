import random

import arcade
from arcade import Texture


class NPC(arcade.Sprite):
    def __init__(self, sheet_path: str, scale: float = 1.0):
        super().__init__(scale=scale)

        sprite_sheet = arcade.load_spritesheet(sheet_path)
        textures = sprite_sheet.get_texture_grid(size=(16, 16), columns=4, count=28)
        self.animations = {
            "down": [textures[i] for i in range(0, 18, 4)],
            "up": [textures[i] for i in range(1, 18, 4)],
            "left": [textures[i] for i in range(2, 18, 4)],
            "right": [textures[i] for i in range(3, 18, 4)],
        }
        self.direction = "down"
        self.direction_timer = 0
        self.change_interval = 0.6
        self.current_frame = 0
        self.frame_timer = 0
        self.frame_duration = 0.1
        self.texture = self.animations[self.direction][0]

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
            self.current_frame = (self.current_frame + 1) % len(
                self.animations[self.direction]
            )
            self.texture = self.animations[self.direction][self.current_frame]
            self.frame_timer = 0

    def update(self, delta_time: float):
        self.direction_timer += delta_time
        self.update_animation(delta_time)

        if self.direction_timer >= self.change_interval:
            self.direction_timer = 0
            direction = random.choice(["up", "down", "left", "right", "idle"])
            speed = 1
            if direction == "up":
                self.change_y = speed
                self.change_x = 0
            elif direction == "down":
                self.change_y = -speed
                self.change_x = 0
            elif direction == "left":
                self.change_x = -speed
                self.change_y = 0
            elif direction == "right":
                self.change_x = speed
                self.change_y = 0
            else:
                self.change_x = 0
                self.change_y = 0

        self.center_x += self.change_x
        self.center_y += self.change_y

        self.overlay_visible = False
