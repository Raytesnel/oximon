import random
from pathlib import Path

import arcade
import os

from arcade import load_tilemap
from arcade.hitbox import HitBoxAlgorithm

SCREEN_WIDTH = 800
SCREEN_HEIGHT = 600
SCREEN_TITLE = "PokeSmash"

ASSETS_PATH = os.path.join(os.path.dirname(__file__), "assets")

class WildPokemon(arcade.Sprite):
    def __init__(self, image_path, maggots_bounds, scale=2.0):
        path_ding = Path(image_path)
        print(path_ding)
        self.animations = {
            "down": [arcade.load_texture(path_ding/ f"player_{i}.png") for i in range(0, 3)],
            "up": [arcade.load_texture(path_ding/ f"player_{i}.png") for i in range(3, 6)],
            "left": [arcade.load_texture(path_ding/ f"player_{i}.png") for i in range(6, 9)],
            "right": [arcade.load_texture(path_ding/ f"player_{i}.png") for i in range(9, 12)],
        }
        print(self.animations)
        super().__init__(path_ding/"player_0.png", scale)
        self.direction = "down"
        self.texture = self.animations[self.direction][0]
        self.maggots_bounds = maggots_bounds  # (left, right, bottom, top)
        self.direction_timer = 0
        self.change_interval = 0.5  # Seconds between direction changes
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

    def update(self, delta_time: float ):
        self.direction_timer += delta_time
        self.update_animation(delta_time)

        if self.direction_timer >= self.change_interval:
            self.direction_timer = 0
            direction = random.choice(['up', 'down', 'left', 'right', 'idle'])
            speed = random.uniform(0.5,2)
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

        # Move and clamp inside area
        self.center_x += self.change_x
        self.center_y += self.change_y

        left, right, bottom, top = self.maggots_bounds
        self.center_x = max(left, min(self.center_x, right))
        self.center_y = max(bottom, min(self.center_y, top))


class Player(arcade.Sprite):
    def __init__(self, asset_path):
        super().__init__()

        self.animations = {
            "down": [arcade.load_texture(os.path.join(asset_path, f"player_{i}.png")) for i in range(0, 3)],
            "up": [arcade.load_texture(os.path.join(asset_path, f"player_{i}.png")) for i in range(3, 6)],
            "left": [arcade.load_texture(os.path.join(asset_path, f"player_{i}.png")) for i in range(6, 9)],
            "right": [arcade.load_texture(os.path.join(asset_path, f"player_{i}.png")) for i in range(9, 12)],
        }
        self.direction = "down"
        self.current_frame = 0
        self.frame_timer = 0
        self.frame_duration = 0.05

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
            self.current_frame = (self.current_frame + 1) % len(self.animations[self.direction])
            self.texture = self.animations[self.direction][self.current_frame]
            self.frame_timer = 0


class WalkDemo(arcade.Window):
    def __init__(self):
        super().__init__(SCREEN_WIDTH, SCREEN_HEIGHT, SCREEN_TITLE)
        arcade.set_background_color(arcade.color.SKY_BLUE)

        self.player = None
        self.player_list = None
        self.camera = None
        self.scene = None
        self.map_sprite = None
        self.walls = None
        self.pokemon = None

    def setup(self):
        self.player_list = arcade.SpriteList()
        asset_path = os.path.join(ASSETS_PATH, "sprites/player")
        self.camera = arcade.Camera2D()
        tile_map = load_tilemap(
            os.path.join(ASSETS_PATH, "map/orthogonal.tmx"),
            scaling=2.0,
            use_spatial_hash=True,
            layer_options={
                "Fringe": {
                    "use_spatial_hash": True,
                }
            }
        )
        self.scene = arcade.Scene.from_tilemap(tile_map)

        map_object = tile_map.object_lists["Objects"]
        start = next((o for o in map_object if o.name == "player-start"), None)

        self.player = Player(asset_path)
        if start:
            print(start)
            self.player.center_x = start.shape[0][0]
            self.player.center_y = start.shape[0][1]
        else:
            self.player.center_x = SCREEN_WIDTH // 2
            self.player.center_y = SCREEN_HEIGHT // 2
        self.player.scale = 2.0
        self.player_list.append(self.player)
        self.walls = self.scene["Fringe"]

        maggots_area = None
        for obj in tile_map.object_lists.get("pokemon", []):
            if obj.name == "maggots":
                maggots_area = obj.shape
                break

        self.wild_pokemon_list = arcade.SpriteList()

        # Use shape corners
        left_top, right_top, right_bottom, left_bottom = maggots_area
        left = left_top[0]
        right = right_top[0]
        bottom = left_bottom[1]
        top = left_top[1]
        maggots_bounds = (left, right, bottom, top)

        # Create wandering Pokémon
        self.pokemon_sprite = WildPokemon(
            image_path=os.path.join(ASSETS_PATH, "sprites/pokemon/pokemon_1/"),
            maggots_bounds=maggots_bounds,
            scale=2.0
        )

        # Random initial position
        spawn_x = random.uniform(left, right)
        spawn_y = random.uniform(bottom, top)
        self.pokemon_sprite.position = (spawn_x, spawn_y)
        self.wild_pokemon_list.append(self.pokemon_sprite)

        self.scene.add_sprite_list("WildPokemon", sprite_list=self.wild_pokemon_list)

    def on_draw(self):
        self.clear()
        self.camera.use()
        self.scene.draw()
        self.player_list.draw()
        self.wild_pokemon_list.draw()

    def on_update(self, delta_time):
        self.player_list.update()
        self.player_list.update_animation(delta_time)
        self.wild_pokemon_list.update()

        if arcade.check_for_collision_with_list(self.player, self.walls):
            self.player.center_x -= self.player.change_x
        if arcade.check_for_collision_with_list(self.player, self.walls):
            self.player.center_y -= self.player.change_y
        self.center_camera_to_player()
        hit_list = arcade.check_for_collision_with_list(self.player, self.wild_pokemon_list)
        if hit_list:
            print("Encountered a wild Pokémon!")
            # TODO: Trigger battle here

    def on_key_press(self, key, modifiers):
        # Cancel movement in both directions first
        self.player.change_x = 0
        self.player.change_y = 0
        self.pokemon_sprite.change_y = 0
        self.pokemon_sprite.change_x = 0

        # Only one direction active at a time
        if key == arcade.key.B:
            self.pokemon_sprite.change_x = 5
        if key == arcade.key.A:
            self.pokemon_sprite.change_x = -5
        if key == arcade.key.UP:
            self.player.change_y = 5
        elif key == arcade.key.DOWN:
            self.player.change_y = -5
        elif key == arcade.key.LEFT:
            self.player.change_x = -5
        elif key == arcade.key.RIGHT:
            self.player.change_x = 5

    def on_key_release(self, key, modifiers):
        # Stop player movement on key release
        if key == arcade.key.UP and self.player.change_y > 0:
            self.player.change_y = 0
        elif key == arcade.key.DOWN and self.player.change_y < 0:
            self.player.change_y = 0
        elif key == arcade.key.LEFT and self.player.change_x < 0:
            self.player.change_x = 0
        elif key == arcade.key.RIGHT and self.player.change_x > 0:
            self.player.change_x = 0

    def center_camera_to_player(self):
        target = arcade.Vec2(
            self.player.center_x,
            self.player.center_y
        )
        self.camera.position = target


if __name__ == "__main__":
    window = WalkDemo()
    window.setup()
    arcade.run()
