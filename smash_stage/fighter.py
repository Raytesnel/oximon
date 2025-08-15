import os
from pathlib import Path
from typing import Optional

import arcade
from loguru import logger
from pydantic import BaseModel

from smash_stage.attacks import Attack, HADUKAN, HADUKAN_BLITZ
from utils import ASSETS_PATH

class KnockBackDamage(BaseModel):
    x_position:float
    y_position:float
    knockback:float
    damage:int

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
        self.stun_duration = 2/60
        self.stun_counter = 0.0
        self.is_stunned = False
        self.pending_knockback: Optional[KnockBackDamage] = None
        self.current_frame = 0
        self.frame_timer = 0
        self.frame_duration = 0.075
        self.texture = self.animations[self.direction][0]
        self.jump_count = 0
        self.max_jumps = 2
        self.lives = 10000
        self.MOVE_SPEED = 2
        self.JUMP_SPEED = 20
        self.name = name
        self.hit_timer = 1/10
        self.hit_counter=0
        self.invincible_frame=2/60
        self.hits_take:Optional[KnockBackDamage]=None

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
    def take_hit(self, knockback_data: KnockBackDamage):
        """Trigger stun + knockback sequence."""
        self.pending_knockback = knockback_data
        self.stun_counter = 0
        if not self.is_stunned:
            self.center_y += 20  # stop movement immediately
            self.lives = max(0, self.lives - self.pending_knockback.damage)

        self.is_stunned = True
        self.change_x = 0

    def update_animation(self, delta_time: float = 1 / 60):
        if self.hits_take:
            logger.debug("TODO: hit animation")
            return
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
        if self.is_stunned:
            self.stun_counter += delta_time
            if self.stun_counter >= self.stun_duration:
                self.is_stunned = False
                if self.pending_knockback:
                    self.change_x = self.pending_knockback.x_position * self.pending_knockback.knockback
                    self.change_y = self.pending_knockback.y_position * self.pending_knockback.knockback
                    self.pending_knockback = None
            else:
                self.change_y =0
                self.change_x =0
            return  # No movement/gravity during stun
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


# TODO: add recovery time
# TODO: while in recovery state, movement in reduced ( terug lopen terwilj je weg wordt geschoten)
# TODO: build up force, combo's builds up a dicrectional force. before unleashing it.
# TODO: attack hit flashy things.
