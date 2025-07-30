import random
from pathlib import Path

import arcade


class WildPokemon(arcade.Sprite):
    def __init__(self, image_path, maggots_bounds, name:str, field:int,scale=2.0,):
        self.path_ding = Path(image_path)
        self.sprite_file_location = Path(image_path)/"banner.png"
        self.name = name
        self.field = field
        self.image_path = image_path
        self.animations = {
            "down": [arcade.load_texture(self.path_ding / f"over_world_{i}.png") for i in range(0, 3)],
            "up": [arcade.load_texture(self.path_ding / f"over_world_{i}.png") for i in range(3, 6)],
            "left": [arcade.load_texture(self.path_ding / f"over_world_{i}.png") for i in range(6, 9)],
            "right": [arcade.load_texture(self.path_ding / f"over_world_{i}.png") for i in range(9, 12)],
        }
        super().__init__(self.path_ding / "over_world_0.png", scale)
        self.direction = "down"
        self.texture = self.animations[self.direction][0]
        self.maggots_bounds = maggots_bounds
        self.direction_timer = 0
        self.change_interval = 0.6
        self.current_frame = 0
        self.frame_timer = 0
        self.frame_duration = 0.05

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
        self.direction_timer += delta_time
        self.update_animation(delta_time)

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

        left, right, bottom, top = self.maggots_bounds
        self.center_x = max(left, min(self.center_x, right))
        self.center_y = max(bottom, min(self.center_y, top))

        self.overlay_visible = False

        # make somthing to make a movement trough grass
