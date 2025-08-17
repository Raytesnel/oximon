from pathlib import Path

import arcade
from loguru import logger
from pydantic import BaseModel

from utils import ASSETS_PATH


class Position(BaseModel):
    x_direction: float
    y_direction: float


class AttackPattern(BaseModel, arbitrary_types_allowed=True):
    texture: arcade.Texture
    damage: float
    knockback_direction: Position
    knockback_strength: int
    explosion_knock_back: int


class Attack(arcade.Sprite):
    def __init__(
        self,
        attack_pattern: list[AttackPattern],
        frame_duration=2 / 10,
    ):
        super().__init__()
        self.is_hit = False
        self.attack_pattern = attack_pattern
        self.attack_flow = None
        self.explosion_knock_back = 0
        self.knockback_strength = 0
        self.textures = [pattern.texture for pattern in attack_pattern]
        self.texture = attack_pattern[0].texture
        self.current_frame = 0
        self.animation_timer = 0
        self.frame_duration = frame_duration
        self.damage = 0
        self.knockback_direction = Position(x_direction=0, y_direction=0)
        self.direction = (0,0) # (1,0) right, (-1,0) left, (0,1) up, (0,-1) down

    def new_attack(self):
        self.attack_flow = iter(self.attack_pattern)
        self.current_frame = 0
        self.is_hit = False
        self.explosion_knock_back = 0
        self.damage = 0
        self.knockback_direction = Position(x_direction=0, y_direction=0)
        self.knockback_strength = 0
        self.animation_timer = 0
        self.texture = self.attack_pattern[0].texture

    def update(self, delta_time):
        try:
            new_attack: AttackPattern = next(self.attack_flow)
        except StopIteration:
            logger.debug("eind of animation")
            self.remove_from_sprite_lists()
            return
        self.texture = new_attack.texture
        self.explosion_knock_back = new_attack.explosion_knock_back
        self.knockback_strength = new_attack.knockback_strength
        self.damage = new_attack.damage
        self.knockback_direction = new_attack.knockback_direction

        self.height = self.texture.height


def collect_attack_sprites(attack_folder: Path) -> list[arcade.Texture]:
    return [
        arcade.load_texture(attack_folder / f"hadukan_{i}.png") for i in range(1, 11)
    ]

class BlueFireBreath(Attack):
    def __init__(self)->None:
        file_name = ASSETS_PATH / "sprites" / "pokemon" /"atacks"/"Fire_2.png"
        self.attack_textures = arcade.load_spritesheet(file_name).get_texture_grid(size=(64, 64), columns=11, count=11)
        attack_patern =  [
            AttackPattern(
                explosion_knock_back=0,
                damage=10,
                knockback_direction=Position(x_direction=1, y_direction=1),
                knockback_strength=10,
                texture=attack_texture
            )
            for attack_texture in self.attack_textures
        ]
        logger.debug("it will rain fire!")
        super().__init__(attack_pattern=attack_patern)
        self.direction = (1,0)
        # TODO: implement, self.is_hit so the ball can trigger effect when hit.

    def update(self,delta_time:float):
        self.animation_timer += delta_time
        idx = self.attack_textures.index(self.texture)
        if idx < len(self.attack_textures) - 6:
            match self.direction:
                case (1, 0):
                    self.center_x += 2
                case (-1,0):
                    self.center_x -= 2
                case (0,1):
                    self.center_y += 2
                case (0,-1):
                    self.center_y -= 2

        else:
            match self.direction:
                case (1, 0):
                    self.center_x += 0.5
                case (-1, 0):
                    self.center_x -= 0.5
                case (0, 1):
                    self.center_y += 0.5
                case (0, -1):
                    self.center_y -= 0.5
        if self.animation_timer >= self.frame_duration:
            # width_last_frame = self.texture.width
            super().update(delta_time)

            self.animation_timer = 0


class FireBreath(Attack):
    def __init__(self)->None:
        file_name = ASSETS_PATH / "sprites" / "pokemon" /"atacks"/"Flame_jet.png"
        self.attack_textures = arcade.load_spritesheet(file_name).get_texture_grid(size=(88, 79), columns=12, count=12)
        attack_patern =  [
            AttackPattern(
                explosion_knock_back=-5,
                damage=2,
                knockback_direction=Position(x_direction=0, y_direction=2),
                knockback_strength=0,
                texture=attack_texture
            )
            for attack_texture in self.attack_textures
        ]
        logger.debug("it will rain fire!")
        super().__init__(attack_pattern=attack_patern)

    def update(self,delta_time:float):
        self.animation_timer += delta_time
        if self.animation_timer >= self.frame_duration:
            # width_last_frame = self.texture.width
            super().update(delta_time)
            self.animation_timer = 0

class ElectroBall(Attack):
    def __init__(self)->None:
        file_name = ASSETS_PATH / "sprites" / "pokemon" /"atacks"/"Charge.png"
        self.attack_textures = arcade.load_spritesheet(file_name).get_texture_grid(size=(64, 64), columns=10, count=10)
        attack_patern =  [
            AttackPattern(
                explosion_knock_back=-5,
                damage=2,
                knockback_direction=Position(x_direction=-2, y_direction=-2),
                knockback_strength=0,
                texture=attack_texture
            )
            for attack_texture in self.attack_textures
        ]
        logger.debug("it will rain fire!")
        super().__init__(attack_pattern=attack_patern)
        self.direction=(0,0)

    def update(self,delta_time:float):
        self.animation_timer += delta_time
        idx = self.attack_textures.index(self.texture)
        match self.direction:
            case (1, 0):
                self.center_x += 0.5
            case (-1,0):
                self.center_x -= 0.5
            case (0,1):
                self.center_y += 0.5
            case (0,-1):
                self.center_y -= 0.5

        if self.animation_timer >= self.frame_duration:
            # width_last_frame = self.texture.width
            super().update(delta_time)

            self.animation_timer = 0

# TODO: add config for possible attacks and config for the player chosen attacks.
# TODO: add hitbox damage for blunt attack (taclke) atacks.