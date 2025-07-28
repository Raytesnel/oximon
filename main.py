import random
import os
from pathlib import Path

import arcade
from arcade import load_tilemap
from arcade.hitbox import HitBoxAlgorithm

from super_smash_platform import SmashStageOnlyView

SCREEN_WIDTH = 800
SCREEN_HEIGHT = 600
SCREEN_TITLE = "PokeSmash"

ASSETS_PATH = os.path.join(os.path.dirname(__file__), "assets")


class WildPokemon(arcade.Sprite):
    def __init__(self, image_path, maggots_bounds, name:str, scale=2.0,):
        self.path_ding = Path(image_path)
        self.sprite_file_location = Path(image_path)/"banner.png"
        self.name = name
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
        self.change_interval = 0.5
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


class OverworldView(arcade.View):
    def __init__(self):
        super().__init__()
        self.count = False
        self.max_pokemon_per_bush = 3 # TODO: take this from the tiled map.
        self.counter_pokemon =0

    def setup(self):
        self.player_list = arcade.SpriteList()
        asset_path = os.path.join(ASSETS_PATH, "sprites/player")
        self.camera = arcade.Camera2D()
        self.tile_map = load_tilemap(
            os.path.join(ASSETS_PATH, "map/probeersel_01.tmx"),
            scaling=2.0,
            use_spatial_hash=True,
        )
        self.scene = arcade.Scene.from_tilemap(self.tile_map)

        try:
            start = next((o for o in self.tile_map.object_lists["objects"] if o.name == "player-start"), None)
            print("found player start")
        except KeyError:
            print(" player start not found")
            start = None
        self.player = Player(asset_path)
        if start:
            self.player.center_x = start.shape[0][0]
            self.player.center_y = start.shape[0][1]
        else:
            self.player.center_x = SCREEN_WIDTH // 2
            self.player.center_y = SCREEN_HEIGHT // 2
        self.player.scale = 1.0
        self.player_list.append(self.player)
        self.walls = self.scene["abandoned"]
        self.wild_pokemon_list = arcade.SpriteList()


    def on_draw(self):
        self.clear()
        self.camera.use()
        self.scene.draw()
        self.player_list.draw()
        self.wild_pokemon_list.draw()
        self.scene.get_sprite_list("voorgrond").draw()

    def on_update(self, delta_time):
        self.player_list.update()
        self.player_list.update_animation(delta_time)
        self.wild_pokemon_list.update()
        if self.max_pokemon_per_bush > len(self.wild_pokemon_list):
            if self.counter_pokemon > 300:
                left_top, right_top, right_bottom, left_bottom= self.tile_map.object_lists["Bulbasaur"][0].shape
                # TODO fix to have multiple grass bushes in different arrays
                bounds = (left_top[0], right_top[0], left_bottom[1], left_top[1])
                name_pokemon = random.choice(["Charmander","Bulbasaur"])
                pokemon_sprite = WildPokemon(
                image_path=os.path.join(ASSETS_PATH, f"sprites/pokemon/{name_pokemon}"),
                maggots_bounds=bounds,
                scale=2.0,
                name=name_pokemon
                )

                pokemon_sprite.position = (
                    random.uniform(bounds[0], bounds[1]),
                    random.uniform(bounds[2], bounds[3])
                )
                self.wild_pokemon_list.append(pokemon_sprite)
                self.counter_pokemon = 0
            else:
                self.counter_pokemon+=1
        if arcade.check_for_collision_with_list(self.player, self.walls):
            self.player.center_x -= self.player.change_x
            self.player.center_y -= self.player.change_y

        self.camera.position = arcade.Vec2(self.player.center_x, self.player.center_y)
        collided_pokemon_list = arcade.check_for_collision_with_list(self.player, self.wild_pokemon_list)
        if collided_pokemon_list:
            collided_pokemon:WildPokemon = collided_pokemon_list[0]  # You can handle multiple later if needed

            splash = BattleSplashView(
                player_sprite_path=os.path.join(ASSETS_PATH, "sprites/pokemon/player_shot.png"),
                banner_path=os.path.join(ASSETS_PATH, "sprites/pokemon/banner.jpg"),
                game_view=self,
                wild_pokemon=collided_pokemon
            )
            self.window.show_view(splash)
            self.wild_pokemon_list.remove(collided_pokemon)
            # TODO make reset world view & walking
            # TODO make the grass move between pokemon

    def on_key_press(self, key, modifiers):
        self.player.change_x = 0
        self.player.change_y = 0
        if key == arcade.key.UP:
            self.player.change_y = 5
        elif key == arcade.key.DOWN:
            self.player.change_y = -5
        elif key == arcade.key.LEFT:
            self.player.change_x = -5
        elif key == arcade.key.RIGHT:
            self.player.change_x = 5

    def on_key_release(self, key, modifiers):
        if key == arcade.key.UP and self.player.change_y > 0:
            self.player.change_y = 0
        elif key == arcade.key.DOWN and self.player.change_y < 0:
            self.player.change_y = 0
        elif key == arcade.key.LEFT and self.player.change_x < 0:
            self.player.change_x = 0
        elif key == arcade.key.RIGHT and self.player.change_x > 0:
            self.player.change_x = 0


class BattleSplashView(arcade.View):
    def __init__(self, player_sprite_path, banner_path, game_view, wild_pokemon):
        super().__init__()
        self.wild_pokemon = wild_pokemon
        self.timer = 0
        self.show_duration = 1.5
        self.game_view = game_view
        banner_widht = 555
        scaling = self.window.width / banner_widht
        self.banner = arcade.Sprite(banner_path,scale=scaling)
        self.player_sprite = arcade.Sprite(player_sprite_path,scale=0.5)
        self.enemy_sprite = arcade.Sprite(self.wild_pokemon.sprite_file_location, scale=0.2)
        self.sprites = arcade.SpriteList()
        self.on_show()
        self.sprites.append(self.banner)
        self.sprites.append(self.player_sprite)
        self.sprites.append(self.enemy_sprite)

    def on_show(self):
        self.banner.center_x = self.game_view.player.center_x
        self.banner.center_y = self.game_view.player.center_y
        self.player_sprite.center_x = self.game_view.player.center_x - self.window.width //2 + self.player_sprite.width//2
        self.player_sprite.center_y = self.game_view.player.center_y
        self.enemy_sprite.center_x = self.game_view.player.center_x + self.window.width //2 - self.enemy_sprite.width//2
        self.enemy_sprite.center_y = self.game_view.player.center_y

    def on_draw(self):
        self.clear()
        self.sprites.draw()

    def on_update(self, delta_time):
        self.timer += delta_time
        if self.timer > self.show_duration:
            self.window.show_view(SmashStageOnlyView(self.game_view))
            # self.window.show_view(self.game_view)


    # def on_key_press(self, key, modifiers):
    #     self.window.show_view(self.game_view)


if __name__ == "__main__":
    window = arcade.Window(SCREEN_WIDTH, SCREEN_HEIGHT, SCREEN_TITLE)
    view = OverworldView()
    view.setup()
    window.show_view(view)
    arcade.run()
