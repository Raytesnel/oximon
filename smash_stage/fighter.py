import os
from pathlib import Path

import arcade

from smash_stage.attacks import Attack, HADUKAN, HADUKAN_BLITZ
from utils import ASSETS_PATH


class Character(arcade.Sprite):
    def __init__(self, asset_path, name:str):
        super().__init__(scale=0.5)
        self.animations = {
            "down": [arcade.load_texture(os.path.join(asset_path, f"player_{i}.png")) for i in range(0, 3)],
            "up": [arcade.load_texture(os.path.join(asset_path, f"player_{i}.png")) for i in range(12, 15)],
            "left": [arcade.load_texture(os.path.join(asset_path, f"player_{i}.png")) for i in range(4, 8)],
            "right": [arcade.load_texture(os.path.join(asset_path, f"player_{i}.png")) for i in range(8, 12)],
        }
        self.direction = "down"
        self.current_frame = 0
        self.frame_timer = 0
        self.frame_duration = 0.075
        self.texture = self.animations[self.direction][0]
        self.jump_count = 0
        self.max_jumps = 2
        self.lives = 100
        self.MOVE_SPEED = 2
        self.JUMP_SPEED = 4
        self.name = name
        self.is_hit = False
        self.hit_timer = 1/20
        self.hit_counter=0

        # Attack animation
        hadouken_path = Path(ASSETS_PATH) / "sprites/pokemon/Charmander/hadukan"
        self.hadouken_frames = [
            arcade.load_texture(hadouken_path / f"hadukan_{i}.png") for i in range(1, 11)
        ]

        # Define attack presets
        self.attacks = {
            "neutral": dict(damage=8, knockback=6, size=(20, 20), offset=(30, 0)),
            "up": dict(damage=10, knockback=10, size=(20, 30), offset=(0, 40)),
            "down": dict(damage=12, knockback=8, size=(20, 20), offset=(0, -40)),
            "left": dict(damage=9, knockback= 1, size=(30, 20), offset=(-40, 0)),
            "right": dict(damage=9, knockback=1, size=(30, 20), offset=(40, 0)),
            "tackle": dict(damage=10, knockback=3, size=(40, 20), offset=(30, 0)),
        }

    def update_animation(self, delta_time: float = 1 / 60):
        if self.change_x == 0 and self.change_y == 0:
            self.current_frame = 0
            self.texture = self.animations[self.direction][0]
            return

        if abs(self.change_x) > abs(self.change_y):
            self.direction = "right" if self.change_x > 0 else "left"

        self.frame_timer += delta_time
        if self.frame_timer > self.frame_duration:
            self.current_frame = (self.current_frame + 1) % len(self.animations[self.direction])
            self.texture = self.animations[self.direction][self.current_frame]
            self.frame_timer = 0

    def update(self, delta_time: float = 1 / 60):
        if self.is_hit:
            self.hit_counter += delta_time
            if self.hit_timer< self.hit_counter:
                self.is_hit = False
                self.hit_counter = 0

        if abs(self.change_x) > 0.1:
            self.change_x *= 0.85
        else:
            self.change_x = 0

    def perform_attack(self, name, direction="neutral"):
        data = self.attacks.get(name)
        if not data:
            return None

        offset = data["offset"]

        attack = HADUKAN_BLITZ

        if direction == "neutral":
            offset = (30 if self.direction == "right" else -30, 0)
            attack = HADUKAN
        if direction == "left":
            attack.facing = -1
        else:
            attack.facing = 1
        attack.new_attack()
        attack.owner = self
        attack.center_y = self.center_y + offset[1]
        attack.center_x = self.center_x + offset[0]
        return attack
