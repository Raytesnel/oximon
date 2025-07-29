import os

import arcade
from arcade.hitbox import HitBox


class Player(arcade.Sprite):
    def __init__(self, asset_path):
        super().__init__()
        self.animations = {
            "down": [arcade.load_texture(os.path.join(asset_path, f"player_{i}.png")) for i in range(0, 4)],
            "left": [arcade.load_texture(os.path.join(asset_path, f"player_{i}.png")) for i in range(4, 8)],
            "right": [arcade.load_texture(os.path.join(asset_path, f"player_{i}.png")) for i in range(8, 12)],
            "up": [arcade.load_texture(os.path.join(asset_path, f"player_{i}.png")) for i in range(12, 15)],
        }
        self.direction = "down"
        self.current_frame = 0
        self.frame_timer = 0
        self.frame_duration = 0.05
        self.texture = self.animations[self.direction][0]
        self._hit_box = HitBox([
            (-16, -30),
            (16, -30),
            (16, -4),
            (-16, -4),
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
